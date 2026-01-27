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

use crate::config::CodeScopeConfig;
use crate::language::LanguageRegistry;
use crate::pipeline::{
    CommentCollector, DefinitionCollector, FilePipeline, ImportCollector, MethodCallCollector,
    UsageCollector,
};
use crate::server::types::{
    CodeAtLocationParams, CommentSearchParams, DefinitionParams, ImportsParams, MethodCallsParams,
    UsagesParams,
};
use crate::symbol::comment::get_code_at_location;

/// CodeScope MCP Server
#[derive(Clone)]
pub struct CodeScopeServer {
    workspace_root: Arc<RwLock<Option<PathBuf>>>,
    config: Arc<RwLock<CodeScopeConfig>>,
    registry: Arc<LanguageRegistry>,
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

        Ok(
            FilePipeline::new(self.registry.clone(), workspace_root, config)
                .with_excludes(exclude_dirs),
        )
    }

    /// Helper to serialize results to JSON
    fn serialize_result<T: serde::Serialize>(result: &T) -> Result<CallToolResult, McpError> {
        let json = serde_json::to_string_pretty(result).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize result: {}", e), None)
        })?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Find symbol definitions in the TypeScript/TSX codebase. Returns file path, line numbers, and source code for each definition found. Set include_docs=true to get JSDoc/comments."
    )]
    async fn symbol_definition(
        &self,
        Parameters(DefinitionParams {
            symbol,
            include_docs,
            exclude_dirs,
        }): Parameters<DefinitionParams>,
    ) -> Result<CallToolResult, McpError> {
        let pipeline = self.create_pipeline(exclude_dirs).await?;
        let collector = DefinitionCollector {
            symbol,
            include_docs: include_docs.unwrap_or(false),
        };

        let results = pipeline.process(&collector);
        Self::serialize_result(&results)
    }

    #[tool(
        description = "Find all usages of a symbol in the TypeScript/TSX codebase. Returns file path, line, column, and usage_kind. Set include_contexts=true for context info."
    )]
    async fn symbol_usages(
        &self,
        Parameters(UsagesParams {
            symbol,
            include_contexts,
            exclude_dirs,
        }): Parameters<UsagesParams>,
    ) -> Result<CallToolResult, McpError> {
        let pipeline = self.create_pipeline(exclude_dirs).await?;
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
        description = "Find method/function calls. Use for patterns like Date.now() or array.map(). Specify object_name to filter by object (e.g., object_name='Date' for Date.now() only)."
    )]
    async fn find_method_calls(
        &self,
        Parameters(MethodCallsParams {
            method_name,
            object_name,
            exclude_dirs,
        }): Parameters<MethodCallsParams>,
    ) -> Result<CallToolResult, McpError> {
        let pipeline = self.create_pipeline(exclude_dirs).await?;
        let collector = MethodCallCollector {
            method_name,
            object_name,
        };

        let results = pipeline.process(&collector);
        Self::serialize_result(&results)
    }

    #[tool(
        description = "Find import statements for a symbol. Use to understand module dependencies and where a symbol is imported from."
    )]
    async fn find_imports(
        &self,
        Parameters(ImportsParams {
            symbol,
            exclude_dirs,
        }): Parameters<ImportsParams>,
    ) -> Result<CallToolResult, McpError> {
        let pipeline = self.create_pipeline(exclude_dirs).await?;
        let collector = ImportCollector { symbol };

        let results = pipeline.process(&collector);
        Self::serialize_result(&results)
    }

    #[tool(
        description = "Search for text within comments in TypeScript/TSX files. Use to find TODOs, FIXMEs, or any text in comments."
    )]
    async fn find_in_comments(
        &self,
        Parameters(CommentSearchParams { text, exclude_dirs }): Parameters<CommentSearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let pipeline = self.create_pipeline(exclude_dirs).await?;
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
}

#[tool_handler]
impl ServerHandler for CodeScopeServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "CodeScope provides TypeScript/TSX symbol search tools.\n\n\
                TOOLS:\n\
                - symbol_definition: Find where symbols are defined. Use include_docs=true for JSDoc.\n\
                - symbol_usages: Find all usages of a symbol\n\
                - find_method_calls: Find method calls (e.g., Date.now()). Use object_name to filter.\n\
                - find_imports: Find import statements for a symbol\n\
                - find_in_comments: Search text in comments (TODO, FIXME, etc.)\n\
                - get_code_at_location: Get code at a specific file:line\n\n\
                WORKFLOW:\n\
                1. Search with find_in_comments/symbol_usages\n\
                2. Get code context with get_code_at_location(file_path, line)"
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
