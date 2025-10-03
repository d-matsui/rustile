# Claude Code 仕様駆動開発

Kiroスタイルの仕様駆動開発をClaude Codeのスラッシュコマンド、フック、エージェントで実装

## プロジェクトコンテキスト

### パス
- Steering: `.kiro/steering/`
- Specs: `.kiro/specs/`
- Commands: `.claude/commands/`

### Steering vs Specification

**Steering** (`.kiro/steering/`) - プロジェクト全体のルールとコンテキストでAIをガイド
**Specs** (`.kiro/specs/`) - 個別機能の開発プロセスを形式化

### アクティブな仕様
- **bsp-balance**: ウィンドウ数に基づいてBSPツリーの比率を手動でバランスするコマンド
- 進捗確認: `/kiro:spec-status [feature-name]`

## 開発ガイドライン
- 日本語で思考し、日本語で応答を生成する

## ワークフロー

### フェーズ0: Steering（オプション）
`/kiro:steering` - steeringドキュメントの作成/更新
`/kiro:steering-custom` - 専門的なコンテキスト用のカスタムsteeringを作成

注意: 新機能や小規模な追加では省略可能。spec-initから直接開始できます。

### フェーズ1: 仕様作成
1. `/kiro:spec-init [詳細な説明]` - 詳細なプロジェクト説明で仕様を初期化
2. `/kiro:spec-requirements [feature]` - 要件定義書を生成
3. `/kiro:spec-design [feature]` - インタラクティブ: "requirements.mdを確認しましたか？ [y/N]"
4. `/kiro:spec-tasks [feature]` - インタラクティブ: 要件と設計の両方の確認を求める

### フェーズ2: 進捗トラッキング
`/kiro:spec-status [feature]` - 現在の進捗とフェーズを確認

## 開発ルール
1. **Steeringを考慮**: 大規模な開発の前に `/kiro:steering` を実行（新機能では省略可）
2. **3フェーズ承認ワークフローに従う**: 要件 → 設計 → タスク → 実装
3. **承認が必要**: 各フェーズは人間のレビューが必要（インタラクティブプロンプトまたは手動）
4. **フェーズをスキップしない**: 設計は承認済み要件が必要、タスクは承認済み設計が必要
5. **タスクステータスを更新**: 作業中のタスクを完了としてマーク
6. **Steeringを最新に保つ**: 重要な変更後に `/kiro:steering` を実行
7. **仕様準拠を確認**: `/kiro:spec-status` で整合性を検証

## コード品質チェック

### コミット前に必須のコマンド
```bash
source ~/.cargo/env  # Ensure cargo is in PATH
cargo fmt           # Format code
cargo build --all-targets --all-features  # Build all targets to catch warnings
cargo clippy --all-targets --all-features -- -D warnings  # Check for lints (treat warnings as errors)
cargo test          # Run all tests
```

## Steering設定

### 現在のSteeringファイル
`/kiro:steering` コマンドで管理。ここの更新はコマンドの変更を反映。

### アクティブなSteeringファイル
- `product.md`: 常に含まれる - プロダクトコンテキストとビジネス目標
- `tech.md`: 常に含まれる - 技術スタックとアーキテクチャ決定
- `structure.md`: 常に含まれる - ファイル構成とコードパターン

### カスタムSteeringファイル
<!-- /kiro:steering-custom コマンドで追加 -->
<!-- フォーマット:
- `filename.md`: モード - パターン - 説明
  モード: Always|Conditional|Manual
  パターン: Conditionalモード用のファイルパターン
-->

### 包含モード
- **Always**: 全てのインタラクションで読み込み（デフォルト）
- **Conditional**: 特定のファイルパターンで読み込み（例: "*.test.js"）
- **Manual**: `@filename.md` 構文で参照
