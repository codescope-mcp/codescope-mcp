mod collectors;

pub use collectors::{
    CommentCollector, DefinitionCollector, ImportCollector, MethodCallCollector, ResultCollector,
    UsageCollector,
};

use std::path::PathBuf;
use std::sync::Arc;

use ignore::WalkBuilder;
use rayon::prelude::*;

use crate::config::CodeScopeConfig;
use crate::language::LanguageRegistry;
use crate::parser::GenericParser;

/// File processing pipeline
///
/// Handles file discovery, filtering, and parallel processing
/// with a given collector.
pub struct FilePipeline {
    registry: Arc<LanguageRegistry>,
    workspace_root: PathBuf,
    config: Arc<CodeScopeConfig>,
    additional_excludes: Option<Vec<String>>,
}

impl FilePipeline {
    pub fn new(
        registry: Arc<LanguageRegistry>,
        workspace_root: PathBuf,
        config: Arc<CodeScopeConfig>,
    ) -> Self {
        Self {
            registry,
            workspace_root,
            config,
            additional_excludes: None,
        }
    }

    pub fn with_excludes(mut self, excludes: Option<Vec<String>>) -> Self {
        self.additional_excludes = excludes;
        self
    }

    /// Get all supported files in the workspace
    fn get_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let walker = WalkBuilder::new(&self.workspace_root)
            .hidden(true)
            .git_ignore(true)
            .build();

        for entry in walker.flatten() {
            let path = entry.path();
            if path.is_file() && self.registry.is_supported(path) {
                // Check exclusions
                if !self.config.should_exclude(
                    path,
                    self.additional_excludes.as_deref(),
                ) {
                    files.push(path.to_path_buf());
                }
            }
        }

        files
    }

    /// Process files with the given collector
    pub fn process<C, T>(&self, collector: &C) -> Vec<T>
    where
        C: ResultCollector<Item = T> + Sync,
        T: Send,
    {
        let files = self.get_files();

        files
            .par_iter()
            .filter_map(|file_path| {
                let mut parser = match GenericParser::new(self.registry.clone()) {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::warn!("Failed to create parser: {}", e);
                        return None;
                    }
                };

                match collector.process_file(&mut parser, file_path) {
                    Ok(items) => Some(items),
                    Err(e) => {
                        tracing::warn!("Failed to process {:?}: {}", file_path, e);
                        None
                    }
                }
            })
            .flatten()
            .collect()
    }
}
