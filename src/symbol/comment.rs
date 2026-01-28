use std::path::Path;

use anyhow::Result;

use crate::symbol::types::{CodeSnippet, CommentMatch, CommentType};

/// Find comments containing the specified text in a file
pub fn find_comments_in_file(file_path: &Path, search_text: &str) -> Result<Vec<CommentMatch>> {
    let source = std::fs::read_to_string(file_path)?;
    let file_path_str = file_path.to_string_lossy().to_string();

    let mut matches = Vec::new();
    let mut in_block_comment = false;
    let mut block_comment_start_line = 0;
    let mut block_comment_start_col = 0;
    let mut block_comment_content = String::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_1indexed = line_num + 1;

        if in_block_comment {
            // Continue collecting block comment content
            if let Some(end_pos) = line.find("*/") {
                // Block comment ends on this line
                block_comment_content.push_str(&line[..end_pos + 2]);
                in_block_comment = false;

                // Check if the block comment contains the search text
                if block_comment_content.contains(search_text) {
                    matches.push(CommentMatch {
                        file_path: file_path_str.clone(),
                        line: block_comment_start_line,
                        column: block_comment_start_col,
                        comment_type: CommentType::Block,
                        content: block_comment_content.clone(),
                    });
                }
                block_comment_content.clear();

                // Check if there's another comment start after the end
                let remaining = &line[end_pos + 2..];
                if let Some(new_start) = remaining.find("/*") {
                    in_block_comment = true;
                    block_comment_start_line = line_1indexed;
                    block_comment_start_col = end_pos + 2 + new_start;
                    block_comment_content = remaining[new_start..].to_string();
                    block_comment_content.push('\n');
                } else if let Some(single_start) = remaining.find("//") {
                    let comment_content = remaining[single_start..].to_string();
                    if comment_content.contains(search_text) {
                        matches.push(CommentMatch {
                            file_path: file_path_str.clone(),
                            line: line_1indexed,
                            column: end_pos + 2 + single_start,
                            comment_type: CommentType::SingleLine,
                            content: comment_content,
                        });
                    }
                }
            } else {
                // Block comment continues
                block_comment_content.push_str(line);
                block_comment_content.push('\n');
            }
        } else {
            // Not in a block comment, look for comment starts
            let mut pos = 0;
            while pos < line.len() {
                let remaining = &line[pos..];

                // Check for block comment start
                if let Some(block_start) = remaining.find("/*") {
                    // Check for single line comment before block comment
                    if let Some(single_start) = remaining.find("//") {
                        if single_start < block_start {
                            // Single line comment comes first
                            let comment_content = remaining[single_start..].to_string();
                            if comment_content.contains(search_text) {
                                matches.push(CommentMatch {
                                    file_path: file_path_str.clone(),
                                    line: line_1indexed,
                                    column: pos + single_start,
                                    comment_type: CommentType::SingleLine,
                                    content: comment_content,
                                });
                            }
                            break;
                        }
                    }

                    // Check if block comment ends on the same line
                    let after_start = &remaining[block_start + 2..];
                    if let Some(end_offset) = after_start.find("*/") {
                        // Block comment ends on this line
                        let comment_content =
                            remaining[block_start..block_start + 2 + end_offset + 2].to_string();
                        if comment_content.contains(search_text) {
                            matches.push(CommentMatch {
                                file_path: file_path_str.clone(),
                                line: line_1indexed,
                                column: pos + block_start,
                                comment_type: CommentType::Block,
                                content: comment_content,
                            });
                        }
                        pos += block_start + 2 + end_offset + 2;
                    } else {
                        // Block comment continues to next line
                        in_block_comment = true;
                        block_comment_start_line = line_1indexed;
                        block_comment_start_col = pos + block_start;
                        block_comment_content = remaining[block_start..].to_string();
                        block_comment_content.push('\n');
                        break;
                    }
                } else if let Some(single_start) = remaining.find("//") {
                    // Single line comment
                    let comment_content = remaining[single_start..].to_string();
                    if comment_content.contains(search_text) {
                        matches.push(CommentMatch {
                            file_path: file_path_str.clone(),
                            line: line_1indexed,
                            column: pos + single_start,
                            comment_type: CommentType::SingleLine,
                            content: comment_content,
                        });
                    }
                    break;
                } else {
                    break;
                }
            }
        }
    }

    // Handle case where file ends while still in a block comment
    if in_block_comment && block_comment_content.contains(search_text) {
        matches.push(CommentMatch {
            file_path: file_path_str,
            line: block_comment_start_line,
            column: block_comment_start_col,
            comment_type: CommentType::Block,
            content: block_comment_content,
        });
    }

    Ok(matches)
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

/// Find comments containing the specified text in a SQL file
///
/// SQL supports:
/// - `--` single-line comments
/// - `/* */` block comments
pub fn find_comments_in_sql_file(file_path: &Path, search_text: &str) -> Result<Vec<CommentMatch>> {
    let source = std::fs::read_to_string(file_path)?;
    let file_path_str = file_path.to_string_lossy().to_string();

    let mut matches = Vec::new();
    let mut in_block_comment = false;
    let mut block_comment_start_line = 0;
    let mut block_comment_start_col = 0;
    let mut block_comment_content = String::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_1indexed = line_num + 1;

        if in_block_comment {
            // Continue collecting block comment content
            if let Some(end_pos) = line.find("*/") {
                // Block comment ends on this line
                block_comment_content.push_str(&line[..end_pos + 2]);
                in_block_comment = false;

                // Check if the block comment contains the search text
                if block_comment_content.contains(search_text) {
                    matches.push(CommentMatch {
                        file_path: file_path_str.clone(),
                        line: block_comment_start_line,
                        column: block_comment_start_col,
                        comment_type: CommentType::Block,
                        content: block_comment_content.clone(),
                    });
                }
                block_comment_content.clear();

                // Check remaining content after block comment end
                let remaining = &line[end_pos + 2..];
                if let Some(new_start) = remaining.find("/*") {
                    in_block_comment = true;
                    block_comment_start_line = line_1indexed;
                    block_comment_start_col = end_pos + 2 + new_start;
                    block_comment_content = remaining[new_start..].to_string();
                    block_comment_content.push('\n');
                } else if let Some(single_start) = remaining.find("--") {
                    let comment_content = remaining[single_start..].to_string();
                    if comment_content.contains(search_text) {
                        matches.push(CommentMatch {
                            file_path: file_path_str.clone(),
                            line: line_1indexed,
                            column: end_pos + 2 + single_start,
                            comment_type: CommentType::SingleLine,
                            content: comment_content,
                        });
                    }
                }
            } else {
                // Block comment continues
                block_comment_content.push_str(line);
                block_comment_content.push('\n');
            }
        } else {
            // Not in a block comment, look for comment starts
            let mut pos = 0;
            while pos < line.len() {
                let remaining = &line[pos..];

                // Check for block comment start
                if let Some(block_start) = remaining.find("/*") {
                    // Check for single line comment before block comment
                    if let Some(single_start) = remaining.find("--") {
                        if single_start < block_start {
                            // Single line comment comes first (rest of line is comment)
                            let comment_content = remaining[single_start..].to_string();
                            if comment_content.contains(search_text) {
                                matches.push(CommentMatch {
                                    file_path: file_path_str.clone(),
                                    line: line_1indexed,
                                    column: pos + single_start,
                                    comment_type: CommentType::SingleLine,
                                    content: comment_content,
                                });
                            }
                            break;
                        }
                    }

                    // Check if block comment ends on the same line
                    let after_start = &remaining[block_start + 2..];
                    if let Some(end_offset) = after_start.find("*/") {
                        // Block comment ends on this line
                        let comment_content =
                            remaining[block_start..block_start + 2 + end_offset + 2].to_string();
                        if comment_content.contains(search_text) {
                            matches.push(CommentMatch {
                                file_path: file_path_str.clone(),
                                line: line_1indexed,
                                column: pos + block_start,
                                comment_type: CommentType::Block,
                                content: comment_content,
                            });
                        }
                        pos += block_start + 2 + end_offset + 2;
                    } else {
                        // Block comment continues to next line
                        in_block_comment = true;
                        block_comment_start_line = line_1indexed;
                        block_comment_start_col = pos + block_start;
                        block_comment_content = remaining[block_start..].to_string();
                        block_comment_content.push('\n');
                        break;
                    }
                } else if let Some(single_start) = remaining.find("--") {
                    // SQL single line comment
                    let comment_content = remaining[single_start..].to_string();
                    if comment_content.contains(search_text) {
                        matches.push(CommentMatch {
                            file_path: file_path_str.clone(),
                            line: line_1indexed,
                            column: pos + single_start,
                            comment_type: CommentType::SingleLine,
                            content: comment_content,
                        });
                    }
                    break;
                } else {
                    break;
                }
            }
        }
    }

    // Handle case where file ends while still in a block comment
    if in_block_comment && block_comment_content.contains(search_text) {
        matches.push(CommentMatch {
            file_path: file_path_str,
            line: block_comment_start_line,
            column: block_comment_start_col,
            comment_type: CommentType::Block,
            content: block_comment_content,
        });
    }

    Ok(matches)
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
}
