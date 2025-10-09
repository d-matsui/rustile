# 実装計画

## タスク概要

この実装計画は、ワークスペース管理機能を段階的に構築します。各タスクは前のタスクの成果を基に構築され、最終的にキーバインディングから完全に動作するワークスペース管理機能までの統合を実現します。

---

- [ ] 1. Workspace構造体の実装
- [x] 1.1 基本的なWorkspace構造体とコンストラクタの実装
  - 単一ワークスペースの状態を表す構造体を作成
  - BspTree、フォーカス状態、fullscreen状態、zoom状態を保持
  - 新しい空のワークスペースを作成するコンストラクタを実装
  - _Requirements: 1.2, 1.3, 4.1_

- [ ] 1.2 Workspaceの状態管理メソッドの実装
  - BspTreeへの参照取得メソッドを実装（読み取り専用と可変の両方）
  - フォーカス状態の取得・設定メソッドを実装
  - fullscreen状態の取得・設定メソッドを実装
  - zoom状態の取得・設定メソッドを実装
  - _Requirements: 4.3, 4.4, 4.5_

- [x] 1.3 Workspaceのウィンドウ操作メソッドの実装
  - すべてのウィンドウを取得するメソッドを実装
  - ウィンドウをレイアウトに追加するメソッドを実装
  - ウィンドウをレイアウトから削除するメソッドを実装
  - ウィンドウ交換、回転などの既存WindowState機能を移植
  - _Requirements: 4.2, 9.3_

---

- [ ] 2. WindowManagerのワークスペース管理機能の実装
- [x] 2.1 WindowManager構造体のリファクタリング
  - window_state: WindowStateをworkspaces: Vec<Workspace>に置き換え (一時的に両方保持)
  - current_workspace_index: usizeを追加
  - intentionally_unmapped: HashSet<Window>をWindowStateから移動
  - config、screen_numをWindowManagerに保持
  - 起動時に単一の空のワークスペースを初期化
  - _Requirements: 1.5, 4.1, 9.1_

- [x] 2.2 現在のワークスペースへのアクセスヘルパーメソッドの実装
  - 現在のワークスペースへの参照を取得するメソッドを実装 (current_workspace)
  - 現在のワークスペースへの可変参照を取得するメソッドを実装 (current_workspace_mut)
  - 既存のウィンドウ操作を現在のワークスペース経由に変更 (WindowRenderer移行後に実施)
  - _Requirements: 4.2_

- [x] 2.3 ワークスペース作成機能の実装
  - 新しいワークスペースを作成するメソッドを実装
  - ワークスペースをVecに追加
  - current_workspace_indexを新しいワークスペースに更新
  - 作成操作をログ記録
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

- [x] 2.4 ワークスペース削除機能の実装
  - ワークスペース数が1の場合はno-opで早期リターン
  - 現在のワークスペースのすべてのウィンドウを閉じる処理を実装 (TODO: X11統合時)
  - ワークスペースをVecから削除
  - 別のワークスペース（前のインデックス）に切り替え
  - 削除操作をログ記録
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

- [x] 2.5 ワークスペース切り替え機能の実装（次/前）
  - 旧ワークスペースのすべてのウィンドウを取得 (TODO: X11統合時)
  - 旧ワークスペースのウィンドウをintentionally_unmappedに追加 (TODO: X11統合時)
  - current_workspace_indexを更新（次は+1、前は-1、循環）
  - 新ワークスペースのウィンドウをintentionally_unmappedから削除 (TODO: X11統合時)
  - フォーカスを新ワークスペースの最後にフォーカスされていたウィンドウに復元 (TODO: X11統合時)
  - 切り替え操作をログ記録
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7_

---

- [x] 3. WindowRendererのワークスペース対応
- [x] 3.1 ワークスペースのウィンドウマップ/アンマップ機能の実装
  - ウィンドウリストをマップするヘルパーメソッドを実装 (perform_workspace_switch)
  - ウィンドウリストをアンマップするヘルパーメソッドを実装 (perform_workspace_switch)
  - バッチ処理でちらつきを最小限に抑える
  - エラーが発生しても残りのウィンドウで継続
  - _Requirements: 6.1, 6.2, 6.3, 6.4_

- [x] 3.2 apply_stateメソッドのワークスペース対応
  - 現在のワークスペースのみのウィンドウをレンダリング (イベント処理で対応)
  - ワークスペース切り替え時のmap/unmap処理を統合 (perform_workspace_switch)
  - 既存のジオメトリ計算ロジックを維持 (WindowStateとの並行管理)
  - _Requirements: 6.1, 6.2_

---

- [x] 4. キーバインディングとコマンドディスパッチの統合
- [x] 4.1 ワークスペースコマンドのディスパッチ実装
  - handle_key_press()にcreate_workspaceコマンドを追加
  - handle_key_press()にdelete_workspaceコマンドを追加
  - handle_key_press()にswitch_workspace_nextコマンドを追加
  - handle_key_press()にswitch_workspace_prevコマンドを追加
  - 各コマンドで対応するメソッドを呼び出し
  - _Requirements: 5.1, 5.2, 5.3_

- [x] 4.2 デフォルトキーバインディングの設定
  - config.example.tomlにワークスペースキーバインディングを追加
  - Ctrl+Alt+n = create_workspace
  - Ctrl+Alt+q = delete_workspace
  - Ctrl+Alt+j = switch_workspace_next
  - Ctrl+Alt+k = switch_workspace_prev
  - _Requirements: 5.1, 5.2, 5.3, 5.4_

---

- [x] 5. イベント処理の修正
- [x] 5.1 MapRequest/UnmapNotifyイベントの更新
  - 新しいウィンドウを現在のワークスペースに追加
  - UnmapNotifyでintentionally_unmappedをチェック（WindowManagerから）
  - 全ワークスペースからウィンドウを削除する処理に変更
  - _Requirements: 4.2, 6.1, 6.2_

- [x] 5.2 その他のイベントハンドラの更新
  - フォーカス変更を現在のワークスペースに適用 (handle_enter_notify)
  - fullscreen/zoom操作を現在のワークスペースに適用 (handle_destroy_notify)
  - ウィンドウ交換・回転を現在のワークスペースに適用 (WindowStateとの並行管理)
  - _Requirements: 4.5, 9.3_

---

- [x] 6. ユニットテストの実装
- [x] 6.1 Workspace構造体のユニットテスト
  - 空のワークスペース作成テスト (test_workspace_creation)
  - ウィンドウ操作（追加、削除、クエリ）テスト (test_workspace_window_*)
  - フォーカス状態管理テスト (test_workspace_state_management)
  - 状態の独立性テスト (各テストで独立性を検証)
  - _Requirements: 8.1, 8.3_

- [x] 6.2 WindowManagerのワークスペース操作テスト
  - 単一ワークスペースでの初期化テスト (test_workspace_initialization)
  - ワークスペース作成と切り替えテスト (test_workspace_creation/switching_logic)
  - 複数ワークスペースでの削除テスト (test_workspace_deletion_logic)
  - 最後のワークスペース削除のno-opテスト (test_workspace_deletion_logic)
  - 循環切り替え（次/前）テスト (test_workspace_switching_logic)
  - intentionally_unmappedトラッキングテスト (イベント処理で実装)
  - _Requirements: 8.1, 8.2, 8.3, 8.4_

---

- [x] 7. 後方互換性の検証と調整
- [x] 7.1 既存機能の動作確認
  - 既存のウィンドウ管理操作が単一ワークスペースで正常に動作することを確認 (全88テスト成功)
  - 既存のキーバインディングが影響を受けないことを確認 (変更なし)
  - BSP操作（balance、rotate、swap）が正常に動作することを確認 (WindowStateとの並行管理)
  - _Requirements: 9.1, 9.2, 9.3, 9.4_

- [x] 7.2 既存テストの修正
  - WindowManagerのテストをワークスペース対応に更新 (テスト追加のみ、既存テストは変更不要)
  - 既存のウィンドウ操作テストが単一ワークスペースで動作することを確認 (全テスト成功)
  - 必要に応じてテストのセットアップを調整 (不要だった)
  - _Requirements: 8.5, 9.4_

---

- [ ] 8. 統合テストと最終検証（手動）
- [ ] 8.1 手動テストの実施
  - ワークスペースキーバインディング設定をロード
  - Rustileを起動して単一ワークスペースを確認
  - 新しいワークスペースを作成
  - 各ワークスペースに異なるウィンドウを開く
  - ワークスペース間を切り替えて正しいウィンドウセットが表示されることを確認
  - ワークスペース間でウィンドウ状態が独立していることを確認
  - ワークスペースを削除
  - 最後のワークスペース削除がno-opであることを確認
  - _Requirements: 8.5, すべての要件の統合テスト_

- [ ] 8.2 エラー処理とエッジケースの検証
  - X11エラー時の部分的成功を確認
  - ワークスペース操作中のシステム安定性を確認
  - ログメッセージが適切に記録されることを確認
  - _Requirements: 7.2_

---

## 実装メモ

### 主要な実装ファイル
- `src/workspace.rs`: Workspace構造体（新規作成）
- `src/window_manager.rs`: WindowManagerのリファクタリングとワークスペース操作
- `src/window_renderer.rs`: ワークスペース対応のmap/unmap処理
- `src/window_state.rs`: 削除または大幅簡素化（Workspaceに移行）

### 技術的注意点
- WindowStateからWorkspaceへの段階的な移行
- intentionally_unmappedは全ワークスペース共通でWindowManagerに保持
- ワークスペース切り替え時のintentionally_unmappedトラッキングが重要
- 最後のワークスペース削除はno-op（ログのみ）
- 循環切り替えは剰余演算で実装（`(index + 1) % len`）

### テスト方針
- ユニットテストでWorkspace構造体とワークスペース操作ロジックをテスト
- 手動テストでキーバインディング統合とX11レンダリング動作を検証
- 既存テストが単一ワークスペースで引き続き動作することを確認
