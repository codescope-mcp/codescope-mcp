---
name: method-calls
description: Find method calls like Date.now(), array.map(), console.log(). Can filter by object name to distinguish Date.now() from performance.now(). UNIQUE feature - no other tool can filter method calls by object.
---

# CodeScope Method Calls Search

Find method/function calls with optional object filtering. This is a UNIQUE feature that no other tool provides.

## When to Use

- Find all `Date.now()` calls (not `performance.now()`)
- Find all `console.log()` calls
- Find all `array.map()` calls
- Find all calls to a specific method across the codebase

## Tool: find_method_calls

**Parameters:**
- `method_name` (required): The method/function name to search for
- `object_name` (optional): Filter by the object the method is called on
- `exclude_dirs` (optional): Directories to exclude (e.g., `["node_modules", "dist"]`)
- `language` (optional): Filter by language (e.g., `"typescript"`, `"python"`)

## Examples

### Find Date.now() calls only
```
find_method_calls(method_name="now", object_name="Date")
```
This will find `Date.now()` but NOT `performance.now()` or other `.now()` calls.

### Find all console.log() calls
```
find_method_calls(method_name="log", object_name="console")
```

### Find all .map() calls (any object)
```
find_method_calls(method_name="map")
```

### Find fetch() calls in TypeScript files only
```
find_method_calls(method_name="fetch", language="typescript")
```

## Supported Languages

TypeScript, JavaScript (including TSX and JSX)

## Why Use This Instead of Grep?

| Grep | find_method_calls |
|------|-------------------|
| `grep "\.now("` matches `Date.now()`, `performance.now()`, strings, comments | Filters by object name precisely |
| Can't distinguish method calls from variable access | AST-aware, only finds actual calls |
| False positives in strings/comments | Only matches code constructs |
