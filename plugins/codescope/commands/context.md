---
description: Get enclosing function/class at file:line
argument-hint: <file:line>
---

# /codescope:context

Get the full function/class containing a specific line.

## Usage
/codescope:context <file-path>:<line-number>

## Examples
/codescope:context src/handler.ts:42
/codescope:context lib/utils.py:128

## How It Works
Uses `get_symbol_at_location` to find the smallest enclosing symbol.
