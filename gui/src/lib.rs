mod board;
mod camera;
mod pieces;
mod renderer;
mod sprite_batch;
mod text_renderer;

use board::BoardRenderer;
use chess_core::{File, GameState, Rank, Square};
use pieces::PieceRenderer;
use renderer::Renderer;
use std::sync::Arc;
use text_renderer::TextRenderer;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

struct ChessGUI {
    window: Arc<Window>,
    renderer: Renderer,
    board: BoardRenderer,
    pieces: PieceRenderer,
    text_renderer: Option<TextRenderer>,
    game_state: GameState,
}

impl ChessGUI {
    async fn new(event_loop: &EventLoop<()>) -> Self {
        let window = Arc::new(
            WindowBuilder::new()
                .with_title("Chess")
                .with_inner_size(winit::dpi::LogicalSize::new(800, 800))
                .build(event_loop)
                .expect("Failed to create window"),
        );

        let renderer = Renderer::new(window.clone()).await;
        let board = BoardRenderer::new(800.0);
        let pieces = PieceRenderer::new();
        let game_state = GameState::new();
        let text_renderer =
            TextRenderer::new(&renderer.device, &renderer.queue, renderer.config.format);

        Self {
            window,
            renderer,
            board,
            pieces,
            text_renderer: Some(text_renderer),
            game_state,
        }
    }
}

pub fn run() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = pollster::block_on(ChessGUI::new(&event_loop));

    // Generate initial vertices
    update_display(&mut app);

    event_loop
        .run(move |event, window_target| {
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
                    render_frame(&mut app);
                }
                Event::AboutToWait => {
                    // Request a redraw before waiting
                    app.window.request_redraw();
                }
                _ => {}
            }
        })
        .unwrap();
}

fn update_display(app: &mut ChessGUI) {
    // Only update board vertices now, pieces will be rendered as text
    let board_vertices = app.board.generate_vertices();
    app.renderer.update_vertices(board_vertices);
}

fn render_frame(app: &mut ChessGUI) {
    match app.renderer.begin_frame() {
        Ok((output, view, mut encoder)) => {
            // First render pass: render the board
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Board Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                if app.renderer.num_vertices > 0 {
                    render_pass.set_pipeline(&app.renderer.render_pipeline);
                    render_pass.set_vertex_buffer(0, app.renderer.vertex_buffer.slice(..));
                    render_pass.draw(0..app.renderer.num_vertices, 0..1);
                }
            }

            // Second render pass: render pieces using text
            if let Some(text_renderer) = &mut app.text_renderer {
                let window_size = app.window.inner_size();
                let board_size = 800.0;
                let square_size = board_size / 8.0;

                // Collect all pieces to render
                let mut pieces = Vec::new();
                for rank in 0..8 {
                    for file in 0..8 {
                        if let (Some(f), Some(r)) = (File::new(file), Rank::new(rank)) {
                            let square = Square::new(f, r);
                            if let Some(piece) = app.game_state.board.piece_at(square) {
                                // Calculate piece position (center of square)
                                // Note: rank 0 is at the bottom in chess, but top in screen coords
                                let x = file as f32 * square_size + square_size / 2.0;
                                let y = (7 - rank) as f32 * square_size + square_size / 2.0;

                                // Convert to NDC
                                let ndc_x = (x / board_size) * 2.0 - 1.0;
                                let ndc_y = 1.0 - (y / board_size) * 2.0;

                                pieces.push((piece.piece_type, piece.color, ndc_x, ndc_y));
                            }
                        }
                    }
                }

                // Prepare text areas
                text_renderer.prepare_pieces(
                    &app.renderer.device,
                    &app.renderer.queue,
                    &pieces,
                    square_size,
                    window_size.width as f32,
                    window_size.height as f32,
                );

                // Render text in a new pass
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Text Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                text_renderer.render(&mut render_pass);
            }

            app.renderer.submit_frame(encoder, output);
        }
        Err(wgpu::SurfaceError::Lost) => app.renderer.resize(app.window.inner_size()),
        Err(wgpu::SurfaceError::OutOfMemory) => std::process::exit(0),
        Err(e) => eprintln!("Render error: {:?}", e),
    }
}
