use serde::Deserialize;
use std::path::Path;

/// Default directories to exclude from search
const DEFAULT_EXCLUDE_DIRS: &[&str] = &["dist", "build", ".next", "out", "coverage"];

/// CodeScope configuration
#[derive(Debug, Default, Deserialize)]
pub struct CodeScopeConfig {
    /// Directories to exclude from search
    #[serde(default)]
    pub exclude_dirs: Vec<String>,
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
        // Return default configuration
        Self::default_config()
    }

    /// Get the default configuration
    pub fn default_config() -> Self {
        Self {
            exclude_dirs: DEFAULT_EXCLUDE_DIRS.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Check if a path should be excluded based on config and additional excludes
    pub fn should_exclude(&self, path: &Path, additional_excludes: Option<&[String]>) -> bool {
        let path_str = path.to_string_lossy();

        // Check config excludes
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

        false
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
}
