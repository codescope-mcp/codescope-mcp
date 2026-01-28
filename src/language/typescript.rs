use crate::symbol::types::SymbolKind;

use super::traits::SymbolKindMapping;

define_language! {
    name: TypeScriptLanguage,
    id: TypeScript,
    display_name: "TypeScript",
    extensions: ["ts"],
    tree_sitter_language: tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
    query_dir: "typescript",
    mappings: TYPESCRIPT_DEFINITION_MAPPINGS,
}

define_language! {
    name: TypeScriptReactLanguage,
    id: TypeScriptReact,
    display_name: "TypeScriptReact",
    extensions: ["tsx"],
    tree_sitter_language: tree_sitter_typescript::LANGUAGE_TSX,
    query_dir: "typescript",
    mappings: TYPESCRIPT_DEFINITION_MAPPINGS,
}

/// Common definition mappings for TypeScript/TSX
pub(super) const TYPESCRIPT_DEFINITION_MAPPINGS: &[SymbolKindMapping] = &[
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
