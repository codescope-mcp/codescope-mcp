---
description: Find import statements for a symbol
argument-hint: <symbol-name>
---

# /codescope:imports

Find where a symbol is imported across the codebase.

## Usage
/codescope:imports <symbol-name>

## Examples
/codescope:imports useState
/codescope:imports lodash
/codescope:imports React

## How It Works
Uses `find_imports` to locate all import/require statements for the symbol.
