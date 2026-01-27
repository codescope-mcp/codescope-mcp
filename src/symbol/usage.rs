use std::collections::HashSet;
use std::path::Path;

use anyhow::Result;
use streaming_iterator::StreamingIterator;

use crate::context::extractor::extract_contexts;
use crate::parser::typescript::TypeScriptParser;
use crate::symbol::types::{SymbolUsage, UsageKind};

/// Find symbol usages in a file
pub fn find_usages_in_file(
    parser: &mut TypeScriptParser,
    file_path: &Path,
    symbol_name: &str,
    include_imports: bool,
    max_contexts: usize,
    object_filter: Option<&str>,
) -> Result<Vec<SymbolUsage>> {
    let source_code = std::fs::read_to_string(file_path)?;
    let tree = parser.parse(file_path, &source_code)?;

    let mut usages = Vec::new();
    let mut seen: HashSet<(usize, usize)> = HashSet::new(); // (line, column) for deduplication
    let query = parser.usages_query();

    let mut cursor = tree_sitter::QueryCursor::new();
    let mut matches = cursor.matches(query, tree.root_node(), source_code.as_bytes());

    while let Some(m) = matches.next() {
        for capture in m.captures {
            let capture_name = &query.capture_names()[capture.index as usize];

            if *capture_name == "usage" {
                let node = capture.node;
                let usage_name = node
                    .utf8_text(source_code.as_bytes())
                    .unwrap_or("")
                    .to_string();

                if usage_name != symbol_name {
                    continue;
                }

                let line = node.start_position().row + 1;
                let column = node.start_position().column;

                // Phase 1: Deduplication - skip if already seen
                if !seen.insert((line, column)) {
                    continue;
                }

                // Check if this is an import statement
                let is_import = is_in_import_statement(node);
                if !include_imports && is_import {
                    continue;
                }

                // Phase 2: Extract semantic context (usage kind and object name)
                let (usage_kind, object_name) = if is_import {
                    (UsageKind::Import, None)
                } else {
                    extract_member_access_info(node, &source_code)
                };

                // Phase 3: Apply object filter
                if let Some(filter) = object_filter {
                    match &object_name {
                        Some(obj) if obj == filter => {}
                        _ => continue, // Skip if object doesn't match filter
                    }
                }

                // Extract contexts
                let contexts = extract_contexts(node, &source_code, max_contexts);

                // Build qualified name
                let qualified_name = match &object_name {
                    Some(obj) => format!("{}.{}", obj, symbol_name),
                    None => symbol_name.to_string(),
                };

                usages.push(SymbolUsage {
                    file_path: file_path.to_string_lossy().to_string(),
                    line,
                    column,
                    qualified_name,
                    usage_kind,
                    object_name,
                    contexts,
                });
            }
        }
    }

    Ok(usages)
}

/// Check if a node is within an import statement
fn is_in_import_statement(node: tree_sitter::Node) -> bool {
    let mut current = node;
    while let Some(parent) = current.parent() {
        let kind = parent.kind();
        if kind == "import_statement" || kind == "import_specifier" || kind == "import_clause" {
            return true;
        }
        current = parent;
    }
    false
}

/// Extract member access information from a node
/// Returns (UsageKind, Option<object_name>)
fn extract_member_access_info(
    node: tree_sitter::Node,
    source: &str,
) -> (UsageKind, Option<String>) {
    if let Some(parent) = node.parent() {
        match parent.kind() {
            "member_expression" => {
                // Check if node is the property (right side) of member_expression
                if let Some(property_node) = parent.child_by_field_name("property") {
                    if property_node.id() == node.id() {
                        // Get the object (left side) of member_expression
                        if let Some(object_node) = parent.child_by_field_name("object") {
                            let object_name = object_node
                                .utf8_text(source.as_bytes())
                                .ok()
                                .map(|s| s.to_string());

                            // Check if this member_expression is part of a call_expression
                            if let Some(grandparent) = parent.parent() {
                                if grandparent.kind() == "call_expression" {
                                    // Verify the member_expression is the function being called
                                    if let Some(func_node) =
                                        grandparent.child_by_field_name("function")
                                    {
                                        if func_node.id() == parent.id() {
                                            return (UsageKind::MethodCall, object_name);
                                        }
                                    }
                                }
                            }
                            return (UsageKind::PropertyAccess, object_name);
                        }
                    }
                }
            }
            "type_annotation" | "type_reference" | "generic_type" => {
                return (UsageKind::TypeReference, None);
            }
            _ => {}
        }

        // Check for type annotation context in parent chain
        let mut current = parent;
        loop {
            match current.kind() {
                "type_annotation" | "type_reference" | "generic_type" => {
                    return (UsageKind::TypeReference, None);
                }
                _ => {}
            }
            if let Some(p) = current.parent() {
                current = p;
            } else {
                break;
            }
        }
    }

    (UsageKind::Identifier, None)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_is_import_statement() {
        // This is a basic unit test - integration tests cover more comprehensive cases
    }
}
