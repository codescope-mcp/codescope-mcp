use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use tree_sitter::Tree;

use super::cache::ParserCache;
use super::generic::GenericParser;
use crate::language::{LanguageRegistry, LanguageSupport};

/// Cached parser that wraps GenericParser with AST caching
///
/// This parser checks the cache before parsing and stores results
/// for subsequent requests. The cache is invalidated automatically
/// when file modification times change.
pub struct CachedParser {
    parser: GenericParser,
    cache: Arc<ParserCache>,
}

impl CachedParser {
    /// Create a new cached parser
    pub fn new(registry: Arc<LanguageRegistry>, cache: Arc<ParserCache>) -> Result<Self> {
        let parser = GenericParser::new(registry)?;
        Ok(Self { parser, cache })
    }

    /// Parse source code with caching
    ///
    /// First checks the cache for a valid AST. If found, returns the cached tree.
    /// Otherwise, parses the source and caches the result.
    pub fn parse(&mut self, path: &Path, source_code: &str) -> Result<Tree> {
        let path_buf = path.to_path_buf();

        // Check cache first
        if let Some(cached_tree) = self.cache.get(&path_buf) {
            return Ok(cached_tree);
        }

        // Parse and cache the result
        let tree = self.parser.parse(path, source_code)?;
        self.cache.insert(path_buf, tree.clone());

        Ok(tree)
    }

    /// Parse source code and return both the tree and language with caching
    ///
    /// This is the primary method for parsing that returns both the AST
    /// and the language support object for further processing.
    pub fn parse_with_language(
        &mut self,
        path: &Path,
        source_code: &str,
    ) -> Result<(Tree, Arc<dyn LanguageSupport>)> {
        let path_buf = path.to_path_buf();

        // Get language first (needed regardless of cache hit)
        let language = self.parser.registry().get_for_path(path).ok_or_else(|| {
            anyhow::anyhow!(
                "Unsupported file type for path '{}'. Supported extensions: {}",
                path.display(),
                self.parser.registry().supported_extensions().join(", ")
            )
        })?;

        // Check cache
        if let Some(cached_tree) = self.cache.get(&path_buf) {
            return Ok((cached_tree, language.clone()));
        }

        // Parse and cache the result
        let (tree, lang) = self.parser.parse_with_language(path, source_code)?;
        self.cache.insert(path_buf, tree.clone());

        Ok((tree, lang))
    }

    /// Check if a file is supported
    pub fn is_supported(&self, path: &Path) -> bool {
        self.parser.is_supported(path)
    }

    /// Get the language registry
    pub fn registry(&self) -> &Arc<LanguageRegistry> {
        self.parser.registry()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_cached_parser_creation() {
        let registry = Arc::new(LanguageRegistry::new().unwrap());
        let cache = Arc::new(ParserCache::new());
        let parser = CachedParser::new(registry, cache);
        assert!(parser.is_ok());
    }

    #[test]
    fn test_cached_parser_parse() {
        let registry = Arc::new(LanguageRegistry::new().unwrap());
        let cache = Arc::new(ParserCache::new());
        let mut parser = CachedParser::new(registry, cache.clone()).unwrap();

        let path = PathBuf::from("test.ts");
        let source = "const x = 1;";

        // First parse
        let tree1 = parser.parse(&path, source);
        assert!(tree1.is_ok());

        // Second parse should use cache (but we can't easily verify internally)
        let tree2 = parser.parse(&path, source);
        assert!(tree2.is_ok());
    }

    #[test]
    fn test_cached_parser_parse_with_language() {
        let registry = Arc::new(LanguageRegistry::new().unwrap());
        let cache = Arc::new(ParserCache::new());
        let mut parser = CachedParser::new(registry, cache).unwrap();

        let path = PathBuf::from("test.tsx");
        let source = "const App = () => <div>Hello</div>;";

        let result = parser.parse_with_language(&path, source);
        assert!(result.is_ok());

        let (_, lang) = result.unwrap();
        assert_eq!(lang.name(), "TypeScriptReact");
    }
}
