use crate::symbol::types::SymbolKind;

use super::traits::SymbolKindMapping;

define_language! {
    name: RustLanguage,
    id: Rust,
    display_name: "Rust",
    extensions: ["rs"],
    tree_sitter_language: tree_sitter_rust::LANGUAGE,
    query_dir: "rust",
    mappings: RUST_DEFINITION_MAPPINGS,
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
