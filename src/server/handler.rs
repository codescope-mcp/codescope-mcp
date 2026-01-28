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
        description = "Find symbol definitions in the codebase. Returns file path, line numbers, and source code for each definition found. Set include_docs=true to get JSDoc/comments. Supports TypeScript, TSX, and Markdown."
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
        description = "Find all usages of a symbol in the codebase. Returns file path, line, column, and usage_kind. Set include_contexts=true for context info. Supports TypeScript, TSX, and Markdown."
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
        description = "Find method/function calls. Use for patterns like Date.now() or array.map(). Specify object_name to filter by object (e.g., object_name='Date' for Date.now() only). Supports TypeScript and TSX."
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
        description = "Find import statements for a symbol. Use to understand module dependencies and where a symbol is imported from. Supports TypeScript and TSX."
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
        description = "Search for text within comments (TypeScript/TSX) or full text (Markdown). Use to find TODOs, FIXMEs, or any text in comments."
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
        description = "Get code at a specific file location. Use after find_in_comments or symbol_usages to see the actual code around a specific line."
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
        description = "Get the enclosing symbol at a specific file location. Returns the full symbol (function, class, heading, etc.) that contains the given line. Use after symbol_usages to get the full context of where a symbol is used."
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
        let source_code = self
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
            .parse_with_language(&path, &source_code)
            .map_err(|e| McpError::internal_error(format!("Failed to parse file: {}", e), None))?;

        // Find the symbol containing the given line
        let target_line = line.saturating_sub(1); // Convert to 0-indexed
        let query = lang.definitions_query();
        let mappings = lang.definition_mappings();

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
                "CodeScope provides TypeScript/TSX/Markdown symbol search tools.\n\n\
                TOOLS:\n\
                - symbol_definition: Find where symbols are defined. Use include_docs=true for JSDoc.\n\
                - symbol_usages: Find all usages of a symbol\n\
                - find_method_calls: Find method calls (e.g., Date.now()). Use object_name to filter.\n\
                - find_imports: Find import statements for a symbol\n\
                - find_in_comments: Search text in comments (TODO, FIXME, etc.)\n\
                - get_code_at_location: Get code at a specific file:line\n\
                - get_symbol_at_location: Get the enclosing symbol at a file:line\n\n\
                WORKFLOW:\n\
                1. Search with find_in_comments/symbol_usages\n\
                2. Get code context with get_code_at_location(file_path, line)\n\
                3. Get full symbol with get_symbol_at_location(file_path, line)"
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
