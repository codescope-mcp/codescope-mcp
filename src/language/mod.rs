mod markdown;
mod registry;
mod traits;
mod typescript;

pub use markdown::MarkdownLanguage;
pub use registry::LanguageRegistry;
pub use traits::{LanguageId, LanguageSupport, SymbolKindMapping};
pub use typescript::{TypeScriptLanguage, TypeScriptReactLanguage};
