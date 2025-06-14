use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

struct ChessGUI {
    window: Window,
}

impl ChessGUI {
    fn new(event_loop: &EventLoop<()>) -> Self {
        let window = WindowBuilder::new()
            .with_title("Chess")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 800))
            .build(event_loop)
            .unwrap();

        Self { window }
    }
}

pub fn run() {
    let event_loop = EventLoop::new().unwrap();
    let app = ChessGUI::new(&event_loop);

    event_loop.run(move |event, window_target| {
        // Set the control flow to wait for events
        window_target.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent {
                window_id,
                event: WindowEvent::CloseRequested,
            } if window_id == app.window.id() => {
                // Properly exit the event loop
                window_target.exit();
            }
            Event::WindowEvent {
                window_id,
                event: WindowEvent::RedrawRequested,
            } if window_id == app.window.id() => {
                // Handle redraw here
                // For now, just a placeholder
            }
            Event::AboutToWait => {
                // Request a redraw before waiting
                app.window.request_redraw();
            }
            _ => {}
        }
    }).unwrap();
}