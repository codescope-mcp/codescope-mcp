use anyhow::{Context, Result};
use tree_sitter::{Language, Query};

use crate::symbol::types::SymbolKind;

use super::traits::{LanguageId, LanguageSupport, SymbolKindMapping};

/// Java language support
pub struct JavaLanguage {
    language: Language,
    definitions_query: Query,
    usages_query: Query,
}

impl JavaLanguage {
    pub fn new() -> Result<Self> {
        let language: Language = tree_sitter_java::LANGUAGE.into();

        let definitions_query_src = include_str!("../../queries/java/definitions.scm");
        let usages_query_src = include_str!("../../queries/java/usages.scm");

        let definitions_query = Query::new(&language, definitions_query_src)
            .context("Failed to parse Java definitions query")?;
        let usages_query =
            Query::new(&language, usages_query_src).context("Failed to parse Java usages query")?;

        Ok(Self {
            language,
            definitions_query,
            usages_query,
        })
    }
}

impl LanguageSupport for JavaLanguage {
    fn id(&self) -> LanguageId {
        LanguageId::Java
    }

    fn name(&self) -> &'static str {
        "Java"
    }

    fn file_extensions(&self) -> &[&'static str] {
        &["java"]
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
        JAVA_DEFINITION_MAPPINGS
    }
}

/// Definition mappings for Java
const JAVA_DEFINITION_MAPPINGS: &[SymbolKindMapping] = &[
    SymbolKindMapping {
        capture_name: "definition.class",
        kind: SymbolKind::Class,
    },
    SymbolKindMapping {
        capture_name: "definition.interface",
        kind: SymbolKind::Interface,
    },
    SymbolKindMapping {
        capture_name: "definition.enum",
        kind: SymbolKind::Enum,
    },
    SymbolKindMapping {
        capture_name: "definition.method",
        kind: SymbolKind::Method,
    },
    SymbolKindMapping {
        capture_name: "definition.constructor",
        kind: SymbolKind::Constructor,
    },
    SymbolKindMapping {
        capture_name: "definition.field",
        kind: SymbolKind::Variable,
    },
    SymbolKindMapping {
        capture_name: "definition.annotation",
        kind: SymbolKind::Interface,
    },
];
