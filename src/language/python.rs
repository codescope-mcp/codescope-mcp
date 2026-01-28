use crate::symbol::types::SymbolKind;

use super::traits::SymbolKindMapping;

define_language! {
    name: PythonLanguage,
    id: Python,
    display_name: "Python",
    extensions: ["py", "pyi"],
    tree_sitter_language: tree_sitter_python::LANGUAGE,
    query_dir: "python",
    mappings: PYTHON_DEFINITION_MAPPINGS,
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
