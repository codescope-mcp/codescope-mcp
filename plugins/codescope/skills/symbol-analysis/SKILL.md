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
- `project_path`: プロジェクトルートパス（必須）
- `symbol_name`: 検索するシンボル名（必須）

### symbol_usages
シンボルの使用箇所を検索します。

**パラメータ:**
- `project_path`: プロジェクトルートパス（必須）
- `symbol_name`: 検索するシンボル名（必須）

### find_method_calls
メソッドや関数の呼び出し箇所を検索します。

**パラメータ:**
- `project_path`: プロジェクトルートパス（必須）
- `method_name`: メソッド/関数名（必須）
- `receiver_type`: レシーバー型（オプション）

### find_imports
インポート文を検索します。

**パラメータ:**
- `project_path`: プロジェクトルートパス（必須）
- `module_name`: モジュール名（オプション）

### find_in_comments
コメント内のテキストを検索します。

**パラメータ:**
- `project_path`: プロジェクトルートパス（必須）
- `search_text`: 検索テキスト（必須）

### get_code_at_location
指定した位置のコードを取得します。

**パラメータ:**
- `project_path`: プロジェクトルートパス（必須）
- `file_path`: ファイルパス（必須）
- `start_line`: 開始行（必須）
- `end_line`: 終了行（必須）

## 使用例

TypeScriptプロジェクトで`UserService`クラスの定義を探す場合：

```
symbol_definition(project_path="/path/to/project", symbol_name="UserService")
```

特定の関数がどこで使われているか調べる場合：

```
symbol_usages(project_path="/path/to/project", symbol_name="handleRequest")
```
