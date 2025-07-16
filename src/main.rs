use x11rb::connection::Connection;
use anyhow::Result;

fn main() -> Result<()> {
    // ログ出力を初期化
    tracing_subscriber::fmt::init();

    // X11サーバーに接続します
    // この時点では、どのディスプレイを使うか自動で判断されます
    let (conn, screen_num) = x11rb::connect(None)?;
    
    // デフォルトスクリーンの情報を取得
    let screen = &conn.setup().roots[screen_num];

    // 接続成功のメッセージと、画面サイズを表示
    tracing::info!("Successfully connected to X11!");
    println!();
    println!("------------------------------------");
    println!("Your screen size is: {}x{}", screen.width_in_pixels, screen.height_in_pixels);
    println!("------------------------------------");
    
    Ok(())
}
