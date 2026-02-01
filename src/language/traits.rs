use tree_sitter::{Language, Query};

use crate::symbol::types::SymbolKind;

/// Unique identifier for a language
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LanguageId {
    #[default]
    TypeScript,
    TypeScriptReact,
    JavaScript,
    JavaScriptReact,
    Markdown,
    Html,
    Css,
    Python,
    Rust,
    Go,
    Java,
    Sql,
}

impl std::fmt::Display for LanguageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LanguageId::TypeScript => write!(f, "TypeScript"),
            LanguageId::TypeScriptReact => write!(f, "TypeScriptReact"),
            LanguageId::JavaScript => write!(f, "JavaScript"),
            LanguageId::JavaScriptReact => write!(f, "JavaScriptReact"),
            LanguageId::Markdown => write!(f, "Markdown"),
            LanguageId::Html => write!(f, "Html"),
            LanguageId::Css => write!(f, "Css"),
            LanguageId::Python => write!(f, "Python"),
            LanguageId::Rust => write!(f, "Rust"),
            LanguageId::Go => write!(f, "Go"),
            LanguageId::Java => write!(f, "Java"),
            LanguageId::Sql => write!(f, "Sql"),
        }
    }
}

/// Mapping from tree-sitter capture name to SymbolKind
#[derive(Debug, Clone)]
pub struct SymbolKindMapping {
    pub capture_name: &'static str,
    pub kind: SymbolKind,
}

/// Trait for language support implementations
///
/// Each language (TypeScript, Python, Rust, etc.) implements this trait
/// to provide language-specific parsing capabilities.
pub trait LanguageSupport: Send + Sync {
    /// Get the unique identifier for this language
    fn id(&self) -> LanguageId;

    /// Get the display name of this language
    fn name(&self) -> &'static str;

    /// Get the file extensions supported by this language (e.g., ["ts", "tsx"])
    fn file_extensions(&self) -> &[&'static str];

    /// Get the tree-sitter Language grammar
    fn tree_sitter_language(&self) -> &Language;

    /// Get the query for finding symbol definitions
    fn definitions_query(&self) -> &Query;

    /// Get the query for finding symbol usages
    fn usages_query(&self) -> &Query;

    /// Get the mappings from capture names to symbol kinds
    fn definition_mappings(&self) -> &[SymbolKindMapping];

    /// Whether this language uses separate documentation statements (e.g., SQL COMMENT ON).
    ///
    /// If true, documentation is extracted from separate statements in the file
    /// rather than from comments immediately before the definition.
    fn uses_separate_docs(&self) -> bool {
        false
    }
}
