//! Codebase statistics collection and aggregation
//!
//! This module provides functionality to collect statistics about a codebase,
//! including file counts, line counts (code, blank, comment), and symbol counts.

use std::collections::HashMap;

use crate::language::LanguageId;
use crate::server::types::{LanguageStats, StatsResponse, StatsSummary, SymbolStats};
use crate::symbol::types::SymbolKind;

/// Statistics collected from a single file
#[derive(Debug, Clone, Default)]
pub struct FileStatistics {
    /// Path to the file
    pub file_path: String,
    /// Language of the file
    pub language_id: LanguageId,
    /// Total number of lines in the file
    pub total_lines: usize,
    /// Number of lines containing code (non-blank, non-comment)
    pub code_lines: usize,
    /// Number of blank lines
    pub blank_lines: usize,
    /// Number of comment lines
    pub comment_lines: usize,
    /// Count of symbols by kind
    pub symbol_counts: HashMap<SymbolKind, usize>,
}

/// Line classification for statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineType {
    /// Empty line (whitespace only)
    Blank,
    /// Comment line (single-line or part of block comment)
    Comment,
    /// Code line (may contain inline comments)
    Code,
}

/// State machine for line classification
#[derive(Debug, Default)]
struct LineClassifier {
    in_block_comment: bool,
}

impl LineClassifier {
    fn new() -> Self {
        Self {
            in_block_comment: false,
        }
    }

    /// Classify a line and update state for block comments
    ///
    /// The `single_line_prefix` parameter specifies the single-line comment prefix
    /// for the current language (e.g., "//" for C-style, "--" for SQL, "#" for Python).
    fn classify(&mut self, line: &str, single_line_prefix: &str) -> LineType {
        let trimmed = line.trim();

        // Empty line
        if trimmed.is_empty() {
            return LineType::Blank;
        }

        // Currently in block comment
        if self.in_block_comment {
            if trimmed.contains("*/") {
                self.in_block_comment = false;
            }
            return LineType::Comment;
        }

        // Check for block comment start
        if trimmed.starts_with("/*") {
            if !trimmed.contains("*/") || trimmed.ends_with("/*") {
                // Block comment doesn't end on this line
                self.in_block_comment = true;
            }
            return LineType::Comment;
        }

        // Check for single-line comment
        if trimmed.starts_with(single_line_prefix) {
            return LineType::Comment;
        }

        // Check for JSDoc-style continuation (line starting with *)
        if trimmed.starts_with('*') && !trimmed.starts_with("*/") {
            return LineType::Comment;
        }

        // Everything else is code
        LineType::Code
    }
}

/// Count lines by type in source code
///
/// Returns (total_lines, code_lines, blank_lines, comment_lines)
pub fn count_lines(source: &str, language_id: LanguageId) -> (usize, usize, usize, usize) {
    let single_line_prefix = get_single_line_comment_prefix(language_id);
    let mut classifier = LineClassifier::new();

    let mut total = 0;
    let mut code = 0;
    let mut blank = 0;
    let mut comment = 0;

    for line in source.lines() {
        total += 1;
        match classifier.classify(line, single_line_prefix) {
            LineType::Blank => blank += 1,
            LineType::Comment => comment += 1,
            LineType::Code => code += 1,
        }
    }

    (total, code, blank, comment)
}

/// Get the single-line comment prefix for a language
fn get_single_line_comment_prefix(language_id: LanguageId) -> &'static str {
    match language_id {
        LanguageId::Python => "#",
        LanguageId::Sql => "--",
        // C-style languages: //, also used for HTML (<!-- is handled differently)
        LanguageId::TypeScript
        | LanguageId::TypeScriptReact
        | LanguageId::JavaScript
        | LanguageId::JavaScriptReact
        | LanguageId::Rust
        | LanguageId::Go
        | LanguageId::Java
        | LanguageId::Css => "//",
        // HTML and Markdown don't have traditional single-line comments
        // For HTML we use a prefix that won't match normal lines
        LanguageId::Html => "<!--",
        // Markdown has no comments, use a non-matching prefix
        LanguageId::Markdown => "<!---",
    }
}

/// Aggregate file statistics into a complete stats response
pub fn aggregate_statistics(file_stats: Vec<FileStatistics>) -> StatsResponse {
    // Aggregate by language
    let mut by_language: HashMap<LanguageId, (usize, usize, usize)> = HashMap::new();
    let mut total_files = 0;
    let mut total_lines = 0;
    let mut code_lines = 0;
    let mut blank_lines = 0;
    let mut comment_lines = 0;
    let mut symbol_counts: HashMap<String, usize> = HashMap::new();
    let mut total_symbols = 0;

    for stats in &file_stats {
        total_files += 1;
        total_lines += stats.total_lines;
        code_lines += stats.code_lines;
        blank_lines += stats.blank_lines;
        comment_lines += stats.comment_lines;

        // Aggregate by language
        let entry = by_language.entry(stats.language_id).or_insert((0, 0, 0));
        entry.0 += 1; // file count
        entry.1 += stats.total_lines; // total lines
        entry.2 += stats.code_lines; // code lines

        // Aggregate symbol counts
        for (kind, count) in &stats.symbol_counts {
            *symbol_counts.entry(kind.to_string()).or_insert(0) += count;
            total_symbols += count;
        }
    }

    // Build language stats sorted by code lines (descending)
    let mut language_stats: Vec<LanguageStats> = by_language
        .into_iter()
        .map(
            |(lang_id, (file_count, lang_total_lines, lang_code_lines))| {
                let percentage = if code_lines > 0 {
                    (lang_code_lines as f64 / code_lines as f64) * 100.0
                } else {
                    0.0
                };
                LanguageStats {
                    language: lang_id.to_string(),
                    file_count,
                    total_lines: lang_total_lines,
                    code_lines: lang_code_lines,
                    percentage: (percentage * 10.0).round() / 10.0, // Round to 1 decimal
                }
            },
        )
        .collect();

    // Sort by code lines descending
    language_stats.sort_by(|a, b| b.code_lines.cmp(&a.code_lines));

    let languages_count = language_stats.len();

    let avg_symbols_per_file = if total_files > 0 {
        (total_symbols as f64 / total_files as f64 * 100.0).round() / 100.0
    } else {
        0.0
    };

    StatsResponse {
        summary: StatsSummary {
            total_files,
            total_lines,
            code_lines,
            blank_lines,
            comment_lines,
            total_symbols,
            languages_count,
        },
        by_language: language_stats,
        symbols: SymbolStats {
            by_kind: symbol_counts,
            avg_symbols_per_file,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_lines_typescript() {
        let source = "// Single line comment
const x = 1;

/*
 * Block comment
 */
function foo() {
    return x;
}";
        let (total, code, blank, comment) = count_lines(source, LanguageId::TypeScript);
        assert_eq!(total, 9);
        assert_eq!(blank, 1); // empty line between const and block comment
        assert_eq!(comment, 4); // single line + block (3 lines)
        assert_eq!(code, 4); // const, function, return, closing brace
    }

    #[test]
    fn test_count_lines_python() {
        let source = "# Comment
def foo():
    pass

# Another comment";
        let (total, code, blank, comment) = count_lines(source, LanguageId::Python);
        assert_eq!(total, 5);
        assert_eq!(blank, 1);
        assert_eq!(comment, 2);
        assert_eq!(code, 2);
    }

    #[test]
    fn test_count_lines_sql() {
        let source = "-- Comment
SELECT * FROM users;
/* Block */";
        let (total, code, blank, comment) = count_lines(source, LanguageId::Sql);
        assert_eq!(total, 3);
        assert_eq!(blank, 0);
        assert_eq!(comment, 2);
        assert_eq!(code, 1);
    }

    #[test]
    fn test_aggregate_statistics() {
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
                    m.insert(SymbolKind::Class, 2);
                    m
                },
            },
            FileStatistics {
                file_path: "bar.rs".to_string(),
                language_id: LanguageId::Rust,
                total_lines: 50,
                code_lines: 40,
                blank_lines: 5,
                comment_lines: 5,
                symbol_counts: {
                    let mut m = HashMap::new();
                    m.insert(SymbolKind::Function, 3);
                    m.insert(SymbolKind::Struct, 1);
                    m
                },
            },
        ];

        let response = aggregate_statistics(stats);

        assert_eq!(response.summary.total_files, 2);
        assert_eq!(response.summary.total_lines, 150);
        assert_eq!(response.summary.code_lines, 120);
        assert_eq!(response.summary.blank_lines, 15);
        assert_eq!(response.summary.comment_lines, 15);
        assert_eq!(response.summary.total_symbols, 11);
        assert_eq!(response.summary.languages_count, 2);

        // Check language breakdown
        assert_eq!(response.by_language.len(), 2);
        assert_eq!(response.by_language[0].language, "TypeScript"); // TypeScript has more code
        assert_eq!(response.by_language[0].code_lines, 80);

        // Check symbol stats
        assert_eq!(response.symbols.by_kind.get("Function"), Some(&8));
        assert_eq!(response.symbols.by_kind.get("Class"), Some(&2));
        assert_eq!(response.symbols.by_kind.get("Struct"), Some(&1));
        assert!((response.symbols.avg_symbols_per_file - 5.5).abs() < 0.01);
    }
}
