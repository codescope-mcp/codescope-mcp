---
description: Find method calls like obj.method()
argument-hint: <method> [object]
---

# /codescope:calls

Find method calls with optional object filtering.

## Usage
/codescope:calls <method-name> [object-name]

## Examples
/codescope:calls now Date        # Date.now() only
/codescope:calls log console     # console.log()
/codescope:calls map             # all .map() calls

## How It Works
Uses `find_method_calls` to find calls. Specify object name to filter precisely.
