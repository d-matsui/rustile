use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize, Position};
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

struct MyUserEvent;

struct State {
    windows: Vec<Window>,
}

impl ApplicationHandler<MyUserEvent> for State {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        // Your application got resumed.
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Handle window event.
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        _event: DeviceEvent,
    ) {
        // Handle device event.
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        for window in &self.windows {
            window.request_redraw();
        }
    }
}

fn main() {
    let event_loop = EventLoop::<MyUserEvent>::with_user_event().build().unwrap();

    let mut windows = Vec::new();
    for i in 0..3 {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();

        window.set_outer_position(Position::Physical(PhysicalPosition::new(
            100 * i as i32,
            100 * i as i32,
        )));

        windows.push(window);
    }

    tile_windows_horizontally(&mut windows);

    let mut state = State { windows };

    let _ = event_loop.run_app(&mut state);
}

fn tile_windows_horizontally(windows: &mut Vec<Window>) {
    let num_windows = windows.len();

    for (i, window) in windows.iter_mut().enumerate() {
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
