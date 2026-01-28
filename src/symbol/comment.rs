use std::path::Path;

use anyhow::Result;

use crate::symbol::types::{CodeSnippet, CommentMatch, CommentType};

/// Find comments containing the specified text in a file (C-style: // and /* */)
pub fn find_comments_in_file(file_path: &Path, search_text: &str) -> Result<Vec<CommentMatch>> {
    find_comments_with_prefix(file_path, search_text, "//")
}

/// Find comments containing the specified text in a SQL file (-- and /* */)
pub fn find_comments_in_sql_file(file_path: &Path, search_text: &str) -> Result<Vec<CommentMatch>> {
    find_comments_with_prefix(file_path, search_text, "--")
}

/// State of the comment scanner
#[derive(Default)]
enum ScanState {
    /// Not inside a block comment
    #[default]
    Normal,
    /// Inside a block comment
    InBlockComment {
        start_line: usize,
        start_col: usize,
        content: String,
    },
}

/// Scanner for finding comments in source code
struct CommentScanner<'a> {
    file_path_str: String,
    search_text: &'a str,
    single_line_prefix: &'a str,
    state: ScanState,
    matches: Vec<CommentMatch>,
}

impl<'a> CommentScanner<'a> {
    fn new(file_path: &Path, search_text: &'a str, single_line_prefix: &'a str) -> Self {
        Self {
            file_path_str: file_path.to_string_lossy().to_string(),
            search_text,
            single_line_prefix,
            state: ScanState::Normal,
            matches: Vec::new(),
        }
    }

    /// Process all lines in the source
    fn scan(mut self, source: &str) -> Vec<CommentMatch> {
        for (line_num, line) in source.lines().enumerate() {
            let line_1indexed = line_num + 1;
            self.process_line(line, line_1indexed);
        }

        // Handle case where file ends while still in a block comment
        self.finalize();
        self.matches
    }

    /// Process a single line
    fn process_line(&mut self, line: &str, line_1indexed: usize) {
        match &mut self.state {
            ScanState::InBlockComment { .. } => {
                self.process_block_comment_continuation(line, line_1indexed);
            }
            ScanState::Normal => {
                self.scan_for_comments(line, line_1indexed);
            }
        }
    }

    /// Continue processing a block comment that started on a previous line
    fn process_block_comment_continuation(&mut self, line: &str, line_1indexed: usize) {
        // Check if block comment ends on this line
        let end_pos = line.find("*/");

        if let Some(end_pos) = end_pos {
            // Block comment ends - take ownership of state to avoid clone
            let old_state = std::mem::take(&mut self.state);
            if let ScanState::InBlockComment {
                start_line,
                start_col,
                mut content,
            } = old_state
            {
                content.push_str(&line[..end_pos + 2]);

                if content.contains(self.search_text) {
                    self.add_match(start_line, start_col, CommentType::Block, content);
                }

                // Check for more comments after the block comment ends
                let remaining = &line[end_pos + 2..];
                let after_col = end_pos + 2;
                self.check_remaining_after_block(remaining, line_1indexed, after_col);
            }
        } else {
            // Block comment continues - mutate content in-place
            if let ScanState::InBlockComment { content, .. } = &mut self.state {
                content.push_str(line);
                content.push('\n');
            }
        }
    }

    /// Check for new comments after a block comment ends on the same line
    fn check_remaining_after_block(
        &mut self,
        remaining: &str,
        line_1indexed: usize,
        col_offset: usize,
    ) {
        if let Some(new_start) = remaining.find("/*") {
            self.state = ScanState::InBlockComment {
                start_line: line_1indexed,
                start_col: col_offset + new_start,
                content: format!("{}\n", &remaining[new_start..]),
            };
        } else if let Some(single_start) = remaining.find(self.single_line_prefix) {
            let comment_content = remaining[single_start..].to_string();
            if comment_content.contains(self.search_text) {
                self.add_match(
                    line_1indexed,
                    col_offset + single_start,
                    CommentType::SingleLine,
                    comment_content,
                );
            }
        }
    }

    /// Scan a line (not in block comment) for comments
    fn scan_for_comments(&mut self, line: &str, line_1indexed: usize) {
        let mut pos = 0;

        while pos < line.len() {
            let remaining = &line[pos..];
            let block_start = remaining.find("/*");
            let single_start = remaining.find(self.single_line_prefix);

            match (block_start, single_start) {
                // Both found - handle whichever comes first
                (Some(bs), Some(ss)) if ss < bs => {
                    // Single-line comment comes first, rest of line is comment
                    self.handle_single_line_comment(remaining, line_1indexed, pos, ss);
                    break;
                }
                (Some(bs), _) => {
                    // Block comment comes first
                    if let Some(new_pos) =
                        self.handle_block_comment_start(remaining, line_1indexed, pos, bs)
                    {
                        pos = new_pos;
                    } else {
                        // Block comment continues to next line
                        break;
                    }
                }
                (None, Some(ss)) => {
                    // Only single-line comment
                    self.handle_single_line_comment(remaining, line_1indexed, pos, ss);
                    break;
                }
                (None, None) => {
                    // No more comments on this line
                    break;
                }
            }
        }
    }

    /// Handle a single-line comment
    fn handle_single_line_comment(
        &mut self,
        remaining: &str,
        line_1indexed: usize,
        pos: usize,
        offset: usize,
    ) {
        let comment_content = remaining[offset..].to_string();
        if comment_content.contains(self.search_text) {
            self.add_match(
                line_1indexed,
                pos + offset,
                CommentType::SingleLine,
                comment_content,
            );
        }
    }

    /// Handle the start of a block comment. Returns the new position if the comment ends on the same line.
    fn handle_block_comment_start(
        &mut self,
        remaining: &str,
        line_1indexed: usize,
        pos: usize,
        block_start: usize,
    ) -> Option<usize> {
        let after_start = &remaining[block_start + 2..];

        if let Some(end_offset) = after_start.find("*/") {
            // Block comment ends on the same line
            let comment_content =
                remaining[block_start..block_start + 2 + end_offset + 2].to_string();
            if comment_content.contains(self.search_text) {
                self.add_match(
                    line_1indexed,
                    pos + block_start,
                    CommentType::Block,
                    comment_content,
                );
            }
            Some(pos + block_start + 2 + end_offset + 2)
        } else {
            // Block comment continues to next line
            self.state = ScanState::InBlockComment {
                start_line: line_1indexed,
                start_col: pos + block_start,
                content: format!("{}\n", &remaining[block_start..]),
            };
            None
        }
    }

    /// Add a match to the results
    fn add_match(
        &mut self,
        line: usize,
        column: usize,
        comment_type: CommentType,
        content: String,
    ) {
        self.matches.push(CommentMatch {
            file_path: self.file_path_str.clone(),
            line,
            column,
            comment_type,
            content,
        });
    }

    /// Finalize scanning - check for unclosed block comment at end of file
    fn finalize(&mut self) {
        if let ScanState::InBlockComment {
            start_line,
            start_col,
            ref content,
        } = self.state
        {
            if content.contains(self.search_text) {
                self.add_match(start_line, start_col, CommentType::Block, content.clone());
            }
        }
    }
}

/// Internal helper that finds comments with configurable single-line prefix.
///
/// Supports:
/// - Single-line comments with the specified prefix (e.g., "//" or "--")
/// - Block comments with /* */
fn find_comments_with_prefix(
    file_path: &Path,
    search_text: &str,
    single_line_prefix: &str,
) -> Result<Vec<CommentMatch>> {
    let source = std::fs::read_to_string(file_path)?;
    let scanner = CommentScanner::new(file_path, search_text, single_line_prefix);
    Ok(scanner.scan(&source))
}

/// Search for text in Markdown files (full text search)
///
/// Since Markdown doesn't have a concept of "comments", this function
/// performs a full text search and treats all matches as block comments.
pub fn find_text_in_markdown_file(
    file_path: &Path,
    search_text: &str,
) -> Result<Vec<CommentMatch>> {
    let source = std::fs::read_to_string(file_path)?;
    let file_path_str = file_path.to_string_lossy().to_string();

    let mut matches = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_1indexed = line_num + 1;

        // Find all occurrences of the search text in this line
        let mut search_start = 0;
        while let Some(pos) = line[search_start..].find(search_text) {
            let column = search_start + pos;
            matches.push(CommentMatch {
                file_path: file_path_str.clone(),
                line: line_1indexed,
                column,
                comment_type: CommentType::Block, // Treat as block for Markdown
                content: line.to_string(),
            });
            search_start = column + search_text.len();
        }
    }

    Ok(matches)
}

/// Get code at a specific location with context lines before and after
pub fn get_code_at_location(
    file_path: &Path,
    line: usize, // 1-indexed
    context_before: usize,
    context_after: usize,
) -> Result<CodeSnippet> {
    let source = std::fs::read_to_string(file_path)?;
    let lines: Vec<&str> = source.lines().collect();
    let total_lines = lines.len();

    // 範囲計算（1-indexed → 0-indexed）
    let target_idx = line.saturating_sub(1);
    let start_idx = target_idx.saturating_sub(context_before);
    let end_idx = (target_idx + context_after + 1).min(total_lines);

    let code = lines[start_idx..end_idx].join("\n");

    Ok(CodeSnippet {
        file_path: file_path.to_string_lossy().to_string(),
        start_line: start_idx + 1,
        end_line: end_idx,
        code,
    })
}

/// Extract documentation comments (JSDoc or regular comments) before a given line
pub fn extract_docs_before_line(source: &str, start_line: usize) -> Option<String> {
    if start_line == 0 {
        return None;
    }

    let lines: Vec<&str> = source.lines().collect();
    let mut doc_lines = Vec::new();
    let mut in_block_comment = false;

    // Start from the line before the definition and go backwards
    for i in (0..start_line).rev() {
        let line = lines.get(i)?;
        let trimmed = line.trim();

        if in_block_comment {
            doc_lines.push(*line);
            if trimmed.starts_with("/*") || trimmed.contains("/*") {
                in_block_comment = false;
            }
        } else if trimmed.ends_with("*/") {
            // Start of a block comment (reading backwards)
            in_block_comment = true;
            doc_lines.push(*line);
        } else if trimmed.starts_with("//") {
            // Single line comment
            doc_lines.push(*line);
        } else if trimmed.starts_with("*") && !trimmed.starts_with("*/") {
            // Middle of a JSDoc/block comment
            doc_lines.push(*line);
        } else if trimmed.is_empty() {
            // Allow empty lines at the beginning, but stop if we've already found comments
            if doc_lines.is_empty() {
                continue;
            } else {
                break;
            }
        } else {
            // Non-comment line, stop
            break;
        }
    }

    if doc_lines.is_empty() {
        None
    } else {
        doc_lines.reverse();
        Some(doc_lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_find_single_line_comments() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "// TODO: fix this").unwrap();
        writeln!(file, "const x = 1;").unwrap();
        writeln!(file, "// Another TODO here").unwrap();

        let matches = find_comments_in_file(file.path(), "TODO").unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line, 1);
        assert_eq!(matches[0].comment_type, CommentType::SingleLine);
        assert_eq!(matches[1].line, 3);
    }

    #[test]
    fn test_find_block_comments() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "/* FIXME: broken */").unwrap();
        writeln!(file, "const x = 1;").unwrap();
        writeln!(file, "/*").unwrap();
        writeln!(file, " * Another FIXME").unwrap();
        writeln!(file, " */").unwrap();

        let matches = find_comments_in_file(file.path(), "FIXME").unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].comment_type, CommentType::Block);
        assert_eq!(matches[1].comment_type, CommentType::Block);
    }

    #[test]
    fn test_extract_jsdoc() {
        let source = r#"/**
 * Process user data
 * @param user - The user object
 */
function processUser(user) {}"#;

        let docs = extract_docs_before_line(source, 4);
        assert!(docs.is_some());
        let docs = docs.unwrap();
        assert!(docs.contains("Process user data"));
        assert!(docs.contains("@param user"));
    }

    #[test]
    fn test_extract_single_line_comments() {
        let source = r#"// Helper function
// Does something useful
function helper() {}"#;

        let docs = extract_docs_before_line(source, 2);
        assert!(docs.is_some());
        let docs = docs.unwrap();
        assert!(docs.contains("Helper function"));
        assert!(docs.contains("Does something useful"));
    }

    #[test]
    fn test_no_docs() {
        let source = r#"const x = 1;
function foo() {}"#;

        let docs = extract_docs_before_line(source, 1);
        assert!(docs.is_none());
    }

    #[test]
    fn test_find_text_in_markdown() {
        let mut file = NamedTempFile::with_suffix(".md").unwrap();
        writeln!(file, "# Test Heading").unwrap();
        writeln!(file).unwrap();
        writeln!(file, "This is a test document.").unwrap();
        writeln!(file, "TODO: add more content").unwrap();
        writeln!(file, "Another TODO item here").unwrap();

        let matches = find_text_in_markdown_file(file.path(), "TODO").unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line, 4);
        assert_eq!(matches[1].line, 5);
        // All Markdown matches are treated as Block type
        assert_eq!(matches[0].comment_type, CommentType::Block);
    }

    #[test]
    fn test_find_text_in_markdown_multiple_per_line() {
        let mut file = NamedTempFile::with_suffix(".md").unwrap();
        writeln!(file, "test test test").unwrap();

        let matches = find_text_in_markdown_file(file.path(), "test").unwrap();
        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0].column, 0);
        assert_eq!(matches[1].column, 5);
        assert_eq!(matches[2].column, 10);
    }

    #[test]
    fn test_find_sql_single_line_comments() {
        let mut file = NamedTempFile::with_suffix(".sql").unwrap();
        writeln!(file, "-- TODO: fix this query").unwrap();
        writeln!(file, "SELECT * FROM users;").unwrap();
        writeln!(file, "-- Another TODO here").unwrap();

        let matches = find_comments_in_sql_file(file.path(), "TODO").unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line, 1);
        assert_eq!(matches[0].comment_type, CommentType::SingleLine);
        assert_eq!(matches[1].line, 3);
    }

    #[test]
    fn test_find_sql_block_comments() {
        let mut file = NamedTempFile::with_suffix(".sql").unwrap();
        writeln!(file, "/* FIXME: broken query */").unwrap();
        writeln!(file, "SELECT * FROM users;").unwrap();
        writeln!(file, "/*").unwrap();
        writeln!(file, " * Another FIXME").unwrap();
        writeln!(file, " */").unwrap();

        let matches = find_comments_in_sql_file(file.path(), "FIXME").unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].comment_type, CommentType::Block);
        assert_eq!(matches[1].comment_type, CommentType::Block);
    }

    #[test]
    fn test_find_sql_mixed_comments() {
        let mut file = NamedTempFile::with_suffix(".sql").unwrap();
        writeln!(file, "-- TODO: single line").unwrap();
        writeln!(file, "/* TODO: block comment */").unwrap();
        writeln!(file, "SELECT * FROM users; -- inline TODO").unwrap();

        let matches = find_comments_in_sql_file(file.path(), "TODO").unwrap();
        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0].line, 1);
        assert_eq!(matches[0].comment_type, CommentType::SingleLine);
        assert_eq!(matches[1].line, 2);
        assert_eq!(matches[1].comment_type, CommentType::Block);
        assert_eq!(matches[2].line, 3);
        assert_eq!(matches[2].comment_type, CommentType::SingleLine);
    }

    // ========== Edge case tests for CommentScanner ==========

    #[test]
    fn test_multiple_block_comments_on_one_line() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "/* TODO: first */ code /* TODO: second */").unwrap();

        let matches = find_comments_in_file(file.path(), "TODO").unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line, 1);
        assert_eq!(matches[0].column, 0);
        assert_eq!(matches[0].comment_type, CommentType::Block);
        assert!(matches[0].content.contains("first"));
        assert_eq!(matches[1].line, 1);
        assert_eq!(matches[1].column, 23);
        assert_eq!(matches[1].comment_type, CommentType::Block);
        assert!(matches[1].content.contains("second"));
    }

    #[test]
    fn test_block_comment_followed_by_single_line_on_same_line() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "/* TODO: block */ code // TODO: single").unwrap();

        let matches = find_comments_in_file(file.path(), "TODO").unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].comment_type, CommentType::Block);
        assert!(matches[0].content.contains("block"));
        assert_eq!(matches[1].comment_type, CommentType::SingleLine);
        assert!(matches[1].content.contains("single"));
    }

    #[test]
    fn test_multiline_block_followed_by_single_line_on_end_line() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "/*").unwrap();
        writeln!(file, " * TODO: multiline block").unwrap();
        writeln!(file, " */ // TODO: trailing single").unwrap();

        let matches = find_comments_in_file(file.path(), "TODO").unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line, 1); // Block starts on line 1
        assert_eq!(matches[0].comment_type, CommentType::Block);
        assert!(matches[0].content.contains("multiline block"));
        assert_eq!(matches[1].line, 3); // Single-line on line 3
        assert_eq!(matches[1].comment_type, CommentType::SingleLine);
        assert!(matches[1].content.contains("trailing single"));
    }

    #[test]
    fn test_multiline_block_followed_by_new_block_on_end_line() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "/*").unwrap();
        writeln!(file, " * TODO: first block").unwrap();
        writeln!(file, " */ /* TODO: second block */").unwrap();

        let matches = find_comments_in_file(file.path(), "TODO").unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line, 1);
        assert_eq!(matches[0].comment_type, CommentType::Block);
        assert!(matches[0].content.contains("first block"));
        assert_eq!(matches[1].line, 3);
        assert_eq!(matches[1].comment_type, CommentType::Block);
        assert!(matches[1].content.contains("second block"));
    }

    #[test]
    fn test_unclosed_block_comment_at_end_of_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "code").unwrap();
        writeln!(file, "/* TODO: unclosed block").unwrap();
        writeln!(file, " * more content").unwrap();
        // No closing */
        let matches = find_comments_in_file(file.path(), "TODO").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].line, 2);
        assert_eq!(matches[0].comment_type, CommentType::Block);
        assert!(matches[0].content.contains("unclosed block"));
        assert!(matches[0].content.contains("more content"));
    }

    #[test]
    fn test_single_line_comment_before_block_on_same_line() {
        let mut file = NamedTempFile::new().unwrap();
        // Single-line comment comes before block comment syntax
        writeln!(file, "// TODO: single /* not a block */").unwrap();

        let matches = find_comments_in_file(file.path(), "TODO").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].comment_type, CommentType::SingleLine);
        // The entire rest of line is part of single-line comment
        assert!(matches[0].content.contains("/* not a block */"));
    }

    #[test]
    fn test_large_multiline_block_comment_no_quadratic_behavior() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "/* TODO: start").unwrap();
        // Add many lines to verify no O(n²) behavior
        for i in 0..100 {
            writeln!(file, " * line {}", i).unwrap();
        }
        writeln!(file, " */").unwrap();

        let matches = find_comments_in_file(file.path(), "TODO").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].line, 1);
        assert_eq!(matches[0].comment_type, CommentType::Block);
        // Verify all lines are captured
        assert!(matches[0].content.contains("line 99"));
    }

    #[test]
    fn test_mixed_block_and_single_line_complex() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "/* MARKER: a */ x /* MARKER: b */ // MARKER: c").unwrap();
        writeln!(file, "// MARKER: d").unwrap();
        writeln!(file, "/*").unwrap();
        writeln!(file, " * MARKER: e").unwrap();
        writeln!(file, " */ /* MARKER: f").unwrap();
        writeln!(file, " */").unwrap();

        let matches = find_comments_in_file(file.path(), "MARKER").unwrap();
        assert_eq!(matches.len(), 6);

        // Line 1: block a, block b, single c
        assert_eq!(matches[0].line, 1);
        assert!(matches[0].content.contains("a"));
        assert_eq!(matches[1].line, 1);
        assert!(matches[1].content.contains("b"));
        assert_eq!(matches[2].line, 1);
        assert!(matches[2].content.contains("c"));

        // Line 2: single d
        assert_eq!(matches[3].line, 2);
        assert!(matches[3].content.contains("d"));

        // Lines 3-5: block e
        assert_eq!(matches[4].line, 3);
        assert!(matches[4].content.contains("e"));

        // Lines 5-6: block f
        assert_eq!(matches[5].line, 5);
        assert!(matches[5].content.contains("f"));
    }
}
