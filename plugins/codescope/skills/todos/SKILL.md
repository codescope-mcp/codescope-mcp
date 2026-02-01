---
name: todos
description: Find TODO, FIXME, HACK comments in codebase. Searches ONLY in comments - excludes code and strings. UNIQUE feature - grep cannot distinguish comments from code.
---

# CodeScope TODO/FIXME Comment Search

Search text ONLY within comments. This is a UNIQUE feature - grep cannot distinguish comments from code or strings.

## When to Use

- Find all TODO comments in the codebase
- Find all FIXME comments that need attention
- Find HACK comments that need cleanup
- Search for any text within comments only

## Tool: find_in_comments

**Parameters:**
- `text` (required): The text to search for within comments
- `exclude_dirs` (optional): Directories to exclude (e.g., `["node_modules", "dist"]`)
- `language` (optional): Filter by language (e.g., `"typescript"`, `"python"`)

## Examples

### Find all TODO comments
```
find_in_comments(text="TODO")
```

### Find all FIXME comments
```
find_in_comments(text="FIXME")
```

### Find HACK comments in TypeScript files
```
find_in_comments(text="HACK", language="typescript")
```

### Find comments mentioning a specific topic
```
find_in_comments(text="refactor")
```

### Find deprecated warnings in comments
```
find_in_comments(text="deprecated")
```

## Supported Languages

TypeScript, JavaScript, Rust, Go, Java, HTML, CSS

Note: Python is not supported because this tool only recognizes C-style comments (`//` and `/* */`), not Python's `#` comments.

## Why Use This Instead of Grep?

| Grep | find_in_comments |
|------|------------------|
| `grep "TODO"` matches code like `const TODO = "value"` | Only matches actual comments |
| Matches strings like `"TODO: example"` | Excludes string literals |
| Can't filter by comment type | AST-aware, precise results |

## Example Comparison

Given this code:
```typescript
// TODO: Fix this later
const message = "TODO: complete this";
const TODO = "task";
/* TODO: Another task */
```

- `grep "TODO"` returns **4 matches** (including variable and string)
- `find_in_comments(text="TODO")` returns **2 matches** (only the actual comments)
