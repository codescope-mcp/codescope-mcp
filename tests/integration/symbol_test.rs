use std::path::PathBuf;

use std::sync::Arc;

use codescope_mcp::language::LanguageRegistry;
use codescope_mcp::parser::generic::GenericParser;
use codescope_mcp::parser::typescript::TypeScriptParser;
use codescope_mcp::symbol::comment::{
    find_comments_in_file, find_text_in_markdown_file, get_code_at_location,
};
use codescope_mcp::symbol::definition::{find_definitions_in_file, find_symbol_at_location};
use codescope_mcp::symbol::types::{CommentType, UsageKind};
use codescope_mcp::symbol::usage::find_usages_in_file;

fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

#[test]
fn test_find_interface_definition() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    let definitions = find_definitions_in_file(&mut parser, &file_path, "User", false)
        .expect("Failed to find definitions");

    assert!(!definitions.is_empty(), "Should find User interface");
    let user_def = &definitions[0];
    assert_eq!(user_def.name, "User");
    assert_eq!(user_def.node_kind.to_string(), "Interface");
}

#[test]
fn test_find_class_definition() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    let definitions = find_definitions_in_file(&mut parser, &file_path, "UserService", false)
        .expect("Failed to find definitions");

    assert!(!definitions.is_empty(), "Should find UserService class");
    let class_def = &definitions[0];
    assert_eq!(class_def.name, "UserService");
    assert_eq!(class_def.node_kind.to_string(), "Class");
}

#[test]
fn test_find_function_definition() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    let definitions = find_definitions_in_file(&mut parser, &file_path, "processUser", false)
        .expect("Failed to find definitions");

    assert!(!definitions.is_empty(), "Should find processUser function");
    let func_def = &definitions[0];
    assert_eq!(func_def.name, "processUser");
    assert_eq!(func_def.node_kind.to_string(), "Function");
}

#[test]
fn test_find_arrow_function_definition() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    let definitions = find_definitions_in_file(&mut parser, &file_path, "createUser", false)
        .expect("Failed to find definitions");

    assert!(
        !definitions.is_empty(),
        "Should find createUser arrow function"
    );
    let arrow_def = &definitions[0];
    assert_eq!(arrow_def.name, "createUser");
    // Could be ArrowFunction or Variable depending on query
}

#[test]
fn test_find_usages() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    let usages = find_usages_in_file(&mut parser, &file_path, "User", false, 3, None)
        .expect("Failed to find usages");

    // User should be used in multiple places
    assert!(!usages.is_empty(), "Should find User usages");
}

#[test]
fn test_find_usages_with_imports() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    let usages_without_imports =
        find_usages_in_file(&mut parser, &file_path, "User", false, 3, None)
            .expect("Failed to find usages");

    let usages_with_imports = find_usages_in_file(&mut parser, &file_path, "User", true, 3, None)
        .expect("Failed to find usages");

    // With imports should be >= without imports
    assert!(
        usages_with_imports.len() >= usages_without_imports.len(),
        "Usages with imports should be >= usages without imports"
    );
}

// Phase 1: Test deduplication
#[test]
fn test_no_duplicate_usages() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    let usages = find_usages_in_file(&mut parser, &file_path, "User", true, 3, None)
        .expect("Failed to find usages");

    // Check for duplicates by (line, column)
    let mut seen = std::collections::HashSet::new();
    for usage in &usages {
        let key = (usage.line, usage.column);
        assert!(
            seen.insert(key),
            "Found duplicate usage at line {} column {}",
            usage.line,
            usage.column
        );
    }
}

// Phase 2: Test usage kind detection
#[test]
fn test_usage_kind_method_call() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    // console.log is a method call
    let usages = find_usages_in_file(&mut parser, &file_path, "log", false, 3, None)
        .expect("Failed to find usages");

    assert!(!usages.is_empty(), "Should find log usages");

    // All log usages should be method calls with object_name "console"
    for usage in &usages {
        assert_eq!(
            usage.usage_kind,
            UsageKind::MethodCall,
            "log should be a method call"
        );
        assert_eq!(
            usage.object_name.as_deref(),
            Some("console"),
            "log should be called on console"
        );
    }
}

#[test]
fn test_usage_kind_type_reference() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    let usages = find_usages_in_file(&mut parser, &file_path, "User", false, 3, None)
        .expect("Failed to find usages");

    // User is used as a type annotation in several places
    let type_refs: Vec<_> = usages
        .iter()
        .filter(|u| u.usage_kind == UsageKind::TypeReference)
        .collect();

    assert!(!type_refs.is_empty(), "Should find User as type reference");
}

// Phase 2: Test semantic context with method calls
#[test]
fn test_method_call_with_object_name() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("semantic_test.ts");

    // Test Date.now() detection
    let usages = find_usages_in_file(&mut parser, &file_path, "now", false, 3, None)
        .expect("Failed to find usages");

    let date_now: Vec<_> = usages
        .iter()
        .filter(|u| u.object_name.as_deref() == Some("Date"))
        .collect();

    assert!(!date_now.is_empty(), "Should find Date.now() calls");
    assert!(
        date_now
            .iter()
            .all(|u| u.usage_kind == UsageKind::MethodCall),
        "Date.now() should be a method call"
    );
}

// Phase 3: Test object filter
#[test]
fn test_object_filter() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("semantic_test.ts");

    // Get all 'now' usages
    let all_usages = find_usages_in_file(&mut parser, &file_path, "now", false, 3, None)
        .expect("Failed to find usages");

    // Get only Date.now() usages
    let date_usages = find_usages_in_file(&mut parser, &file_path, "now", false, 3, Some("Date"))
        .expect("Failed to find usages");

    // Date filter should return subset
    assert!(
        date_usages.len() <= all_usages.len(),
        "Filtered usages should be <= all usages"
    );

    // All filtered results should have object_name "Date"
    for usage in &date_usages {
        assert_eq!(
            usage.object_name.as_deref(),
            Some("Date"),
            "All filtered usages should have object_name 'Date'"
        );
    }
}

#[test]
fn test_object_filter_excludes_different_objects() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("semantic_test.ts");

    // Get only chronia.now() usages
    let chronia_usages =
        find_usages_in_file(&mut parser, &file_path, "now", false, 3, Some("chronia"))
            .expect("Failed to find usages");

    // All filtered results should have object_name "chronia"
    for usage in &chronia_usages {
        assert_eq!(
            usage.object_name.as_deref(),
            Some("chronia"),
            "All filtered usages should have object_name 'chronia'"
        );
    }
}

// Debug test to see actual output format
#[test]
fn test_output_format_debug() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("semantic_test.ts");

    let usages = find_usages_in_file(&mut parser, &file_path, "now", false, 1, None)
        .expect("Failed to find usages");

    println!("\n=== All 'now' usages ===");
    for u in &usages {
        println!(
            "Line {}: usage_kind={:?}, object_name={:?}",
            u.line, u.usage_kind, u.object_name
        );
    }

    let date_usages = find_usages_in_file(&mut parser, &file_path, "now", false, 1, Some("Date"))
        .expect("Failed to find usages");

    println!("\n=== Date.now() only ===");
    for u in &date_usages {
        println!(
            "Line {}: usage_kind={:?}, object_name={:?}",
            u.line, u.usage_kind, u.object_name
        );
    }
}

// ======================================
// New tool tests
// ======================================

// Test: find_method_calls equivalent behavior
#[test]
fn test_find_method_calls_behavior() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("semantic_test.ts");

    // Simulate find_method_calls: contexts なし、import なし、object_filter あり、MethodCall のみ
    let usages = find_usages_in_file(&mut parser, &file_path, "now", false, 0, Some("Date"))
        .expect("Failed to find usages");

    let method_calls: Vec<_> = usages
        .into_iter()
        .filter(|u| u.usage_kind == UsageKind::MethodCall)
        .collect();

    assert!(
        !method_calls.is_empty(),
        "Should find Date.now() method calls"
    );

    // All results should be MethodCall with object_name "Date"
    for usage in &method_calls {
        assert_eq!(usage.usage_kind, UsageKind::MethodCall);
        assert_eq!(usage.object_name.as_deref(), Some("Date"));
        // contexts should be empty (max_contexts = 0)
        assert!(usage.contexts.is_empty(), "contexts should be empty");
    }
}

// Test: find_imports equivalent behavior
#[test]
fn test_find_imports_behavior() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    // Simulate find_imports: contexts なし、import を含める、Import のみ
    let usages = find_usages_in_file(&mut parser, &file_path, "User", true, 0, None)
        .expect("Failed to find usages");

    let imports: Vec<_> = usages
        .into_iter()
        .filter(|u| u.usage_kind == UsageKind::Import)
        .collect();

    // All results should be Import
    for usage in &imports {
        assert_eq!(usage.usage_kind, UsageKind::Import);
        // contexts should be empty (max_contexts = 0)
        assert!(usage.contexts.is_empty(), "contexts should be empty");
    }
}

// Test: symbol_usages simplified behavior (include_contexts = false)
#[test]
fn test_symbol_usages_no_contexts() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    // Simulate symbol_usages with include_contexts = false (default)
    // max_contexts = 0, include_imports = true, object_filter = None
    let usages = find_usages_in_file(&mut parser, &file_path, "User", true, 0, None)
        .expect("Failed to find usages");

    assert!(!usages.is_empty(), "Should find User usages");

    // All results should have empty contexts
    for usage in &usages {
        assert!(
            usage.contexts.is_empty(),
            "contexts should be empty when include_contexts=false"
        );
    }
}

// Test: symbol_usages simplified behavior (include_contexts = true)
#[test]
fn test_symbol_usages_with_contexts() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    // Simulate symbol_usages with include_contexts = true
    // max_contexts = 2, include_imports = true, object_filter = None
    let usages = find_usages_in_file(&mut parser, &file_path, "User", true, 2, None)
        .expect("Failed to find usages");

    assert!(!usages.is_empty(), "Should find User usages");

    // At least some results should have contexts (depending on the usage location)
    let has_contexts = usages.iter().any(|u| !u.contexts.is_empty());
    assert!(
        has_contexts,
        "At least some usages should have contexts when include_contexts=true"
    );

    // All contexts should have at most 2 entries
    for usage in &usages {
        assert!(
            usage.contexts.len() <= 2,
            "contexts should have at most 2 entries"
        );
    }
}

// Test: contexts serialization (skip_serializing_if = "Vec::is_empty")
#[test]
fn test_contexts_serialization() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    // Get usages without contexts
    let usages = find_usages_in_file(&mut parser, &file_path, "User", true, 0, None)
        .expect("Failed to find usages");

    assert!(!usages.is_empty(), "Should find User usages");

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&usages).expect("Failed to serialize");

    // "contexts" should NOT appear in the JSON when empty (skip_serializing_if)
    assert!(
        !json.contains("\"contexts\""),
        "JSON should not contain 'contexts' when empty"
    );
}

// Test: contexts serialization with contexts present
#[test]
fn test_contexts_serialization_with_contexts() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    // Get usages with contexts
    let usages = find_usages_in_file(&mut parser, &file_path, "User", true, 2, None)
        .expect("Failed to find usages");

    assert!(!usages.is_empty(), "Should find User usages");

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&usages).expect("Failed to serialize");

    // "contexts" should appear in the JSON when present
    assert!(
        json.contains("\"contexts\""),
        "JSON should contain 'contexts' when present"
    );
}

// ======================================
// Comment functionality tests
// ======================================

#[test]
fn test_find_todo_comments() {
    let file_path = fixtures_path().join("jsdoc_test.ts");

    let matches = find_comments_in_file(&file_path, "TODO").expect("Failed to find comments");

    assert!(!matches.is_empty(), "Should find TODO comments");
    // Should find at least 2 TODO comments in jsdoc_test.ts
    assert!(matches.len() >= 2, "Should find at least 2 TODO comments");
}

#[test]
fn test_find_fixme_comments() {
    let file_path = fixtures_path().join("jsdoc_test.ts");

    let matches = find_comments_in_file(&file_path, "FIXME").expect("Failed to find comments");

    assert!(!matches.is_empty(), "Should find FIXME comments");
}

#[test]
fn test_comment_types() {
    let file_path = fixtures_path().join("jsdoc_test.ts");

    let matches = find_comments_in_file(&file_path, "TODO").expect("Failed to find comments");

    // Should have both SingleLine and Block comments
    let has_single_line = matches
        .iter()
        .any(|m| m.comment_type == CommentType::SingleLine);
    let has_block = matches.iter().any(|m| m.comment_type == CommentType::Block);

    assert!(
        has_single_line || has_block,
        "Should find at least one comment type"
    );
}

#[test]
fn test_definition_with_docs() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("jsdoc_test.ts");

    // Find Product interface with docs
    let definitions = find_definitions_in_file(&mut parser, &file_path, "Product", true)
        .expect("Failed to find definitions");

    assert!(!definitions.is_empty(), "Should find Product interface");
    let product_def = &definitions[0];

    // Should have JSDoc
    assert!(product_def.docs.is_some(), "Product should have docs");
    let docs = product_def.docs.as_ref().unwrap();
    assert!(
        docs.contains("Represents a product"),
        "Docs should contain description"
    );
}

#[test]
fn test_definition_with_docs_class() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("jsdoc_test.ts");

    // Find ProductService class with docs
    let definitions = find_definitions_in_file(&mut parser, &file_path, "ProductService", true)
        .expect("Failed to find definitions");

    assert!(!definitions.is_empty(), "Should find ProductService class");
    let class_def = &definitions[0];

    // Should have JSDoc
    assert!(class_def.docs.is_some(), "ProductService should have docs");
    let docs = class_def.docs.as_ref().unwrap();
    assert!(docs.contains("@example"), "Docs should contain @example");
}

#[test]
fn test_definition_without_docs() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("jsdoc_test.ts");

    // Find Product interface without docs (include_docs = false)
    let definitions = find_definitions_in_file(&mut parser, &file_path, "Product", false)
        .expect("Failed to find definitions");

    assert!(!definitions.is_empty(), "Should find Product interface");
    let product_def = &definitions[0];

    // Should NOT have docs
    assert!(
        product_def.docs.is_none(),
        "Product should not have docs when include_docs=false"
    );
}

#[test]
fn test_definition_docs_serialization() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("jsdoc_test.ts");

    // Get definitions without docs
    let definitions_no_docs = find_definitions_in_file(&mut parser, &file_path, "Product", false)
        .expect("Failed to find definitions");

    let json_no_docs =
        serde_json::to_string_pretty(&definitions_no_docs).expect("Failed to serialize");

    // "docs" should NOT appear when None (skip_serializing_if)
    assert!(
        !json_no_docs.contains("\"docs\""),
        "JSON should not contain 'docs' when None"
    );

    // Get definitions with docs
    let definitions_with_docs = find_definitions_in_file(&mut parser, &file_path, "Product", true)
        .expect("Failed to find definitions");

    let json_with_docs =
        serde_json::to_string_pretty(&definitions_with_docs).expect("Failed to serialize");

    // "docs" should appear when Some
    assert!(
        json_with_docs.contains("\"docs\""),
        "JSON should contain 'docs' when present"
    );
}

#[test]
fn test_single_line_comment_docs() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("jsdoc_test.ts");

    // Find formatPrice function which has single-line comments above it
    let definitions = find_definitions_in_file(&mut parser, &file_path, "formatPrice", true)
        .expect("Failed to find definitions");

    assert!(!definitions.is_empty(), "Should find formatPrice function");
    let func_def = &definitions[0];

    // Should have single-line comment docs
    assert!(func_def.docs.is_some(), "formatPrice should have docs");
    let docs = func_def.docs.as_ref().unwrap();
    assert!(
        docs.contains("Helper function") || docs.contains("formatted price"),
        "Docs should contain comment text"
    );
}

// ======================================
// get_code_at_location tests
// ======================================

#[test]
fn test_get_code_at_location_basic() {
    let file_path = fixtures_path().join("sample.ts");

    let snippet = get_code_at_location(&file_path, 5, 2, 2).expect("Failed to get code");

    assert_eq!(snippet.file_path, file_path.to_string_lossy());
    assert_eq!(snippet.start_line, 3); // 5 - 2 = 3
    assert_eq!(snippet.end_line, 7); // 5 + 2 = 7
    assert!(!snippet.code.is_empty(), "Code should not be empty");
}

#[test]
fn test_get_code_at_location_at_start() {
    let file_path = fixtures_path().join("sample.ts");

    // Line 1 with context_before=5 should clamp to start
    let snippet = get_code_at_location(&file_path, 1, 5, 2).expect("Failed to get code");

    assert_eq!(snippet.start_line, 1); // Clamped to 1
    assert_eq!(snippet.end_line, 3); // 1 + 2 = 3
}

#[test]
fn test_get_code_at_location_at_end() {
    let file_path = fixtures_path().join("sample.ts");

    // Read file to get total lines
    let source = std::fs::read_to_string(&file_path).expect("Failed to read file");
    let total_lines = source.lines().count();

    // Request past end of file should clamp
    let snippet =
        get_code_at_location(&file_path, total_lines, 2, 100).expect("Failed to get code");

    assert_eq!(snippet.end_line, total_lines); // Clamped to total lines
}

#[test]
fn test_get_code_at_location_default_context() {
    let file_path = fixtures_path().join("sample.ts");

    // Default context (3 lines before and after)
    let snippet = get_code_at_location(&file_path, 10, 3, 3).expect("Failed to get code");

    assert_eq!(snippet.start_line, 7); // 10 - 3 = 7
    assert_eq!(snippet.end_line, 13); // 10 + 3 = 13
}

#[test]
fn test_get_code_at_location_serialization() {
    let file_path = fixtures_path().join("sample.ts");

    let snippet = get_code_at_location(&file_path, 5, 2, 2).expect("Failed to get code");

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&snippet).expect("Failed to serialize");

    // Should contain all expected fields
    assert!(
        json.contains("\"file_path\""),
        "JSON should contain file_path"
    );
    assert!(
        json.contains("\"start_line\""),
        "JSON should contain start_line"
    );
    assert!(
        json.contains("\"end_line\""),
        "JSON should contain end_line"
    );
    assert!(json.contains("\"code\""), "JSON should contain code");
}

// ======================================
// qualified_name tests
// ======================================

#[test]
fn test_qualified_name_simple_identifier() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("semantic_test.ts");

    // Get usages without object filter
    let usages = find_usages_in_file(&mut parser, &file_path, "now", false, 0, None)
        .expect("Failed to find usages");

    assert!(!usages.is_empty(), "Should find 'now' usages");

    // Check that Date.now() has qualified_name "Date.now"
    let date_now_usages: Vec<_> = usages
        .iter()
        .filter(|u| u.object_name.as_deref() == Some("Date"))
        .collect();

    assert!(!date_now_usages.is_empty(), "Should find Date.now() usages");
    for usage in &date_now_usages {
        assert_eq!(
            usage.qualified_name, "Date.now",
            "qualified_name should be 'Date.now'"
        );
    }

    // Check that chronia.now() has qualified_name "chronia.now"
    let chronia_now_usages: Vec<_> = usages
        .iter()
        .filter(|u| u.object_name.as_deref() == Some("chronia"))
        .collect();

    assert!(
        !chronia_now_usages.is_empty(),
        "Should find chronia.now() usages"
    );
    for usage in &chronia_now_usages {
        assert_eq!(
            usage.qualified_name, "chronia.now",
            "qualified_name should be 'chronia.now'"
        );
    }
}

#[test]
fn test_qualified_name_without_object() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    // Get usages of 'User' which should be used as type reference (no object)
    let usages = find_usages_in_file(&mut parser, &file_path, "User", false, 0, None)
        .expect("Failed to find usages");

    assert!(!usages.is_empty(), "Should find 'User' usages");

    // All usages without object_name should have qualified_name = "User"
    for usage in &usages {
        if usage.object_name.is_none() {
            assert_eq!(
                usage.qualified_name, "User",
                "qualified_name should be 'User' when no object"
            );
        }
    }
}

#[test]
fn test_qualified_name_serialization() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("semantic_test.ts");

    let usages = find_usages_in_file(&mut parser, &file_path, "now", false, 0, None)
        .expect("Failed to find usages");

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&usages).expect("Failed to serialize");

    // Should always contain qualified_name
    assert!(
        json.contains("\"qualified_name\""),
        "JSON should always contain 'qualified_name'"
    );
    assert!(
        json.contains("\"Date.now\"") || json.contains("\"chronia.now\""),
        "JSON should contain qualified names like 'Date.now' or 'chronia.now'"
    );
}

#[test]
fn test_qualified_name_distinguishes_same_method() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("semantic_test.ts");

    // Get all 'now' usages
    let usages = find_usages_in_file(&mut parser, &file_path, "now", false, 0, None)
        .expect("Failed to find usages");

    // Group by qualified_name
    let date_now_count = usages
        .iter()
        .filter(|u| u.qualified_name == "Date.now")
        .count();
    let chronia_now_count = usages
        .iter()
        .filter(|u| u.qualified_name == "chronia.now")
        .count();

    // Both should have usages
    assert!(date_now_count > 0, "Should have Date.now usages");
    assert!(chronia_now_count > 0, "Should have chronia.now usages");

    println!("Date.now usages: {}", date_now_count);
    println!("chronia.now usages: {}", chronia_now_count);

    // The qualified_name allows clear distinction
    for usage in &usages {
        println!(
            "Line {}: qualified_name={}",
            usage.line, usage.qualified_name
        );
    }
}

// ======================================
// get_symbol_at_location tests
// ======================================

#[test]
fn test_find_symbol_at_location_method() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    // Line 17 is inside addUser method (lines 16-18)
    let symbol =
        find_symbol_at_location(&mut parser, &file_path, 17).expect("Failed to find symbol");

    assert!(symbol.is_some(), "Should find addUser method at line 17");
    let symbol = symbol.unwrap();
    assert_eq!(symbol.name, "addUser", "Symbol name should be 'addUser'");
    assert_eq!(
        symbol.node_kind.to_string(),
        "Method",
        "Symbol should be a method"
    );
    assert_eq!(symbol.start_line, 16, "Method should start at line 16");
    assert_eq!(symbol.end_line, 18, "Method should end at line 18");
}

#[test]
fn test_find_symbol_at_location_constructor() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    // Line 12 is inside constructor (lines 12-14)
    let symbol =
        find_symbol_at_location(&mut parser, &file_path, 12).expect("Failed to find symbol");

    assert!(symbol.is_some(), "Should find constructor at line 12");
    let symbol = symbol.unwrap();
    assert_eq!(
        symbol.name, "constructor",
        "Symbol name should be 'constructor'"
    );
    assert_eq!(
        symbol.node_kind.to_string(),
        "Constructor",
        "Symbol should be a constructor"
    );
    assert_eq!(symbol.start_line, 12, "Constructor should start at line 12");
    assert_eq!(symbol.end_line, 14, "Constructor should end at line 14");
}

#[test]
fn test_find_symbol_at_location_function() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    // Line 38 is inside processUser function (lines 37-39)
    let symbol =
        find_symbol_at_location(&mut parser, &file_path, 38).expect("Failed to find symbol");

    assert!(
        symbol.is_some(),
        "Should find processUser function at line 38"
    );
    let symbol = symbol.unwrap();
    assert_eq!(
        symbol.name, "processUser",
        "Symbol name should be 'processUser'"
    );
    assert_eq!(
        symbol.node_kind.to_string(),
        "Function",
        "Symbol should be a function"
    );
    assert_eq!(symbol.start_line, 37, "Function should start at line 37");
    assert_eq!(symbol.end_line, 39, "Function should end at line 39");
}

#[test]
fn test_find_symbol_at_location_arrow_function() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    // Line 30 is inside createUser arrow function (lines 29-35)
    let symbol =
        find_symbol_at_location(&mut parser, &file_path, 30).expect("Failed to find symbol");

    assert!(symbol.is_some(), "Should find createUser at line 30");
    let symbol = symbol.unwrap();
    assert_eq!(
        symbol.name, "createUser",
        "Symbol name should be 'createUser'"
    );
    assert_eq!(
        symbol.node_kind.to_string(),
        "ArrowFunction",
        "Symbol should be an arrow function"
    );
    assert_eq!(
        symbol.start_line, 29,
        "Arrow function should start at line 29"
    );
    assert_eq!(symbol.end_line, 35, "Arrow function should end at line 35");
}

#[test]
fn test_find_symbol_at_location_class() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    // Line 10 is inside UserService class (lines 9-27) but outside methods
    let symbol =
        find_symbol_at_location(&mut parser, &file_path, 10).expect("Failed to find symbol");

    assert!(symbol.is_some(), "Should find symbol at line 10");
    let symbol = symbol.unwrap();
    // Should be UserService class since line 10 is the property declaration
    // which is not captured as a symbol
    assert_eq!(symbol.name, "UserService", "Should find UserService class");
}

#[test]
fn test_find_symbol_at_location_interface() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    // Line 4 is inside User interface (lines 3-7)
    let symbol =
        find_symbol_at_location(&mut parser, &file_path, 4).expect("Failed to find symbol");

    assert!(symbol.is_some(), "Should find User interface at line 4");
    let symbol = symbol.unwrap();
    assert_eq!(symbol.name, "User", "Symbol name should be 'User'");
    assert_eq!(
        symbol.node_kind.to_string(),
        "Interface",
        "Symbol should be an interface"
    );
}

#[test]
fn test_find_symbol_at_location_smallest_enclosing() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    // Line 21 is inside findUser method which is inside UserService class
    // Should return findUser (the smallest enclosing symbol)
    let symbol =
        find_symbol_at_location(&mut parser, &file_path, 21).expect("Failed to find symbol");

    assert!(symbol.is_some(), "Should find findUser method at line 21");
    let symbol = symbol.unwrap();
    assert_eq!(
        symbol.name, "findUser",
        "Should find the smallest enclosing symbol (findUser method)"
    );
    assert_eq!(symbol.node_kind.to_string(), "Method");
}

#[test]
fn test_find_symbol_at_location_outside_symbols() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    // Line 1 is a comment outside any symbol
    let symbol =
        find_symbol_at_location(&mut parser, &file_path, 1).expect("Failed to find symbol");

    assert!(
        symbol.is_none(),
        "Should not find symbol at line 1 (comment)"
    );
}

/// Debug test for manual inspection of symbol detection at each line.
/// Run with: cargo test test_find_symbol_at_location_debug -- --ignored --nocapture
#[test]
#[ignore]
fn test_find_symbol_at_location_debug() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    println!("\n=== Debug: Symbol at each line ===");
    for line in 1..=42 {
        let symbol =
            find_symbol_at_location(&mut parser, &file_path, line).expect("Failed to find symbol");
        match symbol {
            Some(s) => println!(
                "Line {}: {} ({}) [{}-{}]",
                line, s.name, s.node_kind, s.start_line, s.end_line
            ),
            None => println!("Line {}: <none>", line),
        }
    }
}

// ======================================
// JavaScript/JSX tests
// ======================================

/// Helper function to find symbol definitions using GenericParser
fn find_js_definitions(file_path: &std::path::Path, symbol_name: &str) -> Vec<(String, String)> {
    let registry = Arc::new(LanguageRegistry::new().expect("Failed to create registry"));
    let mut parser = GenericParser::new(registry).expect("Failed to create parser");

    let source_code = std::fs::read_to_string(file_path).expect("Failed to read file");
    let (tree, lang) = parser
        .parse_with_language(file_path, &source_code)
        .expect("Failed to parse file");

    let query = lang.definitions_query();
    let mut cursor = tree_sitter::QueryCursor::new();

    use streaming_iterator::StreamingIterator;
    let mut matches = cursor.matches(query, tree.root_node(), source_code.as_bytes());

    // Collect all matches first, then deduplicate preferring more specific types
    let mut all_results: Vec<(String, String, usize, usize)> = Vec::new(); // (name, kind, start_row, start_col)
    while let Some(m) = matches.next() {
        let mut name: Option<String> = None;
        let mut kind: Option<String> = None;
        let mut start_row = 0;
        let mut start_col = 0;

        for capture in m.captures {
            let capture_name = &query.capture_names()[capture.index as usize];
            if *capture_name == "name" {
                name = Some(
                    capture
                        .node
                        .utf8_text(source_code.as_bytes())
                        .unwrap_or("")
                        .to_string(),
                );
                start_row = capture.node.start_position().row;
                start_col = capture.node.start_position().column;
            } else if capture_name.starts_with("definition.") {
                kind = Some(capture_name.replace("definition.", ""));
            }
        }

        if let (Some(n), Some(k)) = (name, kind) {
            if n == symbol_name {
                all_results.push((n, k, start_row, start_col));
            }
        }
    }

    // Deduplicate: prefer arrow_function over variable at the same location
    use std::collections::HashMap;
    let mut deduped: HashMap<(usize, usize), (String, String)> = HashMap::new();
    for (name, kind, row, col) in all_results {
        let key = (row, col);
        if let Some((_, existing_kind)) = deduped.get(&key) {
            // Prefer arrow_function over variable
            if existing_kind == "variable" && kind == "arrow_function" {
                deduped.insert(key, (name, kind));
            }
        } else {
            deduped.insert(key, (name, kind));
        }
    }

    deduped.into_values().collect()
}

/// Helper function to find symbol usages using GenericParser
fn find_js_usages(file_path: &std::path::Path, symbol_name: &str) -> Vec<String> {
    let registry = Arc::new(LanguageRegistry::new().expect("Failed to create registry"));
    let mut parser = GenericParser::new(registry).expect("Failed to create parser");

    let source_code = std::fs::read_to_string(file_path).expect("Failed to read file");
    let (tree, lang) = parser
        .parse_with_language(file_path, &source_code)
        .expect("Failed to parse file");

    let query = lang.usages_query();
    let mut cursor = tree_sitter::QueryCursor::new();

    use streaming_iterator::StreamingIterator;
    let mut matches = cursor.matches(query, tree.root_node(), source_code.as_bytes());

    let mut results = Vec::new();
    while let Some(m) = matches.next() {
        for capture in m.captures {
            let capture_name = &query.capture_names()[capture.index as usize];
            if *capture_name == "usage" {
                let usage_text = capture
                    .node
                    .utf8_text(source_code.as_bytes())
                    .unwrap_or("")
                    .to_string();
                if usage_text == symbol_name {
                    results.push(usage_text);
                }
            }
        }
    }

    results
}

#[test]
fn test_js_find_class_definition() {
    let file_path = fixtures_path().join("sample.js");
    let definitions = find_js_definitions(&file_path, "UserService");

    assert!(!definitions.is_empty(), "Should find UserService class");
    assert_eq!(definitions[0].0, "UserService");
    assert_eq!(definitions[0].1, "class");
}

#[test]
fn test_js_find_function_definition() {
    let file_path = fixtures_path().join("sample.js");
    let definitions = find_js_definitions(&file_path, "processUser");

    assert!(!definitions.is_empty(), "Should find processUser function");
    assert_eq!(definitions[0].0, "processUser");
    assert_eq!(definitions[0].1, "function");
}

#[test]
fn test_js_find_arrow_function_definition() {
    let file_path = fixtures_path().join("sample.js");
    let definitions = find_js_definitions(&file_path, "createUser");

    assert!(
        !definitions.is_empty(),
        "Should find createUser arrow function"
    );
    assert_eq!(definitions[0].0, "createUser");
    assert_eq!(
        definitions[0].1, "arrow_function",
        "Should be arrow_function kind"
    );
}

#[test]
fn test_js_find_exported_arrow_function() {
    let file_path = fixtures_path().join("sample.js");
    let definitions = find_js_definitions(&file_path, "formatName");

    assert!(
        !definitions.is_empty(),
        "Should find formatName exported arrow function"
    );
    assert_eq!(definitions[0].0, "formatName");
    assert_eq!(
        definitions[0].1, "arrow_function",
        "Exported arrow function should be arrow_function kind"
    );
}

#[test]
fn test_js_find_exported_function() {
    let file_path = fixtures_path().join("sample.js");
    let definitions = find_js_definitions(&file_path, "validateEmail");

    assert!(
        !definitions.is_empty(),
        "Should find validateEmail exported function"
    );
    assert_eq!(definitions[0].0, "validateEmail");
    assert_eq!(definitions[0].1, "function");
}

#[test]
fn test_js_find_exported_class() {
    let file_path = fixtures_path().join("sample.js");
    let definitions = find_js_definitions(&file_path, "Logger");

    assert!(!definitions.is_empty(), "Should find Logger exported class");
    assert_eq!(definitions[0].0, "Logger");
    assert_eq!(definitions[0].1, "class");
}

#[test]
fn test_js_find_var_variable() {
    let file_path = fixtures_path().join("sample.js");
    let definitions = find_js_definitions(&file_path, "globalConfig");

    assert!(
        !definitions.is_empty(),
        "Should find globalConfig var variable"
    );
    assert_eq!(definitions[0].0, "globalConfig");
    assert_eq!(definitions[0].1, "variable");
}

#[test]
fn test_js_find_constructor() {
    let file_path = fixtures_path().join("sample.js");
    let definitions = find_js_definitions(&file_path, "constructor");

    assert!(!definitions.is_empty(), "Should find constructors");
    // Should find constructors for UserService and Logger
    let constructor_defs: Vec<_> = definitions
        .iter()
        .filter(|(_, k)| k == "constructor")
        .collect();
    assert!(
        constructor_defs.len() >= 2,
        "Should find at least 2 constructors"
    );
}

#[test]
fn test_js_find_method() {
    let file_path = fixtures_path().join("sample.js");
    let definitions = find_js_definitions(&file_path, "addUser");

    assert!(!definitions.is_empty(), "Should find addUser method");
    assert_eq!(definitions[0].0, "addUser");
    assert_eq!(definitions[0].1, "method");
}

#[test]
fn test_js_find_usages() {
    let file_path = fixtures_path().join("sample.js");
    let usages = find_js_usages(&file_path, "console");

    assert!(!usages.is_empty(), "Should find console usages");
}

#[test]
fn test_js_find_todo_comments() {
    let file_path = fixtures_path().join("sample.js");

    let matches = find_comments_in_file(&file_path, "TODO").expect("Failed to find comments");

    assert!(!matches.is_empty(), "Should find TODO comments in JS file");
}

#[test]
fn test_js_generic_parser_handles_js_files() {
    let registry = Arc::new(LanguageRegistry::new().expect("Failed to create registry"));
    let mut parser = GenericParser::new(registry).expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.js");

    let source_code = std::fs::read_to_string(&file_path).expect("Failed to read file");
    let result = parser.parse_with_language(&file_path, &source_code);

    assert!(result.is_ok(), "GenericParser should handle .js files");
    let (tree, lang) = result.unwrap();
    assert!(
        tree.root_node().child_count() > 0,
        "Should produce valid AST with children"
    );
    assert_eq!(
        lang.name(),
        "JavaScript",
        "Should use JavaScript language support"
    );
}

// JSX tests

#[test]
fn test_jsx_find_class_component() {
    let file_path = fixtures_path().join("sample.jsx");
    let definitions = find_js_definitions(&file_path, "UserCard");

    assert!(!definitions.is_empty(), "Should find UserCard class");
    assert_eq!(definitions[0].0, "UserCard");
    assert_eq!(definitions[0].1, "class");
}

#[test]
fn test_jsx_find_function_component() {
    let file_path = fixtures_path().join("sample.jsx");
    let definitions = find_js_definitions(&file_path, "UserProfile");

    assert!(
        !definitions.is_empty(),
        "Should find UserProfile function component"
    );
    assert_eq!(definitions[0].0, "UserProfile");
    assert_eq!(definitions[0].1, "function");
}

#[test]
fn test_jsx_find_arrow_function_component() {
    let file_path = fixtures_path().join("sample.jsx");
    let definitions = find_js_definitions(&file_path, "UserList");

    assert!(
        !definitions.is_empty(),
        "Should find UserList arrow function component"
    );
    assert_eq!(definitions[0].0, "UserList");
    assert_eq!(definitions[0].1, "arrow_function");
}

#[test]
fn test_jsx_find_exported_arrow_function_component() {
    let file_path = fixtures_path().join("sample.jsx");
    let definitions = find_js_definitions(&file_path, "Avatar");

    assert!(
        !definitions.is_empty(),
        "Should find Avatar exported arrow function component"
    );
    assert_eq!(definitions[0].0, "Avatar");
    assert_eq!(
        definitions[0].1, "arrow_function",
        "Exported arrow function component should be arrow_function kind"
    );
}

#[test]
fn test_jsx_find_method() {
    let file_path = fixtures_path().join("sample.jsx");
    let definitions = find_js_definitions(&file_path, "toggleExpand");

    assert!(!definitions.is_empty(), "Should find toggleExpand method");
    assert_eq!(definitions[0].0, "toggleExpand");
    assert_eq!(definitions[0].1, "method");
}

#[test]
fn test_jsx_generic_parser_handles_jsx_files() {
    let registry = Arc::new(LanguageRegistry::new().expect("Failed to create registry"));
    let mut parser = GenericParser::new(registry).expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.jsx");

    let source_code = std::fs::read_to_string(&file_path).expect("Failed to read file");
    let result = parser.parse_with_language(&file_path, &source_code);

    assert!(result.is_ok(), "GenericParser should handle .jsx files");
    let (tree, lang) = result.unwrap();
    assert!(
        tree.root_node().child_count() > 0,
        "Should produce valid AST with children"
    );
    assert_eq!(
        lang.name(),
        "JavaScriptReact",
        "Should use JavaScriptReact language support"
    );
}

#[test]
fn test_jsx_find_usages() {
    let file_path = fixtures_path().join("sample.jsx");
    let usages = find_js_usages(&file_path, "user");

    assert!(!usages.is_empty(), "Should find 'user' usages in JSX file");
}

// ======================================
// Markdown tests
// ======================================

#[test]
fn test_markdown_text_search() {
    let file_path = fixtures_path().join("sample.md");

    let matches = find_text_in_markdown_file(&file_path, "Installation").unwrap();

    assert!(
        !matches.is_empty(),
        "Should find 'Installation' in markdown"
    );
    // Should find the heading at line 14
    assert!(
        matches.iter().any(|m| m.line == 14),
        "Should find Installation heading at line 14"
    );
}

#[test]
fn test_markdown_code_block_search() {
    let file_path = fixtures_path().join("sample.md");

    let matches = find_text_in_markdown_file(&file_path, "npm install").unwrap();

    assert!(
        !matches.is_empty(),
        "Should find 'npm install' in code block"
    );
}

#[test]
fn test_markdown_link_reference_search() {
    let file_path = fixtures_path().join("sample.md");

    let matches = find_text_in_markdown_file(&file_path, "example.com").unwrap();

    assert!(
        !matches.is_empty(),
        "Should find 'example.com' in link references"
    );
}

#[test]
fn test_markdown_heading_detection_with_generic_parser() {
    let registry = Arc::new(LanguageRegistry::new().expect("Failed to create registry"));
    let mut parser = GenericParser::new(registry).expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.md");

    let source_code = std::fs::read_to_string(&file_path).expect("Failed to read file");
    let (tree, lang) = parser
        .parse_with_language(&file_path, &source_code)
        .expect("Failed to parse file");

    let query = lang.definitions_query();
    let mut cursor = tree_sitter::QueryCursor::new();

    use streaming_iterator::StreamingIterator;
    let mut matches = cursor.matches(query, tree.root_node(), source_code.as_bytes());

    let mut headings = Vec::new();
    while let Some(m) = matches.next() {
        for capture in m.captures {
            let capture_name = &query.capture_names()[capture.index as usize];
            if capture_name.starts_with("definition.heading") {
                let text = capture.node.utf8_text(source_code.as_bytes()).unwrap_or("");
                headings.push((capture_name.to_string(), text.to_string()));
            }
        }
    }

    // Should find multiple headings
    assert!(!headings.is_empty(), "Should find headings in markdown");

    // Check for specific heading levels
    let has_h1 = headings
        .iter()
        .any(|(name, _)| name == "definition.heading1");
    let has_h2 = headings
        .iter()
        .any(|(name, _)| name == "definition.heading2");
    let has_h3 = headings
        .iter()
        .any(|(name, _)| name == "definition.heading3");

    assert!(has_h1, "Should find H1 heading");
    assert!(has_h2, "Should find H2 headings");
    assert!(has_h3, "Should find H3 headings");
}

#[test]
fn test_markdown_code_block_with_language() {
    let registry = Arc::new(LanguageRegistry::new().expect("Failed to create registry"));
    let mut parser = GenericParser::new(registry).expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.md");

    let source_code = std::fs::read_to_string(&file_path).expect("Failed to read file");
    let (tree, lang) = parser
        .parse_with_language(&file_path, &source_code)
        .expect("Failed to parse file");

    let query = lang.definitions_query();
    let mut cursor = tree_sitter::QueryCursor::new();

    use streaming_iterator::StreamingIterator;
    let mut matches = cursor.matches(query, tree.root_node(), source_code.as_bytes());

    let mut code_blocks = Vec::new();
    while let Some(m) = matches.next() {
        let mut name: Option<String> = None;
        let mut is_code_block = false;

        for capture in m.captures {
            let capture_name = &query.capture_names()[capture.index as usize];
            if *capture_name == "name" {
                name = Some(
                    capture
                        .node
                        .utf8_text(source_code.as_bytes())
                        .unwrap_or("")
                        .to_string(),
                );
            }
            if *capture_name == "definition.code_block" {
                is_code_block = true;
            }
        }

        if is_code_block {
            if let Some(lang_name) = name {
                code_blocks.push(lang_name);
            }
        }
    }

    // Should find code blocks with language specification
    assert!(
        !code_blocks.is_empty(),
        "Should find code blocks with language"
    );
    assert!(
        code_blocks.contains(&"bash".to_string()),
        "Should find bash code block"
    );
    assert!(
        code_blocks.contains(&"typescript".to_string()),
        "Should find typescript code block"
    );
}

#[test]
fn test_markdown_link_reference_definition() {
    let registry = Arc::new(LanguageRegistry::new().expect("Failed to create registry"));
    let mut parser = GenericParser::new(registry).expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.md");

    let source_code = std::fs::read_to_string(&file_path).expect("Failed to read file");
    let (tree, lang) = parser
        .parse_with_language(&file_path, &source_code)
        .expect("Failed to parse file");

    let query = lang.definitions_query();
    let mut cursor = tree_sitter::QueryCursor::new();

    use streaming_iterator::StreamingIterator;
    let mut matches = cursor.matches(query, tree.root_node(), source_code.as_bytes());

    let mut link_refs = Vec::new();
    while let Some(m) = matches.next() {
        for capture in m.captures {
            let capture_name = &query.capture_names()[capture.index as usize];
            if *capture_name == "definition.link" {
                let text = capture.node.utf8_text(source_code.as_bytes()).unwrap_or("");
                link_refs.push(text.to_string());
            }
        }
    }

    // Should find link reference definitions
    assert!(
        !link_refs.is_empty(),
        "Should find link reference definitions"
    );
}

// ======================================
// Common helper for HTML/CSS tests
// ======================================

/// Generic helper function to find symbol definitions using GenericParser.
/// This is a common helper that can be reused across different language tests.
fn find_definitions(file_path: &std::path::Path, symbol_name: &str) -> Vec<(String, String)> {
    let registry = Arc::new(LanguageRegistry::new().expect("Failed to create registry"));
    let mut parser = GenericParser::new(registry).expect("Failed to create parser");

    let source_code = std::fs::read_to_string(file_path).expect("Failed to read file");
    let (tree, lang) = parser
        .parse_with_language(file_path, &source_code)
        .expect("Failed to parse file");

    let query = lang.definitions_query();
    let mut cursor = tree_sitter::QueryCursor::new();

    use streaming_iterator::StreamingIterator;
    let mut matches = cursor.matches(query, tree.root_node(), source_code.as_bytes());

    let mut results = Vec::new();
    while let Some(m) = matches.next() {
        let mut name: Option<String> = None;
        let mut kind: Option<String> = None;

        for capture in m.captures {
            let capture_name = &query.capture_names()[capture.index as usize];
            if *capture_name == "name" {
                name = Some(
                    capture
                        .node
                        .utf8_text(source_code.as_bytes())
                        .unwrap_or("")
                        .to_string(),
                );
            } else if capture_name.starts_with("definition.") {
                kind = Some(capture_name.replace("definition.", ""));
            }
        }

        if let (Some(n), Some(k)) = (name, kind) {
            if n == symbol_name {
                results.push((n, k));
            }
        }
    }

    results
}

// ======================================
// HTML tests
// ======================================

#[test]
fn test_html_find_element() {
    let file_path = fixtures_path().join("sample.html");
    let definitions = find_definitions(&file_path, "header");

    assert!(!definitions.is_empty(), "Should find header element");
    assert_eq!(definitions[0].0, "header");
    assert_eq!(definitions[0].1, "element");
}

#[test]
fn test_html_find_self_closing_element() {
    let file_path = fixtures_path().join("sample.html");
    let definitions = find_definitions(&file_path, "meta");

    assert!(
        !definitions.is_empty(),
        "Should find self-closing meta element"
    );
    assert_eq!(definitions[0].0, "meta");
    assert_eq!(definitions[0].1, "element");
}

#[test]
fn test_html_find_various_elements() {
    let file_path = fixtures_path().join("sample.html");

    // Test various HTML elements
    let html_defs = find_definitions(&file_path, "html");
    assert!(!html_defs.is_empty(), "Should find html element");

    let body_defs = find_definitions(&file_path, "body");
    assert!(!body_defs.is_empty(), "Should find body element");

    let div_defs = find_definitions(&file_path, "div");
    assert!(!div_defs.is_empty(), "Should find div element");

    let span_defs = find_definitions(&file_path, "span");
    assert!(!span_defs.is_empty(), "Should find span element");

    let footer_defs = find_definitions(&file_path, "footer");
    assert!(!footer_defs.is_empty(), "Should find footer element");
}

#[test]
fn test_html_find_id_attribute() {
    let file_path = fixtures_path().join("sample.html");
    let definitions = find_definitions(&file_path, "main-header");

    assert!(!definitions.is_empty(), "Should find main-header id");
    assert_eq!(definitions[0].0, "main-header");
    assert_eq!(definitions[0].1, "id");
}

#[test]
fn test_html_find_various_ids() {
    let file_path = fixtures_path().join("sample.html");

    // Test various ID attributes
    let content_defs = find_definitions(&file_path, "content");
    assert!(!content_defs.is_empty(), "Should find content id");
    assert_eq!(content_defs[0].1, "id");

    let user_card_defs = find_definitions(&file_path, "user-card");
    assert!(!user_card_defs.is_empty(), "Should find user-card id");
    assert_eq!(user_card_defs[0].1, "id");

    let footer_defs = find_definitions(&file_path, "footer");
    // footer is both an element and an id
    let footer_ids: Vec<_> = footer_defs.iter().filter(|(_, k)| k == "id").collect();
    assert!(!footer_ids.is_empty(), "Should find footer id");
}

#[test]
fn test_html_find_class_attribute() {
    let file_path = fixtures_path().join("sample.html");
    let definitions = find_definitions(&file_path, "header-container");

    assert!(
        !definitions.is_empty(),
        "Should find header-container class"
    );
    assert_eq!(definitions[0].0, "header-container");
    assert_eq!(definitions[0].1, "class");
}

#[test]
fn test_html_find_various_classes() {
    let file_path = fixtures_path().join("sample.html");

    // Test various class attributes
    let main_content_defs = find_definitions(&file_path, "main-content");
    assert!(
        !main_content_defs.is_empty(),
        "Should find main-content class"
    );
    assert_eq!(main_content_defs[0].1, "class");

    let username_defs = find_definitions(&file_path, "username");
    assert!(!username_defs.is_empty(), "Should find username class");
    assert_eq!(username_defs[0].1, "class");

    let footer_container_defs = find_definitions(&file_path, "footer-container");
    assert!(
        !footer_container_defs.is_empty(),
        "Should find footer-container class"
    );
    assert_eq!(footer_container_defs[0].1, "class");
}

#[test]
fn test_html_multi_class_limitation() {
    // This test documents the known limitation where multi-class attributes
    // are captured as a single value rather than separate classes.
    let file_path = fixtures_path().join("sample.html");

    // "card user-card" is captured as a single class definition
    let combined_class = find_definitions(&file_path, "card user-card");
    assert!(
        !combined_class.is_empty(),
        "Multi-class attribute should be captured as single value"
    );

    // Individual class "card" is NOT found separately (known limitation)
    let single_card = find_definitions(&file_path, "card");
    // This will be empty because we capture the entire attribute value
    assert!(
        single_card.is_empty(),
        "Individual class 'card' not found separately (known limitation)"
    );
}

#[test]
fn test_html_generic_parser_handles_html_files() {
    let registry = Arc::new(LanguageRegistry::new().expect("Failed to create registry"));
    let mut parser = GenericParser::new(registry).expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.html");

    let source_code = std::fs::read_to_string(&file_path).expect("Failed to read file");
    let result = parser.parse_with_language(&file_path, &source_code);

    assert!(result.is_ok(), "GenericParser should handle .html files");
    let (tree, lang) = result.unwrap();
    assert!(
        tree.root_node().child_count() > 0,
        "Should produce valid AST with children"
    );
    assert_eq!(lang.name(), "Html", "Should use Html language support");
}

// ======================================
// CSS tests
// ======================================

#[test]
fn test_css_find_class_selector() {
    let file_path = fixtures_path().join("sample.css");
    let definitions = find_definitions(&file_path, "header-container");

    assert!(
        !definitions.is_empty(),
        "Should find header-container class selector"
    );
    assert_eq!(definitions[0].0, "header-container");
    assert_eq!(definitions[0].1, "class_selector");
}

#[test]
fn test_css_find_various_class_selectors() {
    let file_path = fixtures_path().join("sample.css");

    // Test various class selectors
    let card_defs = find_definitions(&file_path, "card");
    assert!(!card_defs.is_empty(), "Should find .card class selector");
    assert_eq!(card_defs[0].1, "class_selector");

    let user_card_defs = find_definitions(&file_path, "user-card");
    assert!(
        !user_card_defs.is_empty(),
        "Should find .user-card class selector"
    );
    assert_eq!(user_card_defs[0].1, "class_selector");

    let username_defs = find_definitions(&file_path, "username");
    assert!(
        !username_defs.is_empty(),
        "Should find .username class selector"
    );
    assert_eq!(username_defs[0].1, "class_selector");
}

#[test]
fn test_css_find_id_selector() {
    let file_path = fixtures_path().join("sample.css");
    let definitions = find_definitions(&file_path, "main-header");

    assert!(
        !definitions.is_empty(),
        "Should find main-header id selector"
    );
    assert_eq!(definitions[0].0, "main-header");
    assert_eq!(definitions[0].1, "id_selector");
}

#[test]
fn test_css_find_various_id_selectors() {
    let file_path = fixtures_path().join("sample.css");

    // Test various ID selectors
    let content_defs = find_definitions(&file_path, "content");
    assert!(!content_defs.is_empty(), "Should find #content id selector");
    assert_eq!(content_defs[0].1, "id_selector");
}

#[test]
fn test_css_find_css_variable() {
    let file_path = fixtures_path().join("sample.css");
    let definitions = find_definitions(&file_path, "--primary-color");

    assert!(
        !definitions.is_empty(),
        "Should find --primary-color CSS variable"
    );
    assert_eq!(definitions[0].0, "--primary-color");
    assert_eq!(definitions[0].1, "variable");
}

#[test]
fn test_css_find_various_css_variables() {
    let file_path = fixtures_path().join("sample.css");

    // Test additional CSS variables
    let secondary_defs = find_definitions(&file_path, "--secondary-color");
    assert!(
        !secondary_defs.is_empty(),
        "Should find --secondary-color CSS variable"
    );
    assert_eq!(secondary_defs[0].1, "variable");

    let text_color_defs = find_definitions(&file_path, "--text-color");
    assert!(
        !text_color_defs.is_empty(),
        "Should find --text-color CSS variable"
    );
    assert_eq!(text_color_defs[0].1, "variable");
}

#[test]
fn test_css_find_keyframes() {
    let file_path = fixtures_path().join("sample.css");
    let definitions = find_definitions(&file_path, "fadeIn");

    assert!(!definitions.is_empty(), "Should find fadeIn keyframes");
    assert_eq!(definitions[0].0, "fadeIn");
    assert_eq!(definitions[0].1, "keyframes");
}

#[test]
fn test_css_find_various_keyframes() {
    let file_path = fixtures_path().join("sample.css");

    // Test additional keyframes
    let slide_in_defs = find_definitions(&file_path, "slideIn");
    assert!(!slide_in_defs.is_empty(), "Should find slideIn keyframes");
    assert_eq!(slide_in_defs[0].1, "keyframes");
}

#[test]
fn test_css_generic_parser_handles_css_files() {
    let registry = Arc::new(LanguageRegistry::new().expect("Failed to create registry"));
    let mut parser = GenericParser::new(registry).expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.css");

    let source_code = std::fs::read_to_string(&file_path).expect("Failed to read file");
    let result = parser.parse_with_language(&file_path, &source_code);

    assert!(result.is_ok(), "GenericParser should handle .css files");
    let (tree, lang) = result.unwrap();
    assert!(
        tree.root_node().child_count() > 0,
        "Should produce valid AST with children"
    );
    assert_eq!(lang.name(), "Css", "Should use Css language support");
}

// ======================================
// Python tests
// ======================================

#[test]
fn test_python_find_class_definition() {
    let file_path = fixtures_path().join("sample.py");
    let definitions = find_definitions(&file_path, "UserService");

    assert!(!definitions.is_empty(), "Should find UserService class");
    assert_eq!(definitions[0].0, "UserService");
    assert_eq!(definitions[0].1, "class");
}

#[test]
fn test_python_find_function_definition() {
    let file_path = fixtures_path().join("sample.py");
    let definitions = find_definitions(&file_path, "process_user");

    assert!(!definitions.is_empty(), "Should find process_user function");
    assert_eq!(definitions[0].0, "process_user");
    assert_eq!(definitions[0].1, "function");
}

#[test]
fn test_python_find_constructor() {
    let file_path = fixtures_path().join("sample.py");
    let definitions = find_definitions(&file_path, "__init__");

    assert!(!definitions.is_empty(), "Should find __init__ constructors");
    // Should find constructors for UserService, User, and Logger
    let constructor_defs: Vec<_> = definitions
        .iter()
        .filter(|(_, k)| k == "constructor")
        .collect();
    assert!(
        constructor_defs.len() >= 3,
        "Should find at least 3 constructors"
    );
}

#[test]
fn test_python_find_method_definition() {
    let file_path = fixtures_path().join("sample.py");
    let definitions = find_definitions(&file_path, "add_user");

    assert!(!definitions.is_empty(), "Should find add_user method");
    assert_eq!(definitions[0].0, "add_user");
    assert_eq!(definitions[0].1, "method");
}

#[test]
fn test_python_find_decorated_method() {
    let file_path = fixtures_path().join("sample.py");

    // Test @property decorated method
    let property_defs = find_definitions(&file_path, "formatted_prefix");
    assert!(
        !property_defs.is_empty(),
        "Should find formatted_prefix property method"
    );
    assert_eq!(property_defs[0].1, "method");

    // Test @classmethod decorated method
    let classmethod_defs = find_definitions(&file_path, "create_default");
    assert!(
        !classmethod_defs.is_empty(),
        "Should find create_default classmethod"
    );
    assert_eq!(classmethod_defs[0].1, "method");
}

#[test]
fn test_python_find_module_variable() {
    let file_path = fixtures_path().join("sample.py");
    let definitions = find_definitions(&file_path, "DEFAULT_CONFIG");

    assert!(
        !definitions.is_empty(),
        "Should find DEFAULT_CONFIG module variable"
    );
    assert_eq!(definitions[0].0, "DEFAULT_CONFIG");
    assert_eq!(definitions[0].1, "variable");
}

/// Helper function to count symbol usages using GenericParser.
/// Returns the number of times the symbol appears in the file.
fn count_usages(file_path: &std::path::Path, symbol_name: &str) -> usize {
    let registry = Arc::new(LanguageRegistry::new().expect("Failed to create registry"));
    let mut parser = GenericParser::new(registry).expect("Failed to create parser");

    let source_code = std::fs::read_to_string(file_path).expect("Failed to read file");
    let (tree, lang) = parser
        .parse_with_language(file_path, &source_code)
        .expect("Failed to parse file");

    let query = lang.usages_query();
    let mut cursor = tree_sitter::QueryCursor::new();

    use streaming_iterator::StreamingIterator;
    let mut matches = cursor.matches(query, tree.root_node(), source_code.as_bytes());

    let mut count = 0;
    while let Some(m) = matches.next() {
        for capture in m.captures {
            let capture_name = &query.capture_names()[capture.index as usize];
            if *capture_name == "usage" {
                let usage_text = capture.node.utf8_text(source_code.as_bytes()).unwrap_or("");
                if usage_text == symbol_name {
                    count += 1;
                }
            }
        }
    }

    count
}

#[test]
fn test_python_find_usages() {
    let file_path = fixtures_path().join("sample.py");
    let count = count_usages(&file_path, "user");

    assert!(count > 0, "Should find 'user' usages in Python file");
}

// Note: test_python_find_todo_comments is not included because
// find_comments_in_file currently only supports // and /* */ style comments.
// Python uses # for comments, which would require extending the comment parser.

#[test]
fn test_python_generic_parser_handles_py_files() {
    let registry = Arc::new(LanguageRegistry::new().expect("Failed to create registry"));
    let mut parser = GenericParser::new(registry).expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.py");

    let source_code = std::fs::read_to_string(&file_path).expect("Failed to read file");
    let result = parser.parse_with_language(&file_path, &source_code);

    assert!(result.is_ok(), "GenericParser should handle .py files");
    let (tree, lang) = result.unwrap();
    assert!(
        tree.root_node().child_count() > 0,
        "Should produce valid AST with children"
    );
    assert_eq!(lang.name(), "Python", "Should use Python language support");
}

// ======================================
// Rust tests
// ======================================

#[test]
fn test_rust_find_struct_definition() {
    let file_path = fixtures_path().join("sample.rs");
    let definitions = find_definitions(&file_path, "User");

    assert!(!definitions.is_empty(), "Should find User struct");
    assert_eq!(definitions[0].0, "User");
    assert_eq!(definitions[0].1, "struct");
}

#[test]
fn test_rust_find_enum_definition() {
    let file_path = fixtures_path().join("sample.rs");
    let definitions = find_definitions(&file_path, "UserRole");

    assert!(!definitions.is_empty(), "Should find UserRole enum");
    assert_eq!(definitions[0].0, "UserRole");
    assert_eq!(definitions[0].1, "enum");
}

#[test]
fn test_rust_find_trait_definition() {
    let file_path = fixtures_path().join("sample.rs");
    let definitions = find_definitions(&file_path, "Validatable");

    assert!(!definitions.is_empty(), "Should find Validatable trait");
    assert_eq!(definitions[0].0, "Validatable");
    assert_eq!(definitions[0].1, "trait");
}

#[test]
fn test_rust_find_function_definition() {
    let file_path = fixtures_path().join("sample.rs");
    let definitions = find_definitions(&file_path, "process_user");

    assert!(!definitions.is_empty(), "Should find process_user function");
    assert_eq!(definitions[0].0, "process_user");
    assert_eq!(definitions[0].1, "function");
}

#[test]
fn test_rust_find_method_definition() {
    let file_path = fixtures_path().join("sample.rs");
    let definitions = find_definitions(&file_path, "display_name");

    assert!(!definitions.is_empty(), "Should find display_name method");
    assert_eq!(definitions[0].0, "display_name");
    assert_eq!(definitions[0].1, "method");
}

#[test]
fn test_rust_find_impl_method() {
    let file_path = fixtures_path().join("sample.rs");
    let definitions = find_definitions(&file_path, "is_admin");

    assert!(!definitions.is_empty(), "Should find is_admin method");
    assert_eq!(definitions[0].0, "is_admin");
    assert_eq!(definitions[0].1, "method");
}

#[test]
fn test_rust_find_module_definition() {
    let file_path = fixtures_path().join("sample.rs");
    let definitions = find_definitions(&file_path, "user_service");

    assert!(!definitions.is_empty(), "Should find user_service module");
    assert_eq!(definitions[0].0, "user_service");
    assert_eq!(definitions[0].1, "module");
}

#[test]
fn test_rust_find_type_alias() {
    let file_path = fixtures_path().join("sample.rs");
    let definitions = find_definitions(&file_path, "UserId");

    assert!(!definitions.is_empty(), "Should find UserId type alias");
    assert_eq!(definitions[0].0, "UserId");
    assert_eq!(definitions[0].1, "type_alias");
}

#[test]
fn test_rust_find_const_definition() {
    let file_path = fixtures_path().join("sample.rs");
    let definitions = find_definitions(&file_path, "MAX_USERS");

    assert!(!definitions.is_empty(), "Should find MAX_USERS const");
    assert_eq!(definitions[0].0, "MAX_USERS");
    assert_eq!(definitions[0].1, "const");
}

#[test]
fn test_rust_find_static_definition() {
    let file_path = fixtures_path().join("sample.rs");
    let definitions = find_definitions(&file_path, "USER_COUNTER");

    assert!(!definitions.is_empty(), "Should find USER_COUNTER static");
    assert_eq!(definitions[0].0, "USER_COUNTER");
    assert_eq!(definitions[0].1, "static");
}

#[test]
fn test_rust_find_macro_definition() {
    let file_path = fixtures_path().join("sample.rs");
    let definitions = find_definitions(&file_path, "create_user");

    assert!(!definitions.is_empty(), "Should find create_user macro");
    assert_eq!(definitions[0].0, "create_user");
    assert_eq!(definitions[0].1, "macro");
}

#[test]
fn test_rust_find_usages() {
    let file_path = fixtures_path().join("sample.rs");
    let count = count_usages(&file_path, "User");

    assert!(count > 0, "Should find 'User' usages in Rust file");
}

#[test]
fn test_rust_generic_parser_handles_rs_files() {
    let registry = Arc::new(LanguageRegistry::new().expect("Failed to create registry"));
    let mut parser = GenericParser::new(registry).expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.rs");

    let source_code = std::fs::read_to_string(&file_path).expect("Failed to read file");
    let result = parser.parse_with_language(&file_path, &source_code);

    assert!(result.is_ok(), "GenericParser should handle .rs files");
    let (tree, lang) = result.unwrap();
    assert!(
        tree.root_node().child_count() > 0,
        "Should produce valid AST with children"
    );
    assert_eq!(lang.name(), "Rust", "Should use Rust language support");
}

// ======================================
// Go tests
// ======================================

#[test]
fn test_go_find_struct_definition() {
    let file_path = fixtures_path().join("sample.go");
    let definitions = find_definitions(&file_path, "User");

    assert!(!definitions.is_empty(), "Should find User struct");
    assert_eq!(definitions[0].0, "User");
    assert_eq!(definitions[0].1, "struct");
}

#[test]
fn test_go_find_interface_definition() {
    let file_path = fixtures_path().join("sample.go");
    let definitions = find_definitions(&file_path, "Validatable");

    assert!(!definitions.is_empty(), "Should find Validatable interface");
    assert_eq!(definitions[0].0, "Validatable");
    assert_eq!(definitions[0].1, "interface");
}

#[test]
fn test_go_find_function_definition() {
    let file_path = fixtures_path().join("sample.go");
    let definitions = find_definitions(&file_path, "NewUser");

    assert!(!definitions.is_empty(), "Should find NewUser function");
    assert_eq!(definitions[0].0, "NewUser");
    assert_eq!(definitions[0].1, "function");
}

#[test]
fn test_go_find_method_definition() {
    let file_path = fixtures_path().join("sample.go");
    let definitions = find_definitions(&file_path, "DisplayName");

    assert!(!definitions.is_empty(), "Should find DisplayName method");
    assert_eq!(definitions[0].0, "DisplayName");
    assert_eq!(definitions[0].1, "method");
}

#[test]
fn test_go_find_const_definition() {
    let file_path = fixtures_path().join("sample.go");
    let definitions = find_definitions(&file_path, "MaxUsers");

    assert!(!definitions.is_empty(), "Should find MaxUsers const");
    assert_eq!(definitions[0].0, "MaxUsers");
    assert_eq!(definitions[0].1, "const");
}

#[test]
fn test_go_find_var_definition() {
    let file_path = fixtures_path().join("sample.go");
    let definitions = find_definitions(&file_path, "globalCounter");

    assert!(
        !definitions.is_empty(),
        "Should find globalCounter variable"
    );
    assert_eq!(definitions[0].0, "globalCounter");
    assert_eq!(definitions[0].1, "variable");
}

#[test]
fn test_go_find_type_alias_definition() {
    let file_path = fixtures_path().join("sample.go");
    let definitions = find_definitions(&file_path, "UserID");

    assert!(!definitions.is_empty(), "Should find UserID type alias");
    assert_eq!(definitions[0].0, "UserID");
    assert_eq!(definitions[0].1, "type_alias");
}

#[test]
fn test_go_find_usages() {
    let file_path = fixtures_path().join("sample.go");
    let count = count_usages(&file_path, "User");

    assert!(count > 0, "Should find 'User' usages in Go file");
}

#[test]
fn test_go_find_todo_comments() {
    let file_path = fixtures_path().join("sample.go");

    let matches = find_comments_in_file(&file_path, "TODO").expect("Failed to find comments");

    assert!(!matches.is_empty(), "Should find TODO comments in Go file");
}

#[test]
fn test_go_generic_parser_handles_go_files() {
    let registry = Arc::new(LanguageRegistry::new().expect("Failed to create registry"));
    let mut parser = GenericParser::new(registry).expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.go");

    let source_code = std::fs::read_to_string(&file_path).expect("Failed to read file");
    let result = parser.parse_with_language(&file_path, &source_code);

    assert!(result.is_ok(), "GenericParser should handle .go files");
    let (tree, lang) = result.unwrap();
    assert!(
        tree.root_node().child_count() > 0,
        "Should produce valid AST with children"
    );
    assert_eq!(lang.name(), "Go", "Should use Go language support");
}

// ======================================
// Java tests
// ======================================

#[test]
fn test_java_find_class_definition() {
    let file_path = fixtures_path().join("sample.java");
    let definitions = find_definitions(&file_path, "User");

    assert!(!definitions.is_empty(), "Should find User class");
    assert_eq!(definitions[0].0, "User");
    assert_eq!(definitions[0].1, "class");
}

#[test]
fn test_java_find_interface_definition() {
    let file_path = fixtures_path().join("sample.java");
    let definitions = find_definitions(&file_path, "Validatable");

    assert!(!definitions.is_empty(), "Should find Validatable interface");
    assert_eq!(definitions[0].0, "Validatable");
    assert_eq!(definitions[0].1, "interface");
}

#[test]
fn test_java_find_enum_definition() {
    let file_path = fixtures_path().join("sample.java");
    let definitions = find_definitions(&file_path, "UserRole");

    assert!(!definitions.is_empty(), "Should find UserRole enum");
    assert_eq!(definitions[0].0, "UserRole");
    assert_eq!(definitions[0].1, "enum");
}

#[test]
fn test_java_find_method_definition() {
    let file_path = fixtures_path().join("sample.java");
    let definitions = find_definitions(&file_path, "getDisplayName");

    assert!(!definitions.is_empty(), "Should find getDisplayName method");
    assert_eq!(definitions[0].0, "getDisplayName");
    assert_eq!(definitions[0].1, "method");
}

#[test]
fn test_java_find_constructor_definition() {
    let file_path = fixtures_path().join("sample.java");
    let definitions = find_definitions(&file_path, "User");

    // Should find both class and constructor named "User"
    let constructor_defs: Vec<_> = definitions
        .iter()
        .filter(|(_, k)| k == "constructor")
        .collect();
    assert!(!constructor_defs.is_empty(), "Should find User constructor");
}

#[test]
fn test_java_find_field_definition() {
    let file_path = fixtures_path().join("sample.java");
    let definitions = find_definitions(&file_path, "id");

    assert!(!definitions.is_empty(), "Should find id field");
    assert_eq!(definitions[0].0, "id");
    assert_eq!(definitions[0].1, "field");
}

#[test]
fn test_java_find_annotation_definition() {
    let file_path = fixtures_path().join("sample.java");
    let definitions = find_definitions(&file_path, "Deprecated");

    assert!(!definitions.is_empty(), "Should find Deprecated annotation");
    assert_eq!(definitions[0].0, "Deprecated");
    assert_eq!(definitions[0].1, "annotation");
}

#[test]
fn test_java_find_usages() {
    let file_path = fixtures_path().join("sample.java");
    let count = count_usages(&file_path, "User");

    assert!(count > 0, "Should find 'User' usages in Java file");
}

#[test]
fn test_java_find_todo_comments() {
    let file_path = fixtures_path().join("sample.java");

    let matches = find_comments_in_file(&file_path, "TODO").expect("Failed to find comments");

    assert!(
        !matches.is_empty(),
        "Should find TODO comments in Java file"
    );
}

#[test]
fn test_java_generic_parser_handles_java_files() {
    let registry = Arc::new(LanguageRegistry::new().expect("Failed to create registry"));
    let mut parser = GenericParser::new(registry).expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.java");

    let source_code = std::fs::read_to_string(&file_path).expect("Failed to read file");
    let result = parser.parse_with_language(&file_path, &source_code);

    assert!(result.is_ok(), "GenericParser should handle .java files");
    let (tree, lang) = result.unwrap();
    assert!(
        tree.root_node().child_count() > 0,
        "Should produce valid AST with children"
    );
    assert_eq!(lang.name(), "Java", "Should use Java language support");
}
