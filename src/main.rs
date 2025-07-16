use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use anyhow::Result;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let (conn, screen_num) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    let mut windows: Vec<Window> = Vec::new();

    // ★修正: 抜けていた2行を追加
    let event_mask = EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY;
    let attributes = ChangeWindowAttributesAux::new().event_mask(event_mask);

    if let Err(e) = conn.change_window_attributes(root, &attributes)?.check() {
        tracing::error!("Another window manager is already running: {:?}", e);
        return Err(anyhow::anyhow!("Failed to become window manager. Is another WM running?"));
    }
    tracing::info!("Successfully became the window manager on screen {}", screen_num);

    loop {
        conn.flush()?;
        let event = conn.wait_for_event()?;
        
        match event {
            Event::MapRequest(ev) => {
                tracing::info!("MapRequest for window {}", ev.window);
                conn.map_window(ev.window)?;

                windows.push(ev.window);
                tracing::info!("Windows managed: {:?}", windows);
            }
            Event::ConfigureRequest(ev) => {
                let values = ConfigureWindowAux::from_configure_request(&ev);
                conn.configure_window(ev.window, &values)?;
            }
            Event::UnmapNotify(ev) => {
                tracing::info!("UnmapNotify for window {}", ev.window);
                windows.retain(|&w| w != ev.window);
                tracing::info!("Windows managed: {:?}", windows);
            }
            _ => {}
        }
    }
}