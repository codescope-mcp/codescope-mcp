# CLAUDE.md

このファイルはClaude Codeがこのリポジトリで作業する際のガイダンスを提供します。

## ビルドとテスト

```bash
# ビルド
cargo build

# 全テスト実行
cargo test

# 統合テストのみ
cargo test --test integration

# 特定のテスト
cargo test test_html
cargo test test_css
```

## コミット前の必須チェック

PRを作成する前に、以下のコマンドを実行してフォーマットとリンティングを確認してください。

```bash
# フォーマットチェック（差分がある場合は失敗）
cargo fmt --check

# フォーマット適用
cargo fmt

# リンティング（警告をエラーとして扱う）
cargo clippy -- -D warnings
```

## プロジェクト構造

- `src/language/` - 言語サポート実装（TypeScript, JavaScript, HTML, CSS, Markdown）
- `src/symbol/` - シンボル検出・解析
- `src/parser/` - パーサー実装
- `queries/` - tree-sitterクエリファイル
- `tests/integration/` - 統合テスト
- `tests/fixtures/` - テスト用サンプルファイル
