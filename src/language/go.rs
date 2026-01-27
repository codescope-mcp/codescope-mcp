use anyhow::{Context, Result};
use tree_sitter::{Language, Query};

use crate::symbol::types::SymbolKind;

use super::traits::{LanguageId, LanguageSupport, SymbolKindMapping};

/// Go language support
pub struct GoLanguage {
    language: Language,
    definitions_query: Query,
    usages_query: Query,
}

impl GoLanguage {
    pub fn new() -> Result<Self> {
        let language: Language = tree_sitter_go::LANGUAGE.into();

        let definitions_query_src = include_str!("../../queries/go/definitions.scm");
        let usages_query_src = include_str!("../../queries/go/usages.scm");

        let definitions_query = Query::new(&language, definitions_query_src)
            .context("Failed to parse Go definitions query")?;
        let usages_query =
            Query::new(&language, usages_query_src).context("Failed to parse Go usages query")?;

        Ok(Self {
            language,
            definitions_query,
            usages_query,
        })
    }
}

impl LanguageSupport for GoLanguage {
    fn id(&self) -> LanguageId {
        LanguageId::Go
    }

    fn name(&self) -> &'static str {
        "Go"
    }

    fn file_extensions(&self) -> &[&'static str] {
        &["go"]
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
        GO_DEFINITION_MAPPINGS
    }
}

/// Definition mappings for Go
const GO_DEFINITION_MAPPINGS: &[SymbolKindMapping] = &[
    SymbolKindMapping {
        capture_name: "definition.function",
        kind: SymbolKind::Function,
    },
    SymbolKindMapping {
        capture_name: "definition.method",
        kind: SymbolKind::Method,
    },
    SymbolKindMapping {
        capture_name: "definition.struct",
        kind: SymbolKind::Struct,
    },
    SymbolKindMapping {
        capture_name: "definition.interface",
        kind: SymbolKind::Interface,
    },
    SymbolKindMapping {
        capture_name: "definition.type_alias",
        kind: SymbolKind::TypeAlias,
    },
    SymbolKindMapping {
        capture_name: "definition.const",
        kind: SymbolKind::Const,
    },
    SymbolKindMapping {
        capture_name: "definition.variable",
        kind: SymbolKind::Variable,
    },
];
