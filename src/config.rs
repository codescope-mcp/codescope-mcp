use serde::Deserialize;
use std::path::Path;

use glob::Pattern;

/// Default directories to exclude from search
const DEFAULT_EXCLUDE_DIRS: &[&str] = &["dist", "build", ".next", "out", "coverage"];

/// Cache configuration
#[derive(Debug, Clone, Deserialize)]
pub struct CacheConfig {
    /// Whether caching is enabled
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,
    /// Maximum number of entries in the cache
    #[serde(default = "default_cache_max_size")]
    pub max_size: usize,
    /// Time-to-live for cache entries in seconds
    #[serde(default = "default_cache_ttl")]
    pub ttl_seconds: u64,
}

fn default_cache_enabled() -> bool {
    true
}

fn default_cache_max_size() -> usize {
    1000
}

fn default_cache_ttl() -> u64 {
    300 // 5 minutes
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: default_cache_enabled(),
            max_size: default_cache_max_size(),
            ttl_seconds: default_cache_ttl(),
        }
    }
}

/// CodeScope configuration
#[derive(Debug, Default, Clone, Deserialize)]
pub struct CodeScopeConfig {
    /// Directories to exclude from search
    #[serde(default)]
    pub exclude_dirs: Vec<String>,

    /// Glob patterns to exclude from search (e.g., "**/*.test.ts", "**/__mocks__/**")
    #[serde(default)]
    pub exclude_patterns: Vec<String>,

    /// Enabled languages (if empty, all registered languages are enabled)
    /// e.g., ["typescript", "tsx", "python"]
    #[serde(default)]
    pub enabled_languages: Vec<String>,

    /// Cache configuration
    #[serde(default)]
    pub cache: CacheConfig,

    /// Maximum number of results to return (None = unlimited)
    #[serde(default)]
    pub max_results: Option<usize>,
}

impl CodeScopeConfig {
    /// Load configuration from .mcp.json in the workspace root
    /// Falls back to default configuration if file doesn't exist or is invalid
    pub fn load(workspace_root: &Path) -> Self {
        let config_path = workspace_root.join(".mcp.json");
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str::<CodeScopeConfig>(&content) {
                    tracing::info!("Loaded config from .mcp.json: {:?}", config);
                    return config;
                } else {
                    tracing::warn!("Failed to parse .mcp.json, using defaults");
                }
            }
        }
        Self::default_config()
    }

    /// Get the default configuration
    pub fn default_config() -> Self {
        Self {
            exclude_dirs: DEFAULT_EXCLUDE_DIRS.iter().map(|s| s.to_string()).collect(),
            exclude_patterns: Vec::new(),
            enabled_languages: Vec::new(), // Empty = all languages
            cache: CacheConfig::default(),
            max_results: None,
        }
    }

    /// Check if a path should be excluded based on config and additional excludes
    pub fn should_exclude(&self, path: &Path, additional_excludes: Option<&[String]>) -> bool {
        let path_str = path.to_string_lossy();

        // Check config exclude_dirs
        for dir in &self.exclude_dirs {
            if path_str.contains(&format!("/{}/", dir))
                || path_str.contains(&format!("\\{}\\", dir))
            {
                return true;
            }
        }

        // Check additional user-specified excludes
        if let Some(dirs) = additional_excludes {
            for dir in dirs {
                if path_str.contains(&format!("/{}/", dir))
                    || path_str.contains(&format!("\\{}\\", dir))
                {
                    return true;
                }
            }
        }

        // Check glob patterns
        for pattern_str in &self.exclude_patterns {
            if let Ok(pattern) = Pattern::new(pattern_str) {
                if pattern.matches(&path_str) {
                    return true;
                }
            }
        }

        false
    }

    /// Check if a language extension is enabled
    pub fn is_language_enabled(&self, extension: &str) -> bool {
        if self.enabled_languages.is_empty() {
            return true; // All languages enabled by default
        }
        self.enabled_languages.iter().any(|lang| {
            lang.eq_ignore_ascii_case(extension)
                || (lang.eq_ignore_ascii_case("typescript") && extension == "ts")
                || (lang.eq_ignore_ascii_case("tsx") && extension == "tsx")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_default_config() {
        let config = CodeScopeConfig::default_config();
        assert!(config.exclude_dirs.contains(&"dist".to_string()));
        assert!(config.exclude_dirs.contains(&"build".to_string()));
        assert!(config.enabled_languages.is_empty());
        assert!(config.cache.enabled);
    }

    #[test]
    fn test_should_exclude() {
        let config = CodeScopeConfig::default_config();

        // Should exclude dist directory
        let path = PathBuf::from("/project/dist/index.js");
        assert!(config.should_exclude(&path, None));

        // Should not exclude src directory
        let path = PathBuf::from("/project/src/index.ts");
        assert!(!config.should_exclude(&path, None));

        // Should exclude with additional excludes
        let path = PathBuf::from("/project/test/sample.ts");
        assert!(config.should_exclude(&path, Some(&["test".to_string()])));
    }

    #[test]
    fn test_exclude_patterns() {
        let config = CodeScopeConfig {
            exclude_patterns: vec!["**/*.test.ts".to_string(), "**/__mocks__/**".to_string()],
            ..Default::default()
        };

        // Should exclude test files
        let test_path = PathBuf::from("/project/src/utils.test.ts");
        assert!(config.should_exclude(&test_path, None));

        // Should exclude mock directories
        let mock_path = PathBuf::from("/project/src/__mocks__/api.ts");
        assert!(config.should_exclude(&mock_path, None));

        // Should not exclude regular files
        let regular_path = PathBuf::from("/project/src/utils.ts");
        assert!(!config.should_exclude(&regular_path, None));
    }

    #[test]
    fn test_is_language_enabled() {
        // Empty enabled_languages = all enabled
        let config = CodeScopeConfig::default_config();
        assert!(config.is_language_enabled("ts"));
        assert!(config.is_language_enabled("tsx"));

        // Specific languages enabled
        let config = CodeScopeConfig {
            enabled_languages: vec!["typescript".to_string()],
            ..Default::default()
        };
        assert!(config.is_language_enabled("ts"));
        assert!(!config.is_language_enabled("tsx"));
    }

    #[test]
    fn test_cache_config_defaults() {
        let cache = CacheConfig::default();
        assert!(cache.enabled);
        assert_eq!(cache.max_size, 1000);
        assert_eq!(cache.ttl_seconds, 300);
    }
}
