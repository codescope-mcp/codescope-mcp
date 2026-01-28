use crate::symbol::types::SymbolKind;

use super::traits::SymbolKindMapping;

// HTML language support
//
// This implementation supports:
// - HTML elements (both regular elements with start_tag and self-closing elements)
// - id attributes (captured as HtmlId)
// - class attributes (captured as HtmlClass)
//
// Known limitations:
// - Class attributes with multiple space-separated values (e.g., `class="card user-card"`)
//   are currently captured as a single class name rather than separate classes.
//   This means "card user-card" would be captured as one definition instead of
//   two separate classes "card" and "user-card".
// - Inline event handlers and other attributes are not captured as symbols.
define_language! {
    name: HtmlLanguage,
    id: Html,
    display_name: "Html",
    extensions: ["html", "htm"],
    tree_sitter_language: tree_sitter_html::LANGUAGE,
    query_dir: "html",
    mappings: HTML_DEFINITION_MAPPINGS,
}

/// Definition mappings for HTML
const HTML_DEFINITION_MAPPINGS: &[SymbolKindMapping] = &[
    SymbolKindMapping {
        capture_name: "definition.element",
        kind: SymbolKind::HtmlElement,
    },
    SymbolKindMapping {
        capture_name: "definition.id",
        kind: SymbolKind::HtmlId,
    },
    SymbolKindMapping {
        capture_name: "definition.class",
        kind: SymbolKind::HtmlClass,
    },
];
