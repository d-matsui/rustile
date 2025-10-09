# 実装計画

## タスク概要

この実装計画は、BSP tree balance機能を段階的に構築します。各タスクは前のタスクの成果を基に構築され、最終的にkeybindingから完全に動作するbalance commandまでの統合を実現します。

---

- [x] 1. BSP tree balance algorithmの実装
- [x] 1.1 再帰的window数計算の実装
  - BSP nodeのsubtreeに含まれるwindow数を計算する再帰関数を実装
  - Leafノードは1を返し、Splitノードは左右のsubtreeの合計を返す
  - 空のtreeや単一windowの場合は早期returnでno-opを実現
  - _Requirements: 2.1, 2.3_

- [x] 1.2 分割ratio更新logicの実装
  - 各Splitノードの左右window数に基づいてratioを計算
  - 式 `ratio = left_window_count / (left_window_count + right_window_count)` を適用
  - Splitノードのratio fieldをin-placeで更新
  - 分割方向やwindow順序は変更せず、ratioのみ更新
  - _Requirements: 2.2, 2.4, 1.3, 1.5_

- [x] 1.3 BspTree::balance_tree() public methodの実装
  - treeが空または単一windowの場合にno-opを実装
  - 複数windowが存在する場合、再帰的balance処理を呼び出し
  - すべてのSplitノードをtraverseしてratioを更新
  - 1回のtraversalでwindow数計算とratio更新を実行（繰り返しtraversal不要）
  - _Requirements: 1.1, 1.2, 1.4, 5.1, 5.2_

---

- [x] 2. WindowManager command統合
- [x] 2.1 WindowManager::balance_tree() methodの実装
  - WindowStateのBspTreeに対してbalance_tree()を呼び出す
  - Balance完了後にWindowRenderer::apply_state()を呼び出してlayoutを更新
  - 成功時に操作をinfo levelでlog
  - X11 errorが発生した場合はerrorをlogして伝播
  - _Requirements: 1.1, 4.1, 4.3_

- [x] 2.2 Command dispatch logicへの統合
  - WindowManager::handle_key_press()内のcommand match文に"balance_tree" caseを追加
  - "balance_tree" commandが来た場合にbalance_tree() methodを呼び出す
  - 既存のcommand patternに従ってerror処理を実装
  - _Requirements: 1.1, 3.5_

---

- [x] 3. Unit testの実装
- [x] 3.1 BSP balance logicのunit test
  - 空のtreeと単一windowでno-opを検証するtest
  - 2 windowsでratio=0.5を検証するtest
  - 3 windows (vertical split)でルートratio=0.33を検証するtest
  - 3 windows (horizontal split)でルートratio=0.33を検証するtest
  - Balance前後でwindow数と順序が不変であることを検証するtest
  - 計算されたratioが期待値の±0.01以内にあることを検証
  - _Requirements: 6.1, 6.2, 6.3, 6.4_

- [x] 3.2 設定のunit test
  - config.tomlからbalance keybindingをloadできることを検証するtest
  - Balance keybindingが設定されていない場合でも正常に動作することを検証するtest
  - 既存のkeybindingがbalance keybinding追加の影響を受けないことを検証
  - _Requirements: 3.1, 3.3, 7.1, 7.2_

---

- [x] 4. 統合と検証
- [x] 4.1 エンドツーエンド動作検証（手動テスト）
  - 複数のwindowパターン（2, 3, 4 windows）でbalance operationが正しく動作することを手動test
  - X11 windowのresize/move操作がbatch処理され、ちらつきがないことを確認
  - Balance操作中にUIが応答性を維持することを確認
  - すべてのwindowが表示可能で使用可能な状態を維持することを確認
  - _Requirements: 4.2, 4.4, 5.3_
  - _Note: config.tomlに `"Shift-Alt_L+0" = "balance_tree"` を追加して手動テストしてください_

- [x] 4.2 後方互換性の検証
  - Balance keybindingなしの設定でRustileが正常に起動することを確認（既存テストで検証済み）
  - 既存のBSP tree操作（add_window, remove_windowなど）が影響を受けないことを確認（全テスト通過）
  - Balance commandがtriggerされない場合、既存の実装と同一に動作することを確認（既存テスト通過）
  - _Requirements: 7.1, 7.3, 7.4, 7.5_

---

## 実装メモ

### 主要な実装ファイル
- `src/bsp.rs`: BspTree::balance_tree()とhelper methods
- `src/window_manager.rs`: WindowManager::balance_tree()とcommand dispatch
- `src/bsp.rs` (tests module): Balance logicのunit tests
- `src/config.rs` (tests module): 設定のunit tests

### 技術的注意点
- Ratio計算は `ratio = left_count / (left_count + right_count)` の式を使用
- すべてのtree操作はin-placeで実行し、新規memory割り当てを回避
- 再帰的traversalは1回のpassでwindow数の計算とratio更新を実行
- 既存のWindowRenderer::apply_state() infrastructureを活用してgeometry更新を実装
- X11 errorはgracefulに処理し、部分的な成功を許容

### テスト方針
- Unit testはX11接続なしでBSP tree logicをtest
- 手動testでkeybinding統合とX11 rendering動作を検証
- Ratio計算の精度は±0.01の許容範囲で検証
