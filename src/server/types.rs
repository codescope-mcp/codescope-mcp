use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::symbol::types::{CommentMatch, SymbolDefinition, SymbolUsage};

/// Parameters for symbol_definition tool
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DefinitionParams {
    /// Symbol name to search for
    pub symbol: String,

    /// Include JSDoc/comments above the definition (default: false)
    pub include_docs: Option<bool>,

    /// Directories to exclude from search (e.g., ["dist", "node_modules"])
    pub exclude_dirs: Option<Vec<String>>,
}

/// Parameters for find_in_comments tool
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CommentSearchParams {
    /// Text to search for in comments
    pub text: String,

    /// Directories to exclude from search (e.g., ["dist", "node_modules"])
    pub exclude_dirs: Option<Vec<String>>,
}

/// Response for find_in_comments tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CommentSearchResponse {
    /// List of comment matches found
    pub matches: Vec<CommentMatch>,
}

/// Response for symbol.definition tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DefinitionResponse {
    /// List of symbol definitions found
    pub definitions: Vec<SymbolDefinition>,
}

/// Parameters for symbol_usages tool (simplified)
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UsagesParams {
    /// Symbol name to search for
    pub symbol: String,

    /// Whether to include context information (default: false)
    /// When true, includes up to 2 contexts per usage
    pub include_contexts: Option<bool>,

    /// Directories to exclude from search (e.g., ["dist", "node_modules"])
    pub exclude_dirs: Option<Vec<String>>,
}

/// Response for symbol.usages tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UsagesResponse {
    /// List of symbol usages found
    pub usages: Vec<SymbolUsage>,
}

/// Parameters for find_method_calls tool
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MethodCallsParams {
    /// Method name to search for (e.g., "now", "map")
    pub method_name: String,

    /// Filter by object name (e.g., "Date" for Date.now())
    pub object_name: Option<String>,

    /// Directories to exclude from search (e.g., ["dist", "node_modules"])
    pub exclude_dirs: Option<Vec<String>>,
}

/// Parameters for find_imports tool
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ImportsParams {
    /// Symbol name to search for
    pub symbol: String,

    /// Directories to exclude from search (e.g., ["dist", "node_modules"])
    pub exclude_dirs: Option<Vec<String>>,
}

/// Parameters for get_code_at_location tool
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CodeAtLocationParams {
    /// File path to read from
    pub file_path: String,

    /// Line number (1-indexed)
    pub line: usize,

    /// Lines to include before the target line (default: 3)
    pub context_before: Option<usize>,

    /// Lines to include after the target line (default: 3)
    pub context_after: Option<usize>,
}
