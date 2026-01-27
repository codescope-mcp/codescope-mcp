use anyhow::{Context, Result};
use tree_sitter::{Language, Query};

use crate::symbol::types::SymbolKind;

use super::traits::{LanguageId, LanguageSupport, SymbolKindMapping};

/// HTML language support
pub struct HtmlLanguage {
    language: Language,
    definitions_query: Query,
    usages_query: Query,
}

impl HtmlLanguage {
    pub fn new() -> Result<Self> {
        let language: Language = tree_sitter_html::LANGUAGE.into();

        let definitions_query_src = include_str!("../../queries/html/definitions.scm");
        let usages_query_src = include_str!("../../queries/html/usages.scm");

        let definitions_query = Query::new(&language, definitions_query_src)
            .context("Failed to parse HTML definitions query")?;
        let usages_query =
            Query::new(&language, usages_query_src).context("Failed to parse HTML usages query")?;

        Ok(Self {
            language,
            definitions_query,
            usages_query,
        })
    }
}

impl LanguageSupport for HtmlLanguage {
    fn id(&self) -> LanguageId {
        LanguageId::Html
    }

    fn name(&self) -> &'static str {
        "Html"
    }

    fn file_extensions(&self) -> &[&'static str] {
        &["html", "htm"]
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
        HTML_DEFINITION_MAPPINGS
    }
}

/// Definition mappings for HTML
const HTML_DEFINITION_MAPPINGS: &[SymbolKindMapping] = &[
    SymbolKindMapping {
        capture_name: "definition.element",
        kind: SymbolKind::HtmlElement,
    },
    SymbolKindMapping {
        capture_name: "definition.id",
        kind: SymbolKind::HtmlId,
    },
    SymbolKindMapping {
        capture_name: "definition.class",
        kind: SymbolKind::HtmlClass,
    },
];
