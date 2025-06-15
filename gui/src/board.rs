use crate::renderer::Vertex;
use chess_core::{Move, Square};

pub struct BoardRenderer {
    vertices: Vec<Vertex>,
    light_color: [f32; 4],
    dark_color: [f32; 4],
    selected_color: [f32; 4],
    valid_move_color: [f32; 4],
    last_move_color: [f32; 4],
    board_size: f32,
    square_size: f32,
    selected_square: Option<Square>,
    valid_moves: Vec<Square>,
    last_move: Option<Move>,
}

impl BoardRenderer {
    pub fn new(board_size: f32) -> Self {
        let light_color = [0.93, 0.93, 0.82, 1.0]; // Light beige
        let dark_color = [0.54, 0.27, 0.07, 1.0]; // Dark brown
        let selected_color = [0.7, 0.7, 0.3, 1.0]; // Yellow highlight
        let valid_move_color = [0.3, 0.7, 0.3, 0.5]; // Semi-transparent green
        let last_move_color = [0.5, 0.3, 0.7, 0.3]; // Semi-transparent purple
        let square_size = board_size / 8.0;

        Self {
            vertices: Vec::with_capacity(8 * 8 * 6), // 6 vertices per square
            light_color,
            dark_color,
            selected_color,
            valid_move_color,
            last_move_color,
            board_size,
            square_size,
            selected_square: None,
            valid_moves: Vec::new(),
            last_move: None,
        }
    }

    pub fn set_selection(&mut self, selected: Option<Square>, valid_moves: Vec<Move>) {
        self.selected_square = selected;
        self.valid_moves = valid_moves.into_iter().map(|m| m.to).collect();
    }

    pub fn set_last_move(&mut self, last_move: Option<Move>) {
        self.last_move = last_move;
    }

    pub fn generate_vertices(&mut self) -> &[Vertex] {
        self.vertices.clear();

        for row in 0..8 {
            for col in 0..8 {
                // Convert to chess square for checking selection
                let rank = 7 - row;
                let square = if let (Some(file), Some(rank)) = (
                    chess_core::File::new(col as u8),
                    chess_core::Rank::new(rank as u8),
                ) {
                    Some(Square::new(file, rank))
                } else {
                    None
                };

                // Determine base color
                let mut color = if (row + col) % 2 == 0 {
                    self.light_color
                } else {
                    self.dark_color
                };

                // Apply selection highlight
                if let Some(sq) = square {
                    if Some(sq) == self.selected_square {
                        color = self.selected_color;
                    }
                }

                let x = col as f32 * self.square_size;
                let y = row as f32 * self.square_size;

                // Convert to normalized device coordinates [-1, 1]
                // Board takes up left 80% of window (from -1.0 to 0.6)
                let board_width = 1.6; // 80% of NDC width
                let ndc_x = (x / self.board_size) * board_width - 1.0;
                let ndc_y = 1.0 - (y / self.board_size) * 2.0; // Flip Y
                let ndc_x2 = ((x + self.square_size) / self.board_size) * board_width - 1.0;
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

        // Add semi-transparent overlay for last move
        if let Some(last_move) = self.last_move {
            for &square in &[last_move.from, last_move.to] {
                let col = square.file().index() as usize;
                let row = 7 - square.rank().index() as usize;

                let x = col as f32 * self.square_size;
                let y = row as f32 * self.square_size;

                let board_width = 1.6;
                let ndc_x = (x / self.board_size) * board_width - 1.0;
                let ndc_y = 1.0 - (y / self.board_size) * 2.0;
                let ndc_x2 = ((x + self.square_size) / self.board_size) * board_width - 1.0;
                let ndc_y2 = 1.0 - ((y + self.square_size) / self.board_size) * 2.0;

                let color = self.last_move_color;

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

        // Add semi-transparent overlays for valid moves
        for &valid_square in &self.valid_moves {
            let col = valid_square.file().index() as usize;
            let row = 7 - valid_square.rank().index() as usize; // Convert chess rank to board row

            let x = col as f32 * self.square_size;
            let y = row as f32 * self.square_size;

            // Convert to normalized device coordinates [-1, 1]
            // Board takes up left 80% of window (from -1.0 to 0.6)
            let board_width = 1.6; // 80% of NDC width
            let ndc_x = (x / self.board_size) * board_width - 1.0;
            let ndc_y = 1.0 - (y / self.board_size) * 2.0; // Flip Y
            let ndc_x2 = ((x + self.square_size) / self.board_size) * board_width - 1.0;
            let ndc_y2 = 1.0 - ((y + self.square_size) / self.board_size) * 2.0;

            let color = self.valid_move_color;

            // Create two triangles for the overlay
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
