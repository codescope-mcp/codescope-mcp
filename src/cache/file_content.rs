use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use anyhow::{Context, Result};
use dashmap::DashMap;

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
    pub fn get_or_read(&self, path: &Path) -> Result<Arc<String>> {
        let path_buf = path.to_path_buf();

        // Check if we have a valid cached entry
        if let Some(entry) = self.cache.get(&path_buf) {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    if modified == entry.modified_time {
                        return Ok(entry.content.clone());
                    }
                }
            }
        }

        // Read file and cache
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        let modified_time = std::fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or_else(SystemTime::now);

        let content = Arc::new(content);
        self.cache.insert(
            path_buf,
            FileContentEntry {
                content: content.clone(),
                modified_time,
            },
        );

        Ok(content)
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
        let content1 = cache.get_or_read(path).unwrap();
        assert!(content1.contains("test content"));

        // Second read should use cache
        let content2 = cache.get_or_read(path).unwrap();
        assert_eq!(content1, content2);

        // Both should be the same Arc instance
        assert!(Arc::ptr_eq(&content1, &content2));
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
}
