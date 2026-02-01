---
description: Find symbol definitions and usages in code
argument-hint: <symbol-name> [--usages] [--docs]
---

# /codescope:symbol

Find where symbols (functions, classes, variables) are defined and used.

## Usage
/codescope:symbol <symbol-name> [options]

## Examples
/codescope:symbol UserService
/codescope:symbol handleRequest --usages
/codescope:symbol useState --docs

## How It Works
1. `symbol_definition` finds where the symbol is defined
2. Add `--usages` to also find all usages via `symbol_usages`
3. Add `--docs` to include JSDoc/docstrings
