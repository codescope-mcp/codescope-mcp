use crate::symbol::types::SymbolKind;

use super::traits::SymbolKindMapping;

define_language! {
    name: GoLanguage,
    id: Go,
    display_name: "Go",
    extensions: ["go"],
    tree_sitter_language: tree_sitter_go::LANGUAGE,
    query_dir: "go",
    mappings: GO_DEFINITION_MAPPINGS,
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
