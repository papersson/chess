use crate::renderer::Vertex;
use chess_core::types::{Color, PieceType};

pub struct PieceRenderer {
    vertices: Vec<Vertex>,
}

impl PieceRenderer {
    pub fn new() -> Self {
        Self {
            vertices: Vec::with_capacity(32 * 6 * 20), // 32 pieces max, 6 vertices per quad, ~20 quads per piece
        }
    }

    pub fn generate_piece_vertices(
        &mut self,
        piece_type: PieceType,
        color: Color,
        x: f32,
        y: f32,
        size: f32,
    ) -> &[Vertex] {
        self.vertices.clear();

        let piece_color = match color {
            Color::White => [0.85, 0.85, 0.85, 1.0], // Light gray instead of near-white
            Color::Black => [0.15, 0.15, 0.15, 1.0], // Dark gray instead of near-black
        };

        // First draw black outline (slightly larger)
        let outline_color = [0.0, 0.0, 0.0, 1.0];
        let outline_scale = 1.1;

        // Draw outline first (slightly larger)
        match piece_type {
            PieceType::Pawn => self.draw_pawn(x, y, size * outline_scale, outline_color),
            PieceType::Knight => self.draw_knight(x, y, size * outline_scale, outline_color),
            PieceType::Bishop => self.draw_bishop(x, y, size * outline_scale, outline_color),
            PieceType::Rook => self.draw_rook(x, y, size * outline_scale, outline_color),
            PieceType::Queen => self.draw_queen(x, y, size * outline_scale, outline_color),
            PieceType::King => self.draw_king(x, y, size * outline_scale, outline_color),
        }

        // Then draw the piece on top
        match piece_type {
            PieceType::Pawn => self.draw_pawn(x, y, size, piece_color),
            PieceType::Knight => self.draw_knight(x, y, size, piece_color),
            PieceType::Bishop => self.draw_bishop(x, y, size, piece_color),
            PieceType::Rook => self.draw_rook(x, y, size, piece_color),
            PieceType::Queen => self.draw_queen(x, y, size, piece_color),
            PieceType::King => self.draw_king(x, y, size, piece_color),
        }

        &self.vertices
    }

    fn draw_pawn(&mut self, cx: f32, cy: f32, size: f32, color: [f32; 4]) {
        // Simple pawn shape: circle on top of cone
        let radius = size * 0.15;

        // Draw circle for head
        self.draw_circle(cx, cy - size * 0.25, radius, color, 8);

        // Draw cone body
        self.add_triangle(
            cx,
            cy - size * 0.15,
            cx - size * 0.25,
            cy + size * 0.3,
            cx + size * 0.25,
            cy + size * 0.3,
            color,
        );

        // Base
        self.add_quad(
            cx - size * 0.3,
            cy + size * 0.25,
            cx + size * 0.3,
            cy + size * 0.25,
            cx + size * 0.3,
            cy + size * 0.35,
            cx - size * 0.3,
            cy + size * 0.35,
            color,
        );
    }

    fn draw_knight(&mut self, cx: f32, cy: f32, size: f32, color: [f32; 4]) {
        // Simple knight shape: stylized horse head
        let scale = size * 0.4;

        // Main body
        self.add_quad(
            cx - scale * 0.6,
            cy - scale * 0.4,
            cx + scale * 0.2,
            cy - scale * 0.6,
            cx + scale * 0.4,
            cy + scale * 0.8,
            cx - scale * 0.8,
            cy + scale * 0.8,
            color,
        );

        // Ear
        self.add_triangle(
            cx - scale * 0.2,
            cy - scale * 0.6,
            cx + scale * 0.2,
            cy - scale * 0.6,
            cx,
            cy - scale,
            color,
        );
    }

    fn draw_bishop(&mut self, cx: f32, cy: f32, size: f32, color: [f32; 4]) {
        // Simple bishop shape: triangle with circle on top
        let radius = size * 0.15;

        // Draw circle for top
        self.draw_circle(cx, cy - size * 0.3, radius, color, 8);

        // Draw triangle for body
        self.add_triangle(
            cx,
            cy - size * 0.2,
            cx - size * 0.35,
            cy + size * 0.4,
            cx + size * 0.35,
            cy + size * 0.4,
            color,
        );
    }

    fn draw_rook(&mut self, cx: f32, cy: f32, size: f32, color: [f32; 4]) {
        // Simple rook shape: rectangle with battlements
        let width = size * 0.6;
        let height = size * 0.7;

        // Main tower
        self.add_quad(
            cx - width / 2.0,
            cy - height / 2.0 + size * 0.1,
            cx + width / 2.0,
            cy - height / 2.0 + size * 0.1,
            cx + width / 2.0,
            cy + height / 2.0,
            cx - width / 2.0,
            cy + height / 2.0,
            color,
        );

        // Battlements
        let battlement_width = width / 3.0;
        for i in 0..3 {
            let x = cx - width / 2.0 + i as f32 * battlement_width;
            self.add_quad(
                x,
                cy - height / 2.0,
                x + battlement_width * 0.7,
                cy - height / 2.0,
                x + battlement_width * 0.7,
                cy - height / 2.0 + size * 0.1,
                x,
                cy - height / 2.0 + size * 0.1,
                color,
            );
        }
    }

    fn draw_queen(&mut self, cx: f32, cy: f32, size: f32, color: [f32; 4]) {
        // Simple queen shape: multiple triangles forming a crown
        let base_y = cy + size * 0.4;
        let peak_y = cy - size * 0.4;

        // Central spike
        self.add_triangle(
            cx - size * 0.1,
            base_y,
            cx + size * 0.1,
            base_y,
            cx,
            peak_y,
            color,
        );

        // Side spikes
        for &offset in &[-0.3, 0.3] {
            self.add_triangle(
                cx + offset * size - size * 0.1,
                base_y,
                cx + offset * size + size * 0.1,
                base_y,
                cx + offset * size,
                peak_y + size * 0.1,
                color,
            );
        }

        // Base
        self.add_quad(
            cx - size * 0.4,
            base_y - size * 0.1,
            cx + size * 0.4,
            base_y - size * 0.1,
            cx + size * 0.4,
            base_y,
            cx - size * 0.4,
            base_y,
            color,
        );
    }

    fn draw_king(&mut self, cx: f32, cy: f32, size: f32, color: [f32; 4]) {
        // Simple king shape: rectangle with cross on top
        let width = size * 0.5;
        let height = size * 0.6;

        // Main body
        self.add_quad(
            cx - width / 2.0,
            cy - height / 2.0 + size * 0.2,
            cx + width / 2.0,
            cy - height / 2.0 + size * 0.2,
            cx + width / 2.0,
            cy + height / 2.0,
            cx - width / 2.0,
            cy + height / 2.0,
            color,
        );

        // Cross vertical
        let cross_width = size * 0.08;
        let cross_height = size * 0.3;
        self.add_quad(
            cx - cross_width / 2.0,
            cy - height / 2.0,
            cx + cross_width / 2.0,
            cy - height / 2.0,
            cx + cross_width / 2.0,
            cy - height / 2.0 + cross_height,
            cx - cross_width / 2.0,
            cy - height / 2.0 + cross_height,
            color,
        );

        // Cross horizontal
        let cross_arm = size * 0.2;
        self.add_quad(
            cx - cross_arm / 2.0,
            cy - height / 2.0 + cross_height * 0.4,
            cx + cross_arm / 2.0,
            cy - height / 2.0 + cross_height * 0.4,
            cx + cross_arm / 2.0,
            cy - height / 2.0 + cross_height * 0.6,
            cx - cross_arm / 2.0,
            cy - height / 2.0 + cross_height * 0.6,
            color,
        );
    }

    fn draw_circle(&mut self, cx: f32, cy: f32, radius: f32, color: [f32; 4], segments: u32) {
        let angle_step = std::f32::consts::PI * 2.0 / segments as f32;

        for i in 0..segments {
            let angle1 = i as f32 * angle_step;
            let angle2 = (i + 1) as f32 * angle_step;

            let x1 = cx + angle1.cos() * radius;
            let y1 = cy + angle1.sin() * radius;
            let x2 = cx + angle2.cos() * radius;
            let y2 = cy + angle2.sin() * radius;

            self.add_triangle(cx, cy, x1, y1, x2, y2, color);
        }
    }

    fn add_triangle(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        color: [f32; 4],
    ) {
        self.vertices.push(Vertex {
            position: [x1, y1],
            color,
        });
        self.vertices.push(Vertex {
            position: [x2, y2],
            color,
        });
        self.vertices.push(Vertex {
            position: [x3, y3],
            color,
        });
    }

    fn add_quad(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        x4: f32,
        y4: f32,
        color: [f32; 4],
    ) {
        // First triangle
        self.add_triangle(x1, y1, x2, y2, x3, y3, color);
        // Second triangle
        self.add_triangle(x1, y1, x3, y3, x4, y4, color);
    }
}
