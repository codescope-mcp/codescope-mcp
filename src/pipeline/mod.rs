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
    config: CodeScopeConfig,
    additional_excludes: Option<Vec<String>>,
}

impl FilePipeline {
    pub fn new(
        registry: Arc<LanguageRegistry>,
        workspace_root: PathBuf,
        config: CodeScopeConfig,
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
    ///
    /// # Implementation Note
    ///
    /// Each parallel thread creates its own `GenericParser` instance because
    /// tree-sitter's `Parser` is not thread-safe and requires mutable access
    /// during parsing. This is a deliberate tradeoff: while creating multiple
    /// parser instances has some overhead, it enables parallel file processing
    /// which significantly improves performance for large codebases.
    ///
    /// The `LanguageRegistry` is shared via `Arc` to avoid duplicating the
    /// compiled queries and language grammars across threads.
    pub fn process<C, T>(&self, collector: &C) -> Vec<T>
    where
        C: ResultCollector<Item = T> + Sync,
        T: Send,
    {
        let files = self.get_files();

        files
            .par_iter()
            .filter_map(|file_path| {
                // Each thread needs its own Parser instance because tree-sitter's
                // Parser requires mutable access and is not thread-safe.
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
