use std::path::PathBuf;
use std::sync::Arc;

use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolResult, Content, Implementation, InitializeResult, ProtocolVersion,
        ServerCapabilities, ServerInfo,
    },
    tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler,
};
use tokio::sync::RwLock;

use crate::cache::CacheManager;
use crate::config::CodeScopeConfig;
use crate::language::LanguageRegistry;
use crate::parser::CachedParser;
use crate::pipeline::{
    CommentCollector, DefinitionCollector, FilePipeline, ImportCollector, MethodCallCollector,
    UsageCollector,
};
use crate::server::types::{
    CodeAtLocationParams, CommentSearchParams, DefinitionParams, ImportsParams, MethodCallsParams,
    SymbolAtLocationParams, SymbolAtLocationResponse, UsagesParams,
};
use crate::symbol::comment::get_code_at_location;
use crate::symbol::types::SymbolDefinition;

/// CodeScope MCP Server
#[derive(Clone)]
pub struct CodeScopeServer {
    workspace_root: Arc<RwLock<Option<PathBuf>>>,
    config: Arc<RwLock<CodeScopeConfig>>,
    registry: Arc<LanguageRegistry>,
    cache_manager: Arc<CacheManager>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl CodeScopeServer {
    /// Create a new CodeScope server
    ///
    /// # Panics
    ///
    /// Panics if the language registry cannot be created. This can only happen
    /// if the embedded tree-sitter queries are malformed, which would indicate
    /// a programming error caught during development/testing rather than a
    /// runtime failure.
    pub fn new() -> Self {
        let registry = Arc::new(LanguageRegistry::new().expect(
            "Failed to create language registry: embedded queries are malformed. \
                This is a programming error that should be caught during development.",
        ));

        Self {
            workspace_root: Arc::new(RwLock::new(None)),
            config: Arc::new(RwLock::new(CodeScopeConfig::default_config())),
            registry,
            cache_manager: Arc::new(CacheManager::new()),
            tool_router: Self::tool_router(),
        }
    }

    /// Try to create a new CodeScope server, returning an error if initialization fails
    pub fn try_new() -> Result<Self, anyhow::Error> {
        let registry = Arc::new(LanguageRegistry::new()?);

        Ok(Self {
            workspace_root: Arc::new(RwLock::new(None)),
            config: Arc::new(RwLock::new(CodeScopeConfig::default_config())),
            registry,
            cache_manager: Arc::new(CacheManager::new()),
            tool_router: Self::tool_router(),
        })
    }

    /// Set the workspace root directory and load config
    pub async fn set_workspace_root(&self, root: PathBuf) {
        let loaded_config = CodeScopeConfig::load(&root);
        {
            let mut config = self.config.write().await;
            *config = loaded_config;
        }

        let mut workspace = self.workspace_root.write().await;
        *workspace = Some(root);
    }

    /// Get the workspace root directory
    async fn get_workspace_root(&self) -> Result<PathBuf, McpError> {
        let workspace = self.workspace_root.read().await;
        match workspace.as_ref() {
            Some(path) => Ok(path.clone()),
            None => std::env::current_dir().map_err(|e| {
                McpError::internal_error(format!("Failed to get current directory: {}", e), None)
            }),
        }
    }

    /// Create a file pipeline with the given excludes
    async fn create_pipeline(
        &self,
        exclude_dirs: Option<Vec<String>>,
    ) -> Result<FilePipeline, McpError> {
        let workspace_root = self.get_workspace_root().await?;
        let config = self.config.read().await.clone();

        Ok(FilePipeline::new(
            self.registry.clone(),
            workspace_root,
            config,
            self.cache_manager.clone(),
        )
        .with_excludes(exclude_dirs))
    }

    /// Helper to serialize results to JSON
    fn serialize_result<T: serde::Serialize>(result: &T) -> Result<CallToolResult, McpError> {
        let json = serde_json::to_string_pretty(result).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize result: {}", e), None)
        })?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Lightweight AST search for symbol definitions. Find functions, classes, methods, variables. Use include_docs=true for JSDoc/docstrings. Simple interface: just symbol name, no path required. Supports 12 languages: TypeScript, TSX, JavaScript, JSX, Python, Rust, Go, Java, HTML, CSS, SQL, Markdown."
    )]
    async fn symbol_definition(
        &self,
        Parameters(DefinitionParams {
            symbol,
            include_docs,
            exclude_dirs,
            language,
        }): Parameters<DefinitionParams>,
    ) -> Result<CallToolResult, McpError> {
        let pipeline = self
            .create_pipeline(exclude_dirs)
            .await?
            .with_language_filter(language);
        let collector = DefinitionCollector {
            symbol,
            include_docs: include_docs.unwrap_or(false),
        };

        let results = pipeline.process(&collector);
        Self::serialize_result(&results)
    }

    #[tool(
        description = "Find all usages of a symbol with usage classification (Import, MethodCall, PropertyAccess, TypeReference, Identifier). Use include_contexts=true for scope hierarchy. Supports 12 languages: TypeScript, TSX, JavaScript, JSX, Python, Rust, Go, Java, HTML, CSS, SQL, Markdown."
    )]
    async fn symbol_usages(
        &self,
        Parameters(UsagesParams {
            symbol,
            include_contexts,
            exclude_dirs,
            language,
        }): Parameters<UsagesParams>,
    ) -> Result<CallToolResult, McpError> {
        let pipeline = self
            .create_pipeline(exclude_dirs)
            .await?
            .with_language_filter(language);
        let collector = UsageCollector {
            symbol,
            include_imports: true,
            max_contexts: if include_contexts.unwrap_or(false) {
                2
            } else {
                0
            },
            object_filter: None,
        };

        let results = pipeline.process(&collector);
        Self::serialize_result(&results)
    }

    #[tool(
        description = "Find method/function calls like Date.now(), array.map(), console.log(). Use object_name to filter (e.g., object_name='Date' finds only Date.now(), not performance.now()). UNIQUE: No other tool can filter method calls by object. Supports: TypeScript, JavaScript."
    )]
    async fn find_method_calls(
        &self,
        Parameters(MethodCallsParams {
            method_name,
            object_name,
            exclude_dirs,
            language,
        }): Parameters<MethodCallsParams>,
    ) -> Result<CallToolResult, McpError> {
        let pipeline = self
            .create_pipeline(exclude_dirs)
            .await?
            .with_language_filter(language);
        let collector = MethodCallCollector {
            method_name,
            object_name,
        };

        let results = pipeline.process(&collector);
        Self::serialize_result(&results)
    }

    #[tool(
        description = "Find import/require statements for a symbol. See where and how a module is imported across the codebase. UNIQUE: Specialized import search, more precise than grep 'import'. Supports: TypeScript, JavaScript."
    )]
    async fn find_imports(
        &self,
        Parameters(ImportsParams {
            symbol,
            exclude_dirs,
            language,
        }): Parameters<ImportsParams>,
    ) -> Result<CallToolResult, McpError> {
        let pipeline = self
            .create_pipeline(exclude_dirs)
            .await?
            .with_language_filter(language);
        let collector = ImportCollector { symbol };

        let results = pipeline.process(&collector);
        Self::serialize_result(&results)
    }

    #[tool(
        description = "Search text ONLY within comments - excludes code and strings. Find TODO, FIXME, HACK, or any text in comments. UNIQUE: Comment-only search, grep cannot distinguish comments from code. Supports: TypeScript, JavaScript, Rust, Go, Java, HTML, CSS."
    )]
    async fn find_in_comments(
        &self,
        Parameters(CommentSearchParams {
            text,
            exclude_dirs,
            language,
        }): Parameters<CommentSearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let pipeline = self
            .create_pipeline(exclude_dirs)
            .await?
            .with_language_filter(language);
        let collector = CommentCollector { text };

        let results = pipeline.process(&collector);
        Self::serialize_result(&results)
    }

    #[tool(
        description = "Get code snippet at a specific file:line with surrounding context. Use after grep or symbol_usages to see actual code around a match. Supports all text files."
    )]
    async fn get_code_at_location(
        &self,
        Parameters(CodeAtLocationParams {
            file_path,
            line,
            context_before,
            context_after,
        }): Parameters<CodeAtLocationParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = PathBuf::from(&file_path);
        let before = context_before.unwrap_or(3);
        let after = context_after.unwrap_or(3);

        let snippet = get_code_at_location(&path, line, before, after)
            .map_err(|e| McpError::internal_error(format!("Failed to read code: {}", e), None))?;

        Self::serialize_result(&snippet)
    }

    #[tool(
        description = "Get the enclosing function/class/method at a specific line. Use after grep or symbol_usages to get full context. Example: Found 'handleError' at line 42 → get the entire function containing it. Supports: TypeScript, TSX, JavaScript, JSX, Python, Rust, Go, Java, HTML, CSS, SQL, Markdown."
    )]
    async fn get_symbol_at_location(
        &self,
        Parameters(SymbolAtLocationParams { file_path, line }): Parameters<SymbolAtLocationParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = PathBuf::from(&file_path);

        // Check if file is supported
        if self.registry.get_for_path(&path).is_none() {
            return Err(McpError::invalid_params(
                format!("Unsupported file type: {}", file_path),
                None,
            ));
        }

        // Read file and parse using cache
        let cached_content = self
            .cache_manager
            .file_cache
            .get_or_read(&path)
            .map_err(|e| McpError::internal_error(format!("Failed to read file: {}", e), None))?;

        let mut parser = CachedParser::new(
            self.registry.clone(),
            self.cache_manager.parser_cache.clone(),
        )
        .map_err(|e| McpError::internal_error(format!("Failed to create parser: {}", e), None))?;

        let (tree, lang) = parser
            .parse_with_language(&path, &cached_content.content, cached_content.modified_time)
            .map_err(|e| McpError::internal_error(format!("Failed to parse file: {}", e), None))?;

        // Find the symbol containing the given line
        let target_line = line.saturating_sub(1); // Convert to 0-indexed
        let query = lang.definitions_query();
        let mappings = lang.definition_mappings();
        let source_code = &cached_content.content;

        let mut cursor = tree_sitter::QueryCursor::new();
        use streaming_iterator::StreamingIterator;
        let mut matches = cursor.matches(query, tree.root_node(), source_code.as_bytes());

        let mut best_symbol: Option<SymbolDefinition> = None;
        let mut best_size: usize = usize::MAX;

        while let Some(m) = matches.next() {
            let mut name: Option<&str> = None;
            let mut definition_node: Option<tree_sitter::Node> = None;
            let mut kind = None;

            for capture in m.captures {
                let capture_name = &query.capture_names()[capture.index as usize];

                if *capture_name == "name" {
                    name = Some(capture.node.utf8_text(source_code.as_bytes()).unwrap_or(""));
                } else {
                    // Check mappings for definition types
                    for mapping in mappings {
                        if *capture_name == mapping.capture_name {
                            definition_node = Some(capture.node);
                            kind = Some(mapping.kind);
                            break;
                        }
                    }
                }
            }

            if let (Some(name_str), Some(node), Some(symbol_kind)) = (name, definition_node, kind) {
                let start_line = node.start_position().row;
                let end_line = node.end_position().row;

                // Check if the target line is within this symbol
                if target_line >= start_line && target_line <= end_line {
                    let size = end_line - start_line;
                    // Prefer the smallest enclosing symbol
                    if size < best_size {
                        best_size = size;
                        let code = node
                            .utf8_text(source_code.as_bytes())
                            .unwrap_or("")
                            .to_string();

                        best_symbol = Some(SymbolDefinition {
                            file_path: file_path.clone(),
                            start_line: start_line + 1,
                            end_line: end_line + 1,
                            node_kind: symbol_kind,
                            code,
                            name: name_str.to_string(),
                            docs: None,
                        });
                    }
                }
            }
        }

        let response = SymbolAtLocationResponse {
            symbol: best_symbol,
        };

        Self::serialize_result(&response)
    }
}

#[tool_handler]
impl ServerHandler for CodeScopeServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "CodeScope: Lightweight AST-based code search (12 languages, no LSP required).\n\n\
                UNIQUE TOOLS (not available elsewhere):\n\
                - find_method_calls: Search obj.method() patterns, filter by object (Date.now vs performance.now)\n\
                - find_imports: Find import statements for a symbol\n\
                - find_in_comments: Search ONLY in comments (TODO, FIXME, etc)\n\
                - get_symbol_at_location: Get enclosing function/class at line number\n\n\
                GENERAL TOOLS:\n\
                - symbol_definition: Find where symbols are defined (simple: just name, no path)\n\
                - symbol_usages: Find all usages with classification (Import/MethodCall/etc)\n\
                - get_code_at_location: Get code snippet at file:line\n\n\
                LANGUAGES: TypeScript/TSX, JavaScript/JSX, Python, Rust, Go, Java, HTML, CSS, SQL, Markdown\n\n\
                USE CASES:\n\
                - 'Find all Date.now() calls' → find_method_calls(method_name='now', object_name='Date')\n\
                - 'Where is useState imported?' → find_imports(symbol='useState')\n\
                - 'Find all TODOs in comments' → find_in_comments(text='TODO')\n\
                - 'Get the function at line 42' → get_symbol_at_location(file_path='...', line=42)"
                    .to_string(),
            ),
        }
    }

    async fn initialize(
        &self,
        _request: rmcp::model::InitializeRequestParam,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        Ok(self.get_info())
    }
}

impl Default for CodeScopeServer {
    fn default() -> Self {
        Self::new()
    }
}
