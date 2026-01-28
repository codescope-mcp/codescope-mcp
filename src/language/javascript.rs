use crate::symbol::types::SymbolKind;

use super::traits::SymbolKindMapping;

define_language! {
    name: JavaScriptLanguage,
    id: JavaScript,
    display_name: "JavaScript",
    extensions: ["js", "mjs", "cjs"],
    tree_sitter_language: tree_sitter_javascript::LANGUAGE,
    query_dir: "javascript",
    mappings: JAVASCRIPT_DEFINITION_MAPPINGS,
}

define_language! {
    name: JavaScriptReactLanguage,
    id: JavaScriptReact,
    display_name: "JavaScriptReact",
    extensions: ["jsx"],
    tree_sitter_language: tree_sitter_javascript::LANGUAGE,
    query_dir: "javascript",
    mappings: JAVASCRIPT_DEFINITION_MAPPINGS,
}

/// Common definition mappings for JavaScript/JSX
/// (excludes Interface, Enum, and TypeAlias which are TypeScript-specific)
const JAVASCRIPT_DEFINITION_MAPPINGS: &[SymbolKindMapping] = &[
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
];
