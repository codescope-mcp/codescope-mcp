---
description: Find TODO/FIXME comments in code
argument-hint: <search-text>
---

# /codescope:todos

Search within comments only. Unlike grep, excludes code and strings.

## Usage
/codescope:todos <search-text>

## Examples
/codescope:todos TODO      # Find all TODOs
/codescope:todos FIXME     # Find FIXMEs
/codescope:todos refactor  # Find "refactor" in comments

## How It Works
Uses `find_in_comments` to search only within actual comments.
