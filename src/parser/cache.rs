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
    pub fn get(&self, path: &PathBuf) -> Option<Tree> {
        let entry = self.cache.get(path)?;

        // Check if file has been modified
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                if modified == entry.modified_time {
                    return Some(entry.tree.clone());
                }
            }
        }

        None
    }

    /// Store a parsed tree in the cache
    pub fn insert(&self, path: PathBuf, tree: Tree) {
        if let Ok(metadata) = std::fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                self.cache.insert(
                    path,
                    CacheEntry {
                        tree,
                        modified_time: modified,
                    },
                );
            }
        }
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
}

impl Default for ParserCache {
    fn default() -> Self {
        Self::new()
    }
}
