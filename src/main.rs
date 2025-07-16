use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use anyhow::Result;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let (conn, screen_num) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    // 1. ウィンドウマネージャとして登録する
    // これにより、ウィンドウの表示要求などが直接処理されず、このプログラムに通知される
    let event_mask = EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY;
    let attributes = ChangeWindowAttributesAux::new().event_mask(event_mask);
    
    // 他にWMがいないことを確認し、ルートウィンドウのイベントを受け取る
    if let Err(e) = conn.change_window_attributes(root, &attributes)?.check() {
        tracing::error!("Another window manager is already running: {:?}", e);
        return Err(anyhow::anyhow!("Failed to become window manager. Is another WM running?"));
    }
    tracing::info!("Successfully became the window manager on screen {}", screen_num);

    // 2. イベントループ
    // X11サーバーからのイベントを待ち、処理を続ける
    loop {
        conn.flush()?;
        let event = conn.wait_for_event()?;
        
        match event {
            Event::MapRequest(ev) => {
                tracing::info!("MapRequest for window {}", ev.window);
                // 新しいウィンドウの表示を許可する
                conn.map_window(ev.window)?;
            }
            Event::ConfigureRequest(ev) => {
                tracing::info!("ConfigureRequest for window {}", ev.window);
                // とりあえずウィンドウからの要求通りに設定を許可する
                let values = ConfigureWindowAux::from_configure_request(&ev);
                conn.configure_window(ev.window, &values)?;
            }
            _ => {
                // 他のイベントは今は無視
            }
        }
    }
}