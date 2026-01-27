use anyhow::{Context, Result};
use tree_sitter::{Language, Query};

use crate::symbol::types::SymbolKind;

use super::traits::{LanguageId, LanguageSupport, SymbolKindMapping};

/// Python language support
pub struct PythonLanguage {
    language: Language,
    definitions_query: Query,
    usages_query: Query,
}

impl PythonLanguage {
    pub fn new() -> Result<Self> {
        let language: Language = tree_sitter_python::LANGUAGE.into();

        let definitions_query_src = include_str!("../../queries/python/definitions.scm");
        let usages_query_src = include_str!("../../queries/python/usages.scm");

        let definitions_query = Query::new(&language, definitions_query_src)
            .context("Failed to parse Python definitions query")?;
        let usages_query = Query::new(&language, usages_query_src)
            .context("Failed to parse Python usages query")?;

        Ok(Self {
            language,
            definitions_query,
            usages_query,
        })
    }
}

impl LanguageSupport for PythonLanguage {
    fn id(&self) -> LanguageId {
        LanguageId::Python
    }

    fn name(&self) -> &'static str {
        "Python"
    }

    fn file_extensions(&self) -> &[&'static str] {
        &["py", "pyi"]
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
        PYTHON_DEFINITION_MAPPINGS
    }
}

/// Definition mappings for Python
const PYTHON_DEFINITION_MAPPINGS: &[SymbolKindMapping] = &[
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
        capture_name: "definition.constructor",
        kind: SymbolKind::Constructor,
    },
    SymbolKindMapping {
        capture_name: "definition.variable",
        kind: SymbolKind::Variable,
    },
];
