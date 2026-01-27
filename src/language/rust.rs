use anyhow::{Context, Result};
use tree_sitter::{Language, Query};

use crate::symbol::types::SymbolKind;

use super::traits::{LanguageId, LanguageSupport, SymbolKindMapping};

/// Rust language support
pub struct RustLanguage {
    language: Language,
    definitions_query: Query,
    usages_query: Query,
}

impl RustLanguage {
    pub fn new() -> Result<Self> {
        let language: Language = tree_sitter_rust::LANGUAGE.into();

        let definitions_query_src = include_str!("../../queries/rust/definitions.scm");
        let usages_query_src = include_str!("../../queries/rust/usages.scm");

        let definitions_query = Query::new(&language, definitions_query_src)
            .context("Failed to parse Rust definitions query")?;
        let usages_query =
            Query::new(&language, usages_query_src).context("Failed to parse Rust usages query")?;

        Ok(Self {
            language,
            definitions_query,
            usages_query,
        })
    }
}

impl LanguageSupport for RustLanguage {
    fn id(&self) -> LanguageId {
        LanguageId::Rust
    }

    fn name(&self) -> &'static str {
        "Rust"
    }

    fn file_extensions(&self) -> &[&'static str] {
        &["rs"]
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
        RUST_DEFINITION_MAPPINGS
    }
}

/// Definition mappings for Rust
const RUST_DEFINITION_MAPPINGS: &[SymbolKindMapping] = &[
    SymbolKindMapping {
        capture_name: "definition.function",
        kind: SymbolKind::Function,
    },
    SymbolKindMapping {
        capture_name: "definition.struct",
        kind: SymbolKind::Struct,
    },
    SymbolKindMapping {
        capture_name: "definition.enum",
        kind: SymbolKind::Enum,
    },
    SymbolKindMapping {
        capture_name: "definition.trait",
        kind: SymbolKind::Trait,
    },
    SymbolKindMapping {
        capture_name: "definition.impl",
        kind: SymbolKind::Impl,
    },
    SymbolKindMapping {
        capture_name: "definition.method",
        kind: SymbolKind::Method,
    },
    SymbolKindMapping {
        capture_name: "definition.type_alias",
        kind: SymbolKind::TypeAlias,
    },
    SymbolKindMapping {
        capture_name: "definition.module",
        kind: SymbolKind::Module,
    },
    SymbolKindMapping {
        capture_name: "definition.const",
        kind: SymbolKind::Const,
    },
    SymbolKindMapping {
        capture_name: "definition.static",
        kind: SymbolKind::Static,
    },
    SymbolKindMapping {
        capture_name: "definition.macro",
        kind: SymbolKind::Macro,
    },
];
