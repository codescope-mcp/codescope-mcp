use crate::symbol::types::SymbolKind;

use super::traits::SymbolKindMapping;

define_language! {
    name: JavaLanguage,
    id: Java,
    display_name: "Java",
    extensions: ["java"],
    tree_sitter_language: tree_sitter_java::LANGUAGE,
    query_dir: "java",
    mappings: JAVA_DEFINITION_MAPPINGS,
}

/// Definition mappings for Java
const JAVA_DEFINITION_MAPPINGS: &[SymbolKindMapping] = &[
    SymbolKindMapping {
        capture_name: "definition.class",
        kind: SymbolKind::Class,
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
        capture_name: "definition.method",
        kind: SymbolKind::Method,
    },
    SymbolKindMapping {
        capture_name: "definition.constructor",
        kind: SymbolKind::Constructor,
    },
    SymbolKindMapping {
        capture_name: "definition.field",
        kind: SymbolKind::Variable,
    },
    SymbolKindMapping {
        capture_name: "definition.annotation",
        kind: SymbolKind::Interface,
    },
];
