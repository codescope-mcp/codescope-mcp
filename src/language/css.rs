use anyhow::{Context, Result};
use tree_sitter::{Language, Query};

use crate::symbol::types::SymbolKind;

use super::traits::{LanguageId, LanguageSupport, SymbolKindMapping};

/// CSS language support
///
/// This implementation supports:
/// - Class selectors (.classname)
/// - ID selectors (#idname)
/// - CSS custom properties/variables (--property-name)
/// - @keyframes animations
///
/// Known limitations:
/// - Pseudo-classes and pseudo-elements are not captured as separate symbols.
/// - Media queries and other at-rules (except @keyframes) are not captured.
/// - Nested selectors in preprocessors (Sass/Less) are not supported.
pub struct CssLanguage {
    language: Language,
    definitions_query: Query,
    usages_query: Query,
}

impl CssLanguage {
    pub fn new() -> Result<Self> {
        let language: Language = tree_sitter_css::LANGUAGE.into();

        let definitions_query_src = include_str!("../../queries/css/definitions.scm");
        let usages_query_src = include_str!("../../queries/css/usages.scm");

        let definitions_query = Query::new(&language, definitions_query_src)
            .context("Failed to parse CSS definitions query")?;
        let usages_query =
            Query::new(&language, usages_query_src).context("Failed to parse CSS usages query")?;

        Ok(Self {
            language,
            definitions_query,
            usages_query,
        })
    }
}

impl LanguageSupport for CssLanguage {
    fn id(&self) -> LanguageId {
        LanguageId::Css
    }

    fn name(&self) -> &'static str {
        "Css"
    }

    fn file_extensions(&self) -> &[&'static str] {
        &["css"]
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
        CSS_DEFINITION_MAPPINGS
    }
}

/// Definition mappings for CSS
const CSS_DEFINITION_MAPPINGS: &[SymbolKindMapping] = &[
    SymbolKindMapping {
        capture_name: "definition.class_selector",
        kind: SymbolKind::CssClassSelector,
    },
    SymbolKindMapping {
        capture_name: "definition.id_selector",
        kind: SymbolKind::CssIdSelector,
    },
    SymbolKindMapping {
        capture_name: "definition.variable",
        kind: SymbolKind::CssVariable,
    },
    SymbolKindMapping {
        capture_name: "definition.keyframes",
        kind: SymbolKind::CssKeyframes,
    },
];
