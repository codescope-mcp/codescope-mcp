use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use tree_sitter::{Parser, Query, Tree};

use crate::language::{LanguageId, LanguageRegistry, LanguageSupport, SymbolKindMapping};

/// Generic parser that works with any registered language
///
/// Uses LanguageRegistry to dynamically select the appropriate
/// language handler based on file extension.
pub struct GenericParser {
    parser: Parser,
    registry: Arc<LanguageRegistry>,
    current_language: Option<LanguageId>,
}

impl GenericParser {
    /// Create a new generic parser
    pub fn new(registry: Arc<LanguageRegistry>) -> Result<Self> {
        let parser = Parser::new();

        Ok(Self {
            parser,
            registry,
            current_language: None,
        })
    }

    /// Set the language for the given file path
    fn set_language_for_file(&mut self, path: &Path) -> Result<&Arc<dyn LanguageSupport>> {
        let language = self
            .registry
            .get_for_path(path)
            .context(format!("Unsupported file type: {:?}", path))?;

        let lang_id = language.id();

        if self.current_language != Some(lang_id) {
            self.parser.set_language(language.tree_sitter_language())?;
            self.current_language = Some(lang_id);
        }

        Ok(language)
    }

    /// Parse source code and return the AST
    pub fn parse(&mut self, path: &Path, source_code: &str) -> Result<Tree> {
        self.set_language_for_file(path)?;

        self.parser
            .parse(source_code, None)
            .context("Failed to parse source code")
    }

    /// Parse source code and return both the tree and language
    pub fn parse_with_language(
        &mut self,
        path: &Path,
        source_code: &str,
    ) -> Result<(Tree, Arc<dyn LanguageSupport>)> {
        let language = self.set_language_for_file(path)?.clone();

        let tree = self
            .parser
            .parse(source_code, None)
            .context("Failed to parse source code")?;

        Ok((tree, language))
    }

    /// Get the definitions query for the current language
    pub fn definitions_query(&self, path: &Path) -> Option<&Query> {
        self.registry
            .get_for_path(path)
            .map(|lang| lang.definitions_query())
    }

    /// Get the usages query for the current language
    pub fn usages_query(&self, path: &Path) -> Option<&Query> {
        self.registry
            .get_for_path(path)
            .map(|lang| lang.usages_query())
    }

    /// Get the definition mappings for the current language
    pub fn definition_mappings(&self, path: &Path) -> Option<&[SymbolKindMapping]> {
        self.registry
            .get_for_path(path)
            .map(|lang| lang.definition_mappings())
    }

    /// Check if a file is supported
    pub fn is_supported(&self, path: &Path) -> bool {
        self.registry.is_supported(path)
    }

    /// Get the language registry
    pub fn registry(&self) -> &Arc<LanguageRegistry> {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_generic_parser_creation() {
        let registry = Arc::new(LanguageRegistry::new().unwrap());
        let parser = GenericParser::new(registry);
        assert!(parser.is_ok());
    }

    #[test]
    fn test_parse_typescript() {
        let registry = Arc::new(LanguageRegistry::new().unwrap());
        let mut parser = GenericParser::new(registry).unwrap();

        let path = PathBuf::from("test.ts");
        let source = "const x = 1;";

        let tree = parser.parse(&path, source);
        assert!(tree.is_ok());
    }

    #[test]
    fn test_parse_tsx() {
        let registry = Arc::new(LanguageRegistry::new().unwrap());
        let mut parser = GenericParser::new(registry).unwrap();

        let path = PathBuf::from("test.tsx");
        let source = "const App = () => <div>Hello</div>;";

        let tree = parser.parse(&path, source);
        assert!(tree.is_ok());
    }

    #[test]
    fn test_unsupported_file() {
        let registry = Arc::new(LanguageRegistry::new().unwrap());
        let mut parser = GenericParser::new(registry).unwrap();

        let path = PathBuf::from("test.py");
        let source = "x = 1";

        let tree = parser.parse(&path, source);
        assert!(tree.is_err());
    }
}
