# 実装タスク

## 概要
このドキュメントは、シャットダウンコマンド機能の実装タスクを定義する。各タスクは要件と設計書に基づいており、順次実行することで機能を完成させる。

---

## タスク一覧

- [x] 1. シャットダウンコマンドの実装
- [x] 1.1 コマンドディスパッチにshutdownケースを追加
  - キー押下イベント処理でshutdownコマンドを認識する
  - ログメッセージ"Shutting down Rustile by user request"を出力する
  - プロセスを正常終了コード0で即座に終了する
  - 既存のコマンドマッチング処理に統合する
  - _Requirements: 1.1, 1.2, 1.3, 3.1, 3.2, 4.1, 4.2, 4.3_

- [x] 2. 設定ファイルへのキーバインディング例示追加
- [x] 2.1 config.example.tomlにデフォルトキーバインドを追加
  - "Shift+Ctrl+Alt+Backspace" = "shutdown"のエントリを追加する
  - コメントで機能説明を記載する（例: "# Shutdown Rustile WM"）
  - 既存のRustile管理コマンドセクションに配置する
  - _Requirements: 2.3_

- [ ] 3. 動作確認とテスト
- [ ] 3.1 手動テストによる動作確認
  - Rustileを起動してShift+Ctrl+Alt+Backspaceを押下し、プロセスが終了することを確認する
  - ログファイル（~/.rustile.log）に終了メッセージが記録されることを確認する
  - _Requirements: 1.1, 1.2, 1.3_

- [x] 4. コード品質チェック
- [x] 4.1 ビルドとリントチェック
  - cargo fmtでコードフォーマットを実行する
  - cargo buildで警告なしでビルドが成功することを確認する
  - cargo clippyでリント警告がないことを確認する
  - cargo testで既存テストが全て通過することを確認する
  - _Requirements: All（コード品質保証）_

---

## 要件カバレッジ確認

| 要件ID | 要件概要 | 対応タスク |
|--------|----------|------------|
| 1.1 | ショートカットでプロセス終了 | 1.1, 3.1 |
| 1.2 | 即座に終了 | 1.1, 3.1 |
| 1.3 | ログ記録 | 1.1, 3.1 |
| 2.1 | config.toml認識 | （既存機能で保証） |
| 2.2 | 異なるキーコンビネーション受容 | （既存機能で保証） |
| 2.3 | config.exampleに例示 | 2.1 |
| 3.1 | ログに終了理由記録 | 1.1, 3.1 |
| 3.2 | 意図的操作として識別可能 | 1.1, 3.1 |
| 4.1 | コマンドマッチング処理 | 1.1 |
| 4.2 | 適切な方法で終了 | 1.1 |
| 4.3 | 他のコマンドと同様に処理 | 1.1 |

全ての要件がタスクでカバーされています。

---

## 実装ノート

### タスク1.1の実装ガイド
- 変更対象ファイル: `src/window_manager.rs`
- 変更箇所: `handle_key_press()`メソッド内のmatchステートメント
- 追加位置: 既存のコマンドケース（"create_workspace", "delete_workspace"など）の後
- ログレベル: `info!`マクロを使用

### タスク2.1の実装ガイド
- 変更対象ファイル: `config.example.toml`
- 追加セクション: `[shortcuts]`セクション内
- 配置位置: Workspace management コメントの近くが適切

### タスク3.1の確認方法
1. Rustileをビルド: `cargo build`
2. X11環境でRustileを起動
3. Shift+Ctrl+Alt+Backspaceを押下
4. プロセスが終了し、ログに"Shutting down Rustile by user request"が記録されることを確認

### タスク4.1のチェックコマンド
```bash
source ~/.cargo/env
cargo fmt
cargo build --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```
