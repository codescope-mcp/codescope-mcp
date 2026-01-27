use std::path::Path;

use anyhow::{Context, Result};
use tree_sitter::{Language, Parser, Query, Tree};

/// TypeScript/TSX parser wrapper
pub struct TypeScriptParser {
    parser: Parser,
    ts_language: Language,
    tsx_language: Language,
    definitions_query_ts: Query,
    definitions_query_tsx: Query,
    usages_query_ts: Query,
    usages_query_tsx: Query,
    current_language: Option<LanguageType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LanguageType {
    TypeScript,
    TypeScriptReact,
}

impl TypeScriptParser {
    /// Create a new TypeScript parser
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();

        let ts_language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();
        let tsx_language = tree_sitter_typescript::LANGUAGE_TSX.into();

        // Load query files
        let definitions_query_src = include_str!("../../queries/typescript/definitions.scm");
        let usages_query_src = include_str!("../../queries/typescript/usages.scm");

        let definitions_query_ts = Query::new(&ts_language, definitions_query_src)
            .context("Failed to parse TS definitions query")?;
        let definitions_query_tsx = Query::new(&tsx_language, definitions_query_src)
            .context("Failed to parse TSX definitions query")?;
        let usages_query_ts = Query::new(&ts_language, usages_query_src)
            .context("Failed to parse TS usages query")?;
        let usages_query_tsx = Query::new(&tsx_language, usages_query_src)
            .context("Failed to parse TSX usages query")?;

        // Set default language
        parser.set_language(&ts_language)?;

        Ok(Self {
            parser,
            ts_language,
            tsx_language,
            definitions_query_ts,
            definitions_query_tsx,
            usages_query_ts,
            usages_query_tsx,
            current_language: Some(LanguageType::TypeScript),
        })
    }

    /// Determine language type from file extension
    fn get_language_type(path: &Path) -> LanguageType {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("tsx") => LanguageType::TypeScriptReact,
            _ => LanguageType::TypeScript,
        }
    }

    /// Set the appropriate language for the given file
    fn set_language_for_file(&mut self, path: &Path) -> Result<()> {
        let lang_type = Self::get_language_type(path);

        if self.current_language != Some(lang_type) {
            let language = match lang_type {
                LanguageType::TypeScript => &self.ts_language,
                LanguageType::TypeScriptReact => &self.tsx_language,
            };
            self.parser.set_language(language)?;
            self.current_language = Some(lang_type);
        }

        Ok(())
    }

    /// Parse source code and return the AST
    pub fn parse(&mut self, path: &Path, source_code: &str) -> Result<Tree> {
        self.set_language_for_file(path)?;

        self.parser
            .parse(source_code, None)
            .context("Failed to parse source code")
    }

    /// Get the definitions query for the current language
    pub fn definitions_query(&self) -> &Query {
        match self.current_language {
            Some(LanguageType::TypeScriptReact) => &self.definitions_query_tsx,
            _ => &self.definitions_query_ts,
        }
    }

    /// Get the usages query for the current language
    pub fn usages_query(&self) -> &Query {
        match self.current_language {
            Some(LanguageType::TypeScriptReact) => &self.usages_query_tsx,
            _ => &self.usages_query_ts,
        }
    }

    /// Check if a file is a TypeScript/TSX file
    pub fn is_typescript_file(path: &Path) -> bool {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("ts") | Some("tsx") => true,
            _ => false,
        }
    }
}
