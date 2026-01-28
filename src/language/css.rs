use crate::symbol::types::SymbolKind;

use super::traits::SymbolKindMapping;

// CSS language support
//
// This implementation supports:
// - Class selectors (.classname)
// - ID selectors (#idname)
// - CSS custom properties/variables (--property-name)
// - @keyframes animations
//
// Known limitations:
// - Pseudo-classes and pseudo-elements are not captured as separate symbols.
// - Media queries and other at-rules (except @keyframes) are not captured.
// - Nested selectors in preprocessors (Sass/Less) are not supported.
define_language! {
    name: CssLanguage,
    id: Css,
    display_name: "Css",
    extensions: ["css"],
    tree_sitter_language: tree_sitter_css::LANGUAGE,
    query_dir: "css",
    mappings: CSS_DEFINITION_MAPPINGS,
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
