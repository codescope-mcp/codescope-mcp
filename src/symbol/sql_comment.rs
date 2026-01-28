//! SQL COMMENT ON statement extraction
//!
//! Handles extracting documentation from COMMENT ON statements for tables and columns.

use std::collections::HashMap;

use streaming_iterator::StreamingIterator;
use tree_sitter::{Query, Tree};

/// Map of symbol keys to their documentation from COMMENT ON statements.
///
/// Key format:
/// - Table: `"table_name"` (e.g., `"users"`)
/// - Column: `"table_name.column_name"` (e.g., `"users.email"`)
pub type SqlCommentMap = HashMap<String, String>;

/// Extract COMMENT ON statements from a SQL file and build a documentation map.
///
/// Returns a map from symbol identifiers to their documentation text.
pub fn extract_sql_comments(tree: &Tree, source: &str, query: &Query) -> SqlCommentMap {
    let mut comment_map = SqlCommentMap::new();
    let mut cursor = tree_sitter::QueryCursor::new();
    let mut matches = cursor.matches(query, tree.root_node(), source.as_bytes());

    while let Some(m) = matches.next() {
        let capture_names: Vec<&str> = m
            .captures
            .iter()
            .map(|c| query.capture_names()[c.index as usize])
            .collect();

        // Check if this is a COMMENT ON TABLE match
        if capture_names.contains(&"comment.table") {
            let mut table_name: Option<&str> = None;
            let mut comment_text: Option<&str> = None;

            for capture in m.captures {
                let name = query.capture_names()[capture.index as usize];
                if name == "comment.table.name" {
                    table_name = capture.node.utf8_text(source.as_bytes()).ok();
                } else if name == "comment.table.text" {
                    comment_text = capture.node.utf8_text(source.as_bytes()).ok();
                }
            }

            if let (Some(name), Some(text)) = (table_name, comment_text) {
                let cleaned_text = clean_sql_literal(text);
                comment_map.insert(name.to_string(), cleaned_text);
            }
        }
        // Check if this is a COMMENT ON COLUMN match
        else if capture_names.contains(&"comment.column") {
            let mut table_name: Option<&str> = None;
            let mut column_name: Option<&str> = None;
            let mut comment_text: Option<&str> = None;

            for capture in m.captures {
                let name = query.capture_names()[capture.index as usize];
                if name == "comment.column.table" {
                    table_name = capture.node.utf8_text(source.as_bytes()).ok();
                } else if name == "comment.column.name" {
                    column_name = capture.node.utf8_text(source.as_bytes()).ok();
                } else if name == "comment.column.text" {
                    comment_text = capture.node.utf8_text(source.as_bytes()).ok();
                }
            }

            if let (Some(table), Some(column), Some(text)) = (table_name, column_name, comment_text)
            {
                let key = format!("{}.{}", table, column);
                let cleaned_text = clean_sql_literal(text);
                comment_map.insert(key, cleaned_text);
            }
        }
    }

    comment_map
}

/// Clean SQL string literal by removing surrounding quotes and escape sequences.
fn clean_sql_literal(text: &str) -> String {
    let trimmed = text.trim();

    // Remove surrounding single quotes
    let without_quotes =
        if trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2 {
            &trimmed[1..trimmed.len() - 1]
        } else {
            trimmed
        };

    // Handle SQL escape sequences ('' -> ')
    without_quotes.replace("''", "'")
}

/// Find the parent table name for a column definition node.
///
/// Walks up the AST to find the enclosing CREATE TABLE statement
/// and returns its table name.
pub fn find_parent_table_name(node: tree_sitter::Node, source: &str) -> Option<String> {
    let mut current = node;

    while let Some(parent) = current.parent() {
        if parent.kind() == "create_table" {
            // Find the object_reference child that contains the table name
            for i in 0..parent.child_count() {
                if let Some(child) = parent.child(i) {
                    if child.kind() == "object_reference" {
                        // Get the identifier inside object_reference
                        for j in 0..child.child_count() {
                            if let Some(id_child) = child.child(j) {
                                if id_child.kind() == "identifier" {
                                    return id_child
                                        .utf8_text(source.as_bytes())
                                        .ok()
                                        .map(|s| s.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        current = parent;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_sql_literal() {
        assert_eq!(clean_sql_literal("'hello'"), "hello");
        assert_eq!(clean_sql_literal("'it''s'"), "it's");
        assert_eq!(
            clean_sql_literal("'User accounts table'"),
            "User accounts table"
        );
    }

    #[test]
    fn test_extract_sql_comments() {
        let language: tree_sitter::Language = tree_sitter_sequel::LANGUAGE.into();
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&language).unwrap();

        let source = r#"
CREATE TABLE users (
    id INT PRIMARY KEY,
    email VARCHAR(255)
);

COMMENT ON TABLE users IS 'User accounts table';
COMMENT ON COLUMN users.email IS 'Primary email address';
"#;

        let tree = parser.parse(source, None).unwrap();
        let query_src = include_str!("../../queries/sql/definitions.scm");
        let query = Query::new(&language, query_src).unwrap();

        let comment_map = extract_sql_comments(&tree, source, &query);

        assert_eq!(
            comment_map.get("users"),
            Some(&"User accounts table".to_string())
        );
        assert_eq!(
            comment_map.get("users.email"),
            Some(&"Primary email address".to_string())
        );
    }
}
