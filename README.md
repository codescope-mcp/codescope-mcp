# CodeScope MCP

TypeScript/TSXプロジェクトのシンボル解析とコードナビゲーションを提供するMCPサーバーです。

## 機能

- **symbol_definition**: シンボル定義の検索（JSDocコメント付き）
- **symbol_usages**: シンボルの使用箇所を検索
- **find_method_calls**: メソッド/関数呼び出しの検索
- **find_imports**: インポート文の検索
- **find_in_comments**: コメント内テキスト検索
- **get_code_at_location**: 指定位置のコード取得

## 前提条件

- Rustツールチェーン（cargo, rustc）

## インストール

### MCPサーバーとして使用

```bash
# ビルド
cargo build --release

# 実行
cargo run --release
```

### Claude Code プラグインとして使用

```bash
# プラグインとして追加
claude plugins add /path/to/codescope-mcp
```

追加後、以下のツールが利用可能になります：

- `symbol_definition`
- `symbol_usages`
- `find_method_calls`
- `find_imports`
- `find_in_comments`
- `get_code_at_location`

スキル `/codescope:symbol-analysis` で使用ガイダンスを確認できます。

## 設定

環境変数:

- `RUST_LOG`: ログレベル設定（例: `info`, `debug`）

## 開発

```bash
# テスト実行
cargo test

# デバッグビルド
cargo build
```

## ライセンス

MIT
