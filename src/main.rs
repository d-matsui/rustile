use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize, Position};
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowId};

struct MyUserEvent;

struct State {
    windows: Vec<Window>,        // List of managed windows
    focused_window_index: usize, // Index of the currently focused window
    cmd_pressed: bool,           // Tracks if the Cmd key is pressed
}

impl State {
    /// Arrange windows horizontally across the screen
    fn tile_windows_horizontally(&mut self) {
        let num_windows = self.windows.len();
        for (i, window) in self.windows.iter_mut().enumerate() {
            let monitor = window.current_monitor().expect("Failed to get monitor");
            let display_size = monitor.size();
            let window_width = display_size.width / num_windows as u32;
            let x_position = i as u32 * window_width;

            // Set window position and resize it
            window.set_outer_position(Position::Physical(PhysicalPosition::new(
                x_position as i32,
                0,
            )));
            let _ = window.request_inner_size(PhysicalSize::new(window_width, display_size.height));
        }
    }

    /// Arrange windows vertically down the screen
    fn tile_windows_vertically(&mut self) {
        let num_windows = self.windows.len();
        for (i, window) in self.windows.iter_mut().enumerate() {
            let monitor = window.current_monitor().expect("Failed to get monitor");
            let display_size = monitor.size();
            let window_height = display_size.height / num_windows as u32;
            let y_position = i as u32 * window_height;

            // Set window position and resize it
            window.set_outer_position(Position::Physical(PhysicalPosition::new(
                0,
                y_position as i32,
            )));
            let _ = window.request_inner_size(PhysicalSize::new(display_size.width, window_height));
        }
    }

    /// Focus on the next window (by index)
    fn focus_next_window(&mut self) {
        if self.windows.is_empty() {
            return;
        }

        if self.focused_window_index == self.windows.len() - 1 {
            self.focused_window_index = 0;
        } else {
            self.focused_window_index += 1;
        }
        self.windows[self.focused_window_index].focus_window();
    }

    /// Focus on the previous window (by index)
    fn focus_previous_window(&mut self) {
        if self.windows.is_empty() {
            return;
        }

        if self.focused_window_index == 0 {
            self.focused_window_index = self.windows.len() - 1;
        } else {
            self.focused_window_index -= 1;
        }
        self.windows[self.focused_window_index].focus_window();
    }

    /// Swap positions of two windows
    fn swap_windows(&mut self, index1: usize, index2: usize) {
        let window1 = &self.windows[index1];
        let window2 = &self.windows[index2];

        let position1 = window1.outer_position().unwrap();
        let size1 = window1.inner_size();
        let position2 = window2.outer_position().unwrap();
        let size2 = window2.inner_size();

        // Swap positions and sizes
        window1.set_outer_position(Position::Physical(PhysicalPosition::new(
            position2.x,
            position2.y,
        )));
        let _ = window1.request_inner_size(size2);
        window2.set_outer_position(Position::Physical(PhysicalPosition::new(
            position1.x,
            position1.y,
        )));
        let _ = window2.request_inner_size(size1);
    }

    /// Swap the current window with the next window (by position)
    fn swap_next_window(&mut self) {
        let mut windows_with_pos: Vec<(usize, PhysicalPosition<i32>)> = self
            .windows
            .iter()
            .enumerate()
            .map(|(i, w)| (i, w.outer_position().unwrap()))
            .collect();

        // Sort windows by their x-coordinate
        windows_with_pos.sort_by_key(|&(_, pos)| pos.x);

        // Determine the next window to swap with
        let next_idx = if let Some((idx, _)) = windows_with_pos
            .iter()
            .cycle()
            .skip_while(|&&(i, _)| i != self.focused_window_index)
            .nth(1)
        {
            *idx
        } else {
            self.focused_window_index
        };

        self.swap_windows(self.focused_window_index, next_idx);
    }

    /// Swap the current window with the previous window (by position)
    fn swap_previous_window(&mut self) {
        let mut windows_with_pos: Vec<(usize, PhysicalPosition<i32>)> = self
            .windows
            .iter()
            .enumerate()
            .map(|(i, w)| (i, w.outer_position().unwrap()))
            .collect();

        // Sort windows by their x-coordinate
        windows_with_pos.sort_by_key(|&(_, pos)| pos.x);

        // Determine the previous window to swap with
        let prev_idx = if let Some((idx, _)) = windows_with_pos
            .iter()
            .rev()
            .cycle()
            .skip_while(|&&(i, _)| i != self.focused_window_index)
            .nth(1)
        {
            *idx
        } else {
            self.focused_window_index
        };

        self.swap_windows(self.focused_window_index, prev_idx);
    }
}

impl ApplicationHandler<MyUserEvent> for State {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    /// Handle keyboard input and window events
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if event.physical_key == KeyCode::SuperLeft && event.state.is_pressed() {
                    self.cmd_pressed = true;
                }
                if event.physical_key == KeyCode::SuperLeft && !event.state.is_pressed() {
                    self.cmd_pressed = false;
                }

                // Tile windows horizontally (Cmd+0)
                if event.physical_key == KeyCode::Digit0 && event.state.is_pressed() {
                    if self.cmd_pressed {
                        self.tile_windows_horizontally();
                    }
                }

                // Tile windows vertically (Cmd+9)
                if event.physical_key == KeyCode::Digit9 && event.state.is_pressed() {
                    if self.cmd_pressed {
                        self.tile_windows_vertically();
                    }
                }

                // Focus next window (j)
                if event.physical_key == KeyCode::KeyJ && event.state.is_pressed() {
                    self.focus_next_window();
                }

                // Focus previous window (k)
                if event.physical_key == KeyCode::KeyK && event.state.is_pressed() {
                    self.focus_previous_window();
                }

                // Swap with the next window (l)
                if event.physical_key == KeyCode::KeyL && event.state.is_pressed() {
                    self.swap_next_window();
                }

                // Swap with the previous window (h)
                if event.physical_key == KeyCode::KeyH && event.state.is_pressed() {
                    self.swap_previous_window();
                }
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        _event: DeviceEvent,
    ) {
    }
}

fn main() {
    let event_loop = EventLoop::<MyUserEvent>::with_user_event().build().unwrap();

    let mut windows = Vec::new();
    for i in 0..3 {
        #[allow(deprecated)]
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();

        window.set_title(&format!("Window {}", i + 1));
        window.set_outer_position(Position::Physical(PhysicalPosition::new(
            100 * i as i32,
            100 * i as i32,
        )));

        windows.push(window);
    }

    let mut state = State {
        windows,
        focused_window_index: 0,
        cmd_pressed: false,
    };

    let _ = event_loop.run_app(&mut state);
}
