mod board;
mod renderer;
mod text_renderer;

use board::BoardRenderer;
use chess_agents::{Agent, MinimaxAgent};
use chess_core::{
    generate_legal_moves, is_checkmate, is_stalemate, Color, File, GameState, Move, PieceType,
    Rank, Square,
};
use renderer::{Renderer, Vertex};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::thread;
use text_renderer::{TextRenderer, UiText};
use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

struct ChessGUI {
    window: Arc<Window>,
    renderer: Renderer,
    board: BoardRenderer,
    text_renderer: Option<TextRenderer>,
    game_state: GameState,
    mouse_position: PhysicalPosition<f64>,
    selected_square: Option<Square>,
    valid_moves: Vec<chess_core::Move>,
    promotion_pending: Option<PromotionState>,
    game_mode: GameMode,
    move_history: Vec<String>,
    ai_thinking: bool,
    mode_selection_active: bool,
    difficulty_selection_active: bool,
    last_move: Option<Move>,
    ai_move_receiver: Option<Receiver<Move>>,
    animating_move: Option<AnimationState>,
    last_frame_time: std::time::Instant,
}

struct AnimationState {
    from: Square,
    to: Square,
    piece: PieceType,
    color: Color,
    start_time: std::time::Instant,
    duration: std::time::Duration,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum GameMode {
    HumanVsHuman,
    HumanVsAI(Color, AIDifficulty), // AI plays this color with difficulty
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum AIDifficulty {
    Easy,   // 100ms
    Medium, // 500ms
    Hard,   // 2000ms
}

struct PromotionState {
    from: Square,
    to: Square,
    color: Color,
}

impl ChessGUI {
    async fn new(event_loop: &EventLoop<()>) -> Self {
        let window = Arc::new(
            WindowBuilder::new()
                .with_title("Chess")
                .with_inner_size(winit::dpi::LogicalSize::new(1000, 800))
                .build(event_loop)
                .expect("Failed to create window"),
        );

        let renderer = Renderer::new(window.clone()).await;
        let board = BoardRenderer::new(800.0);
        let game_state = GameState::new();
        let text_renderer =
            TextRenderer::new(&renderer.device, &renderer.queue, renderer.config.format);

        Self {
            window,
            renderer,
            board,
            text_renderer: Some(text_renderer),
            game_state,
            mouse_position: PhysicalPosition::new(0.0, 0.0),
            selected_square: None,
            valid_moves: Vec::new(),
            promotion_pending: None,
            game_mode: GameMode::HumanVsHuman,
            move_history: Vec::new(),
            ai_thinking: false,
            mode_selection_active: true,
            difficulty_selection_active: false,
            last_move: None,
            ai_move_receiver: None,
            animating_move: None,
            last_frame_time: std::time::Instant::now(),
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
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::CursorMoved { position, .. },
                } if window_id == app.window.id() => {
                    app.mouse_position = position;
                }
                Event::WindowEvent {
                    window_id,
                    event:
                        WindowEvent::MouseInput {
                            state: ElementState::Pressed,
                            button: MouseButton::Left,
                            ..
                        },
                } if window_id == app.window.id() => {
                    handle_mouse_click(&mut app);
                }
                Event::AboutToWait => {
                    // Update animation progress
                    let now = std::time::Instant::now();
                    let needs_redraw = app.animating_move.is_some();

                    if let Some(anim) = &app.animating_move {
                        let elapsed = now.duration_since(anim.start_time);
                        if elapsed >= anim.duration {
                            // Animation complete
                            app.animating_move = None;
                            update_display(&mut app);
                        }
                    }

                    // Check for AI move completion
                    if let Some(receiver) = &app.ai_move_receiver {
                        if let Ok(ai_move) = receiver.try_recv() {
                            // Start animation for AI move
                            if let Some(piece) = app.game_state.board.piece_at(ai_move.from) {
                                app.animating_move = Some(AnimationState {
                                    from: ai_move.from,
                                    to: ai_move.to,
                                    piece: piece.piece_type,
                                    color: piece.color,
                                    start_time: now,
                                    duration: std::time::Duration::from_millis(300),
                                });
                            }

                            // Apply AI move
                            let move_notation = format_move(&app.game_state, ai_move);
                            app.game_state = app.game_state.apply_move(ai_move);
                            app.move_history.push(move_notation);
                            app.last_move = Some(ai_move);
                            app.ai_thinking = false;
                            app.ai_move_receiver = None;
                            update_display(&mut app);
                        }
                    }

                    app.last_frame_time = now;

                    // Request a redraw if animating or before waiting
                    if needs_redraw {
                        app.window.request_redraw();
                    } else {
                        app.window.request_redraw();
                    }
                }
                _ => {}
            }
        })
        .unwrap();
}

fn update_display(app: &mut ChessGUI) {
    // Update board selection state
    app.board
        .set_selection(app.selected_square, app.valid_moves.clone());
    app.board.set_last_move(app.last_move);

    // Update board vertices with highlights
    let mut all_vertices = app.board.generate_vertices().to_vec();

    // Add side panel background with gradient effect
    let panel_bg_color = [0.12, 0.12, 0.12, 1.0];
    let panel_bg_color2 = [0.08, 0.08, 0.08, 1.0];
    all_vertices.extend_from_slice(&[
        // Panel background (right side) with gradient
        Vertex {
            position: [0.6, -1.0],
            color: panel_bg_color,
        },
        Vertex {
            position: [1.0, -1.0],
            color: panel_bg_color2,
        },
        Vertex {
            position: [0.6, 1.0],
            color: panel_bg_color,
        },
        Vertex {
            position: [1.0, -1.0],
            color: panel_bg_color2,
        },
        Vertex {
            position: [1.0, 1.0],
            color: panel_bg_color2,
        },
        Vertex {
            position: [0.6, 1.0],
            color: panel_bg_color,
        },
    ]);

    // Add section dividers
    let divider_color = [0.3, 0.3, 0.3, 1.0];
    let divider_y1 = 0.5; // Between game mode and status
    let divider_y2 = 0.2; // Between status and move history

    // First divider
    all_vertices.extend_from_slice(&[
        Vertex {
            position: [0.62, divider_y1],
            color: divider_color,
        },
        Vertex {
            position: [0.98, divider_y1],
            color: divider_color,
        },
        Vertex {
            position: [0.62, divider_y1 - 0.005],
            color: divider_color,
        },
        Vertex {
            position: [0.98, divider_y1],
            color: divider_color,
        },
        Vertex {
            position: [0.98, divider_y1 - 0.005],
            color: divider_color,
        },
        Vertex {
            position: [0.62, divider_y1 - 0.005],
            color: divider_color,
        },
    ]);

    // Second divider
    all_vertices.extend_from_slice(&[
        Vertex {
            position: [0.62, divider_y2],
            color: divider_color,
        },
        Vertex {
            position: [0.98, divider_y2],
            color: divider_color,
        },
        Vertex {
            position: [0.62, divider_y2 - 0.005],
            color: divider_color,
        },
        Vertex {
            position: [0.98, divider_y2],
            color: divider_color,
        },
        Vertex {
            position: [0.98, divider_y2 - 0.005],
            color: divider_color,
        },
        Vertex {
            position: [0.62, divider_y2 - 0.005],
            color: divider_color,
        },
    ]);

    app.renderer.update_vertices(&all_vertices);
}

fn handle_mouse_click(app: &mut ChessGUI) {
    // Handle mode selection first
    if app.mode_selection_active {
        handle_mode_selection_click(app);
        return;
    }

    // Handle difficulty selection
    if app.difficulty_selection_active {
        handle_difficulty_selection_click(app);
        return;
    }

    // Handle game over click
    if is_game_over(&app.game_state) {
        handle_game_over_click(app);
        return;
    }

    // Don't allow moves if AI is thinking
    if app.ai_thinking {
        return;
    }
    // Handle promotion selection first
    if let Some(promo_state) = &app.promotion_pending {
        let board_size = 800.0;
        let x = app.mouse_position.x as f32;
        let y = app.mouse_position.y as f32;

        // Check if clicking on promotion selection area
        // We'll show 4 pieces horizontally centered on the promotion square
        let square_size = board_size / 8.0;
        let promo_col = promo_state.to.file().index() as f32;
        let promo_row = if promo_state.color == Color::White {
            0.0
        } else {
            7.0
        };

        let promo_x = promo_col * square_size;
        let promo_y = promo_row * square_size;

        // Check if within the promotion selection area (4 squares wide)
        if y >= promo_y
            && y < promo_y + square_size
            && x >= promo_x - 1.5 * square_size
            && x < promo_x + 2.5 * square_size
        {
            let selection_index = ((x - (promo_x - 1.5 * square_size)) / square_size) as usize;
            let promotion_piece = match selection_index {
                0 => Some(PieceType::Queen),
                1 => Some(PieceType::Rook),
                2 => Some(PieceType::Bishop),
                3 => Some(PieceType::Knight),
                _ => None,
            };

            if let Some(piece_type) = promotion_piece {
                let promo_state = app.promotion_pending.take().unwrap();

                // Start animation for promotion move
                app.animating_move = Some(AnimationState {
                    from: promo_state.from,
                    to: promo_state.to,
                    piece: piece_type, // Use the promoted piece type
                    color: promo_state.color,
                    start_time: std::time::Instant::now(),
                    duration: std::time::Duration::from_millis(300),
                });

                let promotion_move =
                    chess_core::Move::new_promotion(promo_state.from, promo_state.to, piece_type);
                let move_notation = format_move(&app.game_state, promotion_move);
                app.game_state = app.game_state.apply_move(promotion_move);
                app.move_history.push(move_notation);
                app.last_move = Some(promotion_move);
                app.selected_square = None;
                app.valid_moves.clear();
                update_display(app);

                // Trigger AI move if applicable
                trigger_ai_move(app);
            }
        }
        return;
    }

    // Convert mouse position to board coordinates
    let x = app.mouse_position.x as f32;
    let y = app.mouse_position.y as f32;

    // Get the square under the mouse
    if let Some((row, col)) = app.board.get_square_at(x, y) {
        // Convert board row/col to chess square
        // Note: board row 0 is at top, but chess rank 0 is at bottom
        let rank = 7 - row;
        if let (Some(file), Some(rank)) = (File::new(col as u8), Rank::new(rank as u8)) {
            let clicked_square = Square::new(file, rank);

            // If no piece selected yet
            if app.selected_square.is_none() {
                // Check if there's a piece at this square of the current player's color
                if let Some(piece) = app.game_state.board.piece_at(clicked_square) {
                    if piece.color == app.game_state.turn {
                        // Select this piece
                        app.selected_square = Some(clicked_square);
                        // Generate legal moves for this piece
                        // Generate legal moves for this piece
                        let all_moves = generate_legal_moves(&app.game_state);
                        app.valid_moves = all_moves
                            .iter()
                            .filter(|m| m.from == clicked_square)
                            .copied()
                            .collect();
                        update_display(app);
                    }
                }
            } else {
                // We have a selected piece
                let from_square = app.selected_square.unwrap();

                // Check if clicking on the same square (deselect)
                if clicked_square == from_square {
                    app.selected_square = None;
                    app.valid_moves.clear();
                    update_display(app);
                    return;
                }

                // Check if this is a valid move
                if let Some(chess_move) = app.valid_moves.iter().find(|m| m.to == clicked_square) {
                    let chess_move = *chess_move;

                    // Check if this is a pawn promotion move
                    if let Some(piece) = app.game_state.board.piece_at(from_square) {
                        if piece.piece_type == PieceType::Pawn {
                            let promotion_rank = if piece.color == Color::White {
                                Rank::EIGHTH
                            } else {
                                Rank::FIRST
                            };
                            if clicked_square.rank() == promotion_rank {
                                // Show promotion selection
                                app.promotion_pending = Some(PromotionState {
                                    from: from_square,
                                    to: clicked_square,
                                    color: piece.color,
                                });
                                update_display(app);
                                return;
                            }
                        }
                    }

                    // Start animation for the move
                    if let Some(piece) = app.game_state.board.piece_at(from_square) {
                        app.animating_move = Some(AnimationState {
                            from: from_square,
                            to: clicked_square,
                            piece: piece.piece_type,
                            color: piece.color,
                            start_time: std::time::Instant::now(),
                            duration: std::time::Duration::from_millis(300),
                        });
                    }

                    // Apply the move
                    let move_notation = format_move(&app.game_state, chess_move);
                    app.game_state = app.game_state.apply_move(chess_move);
                    app.move_history.push(move_notation);
                    app.last_move = Some(chess_move);
                    app.selected_square = None;
                    app.valid_moves.clear();
                    update_display(app);

                    // Trigger AI move if applicable
                    trigger_ai_move(app);
                } else {
                    // Check if selecting a different piece of the same color
                    if let Some(piece) = app.game_state.board.piece_at(clicked_square) {
                        if piece.color == app.game_state.turn {
                            app.selected_square = Some(clicked_square);
                            let all_moves = generate_legal_moves(&app.game_state);
                            app.valid_moves = all_moves
                                .iter()
                                .filter(|m| m.from == clicked_square)
                                .copied()
                                .collect();
                            update_display(app);
                        } else {
                            // Clicked on opponent piece, deselect
                            app.selected_square = None;
                            app.valid_moves.clear();
                            update_display(app);
                        }
                    } else {
                        // Clicked on empty square that's not a valid move, deselect
                        app.selected_square = None;
                        app.valid_moves.clear();
                        update_display(app);
                    }
                }
            }
        }
    }
}

fn render_frame(app: &mut ChessGUI) {
    match app.renderer.begin_frame() {
        Ok((output, view, mut encoder)) => {
            // Render mode selection screen if active
            if app.mode_selection_active {
                render_mode_selection(app, &mut encoder, &view);
                app.renderer.submit_frame(encoder, output);
                return;
            }

            // Render difficulty selection screen if active
            if app.difficulty_selection_active {
                render_difficulty_selection(app, &mut encoder, &view);
                app.renderer.submit_frame(encoder, output);
                return;
            }
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
                let board_pixel_size =
                    (window_size.width as f32 * 0.8).min(window_size.height as f32);
                let square_size = board_pixel_size / 8.0;

                // Collect all pieces to render
                let mut pieces = Vec::new();
                for rank in 0..8 {
                    for file in 0..8 {
                        if let (Some(f), Some(r)) = (File::new(file), Rank::new(rank)) {
                            let square = Square::new(f, r);

                            // Skip piece if it's being animated
                            if let Some(anim) = &app.animating_move {
                                if square == anim.from {
                                    continue; // Don't render at original position
                                }
                                if square == anim.to && anim.start_time.elapsed() < anim.duration {
                                    continue; // Don't render at destination yet
                                }
                            }

                            if let Some(piece) = app.game_state.board.piece_at(square) {
                                // Calculate piece position (center of square)
                                // Note: rank 0 is at the bottom in chess, but top in screen coords
                                let x = file as f32 * square_size + square_size / 2.0;
                                let y = (7 - rank) as f32 * square_size + square_size / 2.0;

                                // Convert to NDC (board takes left 80% of window)
                                let board_width = 1.6; // 80% of NDC width
                                let ndc_x = (x / board_pixel_size) * board_width - 1.0;
                                let ndc_y = 1.0 - (y / board_pixel_size) * 2.0;

                                pieces.push((piece.piece_type, piece.color, ndc_x, ndc_y));
                            }
                        }
                    }
                }

                // Add animated piece if any
                if let Some(anim) = &app.animating_move {
                    let elapsed = anim.start_time.elapsed();
                    if elapsed < anim.duration {
                        let progress = elapsed.as_secs_f32() / anim.duration.as_secs_f32();
                        let progress = progress.min(1.0);

                        // Smooth easing function (ease-in-out)
                        let t = if progress < 0.5 {
                            2.0 * progress * progress
                        } else {
                            1.0 - (-2.0 * progress + 2.0).powi(2) / 2.0
                        };

                        // Calculate interpolated position
                        let from_file = anim.from.file().index() as f32;
                        let from_rank = anim.from.rank().index() as f32;
                        let to_file = anim.to.file().index() as f32;
                        let to_rank = anim.to.rank().index() as f32;

                        let file_pos = from_file + (to_file - from_file) * t;
                        let rank_pos = from_rank + (to_rank - from_rank) * t;

                        let x = file_pos * square_size + square_size / 2.0;
                        let y = (7.0 - rank_pos) * square_size + square_size / 2.0;

                        // Convert to NDC
                        let board_width = 1.6;
                        let ndc_x = (x / board_pixel_size) * board_width - 1.0;
                        let ndc_y = 1.0 - (y / board_pixel_size) * 2.0;

                        pieces.push((anim.piece, anim.color, ndc_x, ndc_y));
                    }
                }

                // Prepare UI text
                let status_text = if app.ai_thinking {
                    "AI is thinking...".to_string()
                } else {
                    get_game_status_text(&app.game_state)
                };

                let ui_text = UiText {
                    game_mode: match app.game_mode {
                        GameMode::HumanVsHuman => "Human vs Human".to_string(),
                        GameMode::HumanVsAI(Color::White, diff) => {
                            format!("AI ({:?}) vs Human", diff)
                        }
                        GameMode::HumanVsAI(Color::Black, diff) => {
                            format!("Human vs AI ({:?})", diff)
                        }
                    },
                    status: status_text,
                    move_history: app.move_history.clone(),
                };

                text_renderer.prepare_pieces(
                    &app.renderer.device,
                    &app.renderer.queue,
                    &pieces,
                    square_size,
                    window_size.width as f32,
                    window_size.height as f32,
                    &ui_text,
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

            // Render promotion selection if pending
            if app.promotion_pending.is_some() {
                render_promotion_selection(app, &mut encoder, &view);
            }

            // Render game over overlay if game is finished
            if is_game_over(&app.game_state) {
                render_game_over_overlay(app, &mut encoder, &view);
            }

            app.renderer.submit_frame(encoder, output);
        }
        Err(wgpu::SurfaceError::Lost) => app.renderer.resize(app.window.inner_size()),
        Err(wgpu::SurfaceError::OutOfMemory) => std::process::exit(0),
        Err(e) => eprintln!("Render error: {:?}", e),
    }
}

fn render_promotion_selection(
    app: &mut ChessGUI,
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
) {
    let promo_state = app.promotion_pending.as_ref().unwrap();
    let window_size = app.window.inner_size();
    let board_pixel_size = (window_size.width as f32 * 0.8).min(window_size.height as f32);
    let square_size = board_pixel_size / 8.0;

    // Generate vertices for promotion overlay background
    let mut vertices = Vec::new();

    // Dark overlay over board area only
    vertices.extend_from_slice(&[
        Vertex {
            position: [-1.0, -1.0],
            color: [0.0, 0.0, 0.0, 0.7],
        },
        Vertex {
            position: [0.6, -1.0],
            color: [0.0, 0.0, 0.0, 0.7],
        },
        Vertex {
            position: [-1.0, 1.0],
            color: [0.0, 0.0, 0.0, 0.7],
        },
        Vertex {
            position: [0.6, -1.0],
            color: [0.0, 0.0, 0.0, 0.7],
        },
        Vertex {
            position: [0.6, 1.0],
            color: [0.0, 0.0, 0.0, 0.7],
        },
        Vertex {
            position: [-1.0, 1.0],
            color: [0.0, 0.0, 0.0, 0.7],
        },
    ]);

    // Light background for promotion choices
    let promo_col = promo_state.to.file().index() as f32;
    let promo_row = if promo_state.color == Color::White {
        0.0
    } else {
        7.0
    };

    for i in 0..4 {
        let x = (promo_col - 1.5 + i as f32) * square_size;
        let y = promo_row * square_size;

        let board_width = 1.6; // 80% of NDC width
        let ndc_x = (x / board_pixel_size) * board_width - 1.0;
        let ndc_y = 1.0 - (y / board_pixel_size) * 2.0;
        let ndc_x2 = ((x + square_size) / board_pixel_size) * board_width - 1.0;
        let ndc_y2 = 1.0 - ((y + square_size) / board_pixel_size) * 2.0;

        let color = [0.9, 0.9, 0.9, 1.0];

        vertices.extend_from_slice(&[
            Vertex {
                position: [ndc_x, ndc_y],
                color,
            },
            Vertex {
                position: [ndc_x2, ndc_y],
                color,
            },
            Vertex {
                position: [ndc_x, ndc_y2],
                color,
            },
            Vertex {
                position: [ndc_x2, ndc_y],
                color,
            },
            Vertex {
                position: [ndc_x2, ndc_y2],
                color,
            },
            Vertex {
                position: [ndc_x, ndc_y2],
                color,
            },
        ]);
    }

    // Create a temporary vertex buffer for the overlay
    let overlay_buffer =
        app.renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Promotion Overlay Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

    // Render the overlay
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Promotion Overlay Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
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

        render_pass.set_pipeline(&app.renderer.render_pipeline);
        render_pass.set_vertex_buffer(0, overlay_buffer.slice(..));
        render_pass.draw(0..vertices.len() as u32, 0..1);
    }

    // Render promotion piece choices using text renderer
    if let Some(text_renderer) = &mut app.text_renderer {
        let window_size = app.window.inner_size();
        let pieces = [
            (PieceType::Queen, promo_state.color),
            (PieceType::Rook, promo_state.color),
            (PieceType::Bishop, promo_state.color),
            (PieceType::Knight, promo_state.color),
        ];

        let mut piece_positions = Vec::new();
        for (i, (piece_type, color)) in pieces.iter().enumerate() {
            let x = (promo_col - 1.5 + i as f32) * square_size + square_size / 2.0;
            let y = promo_row * square_size + square_size / 2.0;

            let board_width = 1.6; // 80% of NDC width
            let ndc_x = (x / board_pixel_size) * board_width - 1.0;
            let ndc_y = 1.0 - (y / board_pixel_size) * 2.0;

            piece_positions.push((*piece_type, *color, ndc_x, ndc_y));
        }

        text_renderer.prepare_pieces(
            &app.renderer.device,
            &app.renderer.queue,
            &piece_positions,
            square_size,
            window_size.width as f32,
            window_size.height as f32,
            &UiText {
                game_mode: String::new(),
                status: String::new(),
                move_history: Vec::new(),
            }, // No UI text during promotion
        );

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Promotion Text Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
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
}

fn get_game_status_text(game_state: &GameState) -> String {
    if is_checkmate(game_state) {
        format!("{} wins by checkmate!", game_state.turn.opponent())
    } else if is_stalemate(game_state) {
        "Stalemate - Draw".to_string()
    } else if game_state.is_fifty_move_draw() {
        "Draw by fifty-move rule".to_string()
    } else if game_state.is_insufficient_material() {
        "Draw by insufficient material".to_string()
    } else if game_state.is_in_check() {
        format!("{} to move - CHECK!", game_state.turn)
    } else {
        format!("{} to move", game_state.turn)
    }
}

fn format_move(game_state: &GameState, chess_move: chess_core::Move) -> String {
    let piece = game_state.board.piece_at(chess_move.from).unwrap();
    let piece_symbol = match piece.piece_type {
        PieceType::King => "K",
        PieceType::Queen => "Q",
        PieceType::Rook => "R",
        PieceType::Bishop => "B",
        PieceType::Knight => "N",
        PieceType::Pawn => "",
    };

    let capture = if game_state.board.piece_at(chess_move.to).is_some() {
        "x"
    } else {
        ""
    };

    let move_number = game_state.fullmove_number;
    let color = if game_state.turn == Color::White {
        "."
    } else {
        "..."
    };

    format!(
        "{}{} {}{}{}{}",
        move_number,
        color,
        piece_symbol,
        capture,
        chess_move.to.file().to_char(),
        chess_move.to.rank().index() + 1
    )
}

fn handle_mode_selection_click(app: &mut ChessGUI) {
    let x = app.mouse_position.x as f32;
    let y = app.mouse_position.y as f32;
    let window_size = app.window.inner_size();

    // Convert to NDC
    let ndc_x = (x / window_size.width as f32) * 2.0 - 1.0;
    let ndc_y = 1.0 - (y / window_size.height as f32) * 2.0;

    // Check if clicking on one of the mode buttons
    // Buttons are centered at Y = 0.0
    if ndc_y >= -0.15 && ndc_y <= 0.15 {
        if ndc_x >= -0.5 && ndc_x <= -0.1 {
            // Human vs Human
            app.game_mode = GameMode::HumanVsHuman;
            app.mode_selection_active = false;
            update_display(app);
        } else if ndc_x >= 0.1 && ndc_x <= 0.5 {
            // Human vs AI - show difficulty selection
            app.mode_selection_active = false;
            app.difficulty_selection_active = true;
            update_display(app);
        }
    }
}

fn trigger_ai_move(app: &mut ChessGUI) {
    if let GameMode::HumanVsAI(ai_color, difficulty) = app.game_mode {
        if app.game_state.turn == ai_color && !is_game_over(&app.game_state) {
            app.ai_thinking = true;
            update_display(app);

            // Clone the game state for the AI thread
            let game_state = app.game_state.clone();
            let (tx, rx) = channel();
            app.ai_move_receiver = Some(rx);

            // Spawn thread for AI computation
            thread::spawn(move || {
                let time_limit = match difficulty {
                    AIDifficulty::Easy => 100,
                    AIDifficulty::Medium => 500,
                    AIDifficulty::Hard => 2000,
                };
                let mut ai_agent = MinimaxAgent::with_time_limit(time_limit);
                if let Some(ai_move) = ai_agent.best_move(&game_state) {
                    let _ = tx.send(ai_move);
                }
            });
        }
    }
}

fn is_game_over(game_state: &GameState) -> bool {
    is_checkmate(game_state)
        || is_stalemate(game_state)
        || game_state.is_fifty_move_draw()
        || game_state.is_insufficient_material()
}

fn handle_game_over_click(app: &mut ChessGUI) {
    let x = app.mouse_position.x as f32;
    let y = app.mouse_position.y as f32;
    let window_size = app.window.inner_size();

    // Convert to NDC
    let ndc_x = (x / window_size.width as f32) * 2.0 - 1.0;
    let ndc_y = 1.0 - (y / window_size.height as f32) * 2.0;

    // Check if clicking on the new game button
    // Button is centered at Y = -0.2
    if ndc_y >= -0.35 && ndc_y <= -0.05 && ndc_x >= -0.2 && ndc_x <= 0.2 {
        // Reset the game
        app.game_state = GameState::new();
        app.selected_square = None;
        app.valid_moves.clear();
        app.promotion_pending = None;
        app.move_history.clear();
        app.ai_thinking = false;
        app.last_move = None;
        app.ai_move_receiver = None;
        app.animating_move = None;

        // If playing against AI and AI plays white, trigger AI move
        if let GameMode::HumanVsAI(Color::White, _) = app.game_mode {
            trigger_ai_move(app);
        }

        update_display(app);
    }
}

fn render_game_over_overlay(
    app: &mut ChessGUI,
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
) {
    let mut vertices = Vec::new();

    // Semi-transparent overlay over entire screen
    vertices.extend_from_slice(&[
        Vertex {
            position: [-1.0, -1.0],
            color: [0.0, 0.0, 0.0, 0.8],
        },
        Vertex {
            position: [1.0, -1.0],
            color: [0.0, 0.0, 0.0, 0.8],
        },
        Vertex {
            position: [-1.0, 1.0],
            color: [0.0, 0.0, 0.0, 0.8],
        },
        Vertex {
            position: [1.0, -1.0],
            color: [0.0, 0.0, 0.0, 0.8],
        },
        Vertex {
            position: [1.0, 1.0],
            color: [0.0, 0.0, 0.0, 0.8],
        },
        Vertex {
            position: [-1.0, 1.0],
            color: [0.0, 0.0, 0.0, 0.8],
        },
    ]);

    // Result display background (centered box)
    let box_color = [0.2, 0.2, 0.2, 0.95];
    vertices.extend_from_slice(&[
        Vertex {
            position: [-0.4, -0.1],
            color: box_color,
        },
        Vertex {
            position: [0.4, -0.1],
            color: box_color,
        },
        Vertex {
            position: [-0.4, 0.3],
            color: box_color,
        },
        Vertex {
            position: [0.4, -0.1],
            color: box_color,
        },
        Vertex {
            position: [0.4, 0.3],
            color: box_color,
        },
        Vertex {
            position: [-0.4, 0.3],
            color: box_color,
        },
    ]);

    // New Game button
    let btn_color = [0.3, 0.5, 0.7, 1.0];
    vertices.extend_from_slice(&[
        Vertex {
            position: [-0.2, -0.35],
            color: btn_color,
        },
        Vertex {
            position: [0.2, -0.35],
            color: btn_color,
        },
        Vertex {
            position: [-0.2, -0.05],
            color: btn_color,
        },
        Vertex {
            position: [0.2, -0.35],
            color: btn_color,
        },
        Vertex {
            position: [0.2, -0.05],
            color: btn_color,
        },
        Vertex {
            position: [-0.2, -0.05],
            color: btn_color,
        },
    ]);

    // Create temporary buffer
    let overlay_buffer =
        app.renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Game Over Overlay Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

    // Render the overlay
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Game Over Overlay Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
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

        render_pass.set_pipeline(&app.renderer.render_pipeline);
        render_pass.set_vertex_buffer(0, overlay_buffer.slice(..));
        render_pass.draw(0..vertices.len() as u32, 0..1);
    }

    // Render text (game result and new game button)
    if let Some(text_renderer) = &mut app.text_renderer {
        let window_size = app.window.inner_size();

        // Get game result text
        let result_text = if is_checkmate(&app.game_state) {
            format!("{} wins by checkmate!", app.game_state.turn.opponent())
        } else if is_stalemate(&app.game_state) {
            "Stalemate - Draw".to_string()
        } else if app.game_state.is_fifty_move_draw() {
            "Draw by fifty-move rule".to_string()
        } else if app.game_state.is_insufficient_material() {
            "Draw by insufficient material".to_string()
        } else {
            "Game Over".to_string()
        };

        text_renderer.prepare_game_over(
            &app.renderer.device,
            &app.renderer.queue,
            window_size.width as f32,
            window_size.height as f32,
            &result_text,
        );

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Game Over Text Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
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
}

fn handle_difficulty_selection_click(app: &mut ChessGUI) {
    let x = app.mouse_position.x as f32;
    let y = app.mouse_position.y as f32;
    let window_size = app.window.inner_size();

    // Convert to NDC
    let ndc_x = (x / window_size.width as f32) * 2.0 - 1.0;
    let ndc_y = 1.0 - (y / window_size.height as f32) * 2.0;

    // Check if clicking on one of the difficulty buttons
    // Buttons are centered at Y = 0.0
    if ndc_y >= -0.15 && ndc_y <= 0.15 {
        if ndc_x >= -0.6 && ndc_x <= -0.2 {
            // Easy
            app.game_mode = GameMode::HumanVsAI(Color::Black, AIDifficulty::Easy);
            app.difficulty_selection_active = false;
            update_display(app);
        } else if ndc_x >= -0.2 && ndc_x <= 0.2 {
            // Medium
            app.game_mode = GameMode::HumanVsAI(Color::Black, AIDifficulty::Medium);
            app.difficulty_selection_active = false;
            update_display(app);
        } else if ndc_x >= 0.2 && ndc_x <= 0.6 {
            // Hard
            app.game_mode = GameMode::HumanVsAI(Color::Black, AIDifficulty::Hard);
            app.difficulty_selection_active = false;
            update_display(app);
        }
    }
}

fn render_difficulty_selection(
    app: &mut ChessGUI,
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
) {
    // Generate vertices for difficulty selection screen
    let mut vertices = Vec::new();

    // Background
    vertices.extend_from_slice(&[
        Vertex {
            position: [-1.0, -1.0],
            color: [0.15, 0.15, 0.15, 1.0],
        },
        Vertex {
            position: [1.0, -1.0],
            color: [0.15, 0.15, 0.15, 1.0],
        },
        Vertex {
            position: [-1.0, 1.0],
            color: [0.15, 0.15, 0.15, 1.0],
        },
        Vertex {
            position: [1.0, -1.0],
            color: [0.15, 0.15, 0.15, 1.0],
        },
        Vertex {
            position: [1.0, 1.0],
            color: [0.15, 0.15, 0.15, 1.0],
        },
        Vertex {
            position: [-1.0, 1.0],
            color: [0.15, 0.15, 0.15, 1.0],
        },
    ]);

    // Button 1: Easy
    let easy_color = [0.3, 0.7, 0.3, 1.0];
    vertices.extend_from_slice(&[
        Vertex {
            position: [-0.6, -0.15],
            color: easy_color,
        },
        Vertex {
            position: [-0.2, -0.15],
            color: easy_color,
        },
        Vertex {
            position: [-0.6, 0.15],
            color: easy_color,
        },
        Vertex {
            position: [-0.2, -0.15],
            color: easy_color,
        },
        Vertex {
            position: [-0.2, 0.15],
            color: easy_color,
        },
        Vertex {
            position: [-0.6, 0.15],
            color: easy_color,
        },
    ]);

    // Button 2: Medium
    let medium_color = [0.7, 0.7, 0.3, 1.0];
    vertices.extend_from_slice(&[
        Vertex {
            position: [-0.2, -0.15],
            color: medium_color,
        },
        Vertex {
            position: [0.2, -0.15],
            color: medium_color,
        },
        Vertex {
            position: [-0.2, 0.15],
            color: medium_color,
        },
        Vertex {
            position: [0.2, -0.15],
            color: medium_color,
        },
        Vertex {
            position: [0.2, 0.15],
            color: medium_color,
        },
        Vertex {
            position: [-0.2, 0.15],
            color: medium_color,
        },
    ]);

    // Button 3: Hard
    let hard_color = [0.7, 0.3, 0.3, 1.0];
    vertices.extend_from_slice(&[
        Vertex {
            position: [0.2, -0.15],
            color: hard_color,
        },
        Vertex {
            position: [0.6, -0.15],
            color: hard_color,
        },
        Vertex {
            position: [0.2, 0.15],
            color: hard_color,
        },
        Vertex {
            position: [0.6, -0.15],
            color: hard_color,
        },
        Vertex {
            position: [0.6, 0.15],
            color: hard_color,
        },
        Vertex {
            position: [0.2, 0.15],
            color: hard_color,
        },
    ]);

    // Create temporary buffer
    let difficulty_buffer =
        app.renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Difficulty Selection Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

    // Render the difficulty selection
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Difficulty Selection Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
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

        render_pass.set_pipeline(&app.renderer.render_pipeline);
        render_pass.set_vertex_buffer(0, difficulty_buffer.slice(..));
        render_pass.draw(0..vertices.len() as u32, 0..1);
    }

    // Render text labels
    if let Some(text_renderer) = &mut app.text_renderer {
        let window_size = app.window.inner_size();

        // Prepare difficulty selection text
        text_renderer.prepare_difficulty_selection(
            &app.renderer.device,
            &app.renderer.queue,
            window_size.width as f32,
            window_size.height as f32,
        );

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Difficulty Text Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
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
}

fn render_mode_selection(
    app: &mut ChessGUI,
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
) {
    // Generate vertices for mode selection screen
    let mut vertices = Vec::new();

    // Background
    vertices.extend_from_slice(&[
        Vertex {
            position: [-1.0, -1.0],
            color: [0.15, 0.15, 0.15, 1.0],
        },
        Vertex {
            position: [1.0, -1.0],
            color: [0.15, 0.15, 0.15, 1.0],
        },
        Vertex {
            position: [-1.0, 1.0],
            color: [0.15, 0.15, 0.15, 1.0],
        },
        Vertex {
            position: [1.0, -1.0],
            color: [0.15, 0.15, 0.15, 1.0],
        },
        Vertex {
            position: [1.0, 1.0],
            color: [0.15, 0.15, 0.15, 1.0],
        },
        Vertex {
            position: [-1.0, 1.0],
            color: [0.15, 0.15, 0.15, 1.0],
        },
    ]);

    // Button 1: Human vs Human
    let btn1_color = [0.3, 0.5, 0.7, 1.0];
    vertices.extend_from_slice(&[
        Vertex {
            position: [-0.5, -0.15],
            color: btn1_color,
        },
        Vertex {
            position: [-0.1, -0.15],
            color: btn1_color,
        },
        Vertex {
            position: [-0.5, 0.15],
            color: btn1_color,
        },
        Vertex {
            position: [-0.1, -0.15],
            color: btn1_color,
        },
        Vertex {
            position: [-0.1, 0.15],
            color: btn1_color,
        },
        Vertex {
            position: [-0.5, 0.15],
            color: btn1_color,
        },
    ]);

    // Button 2: Human vs AI
    let btn2_color = [0.7, 0.3, 0.3, 1.0];
    vertices.extend_from_slice(&[
        Vertex {
            position: [0.1, -0.15],
            color: btn2_color,
        },
        Vertex {
            position: [0.5, -0.15],
            color: btn2_color,
        },
        Vertex {
            position: [0.1, 0.15],
            color: btn2_color,
        },
        Vertex {
            position: [0.5, -0.15],
            color: btn2_color,
        },
        Vertex {
            position: [0.5, 0.15],
            color: btn2_color,
        },
        Vertex {
            position: [0.1, 0.15],
            color: btn2_color,
        },
    ]);

    // Create temporary buffer
    let mode_buffer = app
        .renderer
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mode Selection Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

    // Render the mode selection
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Mode Selection Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
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

        render_pass.set_pipeline(&app.renderer.render_pipeline);
        render_pass.set_vertex_buffer(0, mode_buffer.slice(..));
        render_pass.draw(0..vertices.len() as u32, 0..1);
    }

    // Render text labels
    if let Some(text_renderer) = &mut app.text_renderer {
        let window_size = app.window.inner_size();

        // Prepare mode selection text
        text_renderer.prepare_mode_selection(
            &app.renderer.device,
            &app.renderer.queue,
            window_size.width as f32,
            window_size.height as f32,
        );

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Mode Text Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
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
}
