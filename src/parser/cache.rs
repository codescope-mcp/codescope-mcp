use std::path::PathBuf;
use std::time::SystemTime;

use dashmap::DashMap;
use tree_sitter::Tree;

/// Cache entry for parsed AST
struct CacheEntry {
    tree: Tree,
    modified_time: SystemTime,
}

/// Parser result cache for avoiding redundant parsing
pub struct ParserCache {
    cache: DashMap<PathBuf, CacheEntry>,
}

impl ParserCache {
    /// Create a new parser cache
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
        }
    }

    /// Get a cached tree if it's still valid
    ///
    /// Uses the provided modification time to check validity.
    /// If the cached entry's mtime doesn't match, the entry is removed
    /// to prevent memory bloat from stale entries.
    pub fn get(&self, path: &PathBuf, modified_time: SystemTime) -> Option<Tree> {
        if let Some(entry) = self.cache.get(path) {
            if entry.modified_time == modified_time {
                return Some(entry.tree.clone());
            }
            // Entry is stale - remove it to prevent memory bloat
            drop(entry); // Release the lock before removing
            self.cache.remove(path);
        }
        None
    }

    /// Store a parsed tree in the cache with the given modification time
    ///
    /// The modification time should be obtained from the same source as the
    /// file content to ensure cache consistency.
    pub fn insert(&self, path: PathBuf, tree: Tree, modified_time: SystemTime) {
        self.cache.insert(
            path,
            CacheEntry {
                tree,
                modified_time,
            },
        );
    }

    /// Clear the cache
    #[allow(dead_code)]
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Remove a specific entry from the cache
    #[allow(dead_code)]
    pub fn invalidate(&self, path: &PathBuf) {
        self.cache.remove(path);
    }

    /// Get the number of entries in the cache (for testing)
    #[cfg(test)]
    fn len(&self) -> usize {
        self.cache.len()
    }
}

impl Default for ParserCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Duration;

    use crate::language::LanguageRegistry;
    use crate::parser::GenericParser;

    fn create_test_tree() -> Tree {
        let registry = Arc::new(LanguageRegistry::new().unwrap());
        let mut parser = GenericParser::new(registry).unwrap();
        let path = PathBuf::from("test.ts");
        parser.parse(&path, "const x = 1;").unwrap()
    }

    #[test]
    fn test_parser_cache_creation() {
        let cache = ParserCache::new();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_parser_cache_insert_and_get() {
        let cache = ParserCache::new();
        let path = PathBuf::from("test.ts");
        let tree = create_test_tree();
        let mtime = SystemTime::now();

        // Insert
        cache.insert(path.clone(), tree, mtime);
        assert_eq!(cache.len(), 1);

        // Get with matching mtime should succeed
        let result = cache.get(&path, mtime);
        assert!(result.is_some());
    }

    #[test]
    fn test_parser_cache_miss_on_mtime_mismatch() {
        let cache = ParserCache::new();
        let path = PathBuf::from("test.ts");
        let tree = create_test_tree();
        let mtime = SystemTime::now();

        cache.insert(path.clone(), tree, mtime);

        // Get with different mtime should fail
        let different_mtime = mtime + Duration::from_secs(1);
        let result = cache.get(&path, different_mtime);
        assert!(result.is_none());
    }

    #[test]
    fn test_parser_cache_stale_entry_removed() {
        let cache = ParserCache::new();
        let path = PathBuf::from("test.ts");
        let tree = create_test_tree();
        let old_mtime = SystemTime::UNIX_EPOCH;

        cache.insert(path.clone(), tree, old_mtime);
        assert_eq!(cache.len(), 1);

        // Access with newer mtime should remove the stale entry
        let new_mtime = SystemTime::now();
        let result = cache.get(&path, new_mtime);
        assert!(result.is_none());
        assert_eq!(cache.len(), 0); // Entry should be removed
    }

    #[test]
    fn test_parser_cache_invalidate() {
        let cache = ParserCache::new();
        let path = PathBuf::from("test.ts");
        let tree = create_test_tree();
        let mtime = SystemTime::now();

        cache.insert(path.clone(), tree, mtime);
        assert_eq!(cache.len(), 1);

        cache.invalidate(&path);
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_parser_cache_clear() {
        let cache = ParserCache::new();
        let mtime = SystemTime::now();

        cache.insert(PathBuf::from("a.ts"), create_test_tree(), mtime);
        cache.insert(PathBuf::from("b.ts"), create_test_tree(), mtime);
        assert_eq!(cache.len(), 2);

        cache.clear();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_parser_cache_default() {
        let cache = ParserCache::default();
        assert_eq!(cache.len(), 0);
    }
}
