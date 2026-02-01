---
name: imports
description: Find import/require statements for a symbol. Track module dependencies and see where/how a library is imported across the codebase. More precise than grep.
---

# CodeScope Imports Search

Find import/require statements for a symbol. Specialized import search that's more precise than grep.

## When to Use

- Find where `useState` is imported from
- Track where a library like `lodash` is imported
- Find all files that import a specific module
- Understand module dependencies

## Tool: find_imports

**Parameters:**
- `symbol` (required): The symbol/module name to search for
- `exclude_dirs` (optional): Directories to exclude (e.g., `["node_modules", "dist"]`)
- `language` (optional): Filter by language (e.g., `"typescript"`, `"python"`)

## Examples

### Find where useState is imported
```
find_imports(symbol="useState")
```
Returns all import statements that include `useState`.

### Find lodash imports
```
find_imports(symbol="lodash")
```

### Find React imports across the project
```
find_imports(symbol="React")
```

## Supported Languages

TypeScript, JavaScript (including TSX and JSX)

## What It Finds

| Language | Import Patterns |
|----------|-----------------|
| TypeScript/JavaScript | `import { x } from 'y'`, `import x from 'y'`, `require('y')` |

## Why Use This Instead of Grep?

| Grep | find_imports |
|------|--------------|
| `grep "import.*useState"` may match comments/strings | Only matches actual import statements |
| Regex complexity for all import formats | Understands all language-specific patterns |
| Manual filtering of results | AST-aware, precise results |
