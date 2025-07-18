use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use anyhow::Result;
use std::process::Command;
const XK_T: u32 = 0x0074;

fn tile(conn: &impl Connection, screen: &Screen, windows: &[Window]) -> Result<()> {
    let screen_width = screen.width_in_pixels as i16;
    let screen_height = screen.height_in_pixels as i16;
    if windows.is_empty() { return Ok(()); }
    let num_windows = windows.len() as i16;
    let master_win = windows[0];
    let master_width = if num_windows > 1 { screen_width / 2 } else { screen_width };
    let master_values = ConfigureWindowAux::new().x(0).y(0).width(master_width as u32).height(screen_height as u32);
    conn.configure_window(master_win, &master_values)?;
    if num_windows > 1 {
        let stack_wins = &windows[1..];
        let stack_win_height = screen_height / (num_windows - 1);
        let stack_win_x = screen_width / 2;
        for (i, &win) in stack_wins.iter().enumerate() {
            let stack_values = ConfigureWindowAux::new().x(stack_win_x as i32).y((i as i16 * stack_win_height) as i32).width((screen_width / 2) as u32).height(stack_win_height as u32);
            conn.configure_window(win, &stack_values)?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let (conn, screen_num) = x11rb::connect(None)?;
    let setup = conn.setup();
    let screen = &setup.roots[screen_num];
    let root = screen.root;

    let min_keycode = setup.min_keycode;
    let max_keycode = setup.max_keycode;
    let mapping_reply = conn.get_keyboard_mapping(min_keycode, max_keycode - min_keycode + 1)?.reply()?;
    
    let per = mapping_reply.keysyms_per_keycode as usize;
    let key_t = mapping_reply
        .keysyms
        .chunks(per)
        .position(|keysyms| keysyms.contains(&XK_T))
        .map(|i| (min_keycode + i as u8))
        .ok_or_else(|| anyhow::anyhow!("Could not find keycode for T"))?;
    tracing::info!("Found keycode for T: {}", key_t);

    let mut windows: Vec<Window> = Vec::new();
    let event_mask = EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY;
    let attributes = ChangeWindowAttributesAux::new().event_mask(event_mask);
    if let Err(e) = conn.change_window_attributes(root, &attributes)?.check() {
        tracing::error!("Another window manager is already running: {:?}", e);
        return Err(anyhow::anyhow!("Failed to become window manager. Is another WM running?"));
    }
    tracing::info!("Successfully became the window manager");

    let mod_mask = ModMask::M4;
    // 'T'キーのキーコードでショートカットを登録
    conn.grab_key(true, root, mod_mask, key_t, GrabMode::ASYNC, GrabMode::ASYNC)?;

    loop {
        conn.flush()?;
        let event = conn.wait_for_event()?;
        match event {
            Event::KeyPress(ev) => {
                if ev.state.contains(mod_mask) && ev.detail == key_t {
                    tracing::info!("Mod+T pressed, launching xcalc...");
                    Command::new("xcalc").env("DISPLAY", ":10").spawn()?;
                }
            }
            Event::MapRequest(ev) => {
                conn.map_window(ev.window)?;
                windows.push(ev.window);
                tile(&conn, screen, &windows)?;
            }
            Event::UnmapNotify(ev) => {
                windows.retain(|&w| w != ev.window);
                tile(&conn, screen, &windows)?;
            }
            _ => {}
        }
    }
}