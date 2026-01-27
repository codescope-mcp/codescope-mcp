use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};

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
    }

    #[test]
    fn test_get_by_extension() {
        let registry = LanguageRegistry::new().unwrap();

        let ts_lang = registry.get_by_extension("ts").unwrap();
        assert_eq!(ts_lang.id(), LanguageId::TypeScript);

        let tsx_lang = registry.get_by_extension("tsx").unwrap();
        assert_eq!(tsx_lang.id(), LanguageId::TypeScriptReact);
    }

    #[test]
    fn test_get_for_path() {
        let registry = LanguageRegistry::new().unwrap();

        let ts_path = PathBuf::from("src/index.ts");
        assert!(registry.get_for_path(&ts_path).is_some());

        let tsx_path = PathBuf::from("src/App.tsx");
        assert!(registry.get_for_path(&tsx_path).is_some());

        let unsupported_path = PathBuf::from("README.md");
        assert!(registry.get_for_path(&unsupported_path).is_none());
    }

    #[test]
    fn test_is_supported() {
        let registry = LanguageRegistry::new().unwrap();

        assert!(registry.is_supported(Path::new("test.ts")));
        assert!(registry.is_supported(Path::new("test.tsx")));
        assert!(!registry.is_supported(Path::new("test.py")));
        assert!(!registry.is_supported(Path::new("test.rs")));
    }
}
