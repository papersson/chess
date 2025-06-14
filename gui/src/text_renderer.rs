use chess_core::{Color, PieceType};
use glyphon::{
    Attrs, Buffer, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea,
    TextAtlas, TextBounds, TextRenderer as GlyphonRenderer,
};
use std::collections::HashMap;
use wgpu::{Device, MultisampleState, Queue, TextureFormat};

pub struct TextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    atlas: TextAtlas,
    renderer: GlyphonRenderer,
    // Store buffers for each piece position to avoid lifetime issues
    piece_buffers: HashMap<(i32, i32), Buffer>,
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

        // Build text areas from stored buffers
        let mut text_areas = Vec::new();
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
                    Color::White => glyphon::Color::rgb(255, 255, 255),  // White fill for white pieces
                    Color::Black => glyphon::Color::rgb(0, 0, 0),        // Black fill for black pieces
                }
            } else {
                glyphon::Color::rgb(255, 255, 255)  // Default to white
            };

            // First add black outline (slightly larger)
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
            
            // Then add the actual piece on top with appropriate color
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
