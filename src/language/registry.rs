use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};

use super::css::CssLanguage;
use super::html::HtmlLanguage;
use super::javascript::{JavaScriptLanguage, JavaScriptReactLanguage};
use super::markdown::MarkdownLanguage;
use super::python::PythonLanguage;
use super::traits::{LanguageId, LanguageSupport};
use super::typescript::{TypeScriptLanguage, TypeScriptReactLanguage};

/// Registry for language support implementations
///
/// Manages all supported languages and provides methods to look up
/// the appropriate language handler based on file extension.
pub struct LanguageRegistry {
    languages: HashMap<LanguageId, Arc<dyn LanguageSupport>>,
    extension_map: HashMap<&'static str, LanguageId>,
}

impl LanguageRegistry {
    /// Create a new language registry with default languages
    pub fn new() -> Result<Self> {
        let mut registry = Self {
            languages: HashMap::new(),
            extension_map: HashMap::new(),
        };

        // Register default languages
        registry.register(Arc::new(
            TypeScriptLanguage::new().context("Failed to create TypeScript language")?,
        ))?;
        registry.register(Arc::new(
            TypeScriptReactLanguage::new().context("Failed to create TSX language")?,
        ))?;
        registry.register(Arc::new(
            JavaScriptLanguage::new().context("Failed to create JavaScript language")?,
        ))?;
        registry.register(Arc::new(
            JavaScriptReactLanguage::new().context("Failed to create JSX language")?,
        ))?;
        registry.register(Arc::new(
            MarkdownLanguage::new().context("Failed to create Markdown language")?,
        ))?;
        registry.register(Arc::new(
            HtmlLanguage::new().context("Failed to create HTML language")?,
        ))?;
        registry.register(Arc::new(
            CssLanguage::new().context("Failed to create CSS language")?,
        ))?;
        registry.register(Arc::new(
            PythonLanguage::new().context("Failed to create Python language")?,
        ))?;

        Ok(registry)
    }

    /// Register a new language
    pub fn register(&mut self, language: Arc<dyn LanguageSupport>) -> Result<()> {
        let id = language.id();

        // Register extension mappings
        for ext in language.file_extensions() {
            self.extension_map.insert(ext, id);
        }

        self.languages.insert(id, language);
        Ok(())
    }

    /// Get a language by its ID
    pub fn get(&self, id: LanguageId) -> Option<&Arc<dyn LanguageSupport>> {
        self.languages.get(&id)
    }

    /// Get a language by file extension
    pub fn get_by_extension(&self, extension: &str) -> Option<&Arc<dyn LanguageSupport>> {
        let id = self.extension_map.get(extension)?;
        self.languages.get(id)
    }

    /// Get a language for a given file path
    pub fn get_for_path(&self, path: &Path) -> Option<&Arc<dyn LanguageSupport>> {
        let extension = path.extension()?.to_str()?;
        self.get_by_extension(extension)
    }

    /// Check if a file is supported by any registered language
    pub fn is_supported(&self, path: &Path) -> bool {
        self.get_for_path(path).is_some()
    }

    /// Get all supported file extensions
    pub fn supported_extensions(&self) -> Vec<&'static str> {
        self.extension_map.keys().copied().collect()
    }

    /// Get all registered language IDs
    pub fn registered_languages(&self) -> Vec<LanguageId> {
        self.languages.keys().copied().collect()
    }
}

// Note: LanguageRegistry intentionally does not implement Default because
// new() can fail if the embedded tree-sitter queries are malformed.
// While this is unlikely at runtime (queries are compile-time embedded),
// failing explicitly is preferable to silently returning an empty registry.

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_registry_creation() {
        let registry = LanguageRegistry::new().unwrap();
        assert!(registry.get(LanguageId::TypeScript).is_some());
        assert!(registry.get(LanguageId::TypeScriptReact).is_some());
        assert!(registry.get(LanguageId::JavaScript).is_some());
        assert!(registry.get(LanguageId::JavaScriptReact).is_some());
        assert!(registry.get(LanguageId::Markdown).is_some());
        assert!(registry.get(LanguageId::Html).is_some());
        assert!(registry.get(LanguageId::Css).is_some());
        assert!(registry.get(LanguageId::Python).is_some());
    }

    #[test]
    fn test_get_by_extension() {
        let registry = LanguageRegistry::new().unwrap();

        let ts_lang = registry.get_by_extension("ts").unwrap();
        assert_eq!(ts_lang.id(), LanguageId::TypeScript);

        let tsx_lang = registry.get_by_extension("tsx").unwrap();
        assert_eq!(tsx_lang.id(), LanguageId::TypeScriptReact);

        let js_lang = registry.get_by_extension("js").unwrap();
        assert_eq!(js_lang.id(), LanguageId::JavaScript);

        let mjs_lang = registry.get_by_extension("mjs").unwrap();
        assert_eq!(mjs_lang.id(), LanguageId::JavaScript);

        let cjs_lang = registry.get_by_extension("cjs").unwrap();
        assert_eq!(cjs_lang.id(), LanguageId::JavaScript);

        let jsx_lang = registry.get_by_extension("jsx").unwrap();
        assert_eq!(jsx_lang.id(), LanguageId::JavaScriptReact);

        let md_lang = registry.get_by_extension("md").unwrap();
        assert_eq!(md_lang.id(), LanguageId::Markdown);

        let mdc_lang = registry.get_by_extension("mdc").unwrap();
        assert_eq!(mdc_lang.id(), LanguageId::Markdown);

        let html_lang = registry.get_by_extension("html").unwrap();
        assert_eq!(html_lang.id(), LanguageId::Html);

        let htm_lang = registry.get_by_extension("htm").unwrap();
        assert_eq!(htm_lang.id(), LanguageId::Html);

        let css_lang = registry.get_by_extension("css").unwrap();
        assert_eq!(css_lang.id(), LanguageId::Css);

        let py_lang = registry.get_by_extension("py").unwrap();
        assert_eq!(py_lang.id(), LanguageId::Python);

        let pyi_lang = registry.get_by_extension("pyi").unwrap();
        assert_eq!(pyi_lang.id(), LanguageId::Python);
    }

    #[test]
    fn test_get_for_path() {
        let registry = LanguageRegistry::new().unwrap();

        let ts_path = PathBuf::from("src/index.ts");
        assert!(registry.get_for_path(&ts_path).is_some());

        let tsx_path = PathBuf::from("src/App.tsx");
        assert!(registry.get_for_path(&tsx_path).is_some());

        let js_path = PathBuf::from("src/index.js");
        assert!(registry.get_for_path(&js_path).is_some());

        let jsx_path = PathBuf::from("src/App.jsx");
        assert!(registry.get_for_path(&jsx_path).is_some());

        let md_path = PathBuf::from("README.md");
        assert!(registry.get_for_path(&md_path).is_some());

        let html_path = PathBuf::from("index.html");
        assert!(registry.get_for_path(&html_path).is_some());

        let css_path = PathBuf::from("styles.css");
        assert!(registry.get_for_path(&css_path).is_some());

        let py_path = PathBuf::from("script.py");
        assert!(registry.get_for_path(&py_path).is_some());

        let unsupported_path = PathBuf::from("script.rs");
        assert!(registry.get_for_path(&unsupported_path).is_none());
    }

    #[test]
    fn test_is_supported() {
        let registry = LanguageRegistry::new().unwrap();

        assert!(registry.is_supported(Path::new("test.ts")));
        assert!(registry.is_supported(Path::new("test.tsx")));
        assert!(registry.is_supported(Path::new("test.js")));
        assert!(registry.is_supported(Path::new("test.mjs")));
        assert!(registry.is_supported(Path::new("test.cjs")));
        assert!(registry.is_supported(Path::new("test.jsx")));
        assert!(registry.is_supported(Path::new("test.md")));
        assert!(registry.is_supported(Path::new("test.mdc")));
        assert!(registry.is_supported(Path::new("test.html")));
        assert!(registry.is_supported(Path::new("test.htm")));
        assert!(registry.is_supported(Path::new("test.css")));
        assert!(registry.is_supported(Path::new("test.py")));
        assert!(registry.is_supported(Path::new("test.pyi")));
        assert!(!registry.is_supported(Path::new("test.rs")));
    }
}
