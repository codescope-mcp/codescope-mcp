use anyhow::{Context, Result};
use tree_sitter::{Language, Query};

use crate::symbol::types::SymbolKind;

use super::traits::{LanguageId, LanguageSupport, SymbolKindMapping};

/// Markdown language support
///
/// Note: tree-sitter-md uses a dual-grammar system (block and inline).
/// This implementation only uses the block grammar, which supports:
/// - ATX headings (# Heading)
/// - Fenced code blocks (```code```)
/// - Link reference definitions ([label]: url)
///
/// Inline elements like [text](url) links require the inline grammar
/// and are not currently supported.
pub struct MarkdownLanguage {
    language: Language,
    definitions_query: Query,
    usages_query: Query,
}

impl MarkdownLanguage {
    pub fn new() -> Result<Self> {
        let language: Language = tree_sitter_md::LANGUAGE.into();

        let definitions_query_src = include_str!("../../queries/markdown/definitions.scm");
        let usages_query_src = include_str!("../../queries/markdown/usages.scm");

        let definitions_query = Query::new(&language, definitions_query_src)
            .context("Failed to parse Markdown definitions query")?;
        let usages_query = Query::new(&language, usages_query_src)
            .context("Failed to parse Markdown usages query")?;

        Ok(Self {
            language,
            definitions_query,
            usages_query,
        })
    }
}

impl LanguageSupport for MarkdownLanguage {
    fn id(&self) -> LanguageId {
        LanguageId::Markdown
    }

    fn name(&self) -> &'static str {
        "Markdown"
    }

    fn file_extensions(&self) -> &[&'static str] {
        &["md", "mdc"]
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
        MARKDOWN_DEFINITION_MAPPINGS
    }
}

/// Definition mappings for Markdown
const MARKDOWN_DEFINITION_MAPPINGS: &[SymbolKindMapping] = &[
    SymbolKindMapping {
        capture_name: "definition.heading1",
        kind: SymbolKind::Heading1,
    },
    SymbolKindMapping {
        capture_name: "definition.heading2",
        kind: SymbolKind::Heading2,
    },
    SymbolKindMapping {
        capture_name: "definition.heading3",
        kind: SymbolKind::Heading3,
    },
    SymbolKindMapping {
        capture_name: "definition.heading4",
        kind: SymbolKind::Heading4,
    },
    SymbolKindMapping {
        capture_name: "definition.heading5",
        kind: SymbolKind::Heading5,
    },
    SymbolKindMapping {
        capture_name: "definition.heading6",
        kind: SymbolKind::Heading6,
    },
    SymbolKindMapping {
        capture_name: "definition.code_block",
        kind: SymbolKind::CodeBlock,
    },
    SymbolKindMapping {
        capture_name: "definition.link",
        kind: SymbolKind::Link,
    },
];
