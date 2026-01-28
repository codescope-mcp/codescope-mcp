use std::path::Path;
use std::sync::Arc;

use super::file_content::FileContentCache;
use crate::parser::ParserCache;

/// Unified cache manager for all caching needs
///
/// Provides centralized access to both AST caching (ParserCache)
/// and file content caching (FileContentCache).
pub struct CacheManager {
    /// Cache for parsed AST trees
    pub parser_cache: Arc<ParserCache>,
    /// Cache for file contents
    pub file_cache: Arc<FileContentCache>,
}

impl CacheManager {
    /// Create a new cache manager
    pub fn new() -> Self {
        Self {
            parser_cache: Arc::new(ParserCache::new()),
            file_cache: Arc::new(FileContentCache::new()),
        }
    }

    /// Invalidate all caches for a specific file
    ///
    /// Call this when a file has been modified externally.
    #[allow(dead_code)]
    pub fn invalidate_file(&self, path: &Path) {
        let path_buf = path.to_path_buf();
        self.parser_cache.invalidate(&path_buf);
        self.file_cache.invalidate(path);
    }

    /// Clear all caches
    #[allow(dead_code)]
    pub fn clear(&self) {
        self.parser_cache.clear();
        self.file_cache.clear();
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_manager_creation() {
        let manager = CacheManager::new();
        assert!(Arc::strong_count(&manager.parser_cache) == 1);
        assert!(Arc::strong_count(&manager.file_cache) == 1);
    }

    #[test]
    fn test_cache_manager_default() {
        let manager = CacheManager::default();
        assert!(Arc::strong_count(&manager.parser_cache) == 1);
    }
}
