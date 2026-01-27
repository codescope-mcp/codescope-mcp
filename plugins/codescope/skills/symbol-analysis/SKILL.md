---
name: symbol-analysis
description: TypeScript/TSXコードベースのシンボル検索・解析
---

# CodeScope シンボル解析スキル

TypeScript/TSXプロジェクトのシンボル検索と解析を行うためのガイダンスです。

## 利用可能なツール

CodeScope MCPは以下のツールを提供します：

### symbol_definition
シンボル（関数、クラス、変数など）の定義を検索します。JSDocコメントも含めて取得できます。

**パラメータ:**
- `symbol`: 検索するシンボル名（必須）
- `include_docs`: JSDoc/コメントを含める（デフォルト: false）
- `exclude_dirs`: 除外するディレクトリ（例: `["dist", "node_modules"]`）
- `language`: 言語フィルタ（例: `"typescript"`, `"typescriptreact"`, `"markdown"`）

### symbol_usages
シンボルの使用箇所を検索します。

**パラメータ:**
- `symbol`: 検索するシンボル名（必須）
- `include_contexts`: コンテキスト情報を含める（デフォルト: false）
- `exclude_dirs`: 除外するディレクトリ
- `language`: 言語フィルタ

### find_method_calls
メソッドや関数の呼び出し箇所を検索します（例: `Date.now()`, `array.map()`）。

**パラメータ:**
- `method_name`: メソッド/関数名（必須）
- `object_name`: オブジェクト名でフィルタ（例: `"Date"` で `Date.now()` のみ検索）
- `exclude_dirs`: 除外するディレクトリ
- `language`: 言語フィルタ

### find_imports
インポート文を検索します。

**パラメータ:**
- `symbol`: シンボル名（必須）
- `exclude_dirs`: 除外するディレクトリ
- `language`: 言語フィルタ

### find_in_comments
コメント内のテキストを検索します（TODO、FIXME等）。

**パラメータ:**
- `text`: 検索テキスト（必須）
- `exclude_dirs`: 除外するディレクトリ
- `language`: 言語フィルタ

### get_code_at_location
指定した位置のコードスニペットを取得します。

**パラメータ:**
- `file_path`: ファイルパス（必須）
- `line`: 行番号（1-indexed、必須）
- `context_before`: 対象行の前に含める行数（デフォルト: 3）
- `context_after`: 対象行の後に含める行数（デフォルト: 3）

### get_symbol_at_location
指定した行を含む最小のシンボル（関数、クラス、メソッドなど）を取得します。
`symbol_usages`で使用箇所を見つけた後、その行の完全なコンテキストを取得するのに便利です。

**パラメータ:**
- `file_path`: ファイルパス（必須）
- `line`: 行番号（1-indexed、必須）

**戻り値:**
- `name`: シンボル名
- `node_kind`: 種類（Function, Class, Method, Constructor, Interface, ArrowFunction, Variable, Enum, TypeAlias）
- `start_line`, `end_line`: シンボルの範囲
- `code`: シンボルのコード全体

## 使用例

### シンボル定義の検索

TypeScriptプロジェクトで`UserService`クラスの定義をJSDoc付きで探す場合：

```
symbol_definition(symbol="UserService", include_docs=true)
```

### シンボル使用箇所の検索

特定の関数がどこで使われているか調べる場合：

```
symbol_usages(symbol="handleRequest", include_contexts=true)
```

### 使用箇所の詳細コンテキスト取得

1. まず使用箇所を検索：
```
symbol_usages(symbol="processData")
```

2. 見つかった行の包含シンボルを取得：
```
get_symbol_at_location(file_path="/path/to/file.ts", line=42)
```

これにより、`processData`が使用されている関数やメソッド全体のコードを取得できます。

### メソッド呼び出しの検索

`Date.now()`の呼び出し箇所のみを検索：

```
find_method_calls(method_name="now", object_name="Date")
```

### TODOコメントの検索

プロジェクト内のTODOコメントを検索：

```
find_in_comments(text="TODO")
```
