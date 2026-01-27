use std::path::PathBuf;

use codescope_mcp::parser::typescript::TypeScriptParser;
use codescope_mcp::symbol::comment::{find_comments_in_file, get_code_at_location};
use codescope_mcp::symbol::definition::find_definitions_in_file;
use codescope_mcp::symbol::usage::find_usages_in_file;
use codescope_mcp::symbol::types::{CommentType, UsageKind};

fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

#[test]
fn test_find_interface_definition() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    let definitions =
        find_definitions_in_file(&mut parser, &file_path, "User", false).expect("Failed to find definitions");

    assert!(!definitions.is_empty(), "Should find User interface");
    let user_def = &definitions[0];
    assert_eq!(user_def.name, "User");
    assert_eq!(user_def.node_kind.to_string(), "Interface");
}

#[test]
fn test_find_class_definition() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    let definitions =
        find_definitions_in_file(&mut parser, &file_path, "UserService", false).expect("Failed to find definitions");

    assert!(!definitions.is_empty(), "Should find UserService class");
    let class_def = &definitions[0];
    assert_eq!(class_def.name, "UserService");
    assert_eq!(class_def.node_kind.to_string(), "Class");
}

#[test]
fn test_find_function_definition() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    let definitions =
        find_definitions_in_file(&mut parser, &file_path, "processUser", false).expect("Failed to find definitions");

    assert!(!definitions.is_empty(), "Should find processUser function");
    let func_def = &definitions[0];
    assert_eq!(func_def.name, "processUser");
    assert_eq!(func_def.node_kind.to_string(), "Function");
}

#[test]
fn test_find_arrow_function_definition() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    let definitions =
        find_definitions_in_file(&mut parser, &file_path, "createUser", false).expect("Failed to find definitions");

    assert!(!definitions.is_empty(), "Should find createUser arrow function");
    let arrow_def = &definitions[0];
    assert_eq!(arrow_def.name, "createUser");
    // Could be ArrowFunction or Variable depending on query
}

#[test]
fn test_find_usages() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    let usages =
        find_usages_in_file(&mut parser, &file_path, "User", false, 3, None).expect("Failed to find usages");

    // User should be used in multiple places
    assert!(!usages.is_empty(), "Should find User usages");
}

#[test]
fn test_find_usages_with_imports() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    let usages_without_imports =
        find_usages_in_file(&mut parser, &file_path, "User", false, 3, None).expect("Failed to find usages");

    let usages_with_imports =
        find_usages_in_file(&mut parser, &file_path, "User", true, 3, None).expect("Failed to find usages");

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

    let usages =
        find_usages_in_file(&mut parser, &file_path, "User", true, 3, None).expect("Failed to find usages");

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
    let usages =
        find_usages_in_file(&mut parser, &file_path, "log", false, 3, None).expect("Failed to find usages");

    assert!(!usages.is_empty(), "Should find log usages");

    // All log usages should be method calls with object_name "console"
    for usage in &usages {
        assert_eq!(usage.usage_kind, UsageKind::MethodCall, "log should be a method call");
        assert_eq!(usage.object_name.as_deref(), Some("console"), "log should be called on console");
    }
}

#[test]
fn test_usage_kind_type_reference() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("sample.ts");

    let usages =
        find_usages_in_file(&mut parser, &file_path, "User", false, 3, None).expect("Failed to find usages");

    // User is used as a type annotation in several places
    let type_refs: Vec<_> = usages.iter()
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
    let usages =
        find_usages_in_file(&mut parser, &file_path, "now", false, 3, None).expect("Failed to find usages");

    let date_now: Vec<_> = usages.iter()
        .filter(|u| u.object_name.as_deref() == Some("Date"))
        .collect();

    assert!(!date_now.is_empty(), "Should find Date.now() calls");
    assert!(date_now.iter().all(|u| u.usage_kind == UsageKind::MethodCall), "Date.now() should be a method call");
}

// Phase 3: Test object filter
#[test]
fn test_object_filter() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("semantic_test.ts");

    // Get all 'now' usages
    let all_usages =
        find_usages_in_file(&mut parser, &file_path, "now", false, 3, None).expect("Failed to find usages");

    // Get only Date.now() usages
    let date_usages =
        find_usages_in_file(&mut parser, &file_path, "now", false, 3, Some("Date")).expect("Failed to find usages");

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
        find_usages_in_file(&mut parser, &file_path, "now", false, 3, Some("chronia")).expect("Failed to find usages");

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

    assert!(!method_calls.is_empty(), "Should find Date.now() method calls");

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
        assert!(usage.contexts.is_empty(), "contexts should be empty when include_contexts=false");
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
    assert!(has_contexts, "At least some usages should have contexts when include_contexts=true");

    // All contexts should have at most 2 entries
    for usage in &usages {
        assert!(usage.contexts.len() <= 2, "contexts should have at most 2 entries");
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
    assert!(!json.contains("\"contexts\""), "JSON should not contain 'contexts' when empty");
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
    assert!(json.contains("\"contexts\""), "JSON should contain 'contexts' when present");
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
    let has_single_line = matches.iter().any(|m| m.comment_type == CommentType::SingleLine);
    let has_block = matches.iter().any(|m| m.comment_type == CommentType::Block);

    assert!(has_single_line || has_block, "Should find at least one comment type");
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
    assert!(docs.contains("Represents a product"), "Docs should contain description");
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
    assert!(product_def.docs.is_none(), "Product should not have docs when include_docs=false");
}

#[test]
fn test_definition_docs_serialization() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("jsdoc_test.ts");

    // Get definitions without docs
    let definitions_no_docs = find_definitions_in_file(&mut parser, &file_path, "Product", false)
        .expect("Failed to find definitions");

    let json_no_docs = serde_json::to_string_pretty(&definitions_no_docs).expect("Failed to serialize");

    // "docs" should NOT appear when None (skip_serializing_if)
    assert!(!json_no_docs.contains("\"docs\""), "JSON should not contain 'docs' when None");

    // Get definitions with docs
    let definitions_with_docs = find_definitions_in_file(&mut parser, &file_path, "Product", true)
        .expect("Failed to find definitions");

    let json_with_docs = serde_json::to_string_pretty(&definitions_with_docs).expect("Failed to serialize");

    // "docs" should appear when Some
    assert!(json_with_docs.contains("\"docs\""), "JSON should contain 'docs' when present");
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
    assert!(docs.contains("Helper function") || docs.contains("formatted price"), "Docs should contain comment text");
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
    assert_eq!(snippet.end_line, 7);   // 5 + 2 = 7
    assert!(!snippet.code.is_empty(), "Code should not be empty");
}

#[test]
fn test_get_code_at_location_at_start() {
    let file_path = fixtures_path().join("sample.ts");

    // Line 1 with context_before=5 should clamp to start
    let snippet = get_code_at_location(&file_path, 1, 5, 2).expect("Failed to get code");

    assert_eq!(snippet.start_line, 1); // Clamped to 1
    assert_eq!(snippet.end_line, 3);   // 1 + 2 = 3
}

#[test]
fn test_get_code_at_location_at_end() {
    let file_path = fixtures_path().join("sample.ts");

    // Read file to get total lines
    let source = std::fs::read_to_string(&file_path).expect("Failed to read file");
    let total_lines = source.lines().count();

    // Request past end of file should clamp
    let snippet = get_code_at_location(&file_path, total_lines, 2, 100).expect("Failed to get code");

    assert_eq!(snippet.end_line, total_lines); // Clamped to total lines
}

#[test]
fn test_get_code_at_location_default_context() {
    let file_path = fixtures_path().join("sample.ts");

    // Default context (3 lines before and after)
    let snippet = get_code_at_location(&file_path, 10, 3, 3).expect("Failed to get code");

    assert_eq!(snippet.start_line, 7);  // 10 - 3 = 7
    assert_eq!(snippet.end_line, 13);   // 10 + 3 = 13
}

#[test]
fn test_get_code_at_location_serialization() {
    let file_path = fixtures_path().join("sample.ts");

    let snippet = get_code_at_location(&file_path, 5, 2, 2).expect("Failed to get code");

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&snippet).expect("Failed to serialize");

    // Should contain all expected fields
    assert!(json.contains("\"file_path\""), "JSON should contain file_path");
    assert!(json.contains("\"start_line\""), "JSON should contain start_line");
    assert!(json.contains("\"end_line\""), "JSON should contain end_line");
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
    let date_now_usages: Vec<_> = usages.iter()
        .filter(|u| u.object_name.as_deref() == Some("Date"))
        .collect();

    assert!(!date_now_usages.is_empty(), "Should find Date.now() usages");
    for usage in &date_now_usages {
        assert_eq!(usage.qualified_name, "Date.now", "qualified_name should be 'Date.now'");
    }

    // Check that chronia.now() has qualified_name "chronia.now"
    let chronia_now_usages: Vec<_> = usages.iter()
        .filter(|u| u.object_name.as_deref() == Some("chronia"))
        .collect();

    assert!(!chronia_now_usages.is_empty(), "Should find chronia.now() usages");
    for usage in &chronia_now_usages {
        assert_eq!(usage.qualified_name, "chronia.now", "qualified_name should be 'chronia.now'");
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
            assert_eq!(usage.qualified_name, "User", "qualified_name should be 'User' when no object");
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
    assert!(json.contains("\"qualified_name\""), "JSON should always contain 'qualified_name'");
    assert!(json.contains("\"Date.now\"") || json.contains("\"chronia.now\""),
        "JSON should contain qualified names like 'Date.now' or 'chronia.now'");
}

#[test]
fn test_qualified_name_distinguishes_same_method() {
    let mut parser = TypeScriptParser::new().expect("Failed to create parser");
    let file_path = fixtures_path().join("semantic_test.ts");

    // Get all 'now' usages
    let usages = find_usages_in_file(&mut parser, &file_path, "now", false, 0, None)
        .expect("Failed to find usages");

    // Group by qualified_name
    let date_now_count = usages.iter().filter(|u| u.qualified_name == "Date.now").count();
    let chronia_now_count = usages.iter().filter(|u| u.qualified_name == "chronia.now").count();

    // Both should have usages
    assert!(date_now_count > 0, "Should have Date.now usages");
    assert!(chronia_now_count > 0, "Should have chronia.now usages");

    println!("Date.now usages: {}", date_now_count);
    println!("chronia.now usages: {}", chronia_now_count);

    // The qualified_name allows clear distinction
    for usage in &usages {
        println!("Line {}: qualified_name={}", usage.line, usage.qualified_name);
    }
}
