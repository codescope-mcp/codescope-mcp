use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Symbol kind enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum SymbolKind {
    // TypeScript/TSX
    Function,
    Class,
    Method,
    Interface,
    Enum,
    Variable,
    ArrowFunction,
    Constructor,
    TypeAlias,
    // Markdown
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
    CodeBlock,
    Link,
    // HTML
    HtmlElement,
    HtmlId,
    HtmlClass,
    // CSS
    CssClassSelector,
    CssIdSelector,
    CssVariable,
    CssKeyframes,
    // Rust
    Struct,
    Trait,
    Impl,
    Module,
    Const,
    Static,
    Macro,
    // SQL
    Table,
    View,
    Procedure,
    Index,
    Trigger,
    Column,
}

impl std::fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolKind::Function => write!(f, "Function"),
            SymbolKind::Class => write!(f, "Class"),
            SymbolKind::Method => write!(f, "Method"),
            SymbolKind::Interface => write!(f, "Interface"),
            SymbolKind::Enum => write!(f, "Enum"),
            SymbolKind::Variable => write!(f, "Variable"),
            SymbolKind::ArrowFunction => write!(f, "ArrowFunction"),
            SymbolKind::Constructor => write!(f, "Constructor"),
            SymbolKind::TypeAlias => write!(f, "TypeAlias"),
            SymbolKind::Heading1 => write!(f, "Heading1"),
            SymbolKind::Heading2 => write!(f, "Heading2"),
            SymbolKind::Heading3 => write!(f, "Heading3"),
            SymbolKind::Heading4 => write!(f, "Heading4"),
            SymbolKind::Heading5 => write!(f, "Heading5"),
            SymbolKind::Heading6 => write!(f, "Heading6"),
            SymbolKind::CodeBlock => write!(f, "CodeBlock"),
            SymbolKind::Link => write!(f, "Link"),
            SymbolKind::HtmlElement => write!(f, "HtmlElement"),
            SymbolKind::HtmlId => write!(f, "HtmlId"),
            SymbolKind::HtmlClass => write!(f, "HtmlClass"),
            SymbolKind::CssClassSelector => write!(f, "CssClassSelector"),
            SymbolKind::CssIdSelector => write!(f, "CssIdSelector"),
            SymbolKind::CssVariable => write!(f, "CssVariable"),
            SymbolKind::CssKeyframes => write!(f, "CssKeyframes"),
            SymbolKind::Struct => write!(f, "Struct"),
            SymbolKind::Trait => write!(f, "Trait"),
            SymbolKind::Impl => write!(f, "Impl"),
            SymbolKind::Module => write!(f, "Module"),
            SymbolKind::Const => write!(f, "Const"),
            SymbolKind::Static => write!(f, "Static"),
            SymbolKind::Macro => write!(f, "Macro"),
            SymbolKind::Table => write!(f, "Table"),
            SymbolKind::View => write!(f, "View"),
            SymbolKind::Procedure => write!(f, "Procedure"),
            SymbolKind::Index => write!(f, "Index"),
            SymbolKind::Trigger => write!(f, "Trigger"),
            SymbolKind::Column => write!(f, "Column"),
        }
    }
}

/// Symbol definition information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SymbolDefinition {
    /// File path where the symbol is defined
    pub file_path: String,
    /// Starting line number (1-indexed)
    pub start_line: usize,
    /// Ending line number (1-indexed)
    pub end_line: usize,
    /// Symbol kind
    pub node_kind: SymbolKind,
    /// Source code of the symbol definition
    pub code: String,
    /// Symbol name
    pub name: String,
    /// JSDoc or comment above the definition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
}

/// Usage kind enumeration - describes how a symbol is used
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum UsageKind {
    /// Standalone identifier: foo
    Identifier,
    /// Property access: obj.foo
    PropertyAccess,
    /// Method call: obj.foo()
    MethodCall,
    /// Type reference: let x: Foo
    TypeReference,
    /// Import statement: import { foo }
    Import,
}

impl std::fmt::Display for UsageKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UsageKind::Identifier => write!(f, "Identifier"),
            UsageKind::PropertyAccess => write!(f, "PropertyAccess"),
            UsageKind::MethodCall => write!(f, "MethodCall"),
            UsageKind::TypeReference => write!(f, "TypeReference"),
            UsageKind::Import => write!(f, "Import"),
        }
    }
}

/// Context kind for usage locations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum ContextKind {
    ArrowFunction,
    FunctionDeclaration,
    MethodDeclaration,
    Constructor,
    ClassDeclaration,
    InterfaceDeclaration,
    EnumDeclaration,
    SourceFile,
}

impl std::fmt::Display for ContextKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContextKind::ArrowFunction => write!(f, "ArrowFunction"),
            ContextKind::FunctionDeclaration => write!(f, "FunctionDeclaration"),
            ContextKind::MethodDeclaration => write!(f, "MethodDeclaration"),
            ContextKind::Constructor => write!(f, "Constructor"),
            ContextKind::ClassDeclaration => write!(f, "ClassDeclaration"),
            ContextKind::InterfaceDeclaration => write!(f, "InterfaceDeclaration"),
            ContextKind::EnumDeclaration => write!(f, "EnumDeclaration"),
            ContextKind::SourceFile => write!(f, "SourceFile"),
        }
    }
}

/// Context information for a usage location
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UsageContext {
    /// Context kind
    pub kind: ContextKind,
    /// Name of the context (e.g., function name, class name)
    pub name: Option<String>,
    /// Starting line number (1-indexed)
    pub start_line: usize,
    /// Ending line number (1-indexed)
    pub end_line: usize,
}

/// Symbol usage information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SymbolUsage {
    /// File path where the symbol is used
    pub file_path: String,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (0-indexed)
    pub column: usize,
    /// Fully qualified name (e.g., "Date.now" for Date.now(), "now" for now())
    pub qualified_name: String,
    /// How the symbol is used (e.g., MethodCall, PropertyAccess)
    pub usage_kind: UsageKind,
    /// Object name for member access (e.g., "Date" for Date.now())
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_name: Option<String>,
    /// Context hierarchy from innermost to outermost
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub contexts: Vec<UsageContext>,
}

/// Comment type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum CommentType {
    /// Single line comment: //
    SingleLine,
    /// Block comment: /* */
    Block,
}

impl std::fmt::Display for CommentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommentType::SingleLine => write!(f, "SingleLine"),
            CommentType::Block => write!(f, "Block"),
        }
    }
}

/// Comment match result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CommentMatch {
    /// File path where the comment is found
    pub file_path: String,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (0-indexed)
    pub column: usize,
    /// Comment type
    pub comment_type: CommentType,
    /// Comment content
    pub content: String,
}

/// Code snippet at a specific location
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CodeSnippet {
    /// File path
    pub file_path: String,
    /// Starting line number (1-indexed)
    pub start_line: usize,
    /// Ending line number (1-indexed)
    pub end_line: usize,
    /// Code content
    pub code: String,
}
