use crate::renderer::Vertex;

pub struct BoardRenderer {
    vertices: Vec<Vertex>,
    light_color: [f32; 4],
    dark_color: [f32; 4],
    board_size: f32,
    square_size: f32,
}

impl BoardRenderer {
    pub fn new(board_size: f32) -> Self {
        let light_color = [0.93, 0.93, 0.82, 1.0]; // Light beige
        let dark_color = [0.54, 0.27, 0.07, 1.0]; // Dark brown
        let square_size = board_size / 8.0;

        Self {
            vertices: Vec::with_capacity(8 * 8 * 6), // 6 vertices per square
            light_color,
            dark_color,
            board_size,
            square_size,
        }
    }

    pub fn generate_vertices(&mut self) -> &[Vertex] {
        self.vertices.clear();

        for row in 0..8 {
            for col in 0..8 {
                let color = if (row + col) % 2 == 0 {
                    self.light_color
                } else {
                    self.dark_color
                };

                let x = col as f32 * self.square_size;
                let y = row as f32 * self.square_size;

                // Convert to normalized device coordinates [-1, 1]
                let ndc_x = (x / self.board_size) * 2.0 - 1.0;
                let ndc_y = 1.0 - (y / self.board_size) * 2.0; // Flip Y
                let ndc_x2 = ((x + self.square_size) / self.board_size) * 2.0 - 1.0;
                let ndc_y2 = 1.0 - ((y + self.square_size) / self.board_size) * 2.0;

                // Create two triangles for a square
                // Triangle 1
                self.vertices.push(Vertex {
                    position: [ndc_x, ndc_y],
                    color,
                });
                self.vertices.push(Vertex {
                    position: [ndc_x2, ndc_y],
                    color,
                });
                self.vertices.push(Vertex {
                    position: [ndc_x, ndc_y2],
                    color,
                });

                // Triangle 2
                self.vertices.push(Vertex {
                    position: [ndc_x2, ndc_y],
                    color,
                });
                self.vertices.push(Vertex {
                    position: [ndc_x2, ndc_y2],
                    color,
                });
                self.vertices.push(Vertex {
                    position: [ndc_x, ndc_y2],
                    color,
                });
            }
        }

        &self.vertices
    }

    pub fn get_square_at(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        if x < 0.0 || x >= self.board_size || y < 0.0 || y >= self.board_size {
            return None;
        }

        let col = (x / self.square_size) as usize;
        let row = (y / self.square_size) as usize;

        if col < 8 && row < 8 {
            Some((row, col))
        } else {
            None
        }
    }
}
