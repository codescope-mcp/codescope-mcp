use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;

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
///
/// Note: The modification time must be provided by the caller to ensure
/// consistency with file content caches. This prevents TOCTOU issues
/// where the file could change between reading content and checking mtime.
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
    /// First checks the cache for a valid AST using the provided modification time.
    /// If found, returns the cached tree. Otherwise, parses the source and caches
    /// the result.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path (used as cache key)
    /// * `source_code` - The source code to parse
    /// * `modified_time` - The file's modification time (should come from the same
    ///   source as the content to ensure consistency)
    pub fn parse(
        &mut self,
        path: &Path,
        source_code: &str,
        modified_time: SystemTime,
    ) -> Result<Tree> {
        let path_buf = path.to_path_buf();

        // Check cache first
        if let Some(cached_tree) = self.cache.get(&path_buf, modified_time) {
            return Ok(cached_tree);
        }

        // Parse and cache the result
        let tree = self.parser.parse(path, source_code)?;
        self.cache.insert(path_buf, tree.clone(), modified_time);

        Ok(tree)
    }

    /// Parse source code and return both the tree and language with caching
    ///
    /// This is the primary method for parsing that returns both the AST
    /// and the language support object for further processing.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path (used as cache key and for language detection)
    /// * `source_code` - The source code to parse
    /// * `modified_time` - The file's modification time (should come from the same
    ///   source as the content to ensure consistency)
    pub fn parse_with_language(
        &mut self,
        path: &Path,
        source_code: &str,
        modified_time: SystemTime,
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
        if let Some(cached_tree) = self.cache.get(&path_buf, modified_time) {
            return Ok((cached_tree, language.clone()));
        }

        // Parse and cache the result
        let (tree, lang) = self.parser.parse_with_language(path, source_code)?;
        self.cache.insert(path_buf, tree.clone(), modified_time);

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
    use std::time::Duration;

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
        let mtime = SystemTime::now();

        // First parse
        let tree1 = parser.parse(&path, source, mtime);
        assert!(tree1.is_ok());

        // Second parse should use cache
        let tree2 = parser.parse(&path, source, mtime);
        assert!(tree2.is_ok());
    }

    #[test]
    fn test_cached_parser_cache_miss_on_mtime_change() {
        let registry = Arc::new(LanguageRegistry::new().unwrap());
        let cache = Arc::new(ParserCache::new());
        let mut parser = CachedParser::new(registry, cache.clone()).unwrap();

        let path = PathBuf::from("test.ts");
        let source = "const x = 1;";
        let mtime1 = SystemTime::now();

        // First parse
        let _ = parser.parse(&path, source, mtime1).unwrap();

        // Second parse with different mtime should not use cache
        let mtime2 = mtime1 + Duration::from_secs(1);
        let tree2 = parser.parse(&path, source, mtime2);
        assert!(tree2.is_ok());
    }

    #[test]
    fn test_cached_parser_parse_with_language() {
        let registry = Arc::new(LanguageRegistry::new().unwrap());
        let cache = Arc::new(ParserCache::new());
        let mut parser = CachedParser::new(registry, cache).unwrap();

        let path = PathBuf::from("test.tsx");
        let source = "const App = () => <div>Hello</div>;";
        let mtime = SystemTime::now();

        let result = parser.parse_with_language(&path, source, mtime);
        assert!(result.is_ok());

        let (_, lang) = result.unwrap();
        assert_eq!(lang.name(), "TypeScriptReact");
    }

    #[test]
    fn test_cached_parser_unsupported_file() {
        let registry = Arc::new(LanguageRegistry::new().unwrap());
        let cache = Arc::new(ParserCache::new());
        let mut parser = CachedParser::new(registry, cache).unwrap();

        let path = PathBuf::from("test.xyz");
        let source = "some content";
        let mtime = SystemTime::now();

        let result = parser.parse_with_language(&path, source, mtime);
        assert!(result.is_err());
    }
}
