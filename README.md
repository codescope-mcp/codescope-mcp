# CodeScope MCP

An MCP (Model Context Protocol) server that provides symbol analysis and code navigation for TypeScript, JavaScript, Python, Rust, HTML, CSS, and Markdown projects.

## Features

- **symbol_definition**: Find symbol definitions (with JSDoc/comments support)
- **symbol_usages**: Find all usages of a symbol
- **find_method_calls**: Find method/function calls (e.g., `Date.now()`, `array.map()`)
- **find_imports**: Find import statements for a symbol
- **find_in_comments**: Search text in comments (TypeScript/TSX) or full text (Markdown)
- **get_code_at_location**: Get code snippet at a specific file:line
- **get_symbol_at_location**: Get the enclosing symbol at a specific file:line

### Supported Languages

| Language | Extensions | Symbol Types |
|----------|------------|--------------|
| TypeScript | `.ts` | Functions, Classes, Methods, Constructors, Interfaces, Enums, Variables, Arrow Functions, Type Aliases |
| TypeScript React | `.tsx` | Same as TypeScript |
| JavaScript | `.js`, `.mjs`, `.cjs` | Functions, Classes, Methods, Constructors, Variables, Arrow Functions |
| JavaScript React | `.jsx` | Same as JavaScript |
| Python | `.py`, `.pyi` | Functions, Classes, Methods, Constructors (__init__), Variables |
| Rust | `.rs` | Functions, Structs, Enums, Traits, Impls, Methods, Type Aliases, Modules, Consts, Statics, Macros |
| HTML | `.html`, `.htm` | Elements, IDs, Classes |
| CSS | `.css` | Class Selectors, ID Selectors, Variables, Keyframes |
| Markdown | `.md`, `.mdc` | Headings (H1-H6), Code Blocks, Link References |

## Installation

### As a Claude Code Plugin

Install directly from the GitHub repository:

```bash
claude mcp add codescope -- cargo install --git https://github.com/t0k0sh1/codescope-mcp
```

After installation, the following tools will be available:

- `symbol_definition`
- `symbol_usages`
- `find_method_calls`
- `find_imports`
- `find_in_comments`
- `get_code_at_location`
- `get_symbol_at_location`

Use the skill `/codescope:symbol-analysis` for usage guidance.

### Manual Installation

```bash
# Clone the repository
git clone https://github.com/t0k0sh1/codescope-mcp.git
cd codescope-mcp

# Build
cargo build --release

# Run as standalone MCP server
cargo run --release
```

## Configuration

Environment variables:

- `RUST_LOG`: Log level (e.g., `info`, `debug`, `warn`, `error`)

## Usage Examples

### Find Symbol Definition

```json
{
  "symbol": "UserService",
  "include_docs": true,
  "language": "typescript"
}
```

### Find Symbol Usages

```json
{
  "symbol": "useState",
  "include_contexts": true,
  "language": "typescriptreact"
}
```

### Search in Markdown

```json
{
  "symbol": "Installation",
  "language": "markdown"
}
```

### Language Filter

All tools support the optional `language` parameter:

- `"typescript"` or `"ts"` - TypeScript files only
- `"typescriptreact"` or `"tsx"` - TSX files only
- `"javascript"` or `"js"` - JavaScript files only
- `"javascriptreact"` or `"jsx"` - JSX files only
- `"python"` or `"py"` - Python files only
- `"rust"` or `"rs"` - Rust files only
- `"html"` - HTML files only
- `"css"` - CSS files only
- `"markdown"` or `"md"` - Markdown files only
- `null` or omitted - All supported languages

## Development

```bash
# Run tests
cargo test

# Debug build
cargo build

# Release build
cargo build --release
```

## License

MIT
