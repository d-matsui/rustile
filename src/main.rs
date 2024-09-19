use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

struct MyUserEvent;

struct State {
    windows: Vec<Window>,
    counter: i32,
}

impl ApplicationHandler<MyUserEvent> for State {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Your application got resumed.
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
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
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        // Handle device event.
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        for window in &self.windows {
            window.request_redraw();
        }
        self.counter += 1;
    }
}

fn main() {
    let event_loop = EventLoop::<MyUserEvent>::with_user_event().build().unwrap();

    let mut windows = Vec::new();
    for i in 0..3 {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();
        windows.push(window);
    }

    let mut state = State {
        windows,
        counter: 0,
    };

    let _ = event_loop.run_app(&mut state);
}
