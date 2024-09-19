use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize, Position};
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowId};

struct MyUserEvent;

struct State {
    windows: Vec<Window>,
    focused_window_index: usize,
    cmd_pressed: bool,
}

impl State {
    fn tile_windows_horizontally(&mut self) {
        let num_windows = self.windows.len();

        for (i, window) in self.windows.iter_mut().enumerate() {
            let monitor = window.current_monitor().expect("Failed to get monitor");
            let display_size = monitor.size();

            let window_width = display_size.width / num_windows as u32;
            let x_position = i as u32 * window_width;

            window.set_outer_position(Position::Physical(PhysicalPosition::new(
                x_position as i32,
                0,
            )));
            let _ = window.request_inner_size(PhysicalSize::new(window_width, display_size.height));
        }
    }

    fn tile_windows_vertically(&mut self) {
        let num_windows = self.windows.len();

        for (i, window) in self.windows.iter_mut().enumerate() {
            let monitor = window.current_monitor().expect("Failed to get monitor");
            let display_size = monitor.size();

            let window_height = display_size.height / num_windows as u32;
            let y_position = i as u32 * window_height;

            window.set_outer_position(Position::Physical(PhysicalPosition::new(
                0,
                y_position as i32,
            )));
            let _ = window.request_inner_size(PhysicalSize::new(display_size.width, window_height));
        }
    }

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
}

impl ApplicationHandler<MyUserEvent> for State {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

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
                // TODO: Custom keybindings
                if event.physical_key == KeyCode::SuperLeft && event.state.is_pressed() {
                    self.cmd_pressed = true;
                }
                if event.physical_key == KeyCode::SuperLeft && !event.state.is_pressed() {
                    self.cmd_pressed = false;
                }

                if event.physical_key == KeyCode::Digit0 && event.state.is_pressed() {
                    if self.cmd_pressed {
                        self.tile_windows_horizontally();
                    }
                }
                if event.physical_key == KeyCode::Digit9 && event.state.is_pressed() {
                    if self.cmd_pressed {
                        self.tile_windows_vertically();
                    }
                }

                if event.physical_key == KeyCode::KeyJ && event.state.is_pressed() {
                    self.focus_next_window();
                }
                if event.physical_key == KeyCode::KeyK && event.state.is_pressed() {
                    self.focus_previous_window();
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

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        for window in &self.windows {
            window.request_redraw();
        }
    }
}

fn main() {
    // TODO: extend to accept windows created by other applications (deamonize this app?)
    let event_loop = EventLoop::<MyUserEvent>::with_user_event().build().unwrap();

    let mut windows = Vec::new();
    for i in 0..3 {
        #[allow(deprecated)]
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();

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
