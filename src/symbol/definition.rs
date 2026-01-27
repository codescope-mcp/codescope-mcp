use std::path::Path;

use anyhow::Result;
use streaming_iterator::StreamingIterator;

use crate::parser::typescript::TypeScriptParser;
use crate::symbol::comment::extract_docs_before_line;
use crate::symbol::types::{SymbolDefinition, SymbolKind};

/// Find symbol definitions in a file
pub fn find_definitions_in_file(
    parser: &mut TypeScriptParser,
    file_path: &Path,
    symbol_name: &str,
    include_docs: bool,
) -> Result<Vec<SymbolDefinition>> {
    let source_code = std::fs::read_to_string(file_path)?;
    let tree = parser.parse(file_path, &source_code)?;

    let mut definitions = Vec::new();
    let query = parser.definitions_query();

    let mut cursor = tree_sitter::QueryCursor::new();
    let mut matches = cursor.matches(query, tree.root_node(), source_code.as_bytes());

    while let Some(m) = matches.next() {
        let mut name: Option<&str> = None;
        let mut definition_node: Option<tree_sitter::Node> = None;
        let mut kind: Option<SymbolKind> = None;

        for capture in m.captures {
            let capture_name = &query.capture_names()[capture.index as usize];

            match &**capture_name {
                "name" => {
                    name = Some(capture.node.utf8_text(source_code.as_bytes()).unwrap_or(""));
                }
                "definition.function" => {
                    definition_node = Some(capture.node);
                    kind = Some(SymbolKind::Function);
                }
                "definition.class" => {
                    definition_node = Some(capture.node);
                    kind = Some(SymbolKind::Class);
                }
                "definition.method" => {
                    definition_node = Some(capture.node);
                    kind = Some(SymbolKind::Method);
                }
                "definition.interface" => {
                    definition_node = Some(capture.node);
                    kind = Some(SymbolKind::Interface);
                }
                "definition.enum" => {
                    definition_node = Some(capture.node);
                    kind = Some(SymbolKind::Enum);
                }
                "definition.variable" => {
                    definition_node = Some(capture.node);
                    kind = Some(SymbolKind::Variable);
                }
                "definition.arrow_function" => {
                    definition_node = Some(capture.node);
                    kind = Some(SymbolKind::ArrowFunction);
                }
                "definition.constructor" => {
                    definition_node = Some(capture.node);
                    kind = Some(SymbolKind::Constructor);
                }
                "definition.type_alias" => {
                    definition_node = Some(capture.node);
                    kind = Some(SymbolKind::TypeAlias);
                }
                _ => {}
            }
        }

        if let (Some(name_str), Some(node), Some(symbol_kind)) = (name, definition_node, kind) {
            if name_str == symbol_name {
                let start_line = node.start_position().row + 1;
                let end_line = node.end_position().row + 1;
                let code = node
                    .utf8_text(source_code.as_bytes())
                    .unwrap_or("")
                    .to_string();

                // Extract docs if requested (use 0-indexed row)
                let docs = if include_docs {
                    extract_docs_before_line(&source_code, node.start_position().row)
                } else {
                    None
                };

                definitions.push(SymbolDefinition {
                    file_path: file_path.to_string_lossy().to_string(),
                    start_line,
                    end_line,
                    node_kind: symbol_kind,
                    code,
                    name: name_str.to_string(),
                    docs,
                });
            }
        }
    }

    Ok(definitions)
}

/// Find the enclosing symbol at a specific line in a file.
/// Returns the smallest symbol that contains the given line.
pub fn find_symbol_at_location(
    parser: &mut TypeScriptParser,
    file_path: &Path,
    line: usize,
) -> Result<Option<SymbolDefinition>> {
    let source_code = std::fs::read_to_string(file_path)?;
    let tree = parser.parse(file_path, &source_code)?;

    let query = parser.definitions_query();
    let target_line = line.saturating_sub(1); // Convert to 0-indexed

    let mut cursor = tree_sitter::QueryCursor::new();
    let mut matches = cursor.matches(query, tree.root_node(), source_code.as_bytes());

    let mut best_symbol: Option<SymbolDefinition> = None;
    let mut best_size: usize = usize::MAX;

    while let Some(m) = matches.next() {
        let mut name: Option<&str> = None;
        let mut definition_node: Option<tree_sitter::Node> = None;
        let mut kind: Option<SymbolKind> = None;

        for capture in m.captures {
            let capture_name = &query.capture_names()[capture.index as usize];

            match &**capture_name {
                "name" => {
                    name = Some(capture.node.utf8_text(source_code.as_bytes()).unwrap_or(""));
                }
                "definition.function" => {
                    definition_node = Some(capture.node);
                    kind = Some(SymbolKind::Function);
                }
                "definition.class" => {
                    definition_node = Some(capture.node);
                    kind = Some(SymbolKind::Class);
                }
                "definition.method" => {
                    definition_node = Some(capture.node);
                    kind = Some(SymbolKind::Method);
                }
                "definition.interface" => {
                    definition_node = Some(capture.node);
                    kind = Some(SymbolKind::Interface);
                }
                "definition.enum" => {
                    definition_node = Some(capture.node);
                    kind = Some(SymbolKind::Enum);
                }
                "definition.variable" => {
                    definition_node = Some(capture.node);
                    kind = Some(SymbolKind::Variable);
                }
                "definition.arrow_function" => {
                    definition_node = Some(capture.node);
                    kind = Some(SymbolKind::ArrowFunction);
                }
                "definition.constructor" => {
                    definition_node = Some(capture.node);
                    kind = Some(SymbolKind::Constructor);
                }
                "definition.type_alias" => {
                    definition_node = Some(capture.node);
                    kind = Some(SymbolKind::TypeAlias);
                }
                _ => {}
            }
        }

        if let (Some(name_str), Some(node), Some(symbol_kind)) = (name, definition_node, kind) {
            let start_line = node.start_position().row;
            let end_line = node.end_position().row;

            // Check if the target line is within this symbol
            if target_line >= start_line && target_line <= end_line {
                let size = end_line - start_line;
                // Prefer the smallest enclosing symbol
                if size < best_size {
                    best_size = size;
                    let code = node
                        .utf8_text(source_code.as_bytes())
                        .unwrap_or("")
                        .to_string();

                    best_symbol = Some(SymbolDefinition {
                        file_path: file_path.to_string_lossy().to_string(),
                        start_line: start_line + 1,
                        end_line: end_line + 1,
                        node_kind: symbol_kind,
                        code,
                        name: name_str.to_string(),
                        docs: None,
                    });
                }
            }
        }
    }

    Ok(best_symbol)
}
