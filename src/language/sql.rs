use anyhow::{Context, Result};
use tree_sitter::{Language, Query};

use crate::symbol::types::SymbolKind;

use super::traits::{LanguageId, LanguageSupport, SymbolKindMapping};

/// SQL language support
pub struct SqlLanguage {
    language: Language,
    definitions_query: Query,
    usages_query: Query,
}

impl SqlLanguage {
    pub fn new() -> Result<Self> {
        let language: Language = tree_sitter_sequel::LANGUAGE.into();

        let definitions_query_src = include_str!("../../queries/sql/definitions.scm");
        let usages_query_src = include_str!("../../queries/sql/usages.scm");

        let definitions_query = Query::new(&language, definitions_query_src)
            .context("Failed to parse SQL definitions query")?;
        let usages_query =
            Query::new(&language, usages_query_src).context("Failed to parse SQL usages query")?;

        Ok(Self {
            language,
            definitions_query,
            usages_query,
        })
    }
}

impl LanguageSupport for SqlLanguage {
    fn id(&self) -> LanguageId {
        LanguageId::Sql
    }

    fn name(&self) -> &'static str {
        "Sql"
    }

    fn file_extensions(&self) -> &[&'static str] {
        &["sql"]
    }

    fn tree_sitter_language(&self) -> &Language {
        &self.language
    }

    fn definitions_query(&self) -> &Query {
        &self.definitions_query
    }

    fn usages_query(&self) -> &Query {
        &self.usages_query
    }

    fn definition_mappings(&self) -> &[SymbolKindMapping] {
        SQL_DEFINITION_MAPPINGS
    }
}

/// Definition mappings for SQL
const SQL_DEFINITION_MAPPINGS: &[SymbolKindMapping] = &[
    SymbolKindMapping {
        capture_name: "definition.table",
        kind: SymbolKind::Table,
    },
    SymbolKindMapping {
        capture_name: "definition.view",
        kind: SymbolKind::View,
    },
    SymbolKindMapping {
        capture_name: "definition.function",
        kind: SymbolKind::Function,
    },
    SymbolKindMapping {
        capture_name: "definition.index",
        kind: SymbolKind::Index,
    },
    SymbolKindMapping {
        capture_name: "definition.trigger",
        kind: SymbolKind::Trigger,
    },
    SymbolKindMapping {
        capture_name: "definition.column",
        kind: SymbolKind::Column,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Debug test to dump SQL AST structure.
    /// Run with: cargo test test_sql_ast_dump -- --ignored --nocapture
    #[test]
    #[ignore]
    fn test_sql_ast_dump() {
        let language: Language = tree_sitter_sequel::LANGUAGE.into();
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&language).unwrap();

        let source = r#"
-- Table definition
CREATE TABLE users (
    id INT PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE
);

-- View definition
CREATE VIEW active_users AS
SELECT * FROM users WHERE active = true;

-- Index definition
CREATE INDEX idx_users_email ON users(email);

-- Function definition
CREATE FUNCTION get_user_count()
RETURNS INT
AS $$
    SELECT COUNT(*) FROM users;
$$;

-- Trigger definition
CREATE TRIGGER update_timestamp
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION update_modified_column();

-- Comment on table
COMMENT ON TABLE users IS 'User accounts table';

-- Comment on column
COMMENT ON COLUMN users.email IS 'User email address';
"#;

        let tree = parser.parse(source, None).unwrap();
        print_node(&tree.root_node(), source, 0);
    }

    fn print_node(node: &tree_sitter::Node, source: &str, indent: usize) {
        let prefix = " ".repeat(indent);
        let text = node.utf8_text(source.as_bytes()).unwrap_or("");
        let char_count = text.chars().count();
        let mut short_text: String = text.chars().take(50).collect();
        if char_count > 50 {
            short_text.push_str("...");
        }
        let short_text = short_text.replace('\n', "\\n");

        println!(
            "{}{} [{}-{}] \"{}\"",
            prefix,
            node.kind(),
            node.start_position().row,
            node.end_position().row,
            short_text
        );

        // Print field names
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.is_named() {
                    if let Some(field_name) = node.field_name_for_child(i as u32) {
                        println!("{}  field: {}", prefix, field_name);
                    }
                    print_node(&child, source, indent + 2);
                }
            }
        }
    }
}
