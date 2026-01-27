use anyhow::{Context, Result};
use tree_sitter::{Language, Query};

use crate::symbol::types::SymbolKind;

use super::traits::{LanguageId, LanguageSupport, SymbolKindMapping};

/// JavaScript language support
pub struct JavaScriptLanguage {
    language: Language,
    definitions_query: Query,
    usages_query: Query,
}

impl JavaScriptLanguage {
    pub fn new() -> Result<Self> {
        let language: Language = tree_sitter_javascript::LANGUAGE.into();

        let definitions_query_src = include_str!("../../queries/javascript/definitions.scm");
        let usages_query_src = include_str!("../../queries/javascript/usages.scm");

        let definitions_query = Query::new(&language, definitions_query_src)
            .context("Failed to parse JavaScript definitions query")?;
        let usages_query = Query::new(&language, usages_query_src)
            .context("Failed to parse JavaScript usages query")?;

        Ok(Self {
            language,
            definitions_query,
            usages_query,
        })
    }
}

impl LanguageSupport for JavaScriptLanguage {
    fn id(&self) -> LanguageId {
        LanguageId::JavaScript
    }

    fn name(&self) -> &'static str {
        "JavaScript"
    }

    fn file_extensions(&self) -> &[&'static str] {
        &["js", "mjs", "cjs"]
    }

    fn tree_sitter_language(&self) -> &Language {
        &self.language
    }

    fn definitions_query(&self) -> &Query {
        &self.definitions_query
    }

    fn usages_query(&self) -> &Query {
        &self.usages_query
    }

    fn definition_mappings(&self) -> &[SymbolKindMapping] {
        JAVASCRIPT_DEFINITION_MAPPINGS
    }
}

/// JavaScript React (JSX) language support
pub struct JavaScriptReactLanguage {
    language: Language,
    definitions_query: Query,
    usages_query: Query,
}

impl JavaScriptReactLanguage {
    pub fn new() -> Result<Self> {
        // tree-sitter-javascript uses the same LANGUAGE for both JS and JSX
        let language: Language = tree_sitter_javascript::LANGUAGE.into();

        let definitions_query_src = include_str!("../../queries/javascript/definitions.scm");
        let usages_query_src = include_str!("../../queries/javascript/usages.scm");

        let definitions_query = Query::new(&language, definitions_query_src)
            .context("Failed to parse JSX definitions query")?;
        let usages_query =
            Query::new(&language, usages_query_src).context("Failed to parse JSX usages query")?;

        Ok(Self {
            language,
            definitions_query,
            usages_query,
        })
    }
}

impl LanguageSupport for JavaScriptReactLanguage {
    fn id(&self) -> LanguageId {
        LanguageId::JavaScriptReact
    }

    fn name(&self) -> &'static str {
        "JavaScriptReact"
    }

    fn file_extensions(&self) -> &[&'static str] {
        &["jsx"]
    }

    fn tree_sitter_language(&self) -> &Language {
        &self.language
    }

    fn definitions_query(&self) -> &Query {
        &self.definitions_query
    }

    fn usages_query(&self) -> &Query {
        &self.usages_query
    }

    fn definition_mappings(&self) -> &[SymbolKindMapping] {
        JAVASCRIPT_DEFINITION_MAPPINGS
    }
}

/// Common definition mappings for JavaScript/JSX
/// (excludes Interface, Enum, and TypeAlias which are TypeScript-specific)
const JAVASCRIPT_DEFINITION_MAPPINGS: &[SymbolKindMapping] = &[
    SymbolKindMapping {
        capture_name: "definition.function",
        kind: SymbolKind::Function,
    },
    SymbolKindMapping {
        capture_name: "definition.class",
        kind: SymbolKind::Class,
    },
    SymbolKindMapping {
        capture_name: "definition.method",
        kind: SymbolKind::Method,
    },
    SymbolKindMapping {
        capture_name: "definition.variable",
        kind: SymbolKind::Variable,
    },
    SymbolKindMapping {
        capture_name: "definition.arrow_function",
        kind: SymbolKind::ArrowFunction,
    },
    SymbolKindMapping {
        capture_name: "definition.constructor",
        kind: SymbolKind::Constructor,
    },
];
