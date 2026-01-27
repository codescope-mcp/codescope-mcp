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
use crate::language::{LanguageId, LanguageRegistry};
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
    language_filter: Option<LanguageId>,
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
            language_filter: None,
        }
    }

    pub fn with_excludes(mut self, excludes: Option<Vec<String>>) -> Self {
        self.additional_excludes = excludes;
        self
    }

    /// Filter files by language
    ///
    /// If set, only files matching the specified language will be processed.
    pub fn with_language_filter(mut self, language: Option<String>) -> Self {
        self.language_filter = language.and_then(|lang| parse_language_id(&lang));
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
                if !self
                    .config
                    .should_exclude(path, self.additional_excludes.as_deref())
                {
                    // Check language filter
                    if let Some(filter_lang) = self.language_filter {
                        if let Some(lang) = self.registry.get_for_path(path) {
                            if lang.id() != filter_lang {
                                continue;
                            }
                        }
                    }
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

/// Parse a language name string to LanguageId
fn parse_language_id(name: &str) -> Option<LanguageId> {
    match name.to_lowercase().as_str() {
        "typescript" | "ts" => Some(LanguageId::TypeScript),
        "typescriptreact" | "tsx" => Some(LanguageId::TypeScriptReact),
        "markdown" | "md" => Some(LanguageId::Markdown),
        _ => None,
    }
}
