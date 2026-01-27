mod css;
mod html;
mod javascript;
mod markdown;
mod registry;
mod traits;
mod typescript;

pub use css::CssLanguage;
pub use html::HtmlLanguage;
pub use javascript::{JavaScriptLanguage, JavaScriptReactLanguage};
pub use markdown::MarkdownLanguage;
pub use registry::LanguageRegistry;
pub use traits::{LanguageId, LanguageSupport, SymbolKindMapping};
pub use typescript::{TypeScriptLanguage, TypeScriptReactLanguage};
