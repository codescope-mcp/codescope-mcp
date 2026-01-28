---
name: symbol-analysis
description: AST-aware code search for TypeScript, JavaScript, Python, Rust, Go, Java, HTML, CSS, SQL, Markdown. Use instead of Grep/Read to find symbol definitions, symbol usages, method calls, imports, and comments. Provides precise results by parsing code structure.
---

# CodeScope Symbol Analysis Skill

AST-aware symbol search and code navigation for TypeScript, JavaScript, Python, Rust, Go, Java, HTML, CSS, SQL, and Markdown projects.

## When to Use CodeScope (vs Standard Tools)

### Use CodeScope when:

| Task | CodeScope Tool | Instead of |
|------|----------------|------------|
| Find where a function/class is defined | `symbol_definition` | `grep "function name"` or `grep "class Name"` |
| Find all usages of a symbol | `symbol_usages` | `grep "symbolName"` |
| Find method calls like `obj.method()` | `find_method_calls` | `grep "\.method("` |
| Find import statements | `find_imports` | `grep "import.*symbol"` |
| Get full function/class code at a line | `get_symbol_at_location` | `Read` with manual line range |
| Search TODO/FIXME in comments only | `find_in_comments` | `grep "TODO"` (matches non-comments too) |

### Why CodeScope is Better for Symbol Analysis:

- **AST-aware**: Parses code structure, distinguishes definitions from usages
- **Precise**: Won't match partial names, strings, or comments accidentally
- **Semantic**: Understands code constructs (methods vs functions, classes vs variables)
- **Multi-language**: Same interface for all supported languages with language-specific understanding

### Use Standard Tools (Grep/Read) when:

- Searching for arbitrary text patterns (not code symbols)
- Reading configuration files (JSON, YAML, TOML)
- Searching in unsupported file types
- Need regex pattern matching

## Available Tools

### symbol_definition
Find where symbols (functions, classes, variables, etc.) are defined. Can include JSDoc/docstring comments.

**Parameters:**
- `symbol`: Symbol name to search (required)
- `include_docs`: Include JSDoc/comments (default: false)
- `exclude_dirs`: Directories to exclude (e.g., `["dist", "node_modules"]`)
- `language`: Language filter (e.g., `"typescript"`, `"python"`, `"rust"`, `"sql"`)

**Example:** Find `UserService` class definition with documentation:
```
symbol_definition(symbol="UserService", include_docs=true)
```

### symbol_usages
Find all places where a symbol is used (excluding its definition).

**Parameters:**
- `symbol`: Symbol name to search (required)
- `include_contexts`: Include surrounding code context (default: false)
- `exclude_dirs`: Directories to exclude
- `language`: Language filter

**Example:** Find all usages of `handleRequest`:
```
symbol_usages(symbol="handleRequest", include_contexts=true)
```

### find_method_calls
Find method/function calls with optional object filtering (e.g., `Date.now()`, `array.map()`).

**Parameters:**
- `method_name`: Method/function name (required)
- `object_name`: Filter by object name (e.g., `"Date"` to find only `Date.now()`)
- `exclude_dirs`: Directories to exclude
- `language`: Language filter

**Example:** Find only `Date.now()` calls (not `performance.now()`):
```
find_method_calls(method_name="now", object_name="Date")
```

### find_imports
Find import/require statements for a symbol.

**Parameters:**
- `symbol`: Symbol name (required)
- `exclude_dirs`: Directories to exclude
- `language`: Language filter

**Example:** Find where `useState` is imported:
```
find_imports(symbol="useState")
```

### find_in_comments
Search text within comments only (TODO, FIXME, etc.). Unlike Grep, this won't match code or strings.

**Parameters:**
- `text`: Search text (required)
- `exclude_dirs`: Directories to exclude
- `language`: Language filter

**Example:** Find all TODO comments:
```
find_in_comments(text="TODO")
```

### get_code_at_location
Get a code snippet at a specific file location with context lines.

**Parameters:**
- `file_path`: File path (required)
- `line`: Line number, 1-indexed (required)
- `context_before`: Lines before target (default: 3)
- `context_after`: Lines after target (default: 3)

### get_symbol_at_location
Get the smallest enclosing symbol (function, class, method) at a specific line. Useful after `symbol_usages` to get full context.

**Parameters:**
- `file_path`: File path (required)
- `line`: Line number, 1-indexed (required)

**Returns:** Symbol name, kind, line range, and full code

**Example workflow:**
1. Find usages: `symbol_usages(symbol="processData")`
2. Get full context: `get_symbol_at_location(file_path="src/handler.ts", line=42)`

This retrieves the entire function where `processData` is called.

## Supported Languages

| Language | Extensions | Symbol Types |
|----------|------------|--------------|
| TypeScript | `.ts` | Functions, Classes, Methods, Interfaces, Enums, Variables, Type Aliases |
| TypeScript React | `.tsx` | Same as TypeScript |
| JavaScript | `.js`, `.mjs`, `.cjs` | Functions, Classes, Methods, Variables, Arrow Functions |
| JavaScript React | `.jsx` | Same as JavaScript |
| Python | `.py`, `.pyi` | Functions, Classes, Methods, Variables |
| Rust | `.rs` | Functions, Structs, Enums, Traits, Impls, Methods, Modules, Macros |
| Go | `.go` | Functions, Methods, Structs, Interfaces, Type Aliases |
| Java | `.java` | Classes, Interfaces, Enums, Methods, Fields, Annotations |
| HTML | `.html`, `.htm` | Elements, IDs, Classes |
| CSS | `.css` | Selectors, Variables, Keyframes |
| SQL | `.sql` | Tables, Views, Procedures, Indexes, Triggers |
| Markdown | `.md`, `.mdc` | Headings, Code Blocks, Link References |
