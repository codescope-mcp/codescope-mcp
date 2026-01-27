use crate::symbol::types::{ContextKind, UsageContext};

/// Extract context hierarchy from a node
pub fn extract_contexts(
    node: tree_sitter::Node,
    source_code: &str,
    max_contexts: usize,
) -> Vec<UsageContext> {
    // Early return if no contexts requested
    if max_contexts == 0 {
        return Vec::new();
    }

    let mut contexts = Vec::new();
    let mut current = node;

    while let Some(parent) = current.parent() {
        if let Some(context) = node_to_context(parent, source_code) {
            contexts.push(context);
            if contexts.len() >= max_contexts {
                break;
            }
        }
        current = parent;
    }

    contexts
}

/// Convert a tree-sitter node to a UsageContext if applicable
fn node_to_context(node: tree_sitter::Node, source_code: &str) -> Option<UsageContext> {
    let kind = match node.kind() {
        "arrow_function" => Some(ContextKind::ArrowFunction),
        "function_declaration" => Some(ContextKind::FunctionDeclaration),
        "method_definition" => Some(ContextKind::MethodDeclaration),
        "class_declaration" => Some(ContextKind::ClassDeclaration),
        "interface_declaration" => Some(ContextKind::InterfaceDeclaration),
        "enum_declaration" => Some(ContextKind::EnumDeclaration),
        "program" => Some(ContextKind::SourceFile),
        _ => None,
    }?;

    // Check for constructor
    let kind = if node.kind() == "method_definition" {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = name_node
                .utf8_text(source_code.as_bytes())
                .unwrap_or("");
            if name == "constructor" {
                ContextKind::Constructor
            } else {
                kind
            }
        } else {
            kind
        }
    } else {
        kind
    };

    let name = extract_context_name(node, source_code);
    let start_line = node.start_position().row + 1;
    let end_line = node.end_position().row + 1;

    Some(UsageContext {
        kind,
        name,
        start_line,
        end_line,
    })
}

/// Extract the name from a context node
fn extract_context_name(node: tree_sitter::Node, source_code: &str) -> Option<String> {
    let name_node = match node.kind() {
        "function_declaration" | "class_declaration" | "interface_declaration"
        | "enum_declaration" => node.child_by_field_name("name"),
        "method_definition" => node.child_by_field_name("name"),
        "arrow_function" => {
            // For arrow functions, try to get the variable name from parent
            if let Some(parent) = node.parent() {
                if parent.kind() == "variable_declarator" {
                    parent.child_by_field_name("name")
                } else {
                    None
                }
            } else {
                None
            }
        }
        "program" => None, // SourceFile has no name
        _ => None,
    };

    name_node.and_then(|n| n.utf8_text(source_code.as_bytes()).ok().map(String::from))
}
