use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use anyhow::{Context, Result};
use dashmap::DashMap;

/// Cached file content with its modification time
#[derive(Clone)]
pub struct CachedContent {
    pub content: Arc<String>,
    pub modified_time: SystemTime,
}

/// Cache entry for file content
struct FileContentEntry {
    content: Arc<String>,
    modified_time: SystemTime,
}

/// File content cache for avoiding redundant file reads
///
/// This cache stores file contents keyed by path, and automatically
/// invalidates entries when file modification times change.
pub struct FileContentCache {
    cache: DashMap<PathBuf, FileContentEntry>,
}

impl FileContentCache {
    /// Create a new file content cache
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
        }
    }

    /// Get file content from cache or read from disk
    ///
    /// If the file is in cache and hasn't been modified, returns the cached content.
    /// Otherwise, reads the file and caches the result.
    ///
    /// Returns both the content and modification time to allow callers to pass
    /// the mtime to other caches for consistency.
    pub fn get_or_read(&self, path: &Path) -> Result<CachedContent> {
        let path_buf = path.to_path_buf();

        // Check if we have a valid cached entry
        if let Some(entry) = self.cache.get(&path_buf) {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    if modified == entry.modified_time {
                        return Ok(CachedContent {
                            content: entry.content.clone(),
                            modified_time: entry.modified_time,
                        });
                    }
                }
            }
            // Entry is stale - remove it to prevent memory bloat
            drop(entry); // Release the lock before removing
            self.cache.remove(&path_buf);
        }

        // Get metadata (including modification time) BEFORE reading to minimize TOCTOU window
        // Note: There's still a small window between stat and read, but this is the best
        // we can do without platform-specific atomic APIs. By getting mtime first,
        // if the file changes during read, we'll have an older mtime and the cache
        // will be invalidated on next access (fail-safe behavior).
        let modified_time = std::fs::metadata(path)
            .and_then(|m| m.modified())
            .unwrap_or_else(|_| SystemTime::now());

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        let content = Arc::new(content);
        self.cache.insert(
            path_buf,
            FileContentEntry {
                content: content.clone(),
                modified_time,
            },
        );

        Ok(CachedContent {
            content,
            modified_time,
        })
    }

    /// Clear the cache
    #[allow(dead_code)]
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Remove a specific entry from the cache
    #[allow(dead_code)]
    pub fn invalidate(&self, path: &Path) {
        self.cache.remove(&path.to_path_buf());
    }
}

impl Default for FileContentCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_content_cache_creation() {
        let cache = FileContentCache::new();
        assert!(cache.cache.is_empty());
    }

    #[test]
    fn test_file_content_cache_read() {
        let cache = FileContentCache::new();

        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test content").unwrap();
        let path = temp_file.path();

        // First read
        let result1 = cache.get_or_read(path).unwrap();
        assert!(result1.content.contains("test content"));

        // Second read should use cache
        let result2 = cache.get_or_read(path).unwrap();
        assert_eq!(result1.content, result2.content);

        // Both should be the same Arc instance
        assert!(Arc::ptr_eq(&result1.content, &result2.content));

        // Modification times should match
        assert_eq!(result1.modified_time, result2.modified_time);
    }

    #[test]
    fn test_file_content_cache_returns_mtime() {
        let cache = FileContentCache::new();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test content").unwrap();
        let path = temp_file.path();

        let result = cache.get_or_read(path).unwrap();

        // Verify that mtime is reasonable (not the fallback SystemTime::now())
        let actual_mtime = std::fs::metadata(path).unwrap().modified().unwrap();
        assert_eq!(result.modified_time, actual_mtime);
    }

    #[test]
    fn test_file_content_cache_invalidate() {
        let cache = FileContentCache::new();

        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test content").unwrap();
        let path = temp_file.path().to_path_buf();

        // Read and cache
        let _ = cache.get_or_read(&path).unwrap();
        assert!(!cache.cache.is_empty());

        // Invalidate
        cache.invalidate(&path);
        assert!(cache.cache.is_empty());
    }

    #[test]
    fn test_file_not_found() {
        let cache = FileContentCache::new();
        let result = cache.get_or_read(Path::new("/nonexistent/file.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_stale_entry_removed_on_access() {
        let cache = FileContentCache::new();

        // Create a temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        // Manually insert a stale entry with an old mtime
        let old_mtime = SystemTime::UNIX_EPOCH;
        cache.cache.insert(
            path.clone(),
            FileContentEntry {
                content: Arc::new("old content".to_string()),
                modified_time: old_mtime,
            },
        );

        // Access should detect staleness and remove the entry
        let result = cache.get_or_read(&path).unwrap();

        // Should get fresh content, not "old content"
        assert!(!result.content.contains("old content"));
    }
}
