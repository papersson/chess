use chess_core::{Color, PieceType};
use glyphon::{
    Attrs, Buffer, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea,
    TextAtlas, TextBounds, TextRenderer as GlyphonRenderer,
};
use std::collections::HashMap;
use wgpu::{Device, MultisampleState, Queue, TextureFormat};

pub struct UiText {
    pub game_mode: String,
    pub status: String,
    pub move_history: Vec<String>,
}

pub struct TextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    atlas: TextAtlas,
    renderer: GlyphonRenderer,
    // Store buffers for each piece position to avoid lifetime issues
    piece_buffers: HashMap<(i32, i32), Buffer>,
    // Store buffers for UI text sections
    game_mode_buffer: Option<Buffer>,
    status_buffer: Option<Buffer>,
    move_history_buffer: Option<Buffer>,
}

impl TextRenderer {
    pub fn new(device: &Device, queue: &Queue, format: TextureFormat) -> Self {
        let mut font_system = FontSystem::new();

        // Load the DejaVu Sans font
        let font_data = include_bytes!("../../assets/DejaVuSans.ttf");
        font_system
            .db_mut()
            .load_font_data(Vec::from(&font_data[..]));

        let swash_cache = SwashCache::new();
        let mut atlas = TextAtlas::new(device, queue, format);
        let renderer = GlyphonRenderer::new(&mut atlas, device, MultisampleState::default(), None);

        Self {
            font_system,
            swash_cache,
            atlas,
            renderer,
            piece_buffers: HashMap::new(),
            game_mode_buffer: None,
            status_buffer: None,
            move_history_buffer: None,
        }
    }

    pub fn get_piece_symbol(piece_type: PieceType, color: Color) -> &'static str {
        match (piece_type, color) {
            (PieceType::King, Color::White) => "♔",
            (PieceType::Queen, Color::White) => "♕",
            (PieceType::Rook, Color::White) => "♖",
            (PieceType::Bishop, Color::White) => "♗",
            (PieceType::Knight, Color::White) => "♘",
            (PieceType::Pawn, Color::White) => "♙",
            (PieceType::King, Color::Black) => "♚",
            (PieceType::Queen, Color::Black) => "♛",
            (PieceType::Rook, Color::Black) => "♜",
            (PieceType::Bishop, Color::Black) => "♝",
            (PieceType::Knight, Color::Black) => "♞",
            (PieceType::Pawn, Color::Black) => "♟",
        }
    }

    pub fn prepare_pieces(
        &mut self,
        device: &Device,
        queue: &Queue,
        pieces: &[(PieceType, Color, f32, f32)],
        square_size: f32,
        screen_width: f32,
        screen_height: f32,
        ui_text: &UiText,
    ) {
        // Clear previous buffers
        self.piece_buffers.clear();

        // Create buffers for each piece
        for &(piece_type, color, ndc_x, ndc_y) in pieces {
            let symbol = Self::get_piece_symbol(piece_type, color);

            // Convert from NDC to screen coordinates
            let screen_x = (ndc_x + 1.0) * screen_width / 2.0;
            let screen_y = (1.0 - ndc_y) * screen_height / 2.0;

            // Create a buffer for this piece
            let mut buffer = Buffer::new(
                &mut self.font_system,
                Metrics::new(square_size * 0.8, square_size),
            );
            buffer.set_size(&mut self.font_system, square_size, square_size);
            buffer.set_text(
                &mut self.font_system,
                symbol,
                Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
            );
            buffer.shape_until_scroll(&mut self.font_system);

            // Store buffer with a unique key based on position
            let key = (screen_x as i32, screen_y as i32);
            self.piece_buffers.insert(key, buffer);
        }

        // Prepare UI text sections
        // Game mode text
        {
            let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(20.0, 24.0));
            buffer.set_size(&mut self.font_system, screen_width * 0.2, 40.0);
            buffer.set_text(
                &mut self.font_system,
                &ui_text.game_mode,
                Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
            );
            buffer.shape_until_scroll(&mut self.font_system);
            self.game_mode_buffer = Some(buffer);
        }

        // Status text
        {
            let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(24.0, 28.0));
            buffer.set_size(&mut self.font_system, screen_width * 0.2, 40.0);
            buffer.set_text(
                &mut self.font_system,
                &ui_text.status,
                Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
            );
            buffer.shape_until_scroll(&mut self.font_system);
            self.status_buffer = Some(buffer);
        }

        // Move history text
        if !ui_text.move_history.is_empty() {
            let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(16.0, 20.0));
            buffer.set_size(
                &mut self.font_system,
                screen_width * 0.2,
                screen_height * 0.5,
            );
            let history_text = ui_text.move_history.join("\n");
            buffer.set_text(
                &mut self.font_system,
                &history_text,
                Attrs::new().family(Family::Monospace),
                Shaping::Advanced,
            );
            buffer.shape_until_scroll(&mut self.font_system);
            self.move_history_buffer = Some(buffer);
        }

        // Build text areas from stored buffers
        let mut text_areas = Vec::new();

        let panel_left = screen_width * 0.8 + 20.0; // Right side panel

        // Add game mode text area
        if let Some(buffer) = &self.game_mode_buffer {
            text_areas.push(TextArea {
                buffer,
                left: panel_left,
                top: 20.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: panel_left as i32,
                    top: 0,
                    right: screen_width as i32,
                    bottom: 80,
                },
                default_color: glyphon::Color::rgb(200, 200, 200),
            });
        }

        // Add status text area
        if let Some(buffer) = &self.status_buffer {
            text_areas.push(TextArea {
                buffer,
                left: panel_left,
                top: screen_height * 0.25 + 20.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: panel_left as i32,
                    top: (screen_height * 0.25) as i32,
                    right: screen_width as i32,
                    bottom: (screen_height * 0.4) as i32,
                },
                default_color: glyphon::Color::rgb(255, 255, 255),
            });
        }

        // Add move history text area
        if let Some(buffer) = &self.move_history_buffer {
            text_areas.push(TextArea {
                buffer,
                left: panel_left,
                top: screen_height * 0.4 + 20.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: panel_left as i32,
                    top: (screen_height * 0.4) as i32,
                    right: screen_width as i32,
                    bottom: screen_height as i32,
                },
                default_color: glyphon::Color::rgb(180, 180, 180),
            });
        }

        for ((screen_x, screen_y), buffer) in &self.piece_buffers {
            let screen_x = *screen_x as f32;
            let screen_y = *screen_y as f32;

            // Calculate bounds to center the piece
            let left = screen_x - square_size / 2.0;
            let top = screen_y - square_size / 2.0;

            // Determine piece color from the stored piece data
            let piece_color = if let Some(&(_, color, _, _)) = pieces.iter().find(|(_, _, x, y)| {
                let sx = (*x + 1.0) * screen_width / 2.0;
                let sy = (1.0 - *y) * screen_height / 2.0;
                (sx as i32, sy as i32) == (screen_x as i32, screen_y as i32)
            }) {
                match color {
                    Color::White => glyphon::Color::rgb(255, 255, 255), // White fill for white pieces
                    Color::Black => glyphon::Color::rgb(0, 0, 0), // Black fill for black pieces
                }
            } else {
                glyphon::Color::rgb(255, 255, 255) // Default to white
            };

            // For white pieces, we need to render multiple layers
            let is_white_piece = piece_color == glyphon::Color::rgb(255, 255, 255);
            if is_white_piece {
                // First add thick black outline
                for offset in &[
                    (2.0, 0.0),
                    (-2.0, 0.0),
                    (0.0, 2.0),
                    (0.0, -2.0),
                    (1.5, 1.5),
                    (-1.5, -1.5),
                    (1.5, -1.5),
                    (-1.5, 1.5),
                ] {
                    text_areas.push(TextArea {
                        buffer,
                        left: left + offset.0,
                        top: top + offset.1,
                        scale: 1.0,
                        bounds: TextBounds {
                            left: (left + offset.0) as i32,
                            top: (top + offset.1) as i32,
                            right: (left + square_size + offset.0) as i32,
                            bottom: (top + square_size + offset.1) as i32,
                        },
                        default_color: glyphon::Color::rgb(0, 0, 0),
                    });
                }

                // Then add white fill
                text_areas.push(TextArea {
                    buffer,
                    left,
                    top,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: left as i32,
                        top: top as i32,
                        right: (left + square_size) as i32,
                        bottom: (top + square_size) as i32,
                    },
                    default_color: glyphon::Color::rgb(255, 255, 255),
                });
            } else {
                // For black pieces, just add outline and piece
                text_areas.push(TextArea {
                    buffer,
                    left: left - 1.5,
                    top: top - 1.5,
                    scale: 1.02,
                    bounds: TextBounds {
                        left: (left - 1.5) as i32,
                        top: (top - 1.5) as i32,
                        right: (left + square_size + 1.5) as i32,
                        bottom: (top + square_size + 1.5) as i32,
                    },
                    default_color: glyphon::Color::rgb(0, 0, 0),
                });

                text_areas.push(TextArea {
                    buffer,
                    left,
                    top,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: left as i32,
                        top: top as i32,
                        right: (left + square_size) as i32,
                        bottom: (top + square_size) as i32,
                    },
                    default_color: piece_color,
                });
            }
        }

        self.renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                Resolution {
                    width: screen_width as u32,
                    height: screen_height as u32,
                },
                text_areas,
                &mut self.swash_cache,
            )
            .unwrap();
    }

    pub fn prepare_mode_selection(
        &mut self,
        device: &Device,
        queue: &Queue,
        screen_width: f32,
        screen_height: f32,
    ) {
        // Clear previous buffers
        self.piece_buffers.clear();
        self.game_mode_buffer = None;
        self.status_buffer = None;
        self.move_history_buffer = None;

        // Title
        {
            let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(48.0, 56.0));
            buffer.set_size(&mut self.font_system, screen_width, 100.0);
            buffer.set_text(
                &mut self.font_system,
                "Chess",
                Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
            );
            buffer.shape_until_scroll(&mut self.font_system);
            self.game_mode_buffer = Some(buffer);
        }

        // Subtitle
        {
            let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(24.0, 28.0));
            buffer.set_size(&mut self.font_system, screen_width, 50.0);
            buffer.set_text(
                &mut self.font_system,
                "Select Game Mode",
                Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
            );
            buffer.shape_until_scroll(&mut self.font_system);
            self.status_buffer = Some(buffer);
        }

        // Button 1 text
        {
            let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(20.0, 24.0));
            buffer.set_size(&mut self.font_system, 200.0, 50.0);
            buffer.set_text(
                &mut self.font_system,
                "Human vs Human",
                Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
            );
            buffer.shape_until_scroll(&mut self.font_system);

            let key = (0, 0); // Dummy key for button 1
            self.piece_buffers.insert(key, buffer);
        }

        // Button 2 text
        {
            let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(20.0, 24.0));
            buffer.set_size(&mut self.font_system, 200.0, 50.0);
            buffer.set_text(
                &mut self.font_system,
                "Human vs AI",
                Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
            );
            buffer.shape_until_scroll(&mut self.font_system);

            let key = (1, 0); // Dummy key for button 2
            self.piece_buffers.insert(key, buffer);
        }

        // Now build text areas from stored buffers
        let mut text_areas = Vec::new();

        // Title text area
        if let Some(buffer) = &self.game_mode_buffer {
            text_areas.push(TextArea {
                buffer,
                left: 0.0,
                top: screen_height * 0.2,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: (screen_height * 0.15) as i32,
                    right: screen_width as i32,
                    bottom: (screen_height * 0.35) as i32,
                },
                default_color: glyphon::Color::rgb(255, 255, 255),
            });
        }

        // Subtitle text area
        if let Some(buffer) = &self.status_buffer {
            text_areas.push(TextArea {
                buffer,
                left: 0.0,
                top: screen_height * 0.35,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: (screen_height * 0.3) as i32,
                    right: screen_width as i32,
                    bottom: (screen_height * 0.4) as i32,
                },
                default_color: glyphon::Color::rgb(200, 200, 200),
            });
        }

        // Button 1 text area
        if let Some(buffer) = self.piece_buffers.get(&(0, 0)) {
            text_areas.push(TextArea {
                buffer,
                left: screen_width * 0.25 - 100.0,
                top: screen_height * 0.5 - 12.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: (screen_width * 0.25 - 100.0) as i32,
                    top: (screen_height * 0.5 - 25.0) as i32,
                    right: (screen_width * 0.25 + 100.0) as i32,
                    bottom: (screen_height * 0.5 + 25.0) as i32,
                },
                default_color: glyphon::Color::rgb(255, 255, 255),
            });
        }

        // Button 2 text area
        if let Some(buffer) = self.piece_buffers.get(&(1, 0)) {
            text_areas.push(TextArea {
                buffer,
                left: screen_width * 0.75 - 100.0,
                top: screen_height * 0.5 - 12.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: (screen_width * 0.75 - 100.0) as i32,
                    top: (screen_height * 0.5 - 25.0) as i32,
                    right: (screen_width * 0.75 + 100.0) as i32,
                    bottom: (screen_height * 0.5 + 25.0) as i32,
                },
                default_color: glyphon::Color::rgb(255, 255, 255),
            });
        }

        self.renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                Resolution {
                    width: screen_width as u32,
                    height: screen_height as u32,
                },
                text_areas,
                &mut self.swash_cache,
            )
            .unwrap();
    }

    pub fn prepare_game_over(
        &mut self,
        device: &Device,
        queue: &Queue,
        screen_width: f32,
        screen_height: f32,
        result_text: &str,
    ) {
        // Clear previous buffers
        self.piece_buffers.clear();
        self.game_mode_buffer = None;
        self.status_buffer = None;
        self.move_history_buffer = None;

        // Result text (large, centered)
        {
            let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(36.0, 42.0));
            buffer.set_size(&mut self.font_system, screen_width, 100.0);
            buffer.set_text(
                &mut self.font_system,
                result_text,
                Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
            );
            buffer.shape_until_scroll(&mut self.font_system);
            self.game_mode_buffer = Some(buffer);
        }

        // New Game button text
        {
            let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(24.0, 28.0));
            buffer.set_size(&mut self.font_system, 200.0, 50.0);
            buffer.set_text(
                &mut self.font_system,
                "New Game",
                Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
            );
            buffer.shape_until_scroll(&mut self.font_system);
            self.status_buffer = Some(buffer);
        }

        // Now build text areas from stored buffers
        let mut text_areas = Vec::new();

        // Result text area
        if let Some(buffer) = &self.game_mode_buffer {
            text_areas.push(TextArea {
                buffer,
                left: 0.0,
                top: screen_height * 0.42,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: (screen_height * 0.38) as i32,
                    right: screen_width as i32,
                    bottom: (screen_height * 0.5) as i32,
                },
                default_color: glyphon::Color::rgb(255, 255, 255),
            });
        }

        // New Game button text area
        if let Some(buffer) = &self.status_buffer {
            text_areas.push(TextArea {
                buffer,
                left: screen_width * 0.5 - 100.0,
                top: screen_height * 0.575,
                scale: 1.0,
                bounds: TextBounds {
                    left: (screen_width * 0.5 - 100.0) as i32,
                    top: (screen_height * 0.55) as i32,
                    right: (screen_width * 0.5 + 100.0) as i32,
                    bottom: (screen_height * 0.625) as i32,
                },
                default_color: glyphon::Color::rgb(255, 255, 255),
            });
        }

        self.renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                Resolution {
                    width: screen_width as u32,
                    height: screen_height as u32,
                },
                text_areas,
                &mut self.swash_cache,
            )
            .unwrap();
    }

    pub fn prepare_difficulty_selection(
        &mut self,
        device: &Device,
        queue: &Queue,
        screen_width: f32,
        screen_height: f32,
    ) {
        // Clear previous buffers
        self.piece_buffers.clear();
        self.game_mode_buffer = None;
        self.status_buffer = None;
        self.move_history_buffer = None;

        // Title
        {
            let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(48.0, 56.0));
            buffer.set_size(&mut self.font_system, screen_width, 100.0);
            buffer.set_text(
                &mut self.font_system,
                "Select Difficulty",
                Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
            );
            buffer.shape_until_scroll(&mut self.font_system);
            self.game_mode_buffer = Some(buffer);
        }

        // Easy button text
        {
            let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(24.0, 28.0));
            buffer.set_size(&mut self.font_system, 200.0, 50.0);
            buffer.set_text(
                &mut self.font_system,
                "Easy",
                Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
            );
            buffer.shape_until_scroll(&mut self.font_system);

            let key = (0, 0); // Dummy key for easy button
            self.piece_buffers.insert(key, buffer);
        }

        // Medium button text
        {
            let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(24.0, 28.0));
            buffer.set_size(&mut self.font_system, 200.0, 50.0);
            buffer.set_text(
                &mut self.font_system,
                "Medium",
                Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
            );
            buffer.shape_until_scroll(&mut self.font_system);

            let key = (1, 0); // Dummy key for medium button
            self.piece_buffers.insert(key, buffer);
        }

        // Hard button text
        {
            let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(24.0, 28.0));
            buffer.set_size(&mut self.font_system, 200.0, 50.0);
            buffer.set_text(
                &mut self.font_system,
                "Hard",
                Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
            );
            buffer.shape_until_scroll(&mut self.font_system);

            let key = (2, 0); // Dummy key for hard button
            self.piece_buffers.insert(key, buffer);
        }

        // Now build text areas from stored buffers
        let mut text_areas = Vec::new();

        // Title text area
        if let Some(buffer) = &self.game_mode_buffer {
            text_areas.push(TextArea {
                buffer,
                left: 0.0,
                top: screen_height * 0.25,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: (screen_height * 0.2) as i32,
                    right: screen_width as i32,
                    bottom: (screen_height * 0.35) as i32,
                },
                default_color: glyphon::Color::rgb(255, 255, 255),
            });
        }

        // Easy button text area
        if let Some(buffer) = self.piece_buffers.get(&(0, 0)) {
            text_areas.push(TextArea {
                buffer,
                left: screen_width * 0.2 - 100.0,
                top: screen_height * 0.5 - 14.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: (screen_width * 0.2 - 100.0) as i32,
                    top: (screen_height * 0.5 - 25.0) as i32,
                    right: (screen_width * 0.2 + 100.0) as i32,
                    bottom: (screen_height * 0.5 + 25.0) as i32,
                },
                default_color: glyphon::Color::rgb(255, 255, 255),
            });
        }

        // Medium button text area
        if let Some(buffer) = self.piece_buffers.get(&(1, 0)) {
            text_areas.push(TextArea {
                buffer,
                left: screen_width * 0.5 - 100.0,
                top: screen_height * 0.5 - 14.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: (screen_width * 0.5 - 100.0) as i32,
                    top: (screen_height * 0.5 - 25.0) as i32,
                    right: (screen_width * 0.5 + 100.0) as i32,
                    bottom: (screen_height * 0.5 + 25.0) as i32,
                },
                default_color: glyphon::Color::rgb(255, 255, 255),
            });
        }

        // Hard button text area
        if let Some(buffer) = self.piece_buffers.get(&(2, 0)) {
            text_areas.push(TextArea {
                buffer,
                left: screen_width * 0.8 - 100.0,
                top: screen_height * 0.5 - 14.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: (screen_width * 0.8 - 100.0) as i32,
                    top: (screen_height * 0.5 - 25.0) as i32,
                    right: (screen_width * 0.8 + 100.0) as i32,
                    bottom: (screen_height * 0.5 + 25.0) as i32,
                },
                default_color: glyphon::Color::rgb(255, 255, 255),
            });
        }

        self.renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                Resolution {
                    width: screen_width as u32,
                    height: screen_height as u32,
                },
                text_areas,
                &mut self.swash_cache,
            )
            .unwrap();
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.renderer.render(&self.atlas, render_pass).unwrap();
    }
}
