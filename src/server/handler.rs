use std::path::PathBuf;
use std::sync::Arc;

use ignore::WalkBuilder;
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
use crate::parser::typescript::TypeScriptParser;
use crate::server::types::{CodeAtLocationParams, CommentSearchParams, DefinitionParams, ImportsParams, MethodCallsParams, UsagesParams};
use crate::symbol::comment::{find_comments_in_file, get_code_at_location};
use crate::symbol::definition::find_definitions_in_file;
use crate::symbol::types::{CommentMatch, SymbolDefinition, SymbolUsage, UsageKind};
use crate::symbol::usage::find_usages_in_file;

/// CodeScope MCP Server
#[derive(Clone)]
pub struct CodeScopeServer {
    workspace_root: Arc<RwLock<Option<PathBuf>>>,
    config: Arc<RwLock<CodeScopeConfig>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl CodeScopeServer {
    pub fn new() -> Self {
        Self {
            workspace_root: Arc::new(RwLock::new(None)),
            config: Arc::new(RwLock::new(CodeScopeConfig::default_config())),
            tool_router: Self::tool_router(),
        }
    }

    /// Set the workspace root directory and load config
    pub async fn set_workspace_root(&self, root: PathBuf) {
        // Load config from workspace root
        let loaded_config = CodeScopeConfig::load(&root);
        {
            let mut config = self.config.write().await;
            *config = loaded_config;
        }

        let mut workspace = self.workspace_root.write().await;
        *workspace = Some(root);
    }

    /// Get all TypeScript/TSX files in the workspace
    async fn get_typescript_files(&self) -> Result<Vec<PathBuf>, McpError> {
        let workspace = self.workspace_root.read().await;
        let root = match workspace.as_ref() {
            Some(path) => path.clone(),
            None => {
                // Fall back to current directory if no workspace root is set
                std::env::current_dir().map_err(|e| {
                    McpError::internal_error(
                        format!("Failed to get current directory: {}", e),
                        None,
                    )
                })?
            }
        };

        let mut files = Vec::new();
        let walker = WalkBuilder::new(root).hidden(true).git_ignore(true).build();

        for entry in walker.flatten() {
            let path = entry.path();
            if path.is_file() && TypeScriptParser::is_typescript_file(path) {
                files.push(path.to_path_buf());
            }
        }

        Ok(files)
    }

    /// Helper function to filter files by config and additional exclude_dirs
    fn should_exclude_path(
        path: &PathBuf,
        config: &CodeScopeConfig,
        additional_excludes: &Option<Vec<String>>,
    ) -> bool {
        config.should_exclude(path, additional_excludes.as_ref().map(|v| v.as_slice()))
    }

    #[tool(description = "Find symbol definitions in the TypeScript/TSX codebase. Returns file path, line numbers, and source code for each definition found. Set include_docs=true to get JSDoc/comments.")]
    async fn symbol_definition(
        &self,
        Parameters(DefinitionParams { symbol, include_docs, exclude_dirs }): Parameters<DefinitionParams>,
    ) -> Result<CallToolResult, McpError> {
        let files = self.get_typescript_files().await?;
        let include_docs = include_docs.unwrap_or(false);
        let config = self.config.read().await;

        let mut parser = TypeScriptParser::new().map_err(|e| {
            McpError::internal_error(format!("Failed to create parser: {}", e), None)
        })?;

        let mut all_definitions: Vec<SymbolDefinition> = Vec::new();

        for file_path in files {
            if Self::should_exclude_path(&file_path, &config, &exclude_dirs) {
                continue;
            }

            match find_definitions_in_file(&mut parser, &file_path, &symbol, include_docs) {
                Ok(definitions) => {
                    all_definitions.extend(definitions);
                }
                Err(e) => {
                    tracing::warn!("Failed to parse {:?}: {}", file_path, e);
                }
            }
        }

        let result = serde_json::to_string_pretty(&all_definitions).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize result: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Find all usages of a symbol in the TypeScript/TSX codebase. Returns file path, line, column, and usage_kind. Set include_contexts=true for context info.")]
    async fn symbol_usages(
        &self,
        Parameters(UsagesParams {
            symbol,
            include_contexts,
            exclude_dirs,
        }): Parameters<UsagesParams>,
    ) -> Result<CallToolResult, McpError> {
        let files = self.get_typescript_files().await?;
        let config = self.config.read().await;

        let mut parser = TypeScriptParser::new().map_err(|e| {
            McpError::internal_error(format!("Failed to create parser: {}", e), None)
        })?;

        let mut all_usages: Vec<SymbolUsage> = Vec::new();

        // include_contexts=true の場合は2、false の場合は0（contexts が空になり skip_serializing_if で省略）
        let max_ctx = if include_contexts.unwrap_or(false) { 2 } else { 0 };
        // import は常に含める（消費者側でフィルタ）
        let incl_imports = true;

        for file_path in files {
            if Self::should_exclude_path(&file_path, &config, &exclude_dirs) {
                continue;
            }

            match find_usages_in_file(&mut parser, &file_path, &symbol, incl_imports, max_ctx, None) {
                Ok(usages) => {
                    all_usages.extend(usages);
                }
                Err(e) => {
                    tracing::warn!("Failed to parse {:?}: {}", file_path, e);
                }
            }
        }

        let result = serde_json::to_string_pretty(&all_usages).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize result: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Find method/function calls. Use for patterns like Date.now() or array.map(). Specify object_name to filter by object (e.g., object_name='Date' for Date.now() only).")]
    async fn find_method_calls(
        &self,
        Parameters(MethodCallsParams {
            method_name,
            object_name,
            exclude_dirs,
        }): Parameters<MethodCallsParams>,
    ) -> Result<CallToolResult, McpError> {
        let files = self.get_typescript_files().await?;
        let config = self.config.read().await;

        let mut parser = TypeScriptParser::new().map_err(|e| {
            McpError::internal_error(format!("Failed to create parser: {}", e), None)
        })?;

        let mut all_usages: Vec<SymbolUsage> = Vec::new();

        for file_path in files {
            if Self::should_exclude_path(&file_path, &config, &exclude_dirs) {
                continue;
            }

            // contexts なし、import なし、object_filter あり
            match find_usages_in_file(&mut parser, &file_path, &method_name, false, 0, object_name.as_deref()) {
                Ok(usages) => {
                    // MethodCall のみをフィルタ
                    let method_calls: Vec<_> = usages
                        .into_iter()
                        .filter(|u| u.usage_kind == UsageKind::MethodCall)
                        .collect();
                    all_usages.extend(method_calls);
                }
                Err(e) => {
                    tracing::warn!("Failed to parse {:?}: {}", file_path, e);
                }
            }
        }

        let result = serde_json::to_string_pretty(&all_usages).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize result: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Find import statements for a symbol. Use to understand module dependencies and where a symbol is imported from.")]
    async fn find_imports(
        &self,
        Parameters(ImportsParams { symbol, exclude_dirs }): Parameters<ImportsParams>,
    ) -> Result<CallToolResult, McpError> {
        let files = self.get_typescript_files().await?;
        let config = self.config.read().await;

        let mut parser = TypeScriptParser::new().map_err(|e| {
            McpError::internal_error(format!("Failed to create parser: {}", e), None)
        })?;

        let mut all_usages: Vec<SymbolUsage> = Vec::new();

        for file_path in files {
            if Self::should_exclude_path(&file_path, &config, &exclude_dirs) {
                continue;
            }

            // contexts なし、import を含める
            match find_usages_in_file(&mut parser, &file_path, &symbol, true, 0, None) {
                Ok(usages) => {
                    // Import のみをフィルタ
                    let imports: Vec<_> = usages
                        .into_iter()
                        .filter(|u| u.usage_kind == UsageKind::Import)
                        .collect();
                    all_usages.extend(imports);
                }
                Err(e) => {
                    tracing::warn!("Failed to parse {:?}: {}", file_path, e);
                }
            }
        }

        let result = serde_json::to_string_pretty(&all_usages).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize result: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Search for text within comments in TypeScript/TSX files. Use to find TODOs, FIXMEs, or any text in comments.")]
    async fn find_in_comments(
        &self,
        Parameters(CommentSearchParams { text, exclude_dirs }): Parameters<CommentSearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let files = self.get_typescript_files().await?;
        let config = self.config.read().await;

        let mut all_matches: Vec<CommentMatch> = Vec::new();

        for file_path in files {
            if Self::should_exclude_path(&file_path, &config, &exclude_dirs) {
                continue;
            }

            match find_comments_in_file(&file_path, &text) {
                Ok(matches) => {
                    all_matches.extend(matches);
                }
                Err(e) => {
                    tracing::warn!("Failed to search comments in {:?}: {}", file_path, e);
                }
            }
        }

        let result = serde_json::to_string_pretty(&all_matches).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize result: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Get code at a specific file location. Use after find_in_comments or symbol_usages to see the actual code around a specific line.")]
    async fn get_code_at_location(
        &self,
        Parameters(CodeAtLocationParams { file_path, line, context_before, context_after }): Parameters<CodeAtLocationParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = PathBuf::from(&file_path);
        let before = context_before.unwrap_or(3);
        let after = context_after.unwrap_or(3);

        let snippet = get_code_at_location(&path, line, before, after)
            .map_err(|e| McpError::internal_error(format!("Failed to read code: {}", e), None))?;

        let result = serde_json::to_string_pretty(&snippet)
            .map_err(|e| McpError::internal_error(format!("Failed to serialize: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(result)]))
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
