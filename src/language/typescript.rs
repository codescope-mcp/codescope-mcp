use anyhow::{Context, Result};
use tree_sitter::{Language, Query};

use crate::symbol::types::SymbolKind;

use super::traits::{LanguageId, LanguageSupport, SymbolKindMapping};

/// TypeScript language support
pub struct TypeScriptLanguage {
    language: Language,
    definitions_query: Query,
    usages_query: Query,
}

impl TypeScriptLanguage {
    pub fn new() -> Result<Self> {
        let language: Language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();

        let definitions_query_src = include_str!("../../queries/typescript/definitions.scm");
        let usages_query_src = include_str!("../../queries/typescript/usages.scm");

        let definitions_query = Query::new(&language, definitions_query_src)
            .context("Failed to parse TypeScript definitions query")?;
        let usages_query = Query::new(&language, usages_query_src)
            .context("Failed to parse TypeScript usages query")?;

        Ok(Self {
            language,
            definitions_query,
            usages_query,
        })
    }
}

impl LanguageSupport for TypeScriptLanguage {
    fn id(&self) -> LanguageId {
        LanguageId::TypeScript
    }

    fn name(&self) -> &'static str {
        "TypeScript"
    }

    fn file_extensions(&self) -> &[&'static str] {
        &["ts"]
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
        TYPESCRIPT_DEFINITION_MAPPINGS
    }
}

/// TypeScript React (TSX) language support
pub struct TypeScriptReactLanguage {
    language: Language,
    definitions_query: Query,
    usages_query: Query,
}

impl TypeScriptReactLanguage {
    pub fn new() -> Result<Self> {
        let language: Language = tree_sitter_typescript::LANGUAGE_TSX.into();

        let definitions_query_src = include_str!("../../queries/typescript/definitions.scm");
        let usages_query_src = include_str!("../../queries/typescript/usages.scm");

        let definitions_query = Query::new(&language, definitions_query_src)
            .context("Failed to parse TSX definitions query")?;
        let usages_query =
            Query::new(&language, usages_query_src).context("Failed to parse TSX usages query")?;

        Ok(Self {
            language,
            definitions_query,
            usages_query,
        })
    }
}

impl LanguageSupport for TypeScriptReactLanguage {
    fn id(&self) -> LanguageId {
        LanguageId::TypeScriptReact
    }

    fn name(&self) -> &'static str {
        "TypeScriptReact"
    }

    fn file_extensions(&self) -> &[&'static str] {
        &["tsx"]
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
        TYPESCRIPT_DEFINITION_MAPPINGS
    }
}

/// Common definition mappings for TypeScript/TSX
const TYPESCRIPT_DEFINITION_MAPPINGS: &[SymbolKindMapping] = &[
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
        capture_name: "definition.interface",
        kind: SymbolKind::Interface,
    },
    SymbolKindMapping {
        capture_name: "definition.enum",
        kind: SymbolKind::Enum,
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
    SymbolKindMapping {
        capture_name: "definition.type_alias",
        kind: SymbolKind::TypeAlias,
    },
];
