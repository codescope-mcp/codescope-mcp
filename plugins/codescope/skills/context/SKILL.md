---
name: context
description: Get the enclosing function/class/method at a specific file:line. Use after grep or search to understand the full context of a code location.
argument-hint: "[file:line]"
---

# CodeScope Context Retrieval

Get the enclosing function/class/method at a specific line. Use after grep or symbol_usages to get full context of a code location.

## When to Use

- After grep finds a match, get the full function containing it
- After symbol_usages, understand the context where a symbol is used
- Get the complete implementation of a function at a known line
- Understand what function/class surrounds a specific line

## Tool: get_symbol_at_location

**Parameters:**
- `file_path` (required): Path to the file
- `line` (required): Line number (1-indexed)

**Returns:**
- Symbol name
- Symbol kind (function, class, method, etc.)
- Start and end line numbers
- Full source code of the symbol

## Workflow Examples

### After Grep
1. `grep "handleError"` finds match at `src/handler.ts:42`
2. `get_symbol_at_location(file_path="src/handler.ts", line=42)` returns the entire function containing line 42

### After symbol_usages
1. `symbol_usages(symbol="processData")` finds usage at `src/service.ts:128`
2. `get_symbol_at_location(file_path="src/service.ts", line=128)` returns the complete function where `processData` is called

### Direct Lookup
If you know a function is around line 50:
```
get_symbol_at_location(file_path="src/main.rs", line=50)
```

## Supported Languages

TypeScript, TSX, JavaScript, JSX, Python, Rust, Go, Java, HTML, CSS, SQL, Markdown

## Behavior

- Returns the **smallest enclosing symbol** at the given line
- If line 42 is inside a method inside a class, returns the method (not the class)
- If no symbol contains the line, returns null

## Example

Given this code:
```typescript
class UserService {           // line 1
  async getUser(id: string) { // line 2
    const user = await db.find(id); // line 3
    return user;              // line 4
  }                           // line 5
}                             // line 6
```

`get_symbol_at_location(file_path="...", line=3)` returns:
- **name**: `getUser`
- **kind**: Method
- **start_line**: 2
- **end_line**: 5
- **code**: The entire `getUser` method body

## Related Tool: get_code_at_location

If you just need a code snippet (not the full enclosing symbol):
```
get_code_at_location(file_path="src/main.ts", line=42, context_before=5, context_after=5)
```
Returns lines 37-47 without determining symbol boundaries.
