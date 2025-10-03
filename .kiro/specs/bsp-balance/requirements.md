# 要件定義書

## はじめに

BSP Balance機能は、BSP treeの分割比率を最適化するための手動commandを提供し、最適な空間利用を実現します。現在、Rustileは新しいwindowを追加する際に固定の分割比率（0.5）を使用しており、時間の経過とともに最適でない空間配分につながる可能性があります。この機能により、userは設定可能なkeybindingを介してオンデマンドで再balanceをtriggerでき、window数に基づいて分割比率を調整することで、画面領域の利用率を向上させます。

この機能のuserへのメリット：
- 必要に応じたオンデマンドのlayout最適化
- 各subtreeのwindow数に基づいた比例的な分割比率計算
- 素早い再balance用の簡単なkeybinding（例：Shift-Alt_L+0）
- balance実行timingのuser制御

## 要件

### 要件1: 手動balance command
**目的:** window managerのuserとして、keybindingでBSP treeのbalanceをtriggerし、自動的な中断なしに必要なときにlayoutを最適化したい。

#### 受入基準

1. userが設定されたbalance keybindingを押したとき、RustileはBSP tree全体を再balanceすること
2. balance commandが実行されたとき、Rustileは各subtreeのwindow数に基づいて最適な分割比率を計算すること
3. 分割nodeが存在する場合、Rustileは左右のsubtreeのwindow数に基づいて比例的に空間を割り当てるように比率を更新すること
4. BSP treeが空であるか、1つのwindowのみを含む場合、Rustileはbalance commandを無視すること（no-op）
5. balance操作を実行する間、Rustileは既存のwindowの順序と分割方向を維持すること

### 要件2: 比率計算algorithm
**目的:** 開発者として、保守可能でtest可能で予測可能な、明確に定義されたbalance algorithmを持ちたい。

#### 受入基準

1. nodeが分割である場合、Rustileは左右のsubtreeのwindow数を再帰的に計算すること
2. 分割の最適比率を計算する際、Rustileは式を使用すること：ratio = left_window_count / (left_window_count + right_window_count)
3. treeにbalanceを適用する際、Rustileはすべての分割nodeをtraverseして比率を更新すること
4. 複数の分割が存在する場合、Rustileはsubtreeのwindow数に基づいて各分割を独立してbalanceすること

### 要件3: Keybinding設定
**目的:** window managerのuserとして、workflowに合わせて他のshortcutと競合しないように、balance keybindingを設定したい。

#### 受入基準

1. Rustileが設定をloadする際、systemはconfig.tomlからbalance commandのkeybindingを読み取ること
2. Balance keybindingが定義されている場合、設定は他のkeybindingと同じ形式に従うこと（例："Shift-Alt_L+0"）
3. Balance keybindingが設定されていない場合、Rustileはbalance commandをどのkeyにもbindしないこと
4. Keybindingの競合が検出された場合、Rustileは警告をlogに記録し、競合するkeyのbindを拒否すること
5. Userがbalance keybindingを設定している場合、Rustileはkeyの組み合わせが押されたときにbalance commandを実行すること

### 要件4: Window geometryの更新
**目的:** window managerのuserとして、balance後にwindowがsmoothにresizeされ、視覚的な遷移がcleanで応答性が高くなるようにしたい。

#### 受入基準

1. Balance commandが比率計算を完了したとき、Rustileはwindow layoutの再計算をtriggerすること
2. Window geometryの変更が必要な場合、Rustileは視覚的なちらつきを最小限に抑えるためにX11 windowのresize/move操作をbatch処理すること
3. Windowがresizeされる際、Rustileは既存のwindow rendering infrastructure（window_renderer module）を使用すること
4. Window位置を更新する間、Rustileはすべてのwindowが表示可能で使用可能な状態を維持すること

### 要件5: Performanceと制約
**目的:** window managerのuserとして、balanceが高速で応答性が高く、workflowを中断しないようにしたい。

#### 受入基準

1. Balance計算がtree traversalを必要とする場合、Rustileは冗長な計算なしに効率的なalgorithmを使用すること
2. Window数を再帰的に計算する際、Rustileは繰り返しtraversalを避けるためにsubtree countをcacheすること
3. Balance中、Rustileはblocking操作を回避することでUIの応答性を維持すること

### 要件6: Testと検証
**目的:** 開発者として、regressionが早期に検出され、動作が十分に文書化されるように、balance機能の包括的なtestを持ちたい。

#### 受入基準

1. Unit testが実行される際、test suiteは1、2、3、5、10個のwindowを含むtreeのbalance計算testを含むこと
2. さまざまなtree構造でbalanceがtriggerされた場合、testはbalanceされたtreeとunbalancedなtreeの両方で正しい比率計算を検証すること
3. Edge caseをtestする際、suiteは単一windowのtree、空のtree、深くnestedされた分割を持つtreeのscenarioを含むこと
4. Balance比率が計算される場合、testは比率が理想値の期待される許容範囲（±0.01）内にあることを検証すること

### 要件7: 後方互換性
**目的:** Rustile userとして、既存の設定とworkflowが引き続き機能し、upgradeによってsetupが壊れないようにしたい。

#### 受入基準

1. Rustileがbalance keybindingなしの設定をloadする場合、systemはbalance機能なしで正常に機能すること
2. Userが既存のkeybindingを設定している場合、balance keybindingの追加はそれらに影響を与えないこと
3. 既存のBSP tree構造が存在する場合、balance commandはtreeがどのように作成されたかに関係なく正しく機能すること
4. Balance commandがtriggerされない場合、Rustileは現在の実装と同一に動作すること
5. BspTree APIが他のmoduleで使用されている場合、balance機能は既存の`add_window()`または`remove_window()` method signatureの変更を必要としないこと
