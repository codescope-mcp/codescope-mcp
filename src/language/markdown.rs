use crate::symbol::types::SymbolKind;

use super::traits::SymbolKindMapping;

// Markdown language support
//
// Note: tree-sitter-md uses a dual-grammar system (block and inline).
// This implementation only uses the block grammar, which supports:
// - ATX headings (# Heading)
// - Fenced code blocks (```code```)
// - Link reference definitions ([label]: url)
//
// Inline elements like [text](url) links require the inline grammar
// and are not currently supported.
define_language! {
    name: MarkdownLanguage,
    id: Markdown,
    display_name: "Markdown",
    extensions: ["md", "mdc"],
    tree_sitter_language: tree_sitter_md::LANGUAGE,
    query_dir: "markdown",
    mappings: MARKDOWN_DEFINITION_MAPPINGS,
}

/// Definition mappings for Markdown
const MARKDOWN_DEFINITION_MAPPINGS: &[SymbolKindMapping] = &[
    SymbolKindMapping {
        capture_name: "definition.heading1",
        kind: SymbolKind::Heading1,
    },
    SymbolKindMapping {
        capture_name: "definition.heading2",
        kind: SymbolKind::Heading2,
    },
    SymbolKindMapping {
        capture_name: "definition.heading3",
        kind: SymbolKind::Heading3,
    },
    SymbolKindMapping {
        capture_name: "definition.heading4",
        kind: SymbolKind::Heading4,
    },
    SymbolKindMapping {
        capture_name: "definition.heading5",
        kind: SymbolKind::Heading5,
    },
    SymbolKindMapping {
        capture_name: "definition.heading6",
        kind: SymbolKind::Heading6,
    },
    SymbolKindMapping {
        capture_name: "definition.code_block",
        kind: SymbolKind::CodeBlock,
    },
    SymbolKindMapping {
        capture_name: "definition.link",
        kind: SymbolKind::Link,
    },
];
