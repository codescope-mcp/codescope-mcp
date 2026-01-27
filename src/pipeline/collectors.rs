use std::collections::HashSet;
use std::path::Path;

use anyhow::Result;
use streaming_iterator::StreamingIterator;

use crate::context::extractor::extract_contexts;
use crate::parser::GenericParser;
use crate::symbol::comment::{extract_docs_before_line, find_comments_in_file};
use crate::symbol::types::{CommentMatch, SymbolDefinition, SymbolKind, SymbolUsage, UsageKind};

/// Trait for collecting results from parsed files
pub trait ResultCollector: Sync {
    type Item: Send;

    fn process_file(&self, parser: &mut GenericParser, path: &Path) -> Result<Vec<Self::Item>>;
}

/// Collector for symbol definitions
pub struct DefinitionCollector {
    pub symbol: String,
    pub include_docs: bool,
}

impl ResultCollector for DefinitionCollector {
    type Item = SymbolDefinition;

    fn process_file(&self, parser: &mut GenericParser, path: &Path) -> Result<Vec<Self::Item>> {
        let source_code = std::fs::read_to_string(path)?;
        let (tree, language) = parser.parse_with_language(path, &source_code)?;

        let mut definitions = Vec::new();
        let query = language.definitions_query();
        let mappings = language.definition_mappings();

        let mut cursor = tree_sitter::QueryCursor::new();
        let mut matches = cursor.matches(query, tree.root_node(), source_code.as_bytes());

        while let Some(m) = matches.next() {
            let mut name: Option<&str> = None;
            let mut definition_node: Option<tree_sitter::Node> = None;
            let mut kind: Option<SymbolKind> = None;

            for capture in m.captures {
                let capture_name = &query.capture_names()[capture.index as usize];

                if *capture_name == "name" {
                    name = Some(capture.node.utf8_text(source_code.as_bytes()).unwrap_or(""));
                } else {
                    // Check mappings for definition types
                    for mapping in mappings {
                        if *capture_name == mapping.capture_name {
                            definition_node = Some(capture.node);
                            kind = Some(mapping.kind);
                            break;
                        }
                    }
                }
            }

            if let (Some(name_str), Some(node), Some(symbol_kind)) = (name, definition_node, kind) {
                if name_str == self.symbol {
                    let start_line = node.start_position().row + 1;
                    let end_line = node.end_position().row + 1;
                    let code = node
                        .utf8_text(source_code.as_bytes())
                        .unwrap_or("")
                        .to_string();

                    let docs = if self.include_docs {
                        extract_docs_before_line(&source_code, node.start_position().row)
                    } else {
                        None
                    };

                    definitions.push(SymbolDefinition {
                        file_path: path.to_string_lossy().to_string(),
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
}

/// Collector for symbol usages
pub struct UsageCollector {
    pub symbol: String,
    pub include_imports: bool,
    pub max_contexts: usize,
    pub object_filter: Option<String>,
}

impl ResultCollector for UsageCollector {
    type Item = SymbolUsage;

    fn process_file(&self, parser: &mut GenericParser, path: &Path) -> Result<Vec<Self::Item>> {
        let source_code = std::fs::read_to_string(path)?;
        let (tree, language) = parser.parse_with_language(path, &source_code)?;

        let mut usages = Vec::new();
        let mut seen: HashSet<(usize, usize)> = HashSet::new();
        let query = language.usages_query();

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

                    if usage_name != self.symbol {
                        continue;
                    }

                    let line = node.start_position().row + 1;
                    let column = node.start_position().column;

                    if !seen.insert((line, column)) {
                        continue;
                    }

                    let is_import = is_in_import_statement(node);
                    if !self.include_imports && is_import {
                        continue;
                    }

                    let (usage_kind, object_name) = if is_import {
                        (UsageKind::Import, None)
                    } else {
                        extract_member_access_info(node, &source_code)
                    };

                    if let Some(ref filter) = self.object_filter {
                        match &object_name {
                            Some(obj) if obj == filter => {}
                            _ => continue,
                        }
                    }

                    let contexts = extract_contexts(node, &source_code, self.max_contexts);

                    let qualified_name = match &object_name {
                        Some(obj) => format!("{}.{}", obj, self.symbol),
                        None => self.symbol.clone(),
                    };

                    usages.push(SymbolUsage {
                        file_path: path.to_string_lossy().to_string(),
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
}

/// Collector for method calls
pub struct MethodCallCollector {
    pub method_name: String,
    pub object_name: Option<String>,
}

impl ResultCollector for MethodCallCollector {
    type Item = SymbolUsage;

    fn process_file(&self, parser: &mut GenericParser, path: &Path) -> Result<Vec<Self::Item>> {
        let collector = UsageCollector {
            symbol: self.method_name.clone(),
            include_imports: false,
            max_contexts: 0,
            object_filter: self.object_name.clone(),
        };

        let usages = collector.process_file(parser, path)?;

        // Filter for MethodCall only
        Ok(usages
            .into_iter()
            .filter(|u| u.usage_kind == UsageKind::MethodCall)
            .collect())
    }
}

/// Collector for imports
pub struct ImportCollector {
    pub symbol: String,
}

impl ResultCollector for ImportCollector {
    type Item = SymbolUsage;

    fn process_file(&self, parser: &mut GenericParser, path: &Path) -> Result<Vec<Self::Item>> {
        let collector = UsageCollector {
            symbol: self.symbol.clone(),
            include_imports: true,
            max_contexts: 0,
            object_filter: None,
        };

        let usages = collector.process_file(parser, path)?;

        // Filter for Import only
        Ok(usages
            .into_iter()
            .filter(|u| u.usage_kind == UsageKind::Import)
            .collect())
    }
}

/// Collector for comments containing specific text
pub struct CommentCollector {
    pub text: String,
}

impl ResultCollector for CommentCollector {
    type Item = CommentMatch;

    fn process_file(&self, _parser: &mut GenericParser, path: &Path) -> Result<Vec<Self::Item>> {
        // Delegate to the existing comprehensive comment search implementation
        // which properly handles multi-line block comments, state management,
        // and edge cases that a simple line-by-line search would miss.
        find_comments_in_file(path, &self.text)
    }
}

// Helper functions moved from usage.rs

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

fn extract_member_access_info(
    node: tree_sitter::Node,
    source: &str,
) -> (UsageKind, Option<String>) {
    if let Some(parent) = node.parent() {
        match parent.kind() {
            "member_expression" => {
                if let Some(property_node) = parent.child_by_field_name("property") {
                    if property_node.id() == node.id() {
                        if let Some(object_node) = parent.child_by_field_name("object") {
                            let object_name = object_node
                                .utf8_text(source.as_bytes())
                                .ok()
                                .map(|s| s.to_string());

                            if let Some(grandparent) = parent.parent() {
                                if grandparent.kind() == "call_expression" {
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
