# ワークスペース管理機能実装の振り返り

## はじめに

このドキュメントは、Rustileウィンドウマネージャーにワークスペース管理機能を追加する過程で、当初の設計から実装がどのように乖離したか、なぜそうなったのか、そして今後どう改善できるかを振り返るものです。

## 当初の設計とコンセプト

### 設計の核心アイデア（design.md より）

**決定1: WindowManagerが直接Vec<Workspace>を管理**

設計書では以下のように明確に記述されていました：

> **選択したアプローチ**: WindowManagerがVec<Workspace>とcurrent_workspace_indexを直接保持します。Workspace構造体はWindowStateからワークスペース固有の状態（BspTree、フォーカス、fullscreen/zoom）を抽出したものです。

重要なポイント：
- **WindowStateの役割**: 「WindowStateの責任をWorkspace構造体に移行し、WindowManagerが直接ワークスペースを管理します」（9行目）
- **WindowStateの処遇**: 「WindowState - Workspace構造体に機能を移行し、削除または大幅簡素化」（78行目）

つまり、設計の意図は：
1. WindowStateの機能をWorkspaceに移す
2. WindowStateは削除するか、大幅に簡素化する
3. WindowManagerが直接Workspaceを管理する

### Workspace構造体の設計（design.md 256-300行）

```rust
pub struct Workspace {
    bsp_tree: BspTree,
    focused_window: Option<Window>,
    fullscreen_window: Option<Window>,
    zoomed_window: Option<Window>,
}
```

Workspaceが持つべき機能：
- BspTreeへのアクセス（bsp_tree()、bsp_tree_mut()）
- フォーカス管理（focused_window()、set_focused_window()）
- fullscreen/zoom管理
- ウィンドウ追加・削除（add_window()、remove_window()）
- すべてのウィンドウ取得（all_windows()）

## 実装での乖離

### 何が違ったか

#### 1. WindowStateが残存

**設計**: WindowStateを削除または大幅簡素化
**実装**: WindowStateが完全に残り、一時的なレンダリング用バッファとして機能

現在の実装：
```rust
pub struct WindowManager<C: Connection> {
    // ...
    pub(crate) workspaces: Vec<Workspace>,
    pub(crate) current_workspace_index: usize,
    // Temporary rendering state - synced with current workspace before rendering
    pub(crate) window_state: WindowState,  // ← 設計にない！
    pub(crate) window_renderer: WindowRenderer,
}
```

#### 2. 複雑な同期メカニズムの出現

**設計**: WindowRendererはWorkspaceを直接読み取る
**実装**: Workspace ↔ WindowState間の双方向同期が必要に

実装された同期関数：
- `sync_window_state_with_current_workspace()` - Workspace → WindowState
- `sync_workspace_from_window_state()` - WindowState → Workspace (focus, fullscreen, zoom)
- `sync_bsp_tree_to_workspace()` - WindowState.bsp_tree → Workspace.bsp_tree

#### 3. Workspaceの機能が不完全

**設計**: Workspaceがadd_window()、remove_window()などを持つ
**実装**: これらのメソッドはあるが、実際にはWindowState経由でしか使えない

設計で定義されているが実装されていない/使われていないメソッド：
- Workspace.add_window() - あるが、WindowState.add_window_to_layout()が削除されたため正しく動作しない
- BspTreeへの直接操作が必要になった

### なぜこうなったか - 根本原因の分析

#### 1. WindowRendererの依存関係

**問題の核心**: WindowRendererがWindowStateに強く依存していた

WindowRendererの主要メソッド：
```rust
pub fn apply_state<C: Connection>(
    &mut self,
    conn: &mut C,
    state: &mut WindowState,  // ← WindowStateに依存
) -> Result<()>
```

WindowRendererはWindowStateの以下のメソッドを使用：
- `window_count()`
- `get_all_windows()`
- `calculate_window_geometries()`
- `border_color_for_window()`
- `layout_params()`
- その他多数

**設計での想定**: WindowRendererをWorkspaceに対応させる
**実装での障壁**: WindowRendererの全面的な書き換えは大規模すぎる

#### 2. 段階的実装の罠

実装は以下の順序で進みました：

1. **タスク1-2**: Workspace構造体の作成、WindowManagerへの統合
   - この時点でWorkspaceとWindowStateが併存

2. **タスク3-5**: ワークスペース操作の実装
   - WindowRendererはWindowStateを使い続ける
   - 一時的な同期メカニズムとして`sync_window_state_with_current_workspace()`を導入

3. **バグ修正フェーズ**:
   - ウィンドウが重なる問題 → さらに複雑な同期ロジック
   - fullscreenが壊れる問題 → fullscreen処理をWindowManagerに移動
   - rotateが2回目失敗 → BSPツリーの直接コピー

**問題**: 「一時的な」同期メカニズムが「永続的な」設計になってしまった

#### 3. 設計の不完全性

design.mdにも実は曖昧な部分がありました：

**WindowRendererの変更について**（412-460行）：
```rust
/// ワークスペースのウィンドウをマップ
pub fn map_workspace_windows<C: Connection>(
    &self,
    conn: &C,
    windows: &[Window],
) -> Result<()>;
```

しかし、apply_state()の扱いについては明記されていませんでした：
- apply_state()はWindowStateを受け取り続けるのか？
- それともWorkspaceを受け取るように変更するのか？
- WindowStateからWorkspaceへの移行をどう進めるのか？

## 現在の実装の問題点

### 1. アーキテクチャの複雑性

**二重管理**: WorkspaceとWindowStateの両方が状態を保持
- Workspaceが真実のソース（source of truth）
- WindowStateは一時的なレンダリングバッファ
- しかし、この区別は明示的でなく、バグの温床

### 2. パフォーマンスへの影響

各操作でBSPツリーの完全コピーが発生：
```rust
let bsp_tree_clone = self.current_workspace().bsp_tree().clone();
*self.window_state.bsp_tree_mut() = bsp_tree_clone;
```

これはO(N)の操作（N = ウィンドウ数）で、設計の意図に反します。

### 3. 保守性の低下

バグ修正の度に同期ロジックが増加：
- focus操作 → sync_workspace_from_window_state()
- swap操作 → sync_bsp_tree_to_workspace()
- fullscreen → 完全に別処理

**新しい機能を追加する難しさ**: どこに何を実装すべきか不明確

### 4. テスト戦略との乖離

design.mdのテスト戦略（625-641行）：
- Workspace構造体のユニットテスト
- WorkspaceManagerのユニットテスト

**実装**:
- Workspaceのテストは削除された（dead_code除去の方針で）
- 同期ロジックのテストが存在しない

## 学んだこと

### 1. 「一時的な」解決策の危険性

**教訓**: 一時的な回避策（sync関数）が永続的な設計になる

**兆候**:
- 「とりあえず」動かすために同期関数を追加
- 「後で直す」という計画のまま進む
- バグ修正の度に同期ロジックが増える

**対策**:
- 一時的な解決策には必ず期限と撤廃計画を設ける
- コードコメントに`TODO: Remove this sync once WindowRenderer is refactored`を明記
- 技術的負債としてissueを立てる

### 2. 段階的移行の重要性と難しさ

**問題**: WindowState → Workspace移行を完遂できなかった

**原因**:
- 移行計画が不明確（design.mdにも明記されていない）
- WindowRendererの依存関係が大きすぎた
- タスク分解が不十分

**対策**:
- Strangler Fig パターンの採用:
  1. 新しいインターフェース（Workspace）を作る
  2. 古いインターフェース（WindowState）をラップして内部で新しいものを呼ぶ
  3. 呼び出し側を段階的に移行
  4. 古いインターフェースを削除

- 具体的な移行ステップをタスクに：
  - タスクA: WorkspaceRendererを作成（WindowRendererのWorkspace版）
  - タスクB: WindowManagerの1つのメソッドをWorkspaceRendererに切り替え
  - タスクC: 全メソッド移行後、WindowRendererを削除

### 3. 設計書の粒度

**問題**: design.mdは高レベルな設計は良いが、移行戦略の詳細がない

**不足していた情報**:
- WindowStateからWorkspaceへの移行の具体的なステップ
- WindowRendererをどう変更するか（シグネチャの変更、段階的移行など）
- 過渡期のアーキテクチャ（両方が存在する期間をどう管理するか）

**対策**:
- 設計書に「移行戦略」セクションを追加
- 各ステップでのアーキテクチャ図を用意
- 各ステップの完了条件を明確化

### 4. 「dead_code許さない」方針の功罪

**功**: コードベースがクリーンに保たれる
**罪**: 使っていないが設計上必要なメソッドも削除してしまった

例：Workspace.add_window()は設計上必要だが、現在の実装では使われていないため削除圧力がかかる

**対策**:
- インターフェースとして重要なメソッドは`#[cfg(test)]`でテストを残す
- または、"この機能は将来の移行で使用予定"とコメント
- Architecture Decision Record (ADR)で「なぜこのメソッドが必要か」を文書化

## 今後の改善策

### 短期的改善（1-2週間）

#### 1. 現状の明示化

```rust
/// Temporary rendering buffer.
///
/// TECHNICAL DEBT: This should be removed once WindowRenderer is refactored
/// to work directly with Workspace. See issue #XXX.
///
/// Current architecture:
/// - Workspace is the source of truth
/// - WindowState is synced from Workspace before rendering
/// - Changes from WindowRenderer are synced back to Workspace
pub(crate) window_state: WindowState,
```

#### 2. 同期ロジックのドキュメント化

同期関数に詳細なコメントを追加：
```rust
/// Syncs WindowState with the current workspace
///
/// This is a temporary workaround until WindowRenderer is refactored.
///
/// What gets synced:
/// - BSP tree (full clone)
/// - focused_window
/// - fullscreen_window
/// - zoomed_window
///
/// Performance: O(N) where N = number of windows (due to BSP clone)
fn sync_window_state_with_current_workspace(&mut self)
```

#### 3. テストの追加

同期ロジックの正しさをテスト：
```rust
#[test]
fn test_sync_preserves_bsp_structure() {
    // rotateやswapの後でもBSP構造が保たれることを検証
}

#[test]
fn test_sync_preserves_focus() {
    // focus変更が正しく同期されることを検証
}
```

### 中期的改善（1-2ヶ月）

#### 1. WorkspaceRendererの段階的導入

**ステップ1**: WorkspaceRenderer traitを定義
```rust
pub trait WorkspaceRenderer {
    fn apply_workspace<C: Connection>(
        &mut self,
        conn: &mut C,
        workspace: &Workspace,
        config: &Config,
        screen_num: usize,
    ) -> Result<()>;
}
```

**ステップ2**: 既存WindowRendererにtraitを実装（内部でWindowStateを使用）
```rust
impl WorkspaceRenderer for WindowRenderer {
    fn apply_workspace<C: Connection>(...) -> Result<()> {
        // 内部でWindowStateを作成して既存のapply_state()を呼ぶ
        let mut state = WindowState::from_workspace(workspace, config, screen_num);
        self.apply_state(conn, &mut state)
    }
}
```

**ステップ3**: WindowManagerの呼び出しを段階的に移行

**ステップ4**: WindowRenderer内部をWorkspace直接操作に書き換え

**ステップ5**: WindowStateを完全削除

#### 2. Workspace APIの充実

設計書通りのメソッドを実装：
```rust
impl Workspace {
    /// Adds a window to this workspace
    ///
    /// This will be used once WindowRenderer is refactored.
    pub fn add_window(&mut self, window: Window, config: &Config) {
        let split_ratio = config.bsp_split_ratio();
        self.bsp_tree.add_window(window, self.focused_window, split_ratio);
    }

    // ... その他の設計書のメソッド
}
```

### 長期的改善（3-6ヶ月）

#### 1. アーキテクチャの再設計

完全に設計書通りの実装に：
- WindowStateを完全削除
- WindowManagerがWorkspaceを直接管理
- WindowRendererがWorkspaceを直接読み取り

#### 2. ドキュメントの改善

- 実装とdesign.mdの差分を明記
- 移行計画をRoadmap化
- ADRで設計判断を記録

#### 3. 自動化テスト

- 統合テストで実際のX11操作をテスト
- パフォーマンステスト（BSPツリーのクローンコストなど）

## プロセスの改善提案

### 設計フェーズ

1. **詳細な移行戦略の策定**
   - 既存コードから新設計への移行ステップを明示
   - 各ステップでのアーキテクチャ図
   - 完了条件の明確化

2. **依存関係の可視化**
   - どのコンポーネントがどのコンポーネントに依存しているか
   - 変更の影響範囲の特定

3. **リスクの特定**
   - 大規模な変更が必要な箇所（WindowRendererなど）
   - 段階的移行が難しい箇所

### 実装フェーズ

1. **一時的な解決策の管理**
   - 必ず期限を設定
   - 技術的負債としてissue化
   - コード内にTODOコメントとissue番号を記載

2. **段階的なリファクタリング**
   - 大きな変更を小さなPRに分割
   - 各PRで動作する状態を維持
   - リファクタリング専用のブランチ戦略

3. **コードレビューの重点**
   - 「一時的」とされるコードの精査
   - 設計書との差分の確認
   - 技術的負債の蓄積防止

### レビューフェーズ

1. **定期的な設計との突合**
   - 実装が設計からどれだけ乖離しているか定期確認
   - 乖離が大きくなる前に軌道修正

2. **振り返りの習慣化**
   - 各機能実装後に必ず振り返り
   - 学びを次の実装に活かす

## まとめ

### 成功したこと

✅ ワークスペース管理機能は動作している
✅ ユーザーの要求機能は実装されている
✅ テストは全て通っている

### 改善が必要なこと

❌ アーキテクチャが設計と乖離している
❌ 複雑な同期ロジックが保守性を下げている
❌ 性能上のオーバーヘッドがある（BSPツリーのクローン）

### 最大の学び

**「動く」と「正しい設計」は別物**

今回の実装は動作していますが、設計書が描いていた美しいアーキテクチャにはなっていません。これは技術的負債として認識し、計画的に返済していく必要があります。

**一時的な解決策は永続化する傾向がある**

"とりあえず動かす"ための同期ロジックが、システムの一部として定着してしまいました。一時的な解決策には必ず撤廃計画が必要です。

**設計書には移行戦略も含めるべき**

高レベルな設計だけでなく、既存システムから新システムへの移行の具体的なステップも設計の一部として記述すべきでした。

---

このドキュメントが、同じような状況に直面した開発者の助けになることを願っています。

**執筆者より**: このドキュメントは実装の「失敗」を記録したものではなく、「学び」を記録したものです。完璧な設計から始めることは難しく、重要なのは問題を認識し、改善し続けることです。
