//! Integration tests for codebase statistics

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

use codescope_mcp::cache::CachedContent;
use codescope_mcp::language::{LanguageId, LanguageRegistry};
use codescope_mcp::parser::{CachedParser, ParserCache};
use codescope_mcp::pipeline::stats::{aggregate_statistics, count_lines, FileStatistics};
use codescope_mcp::pipeline::{ResultCollector, StatsCollector};
use codescope_mcp::symbol::types::SymbolKind;

fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn create_test_parser() -> (CachedParser, Arc<LanguageRegistry>) {
    let registry = Arc::new(LanguageRegistry::new().expect("Failed to create registry"));
    let parser_cache = Arc::new(ParserCache::new());
    let parser =
        CachedParser::new(registry.clone(), parser_cache).expect("Failed to create parser");
    (parser, registry)
}

#[test]
fn test_count_lines_typescript_fixture() {
    let file_path = fixtures_path().join("sample.ts");
    let source = std::fs::read_to_string(&file_path).expect("Failed to read sample.ts");

    let (total, code, _blank, _comment) = count_lines(&source, LanguageId::TypeScript);

    // The file should have lines
    assert!(total > 0, "Should have lines");
    assert!(code > 0, "Should have code lines");
    // May or may not have blank/comment lines depending on fixture
}

#[test]
fn test_count_lines_rust_fixture() {
    // Use this file as a fixture
    let source = "// This is a comment
fn main() {
    println!(\"Hello\");
}

/* Block
 * comment
 */";

    let (total, code, blank, comment) = count_lines(source, LanguageId::Rust);

    assert_eq!(total, 8);
    assert_eq!(blank, 1); // empty line between } and /*
    assert_eq!(comment, 4); // single line + block (3 lines)
    assert_eq!(code, 3); // fn, println, closing brace
}

#[test]
fn test_count_lines_python() {
    let source = "# Python comment
def hello():
    \"\"\"Docstring\"\"\"
    print(\"Hi\")

# Another comment
x = 1";

    let (total, code, blank, comment) = count_lines(source, LanguageId::Python);

    assert_eq!(total, 7);
    assert_eq!(blank, 1);
    assert_eq!(comment, 2); // # comments
    assert_eq!(code, 4); // def, docstring, print, x
}

#[test]
fn test_count_lines_sql() {
    let source = "-- SQL comment
SELECT * FROM users;

/* Block
comment */
INSERT INTO users VALUES (1);";

    let (total, code, blank, comment) = count_lines(source, LanguageId::Sql);

    assert_eq!(total, 6);
    assert_eq!(blank, 1);
    assert_eq!(comment, 3); // -- comment + block (2 lines)
    assert_eq!(code, 2); // SELECT, INSERT
}

#[test]
fn test_stats_collector_typescript() {
    let (mut parser, _registry) = create_test_parser();
    let file_path = fixtures_path().join("sample.ts");

    let source = std::fs::read_to_string(&file_path).expect("Failed to read file");
    let cached_content = CachedContent {
        content: Arc::new(source),
        modified_time: SystemTime::now(),
    };

    let collector = StatsCollector;
    let results = collector
        .process_file(&mut parser, &file_path, &cached_content)
        .expect("Failed to process file");

    assert_eq!(results.len(), 1, "Should return one FileStatistics");

    let stats = &results[0];
    assert_eq!(stats.language_id, LanguageId::TypeScript);
    assert!(stats.total_lines > 0, "Should have lines");
    assert!(!stats.symbol_counts.is_empty(), "Should have symbols");

    // The sample.ts fixture has various symbols
    let total_symbols: usize = stats.symbol_counts.values().sum();
    assert!(total_symbols > 0, "Should count symbols");
}

#[test]
fn test_aggregate_statistics_single_file() {
    let stats = vec![FileStatistics {
        file_path: "test.ts".to_string(),
        language_id: LanguageId::TypeScript,
        total_lines: 100,
        code_lines: 80,
        blank_lines: 10,
        comment_lines: 10,
        symbol_counts: {
            let mut m = HashMap::new();
            m.insert(SymbolKind::Function, 5);
            m.insert(SymbolKind::Class, 2);
            m
        },
    }];

    let response = aggregate_statistics(stats);

    assert_eq!(response.summary.total_files, 1);
    assert_eq!(response.summary.total_lines, 100);
    assert_eq!(response.summary.code_lines, 80);
    assert_eq!(response.summary.blank_lines, 10);
    assert_eq!(response.summary.comment_lines, 10);
    assert_eq!(response.summary.total_symbols, 7);
    assert_eq!(response.summary.languages_count, 1);

    assert_eq!(response.by_language.len(), 1);
    assert_eq!(response.by_language[0].language, "TypeScript");
    assert_eq!(response.by_language[0].file_count, 1);
    assert_eq!(response.by_language[0].percentage, 100.0);

    assert_eq!(response.symbols.by_kind.get("Function"), Some(&5));
    assert_eq!(response.symbols.by_kind.get("Class"), Some(&2));
}

#[test]
fn test_aggregate_statistics_multiple_languages() {
    let stats = vec![
        FileStatistics {
            file_path: "foo.ts".to_string(),
            language_id: LanguageId::TypeScript,
            total_lines: 100,
            code_lines: 80,
            blank_lines: 10,
            comment_lines: 10,
            symbol_counts: {
                let mut m = HashMap::new();
                m.insert(SymbolKind::Function, 5);
                m
            },
        },
        FileStatistics {
            file_path: "bar.ts".to_string(),
            language_id: LanguageId::TypeScript,
            total_lines: 50,
            code_lines: 40,
            blank_lines: 5,
            comment_lines: 5,
            symbol_counts: {
                let mut m = HashMap::new();
                m.insert(SymbolKind::Function, 3);
                m
            },
        },
        FileStatistics {
            file_path: "lib.rs".to_string(),
            language_id: LanguageId::Rust,
            total_lines: 200,
            code_lines: 150,
            blank_lines: 30,
            comment_lines: 20,
            symbol_counts: {
                let mut m = HashMap::new();
                m.insert(SymbolKind::Function, 10);
                m.insert(SymbolKind::Struct, 5);
                m
            },
        },
    ];

    let response = aggregate_statistics(stats);

    assert_eq!(response.summary.total_files, 3);
    assert_eq!(response.summary.total_lines, 350);
    assert_eq!(response.summary.code_lines, 270);
    assert_eq!(response.summary.blank_lines, 45);
    assert_eq!(response.summary.comment_lines, 35);
    assert_eq!(response.summary.total_symbols, 23);
    assert_eq!(response.summary.languages_count, 2);

    // Languages sorted by code lines (Rust: 150, TypeScript: 120)
    assert_eq!(response.by_language.len(), 2);
    assert_eq!(response.by_language[0].language, "Rust");
    assert_eq!(response.by_language[0].file_count, 1);
    assert_eq!(response.by_language[0].code_lines, 150);
    assert_eq!(response.by_language[1].language, "TypeScript");
    assert_eq!(response.by_language[1].file_count, 2);
    assert_eq!(response.by_language[1].code_lines, 120);

    // Symbol aggregation
    assert_eq!(response.symbols.by_kind.get("Function"), Some(&18)); // 5+3+10
    assert_eq!(response.symbols.by_kind.get("Struct"), Some(&5));
}

#[test]
fn test_aggregate_statistics_empty() {
    let stats: Vec<FileStatistics> = vec![];
    let response = aggregate_statistics(stats);

    assert_eq!(response.summary.total_files, 0);
    assert_eq!(response.summary.total_lines, 0);
    assert_eq!(response.summary.code_lines, 0);
    assert_eq!(response.summary.languages_count, 0);
    assert!(response.by_language.is_empty());
    assert!(response.symbols.by_kind.is_empty());
    assert_eq!(response.symbols.avg_symbols_per_file, 0.0);
}

#[test]
fn test_percentage_calculation() {
    let stats = vec![
        FileStatistics {
            file_path: "a.ts".to_string(),
            language_id: LanguageId::TypeScript,
            total_lines: 100,
            code_lines: 75,
            blank_lines: 15,
            comment_lines: 10,
            symbol_counts: HashMap::new(),
        },
        FileStatistics {
            file_path: "b.py".to_string(),
            language_id: LanguageId::Python,
            total_lines: 100,
            code_lines: 25,
            blank_lines: 50,
            comment_lines: 25,
            symbol_counts: HashMap::new(),
        },
    ];

    let response = aggregate_statistics(stats);

    // Total code lines: 100
    // TypeScript: 75 -> 75%
    // Python: 25 -> 25%
    assert_eq!(response.by_language[0].language, "TypeScript");
    assert_eq!(response.by_language[0].percentage, 75.0);
    assert_eq!(response.by_language[1].language, "Python");
    assert_eq!(response.by_language[1].percentage, 25.0);
}

#[test]
fn test_avg_symbols_per_file() {
    let stats = vec![
        FileStatistics {
            file_path: "a.ts".to_string(),
            language_id: LanguageId::TypeScript,
            total_lines: 50,
            code_lines: 40,
            blank_lines: 5,
            comment_lines: 5,
            symbol_counts: {
                let mut m = HashMap::new();
                m.insert(SymbolKind::Function, 4);
                m
            },
        },
        FileStatistics {
            file_path: "b.ts".to_string(),
            language_id: LanguageId::TypeScript,
            total_lines: 50,
            code_lines: 40,
            blank_lines: 5,
            comment_lines: 5,
            symbol_counts: {
                let mut m = HashMap::new();
                m.insert(SymbolKind::Function, 6);
                m
            },
        },
    ];

    let response = aggregate_statistics(stats);

    // 10 symbols / 2 files = 5.0
    assert_eq!(response.symbols.avg_symbols_per_file, 5.0);
}
