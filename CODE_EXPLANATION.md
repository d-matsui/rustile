# Rustile コード解説書

このドキュメントでは、Rustile ウィンドウマネージャーの全ての動作について、エントリーポイントからウィンドウタイリングアルゴリズムまで詳しく解説します。

## 目次

1. [プロジェクト概要](#プロジェクト概要)
2. [メインエントリーポイント (main.rs)](#メインエントリーポイント-mainrs)
3. [設定システム (config.rs)](#設定システム-configrs)
4. [ウィンドウマネージャーコア (window_manager.rs)](#ウィンドウマネージャーコア-window_managerrs)
5. [レイアウトシステム (layout.rs)](#レイアウトシステム-layoutrs)
6. [キーボード管理 (keyboard.rs)](#キーボード管理-keyboardrs)
7. [キーパーサー (keys.rs)](#キーパーサー-keysrs)
8. [ライブラリ構造 (lib.rs)](#ライブラリ構造-librs)
9. [コンポーネントの相互作用](#コンポーネントの相互作用)
10. [イベントフロー](#イベントフロー)
11. [テスト](#テスト)

---

## プロジェクト概要

Rustile は Rust で書かれたタイリングウィンドウマネージャーで、ウィンドウを重複することなく自動的に配置します。X11 プロトコルを使用してディスプレイサーバーと通信し、ウィンドウを管理します。

**主要な概念:**
- **ウィンドウマネージャー**: ウィンドウの表示方法を制御するプログラム
- **タイリング**: ウィンドウを重複なく画面を埋めるように自動配置すること
- **X11**: Linux システムで使用されるディスプレイサーバープロトコル
- **イベント駆動**: プログラムがイベント（ウィンドウ開く、キー押下など）に応答すること

**アーキテクチャ:**
```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│   main.rs   │───▶│WindowManager │───▶│   X11       │
└─────────────┘    └──────┬───────┘    │   Server    │
                          │            └─────────────┘
                          ▼
              ┌─────────────────────────┐
              │     コンポーネント:      │
              │ ┌─────────────────────┐ │
              │ │   LayoutManager     │ │
              │ │  (ウィンドウ配置)    │ │
              │ └─────────────────────┘ │
              │ ┌─────────────────────┐ │
              │ │  KeyboardManager    │ │
              │ │  (ショートカット)    │ │
              │ └─────────────────────┘ │
              └─────────────────────────┘
```

---

## メインエントリーポイント (main.rs)

```rust
//! ウィンドウマネージャーのエントリーポイント。ログを初期化してウィンドウマネージャーを開始します。

use anyhow::Result;
use rustile::window_manager::WindowManager;
use tracing::info;

fn main() -> Result<()> {
    // デバッグメッセージを表示するためのロギングシステムを初期化
    tracing_subscriber::fmt::init();
    
    info!("Rustile ウィンドウマネージャーを開始しています");
    
    // X11 サーバー（ディスプレイサーバー）に接続
    // 接続とスクリーン番号を返す
    let (conn, screen_num) = x11rb::connect(None)?;
    info!("X11 ディスプレイに接続しました、スクリーン {}", screen_num);
    
    // ウィンドウマネージャーを作成して実行
    let wm = WindowManager::new(conn, screen_num)?;
    wm.run()
}
```

**ここで何が起きているか:**

1. **ログ設定**: `tracing_subscriber::fmt::init()` でログを設定し、ウィンドウマネージャーの動作を確認できるようにします
2. **X11 接続**: `x11rb::connect(None)` で X11 ディスプレイサーバーに接続します
   - `conn`: X11 にコマンドを送るための接続オブジェクト
   - `screen_num`: 使用するモニター/スクリーン（通常はプライマリディスプレイの 0）
3. **ウィンドウマネージャー作成**: `WindowManager::new()` でウィンドウマネージャーのインスタンスを作成
4. **イベントループ**: `wm.run()` でイベントを処理する無限ループを開始

**エラーハンドリング**: `Result<()>` の戻り値の型は、この関数がエラーで失敗する可能性があることを意味し、`?` 演算子はエラーを呼び出し元に伝播させます。

---

## 設定システム (config.rs)

Rustile は現在、TOML ファイルから設定を読み込む動的設定システムを使用しています。

```rust
//! ウィンドウマネージャーの設定読み込みと管理

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub shortcuts: HashMap<String, String>,
    pub layout: LayoutConfig,
    pub general: GeneralConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LayoutConfig {
    pub master_ratio: f32,
    pub gap_size: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GeneralConfig {
    pub default_display: String,
}
```

**設定システム:**

1. **読み込み順序**:
   - 最初に試行: `~/.config/rustile/config.toml`
   - 見つからない場合はデフォルト値にフォールバック

2. **設定構造**:
   - **shortcuts**: キーの組み合わせをコマンドにマップ
     - 例: `"Super+t" = "xterm"`
   - **layout**: ウィンドウレイアウト設定
     - `master_ratio`: 0.0-1.0（デフォルト 0.5）
     - `gap_size`: ウィンドウ間のピクセル数（ギャップシステム対応）
   - **general**: 一般設定
     - `default_display`: アプリ起動用のX11ディスプレイ

3. **設定ファイル例**:
```toml
[general]
default_display = ":1"

[layout]
master_ratio = 0.5
gap_size = 10

[shortcuts]
"Shift+Alt+1" = "gnome-terminal"
"Shift+Alt+2" = "code"
"Super+Return" = "xterm"
```

**TOML 設定の利点:**
- ユーザーフレンドリーな形式
- 変更のための再コンパイルが不要
- 複雑なキーの組み合わせに対応
- 設定の共有が容易
- 視覚的なギャップシステムとフォーカス管理に対応

---

## ウィンドウマネージャーコア (window_manager.rs)

これはすべてを調整するウィンドウマネージャーの中核部分です。

### データ構造

```rust
/// メインウィンドウマネージャー構造体
pub struct WindowManager<C: Connection> {
    /// X11 接続
    conn: C,
    /// スクリーン情報
    screen_num: usize,
    /// 現在管理されているウィンドウ
    windows: Vec<Window>,
    /// フォーカスされているウィンドウ（視覚的ボーダー用）
    focused_window: Option<Window>,
    /// ウィンドウ配置用レイアウトマネージャー
    layout_manager: LayoutManager,
    /// ショートカット用キーボードマネージャー
    keyboard_manager: KeyboardManager,
    /// 設定
    config: Config,
}
```

**フィールド説明:**
- `conn`: X11 サーバーにコマンドを送るための接続
- `screen_num`: 管理するモニター
- `windows`: 現在管理中のウィンドウのリスト
- `focused_window`: フォーカスされているウィンドウ（視覚的ボーダー表示用）
- `layout_manager`: ウィンドウの位置とサイズを管理
- `keyboard_manager`: キーボードショートカットを処理
- `config`: TOML から読み込まれた設定

### 初期化 (`new()`)

```rust
pub fn new(conn: C, screen_num: usize) -> Result<Self> {
    // 設定を読み込み
    let config = Config::load()?;
    info!("設定を読み込みました、ショートカット数: {}", config.shortcuts().len());

    let setup = conn.setup();
    let screen = &setup.roots[screen_num];
    let root = screen.root;

    // キーボードマネージャーを初期化
    let mut keyboard_manager = KeyboardManager::new(&conn, setup)?;

    // ウィンドウマネージャーとして登録
    let event_mask = EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY;
    let attributes = ChangeWindowAttributesAux::new().event_mask(event_mask);
    
    if let Err(e) = conn.change_window_attributes(root, &attributes)?.check() {
        error!("他のウィンドウマネージャーが既に動作中です: {:?}", e);
        return Err(anyhow::anyhow!("ウィンドウマネージャーになれませんでした。他のWMが動作していませんか？"));
    }
    
    info!("ウィンドウマネージャーとしての登録に成功しました");

    // 設定からキーボードショートカットを登録
    keyboard_manager.register_shortcuts(&conn, root, config.shortcuts())?;

    Ok(Self {
        conn,
        screen_num,
        windows: Vec::new(),
        focused_window: None,
        layout_manager: LayoutManager::new(),
        keyboard_manager,
        config,
    })
}
```

**初期化ステップ:**

1. **設定読み込み**: TOML ファイルから設定を読み込み
2. **スクリーン情報取得**: X11 からモニター情報を取得
3. **キーボードマネージャー作成**: キーボード処理を設定
4. **ウィンドウマネージャー登録**: X11 にウィンドウの制御権を要求
   - `SUBSTRUCTURE_REDIRECT`: ウィンドウの配置を制御
   - `SUBSTRUCTURE_NOTIFY`: ウィンドウの作成/削除の通知を受信
5. **エラーチェック**: 他のウィンドウマネージャーが動作中なら失敗
6. **ショートカット登録**: 設定されたキーの組み合わせを X11 に登録
7. **インスタンス作成**: 全コンポーネントを初期化

### メインイベントループ (`run()`)

```rust
pub fn run(mut self) -> Result<()> {
    info!("ウィンドウマネージャーのイベントループを開始します");
    
    loop {
        self.conn.flush()?;
        let event = self.conn.wait_for_event()?;
        
        if let Err(e) = self.handle_event(event) {
            error!("イベント処理エラー: {:?}", e);
        }
    }
}
```

**イベントループのステップ:**
1. **フラッシュ**: X11 への保留中のコマンドを送信
2. **待機**: X11 からのイベントを受信するまでブロック
3. **処理**: イベントを処理（特定のハンドラーに委託）
4. **エラーハンドリング**: エラーをログ出力するがクラッシュしない
5. **繰り返し**: ステップ1に戻る

### イベントハンドラー

#### キープレスハンドラー（フォーカス管理付き）
```rust
fn handle_key_press(&mut self, event: KeyPressEvent) -> Result<()> {
    if let Some(command) = self.keyboard_manager.handle_key_press(&event) {
        info!("ショートカットが押されました、実行中: {}", command);
        
        // コマンドをパース（単純な実装、改善の余地あり）
        let parts: Vec<&str> = command.split_whitespace().collect();
        if let Some(program) = parts.first() {
            let mut cmd = Command::new(program);
            
            // 引数があれば追加
            if parts.len() > 1 {
                cmd.args(&parts[1..]);
            }
            
            // ディスプレイ環境を設定
            cmd.env("DISPLAY", self.config.default_display());
            
            match cmd.spawn() {
                Ok(_) => info!("起動に成功: {}", command),
                Err(e) => error!("{} の起動に失敗: {}", command, e),
            }
        }
    }
    
    // フォーカス管理：ウィンドウクリック時
    if let Some(window) = self.get_window_at_position(event.event_x, event.event_y) {
        self.set_focus(window)?;
    }
    
    Ok(())
}
```

#### マップリクエストハンドラー（新しいウィンドウ）
```rust
fn handle_map_request(&mut self, event: MapRequestEvent) -> Result<()> {
    let window = event.window;
    info!("ウィンドウをマッピング中: {:?}", window);
    
    // ウィンドウを可視化
    self.conn.map_window(window)?;
    
    // 管理ウィンドウに追加
    self.windows.push(window);
    
    // 最新ウィンドウにフォーカス設定
    self.set_focus(window)?;
    
    // 設定されたマスター比率でレイアウトを適用
    self.apply_layout()?;
    
    Ok(())
}
```

#### フォーカス管理（視覚的ボーダー付き）
```rust
fn set_focus(&mut self, window: Window) -> Result<()> {
    // 以前のフォーカスウィンドウのボーダーをクリア
    if let Some(old_focused) = self.focused_window {
        self.set_window_border(old_focused, 0x808080, 1)?; // グレーの細いボーダー
    }
    
    // 新しいフォーカスウィンドウに明確なボーダーを設定
    self.set_window_border(window, 0x0066CC, 3)?; // 青の太いボーダー
    
    // X11 入力フォーカスを設定
    self.conn.set_input_focus(InputFocus::POINTER_ROOT, window, CURRENT_TIME)?;
    
    self.focused_window = Some(window);
    info!("ウィンドウにフォーカスを設定: {:?}", window);
    
    Ok(())
}

fn set_window_border(&self, window: Window, color: u32, width: u32) -> Result<()> {
    let attributes = ChangeWindowAttributesAux::new()
        .border_pixel(color)
        .border_width(width);
    
    self.conn.change_window_attributes(window, &attributes)?.check()?;
    Ok(())
}
```

---

## レイアウトシステム (layout.rs)

レイアウトシステムは、ウィンドウがどこに配置され、どのサイズになるかを決定します。

### レイアウトタイプ

```rust
/// 異なるタイリングレイアウトを表現
#[derive(Debug, Clone, Copy)]
pub enum Layout {
    /// マスタースタックレイアウト：左側にマスターウィンドウ、右側にスタック
    MasterStack,
}

/// ウィンドウレイアウトマネージャー（ギャップシステム対応）
pub struct LayoutManager {
    current_layout: Layout,
}
```

現在はレイアウトは一つ（MasterStack）のみですが、この設計により以下のようなレイアウトを簡単に追加できます：
- 水平分割
- グリッドレイアウト
- フィボナッチスパイラル
- フローティングウィンドウ

### マスタースタックアルゴリズム（ギャップシステム付き）

これがウィンドウを配置する核心アルゴリズムです：

```rust
fn tile_master_stack(&self, conn: &impl Connection, screen: &Screen, windows: &[Window], gap_size: u32) -> Result<()> {
    // 空の場合の処理
    if windows.is_empty() {
        return Ok(());
    }

    let screen_width = screen.width_in_pixels as i16;   // 例：1280
    let screen_height = screen.height_in_pixels as i16; // 例：720
    let num_windows = windows.len() as i16;
    let gap = gap_size as i16;

    // マスターウィンドウ（最初のウィンドウ）を設定
    let master_window = windows[0];
    let master_width = if num_windows > 1 {
        ((screen_width - gap * 3) as f32 * MASTER_RATIO) as i16  // ギャップを考慮したマスター幅
    } else {
        screen_width - gap * 2  // 単一ウィンドウでも左右にギャップ
    };

    let master_config = ConfigureWindowAux::new()
        .x(gap as i32)                              // 左端からギャップ分離す
        .y(gap as i32)                              // 上端からギャップ分離す
        .width((master_width - gap) as u32)         // ギャップ分幅を減らす
        .height((screen_height - gap * 2) as u32);  // 上下ギャップ分高さを減らす

    conn.configure_window(master_window, &master_config)?;

    // スタックウィンドウ（残りのウィンドウ）を設定
    if num_windows > 1 {
        let stack_windows = &windows[1..];  // 最初以外すべて
        let stack_x = master_width + gap * 2;       // マスター終了位置＋ギャップ
        let stack_width = screen_width - master_width - gap * 3;  // 残り幅からギャップ分引く
        let stack_height = (screen_height - gap * (num_windows + 1)) / (num_windows - 1);  // 高さをギャップ考慮して分割

        for (index, &window) in stack_windows.iter().enumerate() {
            let stack_y = gap + (index as i16) * (stack_height + gap);  // 垂直にスタック、ギャップ付き

            let stack_config = ConfigureWindowAux::new()
                .x(stack_x as i32)              // スクリーン右半分
                .y(stack_y as i32)              // スタック位置
                .width(stack_width as u32)      // 右半分の幅
                .height(stack_height as u32);   // 分割された高さ

            conn.configure_window(window, &stack_config)?;
        }
    }

    Ok(())
}
```

**ギャップシステム付きビジュアル例:**

```
ギャップなし:                ギャップ=10px:               
┌────────┬────────┐         ┌──────┬──────┐        
│        │        │         │      │      │        
│   W1   │   W2   │         │  W1  │  W2  │        
│        │        │         │      ├──────┤        
│        │        │         │      │  W3  │        
└────────┴────────┘         └──────┴──────┘        
```

**アルゴリズムステップ:**

1. **マスターウィンドウ**:
   - 常にリストの最初のウィンドウ
   - スクリーンの左側を占有
   - 幅 = MASTER_RATIO * (screen_width - gaps) (デフォルト50%)
   - 高さ = フルスクリーン高さ - 上下ギャップ

2. **スタックウィンドウ**:
   - その他すべてのウィンドウ
   - スクリーンの右側を共有
   - それぞれ均等な高さ: (screen_height - gaps) / スタックウィンドウ数
   - すべて同じ幅: 残りのスクリーン幅

3. **ギャップシステム**:
   - 画面端: gap ピクセル
   - ウィンドウ間: gap ピクセル
   - 設定可能: config.toml の gap_size

---

## キーボード管理 (keyboard.rs)

キーボードシステムは、キーマッピングとショートカット処理を管理します。

### キー概念

- **Keysym**: 汎用キー識別子（例：0x0074 は 'T'）
- **Keycode**: 特定のキーボードの物理キー番号
- **Modifier**: Shift、Ctrl、Alt、Super などの修飾キー

### データ構造

```rust
/// キーボードマッピングとショートカットを管理
pub struct KeyboardManager {
    /// keysym から keycode へのマップ
    keycode_map: HashMap<u32, u8>,
    /// 登録されたショートカット
    shortcuts: Vec<Shortcut>,
}

/// キーボードショートカットを表現
#[derive(Debug, Clone)]
pub struct Shortcut {
    pub modifiers: KeyButMask,
    pub keycode: u8,
    pub command: String,
}
```

キーボードマネージャーは現在、keycode マッピングと関連するコマンドと共に登録されたショートカットのリストの両方を保存します。

### 初期化

```rust
pub fn new<C: Connection>(conn: &C, setup: &Setup) -> Result<Self> {
    let min_keycode = setup.min_keycode;
    let max_keycode = setup.max_keycode;
    
    // X サーバーからキーボードマッピングを取得
    let mapping_reply = conn
        .get_keyboard_mapping(min_keycode, max_keycode - min_keycode + 1)?
        .reply()?;
    
    let keysyms_per_keycode = mapping_reply.keysyms_per_keycode as usize;
    let mut keycode_map = HashMap::new();
    
    // keycode マップを構築
    for (index, chunk) in mapping_reply.keysyms.chunks(keysyms_per_keycode).enumerate() {
        let keycode = min_keycode + index as u8;
        
        // 各 keycode の最初の keysym を保存（非シフト）
        if let Some(&keysym) = chunk.first() {
            if keysym != 0 {
                keycode_map.insert(keysym, keycode);
            }
        }
    }
    
    info!("キーボードマネージャーを初期化しました、keycode 数: {}", keycode_map.len());
    
    Ok(Self { 
        keycode_map,
        shortcuts: Vec::new(),
    })
}
```

**これが行うこと:**
1. X11 にキーボードマッピングテーブルを要求
2. 各物理キーについて、それが表すシンボルを取得
3. マップを構築: keysym → keycode
4. 例：0x0074 ('T') → keycode 28

### 設定からショートカット登録

```rust
pub fn register_shortcuts<C: Connection>(
    &mut self,
    conn: &C,
    root: Window,
    shortcuts: &HashMap<String, String>,
) -> Result<()> {
    for (key_combo, command) in shortcuts {
        // キーの組み合わせをパース（例："Super+t"）
        let (modifiers, keysym) = parse_key_combination(key_combo)?;
        
        // 物理 keycode を取得
        let keycode = self.get_keycode(keysym);
        if keycode == 0 {
            warn!("キー '{}' の keycode が見つかりません、スキップします", key_combo);
            continue;
        }
        
        // ModMask を X11 用の KeyButMask に変換
        let key_but_mask = KeyButMask::from(modifiers.bits());
        
        // キーの組み合わせをグラブ（捕獲）
        conn.grab_key(
            true,
            root,
            key_but_mask,
            keycode,
            GrabMode::ASYNC,
            GrabMode::ASYNC,
        )?;
        
        // ショートカットを保存
        self.shortcuts.push(Shortcut {
            modifiers: key_but_mask,
            keycode,
            command: command.clone(),
        });
    }
    
    Ok(())
}
```

**これが行うこと:**
1. 設定されたすべてのショートカットを反復
2. 人間が読めるキーの組み合わせをパース（keys.rs で処理）
3. 物理 keycode に変換
4. 各組み合わせを X11 に登録
5. 後でマッチングするためにショートカットを保存

---

## キーパーサー (keys.rs)

キーパーサーモジュールは、人間が読めるキーの組み合わせを X11 の keysym と修飾子に変換する処理を行います。これにより、生の16進値の代わりに「Super+t」のようなキーを使用できる、ユーザーフレンドリーな設定システムが実現されます。

### コア関数

```rust
/// "Super+t" や "Ctrl+Alt+Delete" のようなキーの組み合わせ文字列をパース
pub fn parse_key_combination(combo: &str) -> Result<(ModMask, u32)> {
    let parts: Vec<&str> = combo.split('+').collect();
    
    if parts.is_empty() {
        return Err(anyhow::anyhow!("空のキーの組み合わせです"));
    }
    
    let mut modifiers = ModMask::from(0u16);
    let key_part;
    
    // 修飾子とキーをパース
    if parts.len() == 1 {
        // 修飾子なしの単一キー
        key_part = parts[0];
    } else {
        // 複数部分 - 最後以外はすべて修飾子
        for modifier in &parts[..parts.len() - 1] {
            match modifier.to_lowercase().as_str() {
                "super" | "mod4" | "win" | "windows" | "cmd" => modifiers |= ModMask::M4,
                "alt" | "mod1" | "meta" => modifiers |= ModMask::M1,
                "ctrl" | "control" | "ctl" => modifiers |= ModMask::CONTROL,
                "shift" => modifiers |= ModMask::SHIFT,
                "mod2" | "numlock" | "num" => modifiers |= ModMask::M2,
                "mod3" | "scrolllock" | "scroll" => modifiers |= ModMask::M3,
                "mod5" | "altgr" | "altgraph" => modifiers |= ModMask::M5,
                "hyper" => {
                    // Hyper = Super+Alt+Ctrl+Shift
                    modifiers |= ModMask::M4 | ModMask::M1 | 
                                ModMask::CONTROL | ModMask::SHIFT;
                }
                _ => return Err(anyhow::anyhow!("不明な修飾子: {}", modifier)),
            }
        }
        key_part = parts.last().unwrap();
    }
    
    // キー名を keysym に変換
    let keysym = get_keysym_from_name(key_part)?;
    
    Ok((modifiers, keysym))
}
```

### 修飾子サポート

キーパーサーは、クロスプラットフォーム対応のため代替名を含む包括的な修飾子キーをサポートします：

**主要修飾子:**
- `Super`, `Mod4`, `Win`, `Windows`, `Cmd` → Superキー（Windows/Cmdキー）
- `Alt`, `Mod1`, `Meta` → Altキー
- `Ctrl`, `Control`, `Ctl` → Controlキー
- `Shift` → Shiftキー

**使用頻度の低い修飾子:**
- `Mod2`, `NumLock`, `Num` → Num Lock
- `Mod3`, `ScrollLock`, `Scroll` → Scroll Lock
- `Mod5`, `AltGr`, `AltGraph` → AltGr（国際キーボードの右Alt）

**特殊な組み合わせ:**
- `Hyper` → 4つの主要修飾子すべての組み合わせ（Super+Alt+Ctrl+Shift）

### キー名マッピング

```rust
fn get_keysym_from_name(name: &str) -> Result<u32> {
    let normalized = name.to_lowercase();
    
    match normalized.as_str() {
        // 文字（a-z）
        c if c.len() == 1 && c.chars().next().unwrap().is_ascii_lowercase() => {
            Ok(c.chars().next().unwrap() as u32)
        }
        
        // 数字（0-9）
        c if c.len() == 1 && c.chars().next().unwrap().is_ascii_digit() => {
            Ok(c.chars().next().unwrap() as u32)
        }
        
        // 特殊キー
        "space" => Ok(0x0020),
        "return" | "enter" => Ok(0xff0d),
        "tab" => Ok(0xff09),
        "escape" | "esc" => Ok(0xff1b),
        "backspace" => Ok(0xff08),
        "delete" | "del" => Ok(0xffff),
        
        // ファンクションキー
        "f1" => Ok(0xffbe),
        "f2" => Ok(0xffbf),
        "f3" => Ok(0xffc0),
        "f4" => Ok(0xffc1),
        "f5" => Ok(0xffc2),
        "f6" => Ok(0xffc3),
        "f7" => Ok(0xffc4),
        "f8" => Ok(0xffc5),
        "f9" => Ok(0xffc6),
        "f10" => Ok(0xffc7),
        "f11" => Ok(0xffc8),
        "f12" => Ok(0xffc9),
        
        // 矢印キー
        "up" => Ok(0xff52),
        "down" => Ok(0xff54),
        "left" => Ok(0xff51),
        "right" => Ok(0xff53),
        
        _ => Err(anyhow::anyhow!("不明なキー名: {}", name)),
    }
}
```

### 使用例

```rust
// 単純キー
parse_key_combination("t") → (ModMask::empty(), 0x0074)

// 単一修飾子
parse_key_combination("Super+t") → (ModMask::M4, 0x0074)

// 複数修飾子  
parse_key_combination("Ctrl+Alt+Delete") → (ModMask::CONTROL | ModMask::M1, 0xffff)

// 代替名
parse_key_combination("Cmd+space") → (ModMask::M4, 0x0020)  // Super+spaceと同じ
parse_key_combination("Win+Return") → (ModMask::M4, 0xff0d)  // Super+Returnと同じ

// 複雑な組み合わせ
parse_key_combination("Hyper+F12") → (ModMask::M4 | ModMask::M1 | ModMask::CONTROL | ModMask::SHIFT, 0xffc9)
```

### 大文字小文字の区別なし

ユーザーの利便性のため、すべてのパースは大文字小文字を区別しません：

```rust
"SUPER+T" == "super+t" == "Super+T" == "SuPeR+t"
```

### エラーハンドリング

パーサーは役立つエラーメッセージを提供します：

```rust
parse_key_combination("") → エラー: "空のキーの組み合わせです"
parse_key_combination("Unknown+t") → エラー: "不明な修飾子: unknown"  
parse_key_combination("Super+xyz") → エラー: "不明なキー名: xyz"
```

### キーボードマネージャーとの統合

キーパーサーは、ショートカット登録時にキーボードマネージャーによって使用されます：

```rust
pub fn register_shortcuts<C: Connection>(
    &mut self,
    conn: &C,
    root: Window,
    shortcuts: &HashMap<String, String>,
) -> Result<()> {
    for (key_combo, command) in shortcuts {
        // 人間が読めるキーの組み合わせをパース
        match parse_key_combination(key_combo) {
            Ok((modifiers, keysym)) => {
                // keycode に変換して X11 に登録
                let keycode = self.get_keycode(keysym);
                if keycode != 0 {
                    self.register_shortcut(conn, root, modifiers, keycode, command)?;
                }
            }
            Err(e) => warn!("キーの組み合わせ '{}' のパースに失敗: {}", key_combo, e),
        }
    }
    Ok(())
}
```

これにより、ユーザーは以下のような設定を書くことができます：

```toml
[shortcuts]
"Super+Return" = "xterm"
"Ctrl+Alt+t" = "gnome-terminal" 
"Shift+Alt+1" = "firefox"
"Win+space" = "dmenu_run"  # Superの代替名
```

### 利点

1. **ユーザーフレンドリー**: 16進コードの代わりに自然なキーの組み合わせ
2. **クロスプラットフォーム**: 代替修飾子名（Cmd、Win、Meta）
3. **柔軟性**: 大文字小文字の区別なし、複数の命名オプション
4. **堅牢性**: 包括的なエラーハンドリングと検証
5. **拡張可能**: 新しいキー名と修飾子の追加が容易

---

## ライブラリ構造 (lib.rs)

```rust
//! Rustile - Rust で書かれたタイリングウィンドウマネージャー
//! 
//! このウィンドウマネージャーはマスタースタックレイアウトによる自動ウィンドウタイリングを提供します。
//! シンプル、効率的、拡張可能になるよう設計されています。

pub mod config;
pub mod keyboard;
pub mod keys;
pub mod layout;
pub mod window_manager;
```

このファイルは、ライブラリのどの部分が公開されるかを定義します。他のコード（main.rs など）がモジュールを使用できるようにします。

---

## コンポーネントの相互作用

すべてのピースがどのように連携するかを以下に示します：

### 起動シーケンス

```
1. main.rs
   ├── ログを初期化
   ├── X11 に接続
   └── WindowManager を作成
       ├── TOML から設定を読み込み
       │   └── ~/.config/rustile/config.toml を試行
       ├── KeyboardManager を作成
       │   └── X11 からキーボードマッピングを読み込み
       ├── LayoutManager を作成
       │   └── デフォルトレイアウト（MasterStack）を設定
       ├── ウィンドウマネージャーとして登録
       │   └── X11 にウィンドウ配置の制御を通知
       └── 設定からショートカットを登録
           └── 設定されたすべてのキーの組み合わせをパースしてグラブ

2. イベントループを開始
   └── X11 イベントを永遠に待機
```

### イベント処理フロー

```
X11 イベント → WindowManager.handle_event()
├── KeyPress → handle_key_press()
│   ├── KeyboardManager が登録されたショートカットと照合
│   ├── keycode と修飾子でマッチするショートカットを検索
│   └── マッチした場合、関連するコマンドを実行
├── MapRequest → handle_map_request()
│   ├── ウィンドウを可視化
│   ├── ウィンドウリストに追加
│   ├── フォーカスを設定（視覚的ボーダー付き）
│   └── レイアウトアルゴリズムを適用
└── UnmapNotify → handle_unmap_notify()
    ├── ウィンドウリストから削除
    ├── フォーカスをクリア（該当する場合）
    └── レイアウトアルゴリズムを再適用
```

### レイアウト適用フロー

```
apply_layout()
├── X11 からスクリーン寸法を取得
├── LayoutManager.apply_layout() を呼び出し
│   └── tile_master_stack()
│       ├── マスターウィンドウのサイズ/位置を計算（ギャップ考慮）
│       ├── スタックウィンドウのサイズ/位置を計算（ギャップ考慮）
│       └── X11 に設定コマンドを送信
└── X11 がすべてのウィンドウを移動/リサイズ
```

---

## イベントフロー

ウィンドウマネージャーを使用するときに何が起きるかを以下に示します：

### ウィンドウを開く

```
ユーザーが実行: xclock
    ↓
X11 がウィンドウを作成するが表示しない
    ↓
X11 がウィンドウマネージャーに MapRequest を送信
    ↓
WindowManager.handle_map_request()
    ├── conn.map_window() - 可視化
    ├── windows.push() - リストに追加
    ├── set_focus() - フォーカス設定（視覚的ボーダー付き）
    └── apply_layout() - すべてを再配置
        ↓
LayoutManager.tile_master_stack()
    ├── すべてのウィンドウの新しい位置を計算（ギャップ付き）
    └── 各ウィンドウに conn.configure_window()
        ↓
X11 がウィンドウを移動/リサイズ
    ↓
ユーザーがタイル化されたウィンドウを確認
```

### ウィンドウを閉じる

```
ユーザーがウィンドウを閉じる（Xボタンまたは Alt+F4）
    ↓
X11 がウィンドウを破棄
    ↓
X11 がウィンドウマネージャーに UnmapNotify を送信
    ↓
WindowManager.handle_unmap_notify()
    ├── windows.retain() - リストから削除
    ├── フォーカスをクリア（該当する場合）
    └── apply_layout() - 残りウィンドウを再配置
        ↓
LayoutManager.tile_master_stack()
    ├── 残りウィンドウの新しい位置を計算
    └── 各ウィンドウに conn.configure_window()
        ↓
X11 が残りウィンドウを移動/リサイズ
    ↓
ユーザーが残りウィンドウがスペースを埋めるのを確認
```

### 設定されたショートカットを押す

```
ユーザーが Shift+Alt+1 を押す（gnome-terminal 用に設定済み）
    ↓
X11 がウィンドウマネージャーに KeyPress イベントを送信
    ↓
WindowManager.handle_key_press()
    ↓
KeyboardManager.handle_key_press()
    ├── keycode + 修飾子にマッチするショートカットを検索
    ├── コマンド "gnome-terminal" を返す
    └── WindowManager が実行: Command::new("gnome-terminal").spawn()
        ↓
新しい gnome-terminal プロセスが開始
    ↓
gnome-terminal がウィンドウを作成 → MapRequest イベント
    ↓
（上記「ウィンドウを開く」フローに従う）
```

### フォーカス切り替え（視覚フィードバック付き）

```
ユーザーがウィンドウをクリック
    ↓
X11 が KeyPress/ButtonPress イベントを送信
    ↓
WindowManager.set_focus()
    ├── 前のフォーカスウィンドウのボーダーをグレーに設定
    ├── 新しいウィンドウに青い太いボーダーを設定
    └── X11 入力フォーカスを設定
        ↓
ユーザーが視覚的なフォーカス変更を確認
```

---

## テスト

プロジェクトには信頼性を確保するための包括的なテストが含まれています：

### ユニットテスト

各モジュールファイル内（`#[cfg(test)]` セクション）に配置：

**設定テスト**:
- TOML からの設定読み込みをテスト
- デフォルト値を検証
- アクセサメソッドをテスト

**レイアウトテスト**:
- レイアウトマネージャーの作成をテスト
- 空のウィンドウリストを処理
- 寸法計算を検証
- ギャップシステムの計算をテスト

**キーボードテスト**:
- keycode 検索をテスト
- ショートカットマッチングをテスト
- 不足キーを処理

**キーテスト**（22テスト総計）:
- 単純キーと修飾子をパース
- 代替修飾子名をテスト（Win、Cmd、Meta）
- 大文字小文字を区別しないパース
- 複雑な修飾子の組み合わせ
- 特殊キー（Return、space、F-キー）
- 不明キーのエラーハンドリング

### マニュアルテスト

インタラクティブテストに `test_rustile.sh` を使用：
1. Xephyr（ネストされたX サーバー）を開始
2. デバッグログ付きで rustile を実行
3. テストウィンドウを開く
4. マニュアル操作を許可

### テストの実行

```bash
# すべてのテストを実行
cargo test

# 出力付きで実行
cargo test -- --nocapture

# 特定のテストを実行
cargo test test_master_window_dimensions

# マニュアルテスト
./test_rustile.sh
```

---

## まとめ

Rustile は以下を実演するシンプルだが完全なタイリングウィンドウマネージャーです：

1. **X11 プロトコル**: ディスプレイサーバーとの通信方法
2. **イベント駆動プログラミング**: ユーザーアクションへの応答
3. **モジュラー設計**: 各コンポーネントが明確な責任を持つ
4. **Rust の安全性**: メモリ安全性とエラーハンドリング
5. **テスト**: 信頼性のためのユニット・統合テスト

コードは以下のよう設計されています：
- **読みやすい**: 明確な名前とドキュメンテーション
- **保守しやすい**: モジュラー構造
- **拡張可能**: 新機能の追加が容易
- **安全**: Rust の型システムが一般的なバグを防ぐ

最近の改良点：
- ✅ 設定ファイルサポート（TOML）
- ✅ 人間が読めるキーの組み合わせ
- ✅ すべての X11 修飾子をサポート
- ✅ クロスプラットフォーム修飾子命名
- ✅ ウィンドウフォーカス管理と視覚的ボーダー
- ✅ 設定可能なギャップシステム

将来追加する機能：
- 複数レイアウト
- ウィンドウナビゲーションショートカット
- マルチモニターサポート
- ウィンドウデコレーション
- ステータスバー

このウィンドウマネージャーは、比較的少量の適切に構造化されたコードで、機能的なデスクトップ環境を作成する方法を示しています。