mod renderer;

use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use renderer::Renderer;

struct ChessGUI {
    window: Arc<Window>,
    renderer: Renderer,
}

impl ChessGUI {
    async fn new(event_loop: &EventLoop<()>) -> Self {
        let window = Arc::new(
            WindowBuilder::new()
                .with_title("Chess")
                .with_inner_size(winit::dpi::LogicalSize::new(800, 800))
                .build(event_loop)
                .unwrap(),
        );

        let renderer = Renderer::new(window.clone()).await;

        Self { window, renderer }
    }
}

pub fn run() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = pollster::block_on(ChessGUI::new(&event_loop));

    event_loop.run(move |event, window_target| {
        // Set the control flow to wait for events
        window_target.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent {
                window_id,
                event: WindowEvent::Resized(physical_size),
            } if window_id == app.window.id() => {
                app.renderer.resize(physical_size);
            }
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
                match app.renderer.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => app.renderer.resize(app.window.inner_size()),
                    Err(wgpu::SurfaceError::OutOfMemory) => window_target.exit(),
                    Err(e) => eprintln!("Render error: {:?}", e),
                }
            }
            Event::AboutToWait => {
                // Request a redraw before waiting
                app.window.request_redraw();
            }
            _ => {}
        }
    }).unwrap();
}