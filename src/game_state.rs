use crate::board::*;
/// Complete game state including board, turn, castling rights, etc.
/// This module provides the main interface for chess game management.
use crate::types::*;

/// Complete state of a chess game, matching FEN components.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    /// The current board position
    pub board: BoardState,
    /// Which side is to move
    pub turn: Color,
    /// Castling rights for both sides
    pub castling: CastlingRights,
    /// En passant target square (if a pawn just made a double move)
    pub en_passant: Option<Square>,
    /// Half-move clock for 50-move rule
    pub halfmove_clock: u16,
    /// Full move number (incremented after Black's move)
    pub fullmove_number: u16,
}

impl GameState {
    /// Creates a new game in the starting position.
    pub fn new() -> Self {
        Self {
            board: BoardState::starting_position(),
            turn: Color::White,
            castling: CastlingRights::all(),
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    /// Creates an empty game state for testing.
    pub fn empty() -> Self {
        Self {
            board: BoardState::empty(),
            turn: Color::White,
            castling: CastlingRights::none(),
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    /// Returns true if the game is drawn by the 50-move rule.
    pub fn is_fifty_move_draw(&self) -> bool {
        self.halfmove_clock >= 100
    }

    /// Returns true if there is insufficient material to checkmate.
    pub fn is_insufficient_material(&self) -> bool {
        let white_material = self.count_material(Color::White);
        let black_material = self.count_material(Color::Black);

        // King vs King
        if white_material.is_bare_king() && black_material.is_bare_king() {
            return true;
        }

        // King and minor piece vs King
        if (white_material.is_king_and_minor() && black_material.is_bare_king())
            || (black_material.is_king_and_minor() && white_material.is_bare_king())
        {
            return true;
        }

        // King and two knights vs King (cannot force mate)
        if (white_material.is_king_and_two_knights() && black_material.is_bare_king())
            || (black_material.is_king_and_two_knights() && white_material.is_bare_king())
        {
            return true;
        }

        false
    }

    /// Counts material for the given color.
    fn count_material(&self, color: Color) -> MaterialCount {
        let mut count = MaterialCount::default();

        for i in 0..64 {
            if let Some(square) = Square::from_index(i) {
                if let Some(piece) = self.board.piece_at(square) {
                    if piece.color == color {
                        match piece.piece_type {
                            PieceType::Pawn => count.pawns += 1,
                            PieceType::Knight => count.knights += 1,
                            PieceType::Bishop => count.bishops += 1,
                            PieceType::Rook => count.rooks += 1,
                            PieceType::Queen => count.queens += 1,
                            PieceType::King => {} // King is always present
                        }
                    }
                }
            }
        }

        count
    }

    /// Applies a move to the game state, returning a new state.
    /// This does NOT check if the move is legal.
    pub fn apply_move(&self, mv: Move) -> Self {
        let mut new_state = self.clone();

        // Get the moving piece
        let piece = self
            .board
            .piece_at(mv.from)
            .expect("No piece at source square");

        // Handle castling
        if piece.piece_type == PieceType::King && mv.from.distance(mv.to) == 2 {
            new_state.apply_castle(mv);
        } else {
            // Normal move or capture
            let captured = new_state.board.move_piece(mv.from, mv.to);

            // Handle en passant capture
            if piece.piece_type == PieceType::Pawn && Some(mv.to) == self.en_passant {
                let capture_square = Square::new(mv.to.file(), mv.from.rank());
                new_state.board.array_board.set_piece(capture_square, None);
                // Update bitboards
                new_state.board.bitboards = BitBoardSet::from_board(&new_state.board.array_board);
            }

            // Handle promotion
            if let Some(promotion) = mv.promotion {
                new_state
                    .board
                    .array_board
                    .set_piece(mv.to, Some(Piece::new(promotion, piece.color)));
                // Update bitboards
                new_state.board.bitboards = BitBoardSet::from_board(&new_state.board.array_board);
            }

            // Update en passant square
            new_state.en_passant = None;
            if piece.piece_type == PieceType::Pawn && mv.from.distance(mv.to) == 2 {
                let ep_square = Square::new(
                    mv.from.file(),
                    Rank::new((mv.from.rank().index() + mv.to.rank().index()) / 2).unwrap(),
                );
                new_state.en_passant = Some(ep_square);
            }

            // Update halfmove clock
            if piece.piece_type == PieceType::Pawn || captured.is_some() {
                new_state.halfmove_clock = 0;
            } else {
                new_state.halfmove_clock += 1;
            }
        }

        // Update castling rights
        new_state.castling = self.castling.update_after_move(mv.from, mv.to);

        // Update turn and move number
        if self.turn == Color::Black {
            new_state.fullmove_number += 1;
        }
        new_state.turn = self.turn.opponent();

        new_state
    }

    /// Applies a castling move.
    fn apply_castle(&mut self, mv: Move) {
        let (rook_from, rook_to) = if mv.to.file().index() > mv.from.file().index() {
            // Kingside castling
            let rank = mv.from.rank();
            (
                Square::new(File::new(7).unwrap(), rank), // h-file
                Square::new(File::new(5).unwrap(), rank), // f-file
            )
        } else {
            // Queenside castling
            let rank = mv.from.rank();
            (
                Square::new(File::new(0).unwrap(), rank), // a-file
                Square::new(File::new(3).unwrap(), rank), // d-file
            )
        };

        // Move king
        self.board.move_piece(mv.from, mv.to);
        // Move rook
        self.board.move_piece(rook_from, rook_to);

        // Castling doesn't reset halfmove clock
        self.halfmove_clock += 1;
    }

    /// Returns the side to move.
    pub fn side_to_move(&self) -> Color {
        self.turn
    }

    /// Returns true if the given square is attacked by the given color.
    pub fn is_attacked_by(&self, square: Square, attacker: Color) -> bool {
        // Check pawn attacks
        if self.is_pawn_attacked(square, attacker) {
            return true;
        }

        // Check knight attacks
        if self.is_knight_attacked(square, attacker) {
            return true;
        }

        // Check sliding piece attacks (bishop, rook, queen)
        if self.is_slider_attacked(square, attacker) {
            return true;
        }

        // Check king attacks
        if self.is_king_attacked(square, attacker) {
            return true;
        }

        false
    }

    /// Returns true if the given square is attacked by enemy pawns.
    fn is_pawn_attacked(&self, square: Square, attacker: Color) -> bool {
        let pawn_attacks = match attacker {
            Color::White => {
                // White pawns attack diagonally upward
                let mut attacks = BitBoard::EMPTY;
                if let Some(left) = square.file().offset(-1) {
                    if let Some(down) = square.rank().offset(-1) {
                        attacks = attacks.set(Square::new(left, down));
                    }
                }
                if let Some(right) = square.file().offset(1) {
                    if let Some(down) = square.rank().offset(-1) {
                        attacks = attacks.set(Square::new(right, down));
                    }
                }
                attacks
            }
            Color::Black => {
                // Black pawns attack diagonally downward
                let mut attacks = BitBoard::EMPTY;
                if let Some(left) = square.file().offset(-1) {
                    if let Some(up) = square.rank().offset(1) {
                        attacks = attacks.set(Square::new(left, up));
                    }
                }
                if let Some(right) = square.file().offset(1) {
                    if let Some(up) = square.rank().offset(1) {
                        attacks = attacks.set(Square::new(right, up));
                    }
                }
                attacks
            }
        };

        let enemy_pawns = self.board.bitboards.pieces(PieceType::Pawn, attacker);
        !pawn_attacks.intersection(enemy_pawns).is_empty()
    }

    /// Returns true if the given square is attacked by enemy knights.
    fn is_knight_attacked(&self, square: Square, attacker: Color) -> bool {
        const KNIGHT_MOVES: [(i8, i8); 8] = [
            (-2, -1),
            (-2, 1),
            (-1, -2),
            (-1, 2),
            (1, -2),
            (1, 2),
            (2, -1),
            (2, 1),
        ];

        let mut knight_attacks = BitBoard::EMPTY;
        for &(df, dr) in &KNIGHT_MOVES {
            if let Some(file) = square.file().offset(df) {
                if let Some(rank) = square.rank().offset(dr) {
                    knight_attacks = knight_attacks.set(Square::new(file, rank));
                }
            }
        }

        let enemy_knights = self.board.bitboards.pieces(PieceType::Knight, attacker);
        !knight_attacks.intersection(enemy_knights).is_empty()
    }

    /// Returns true if the given square is attacked by enemy sliding pieces.
    fn is_slider_attacked(&self, square: Square, attacker: Color) -> bool {
        // Check diagonal attacks (bishop and queen)
        const DIAGONALS: [(i8, i8); 4] = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
        for &(df, dr) in &DIAGONALS {
            if self.is_attacked_along_ray(square, df, dr, attacker, true) {
                return true;
            }
        }

        // Check straight attacks (rook and queen)
        const STRAIGHTS: [(i8, i8); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        for &(df, dr) in &STRAIGHTS {
            if self.is_attacked_along_ray(square, df, dr, attacker, false) {
                return true;
            }
        }

        false
    }

    /// Checks if a square is attacked along a ray.
    fn is_attacked_along_ray(
        &self,
        square: Square,
        df: i8,
        dr: i8,
        attacker: Color,
        diagonal: bool,
    ) -> bool {
        let mut current_file = square.file();
        let mut current_rank = square.rank();

        loop {
            current_file = match current_file.offset(df) {
                Some(f) => f,
                None => break,
            };
            current_rank = match current_rank.offset(dr) {
                Some(r) => r,
                None => break,
            };

            let current_square = Square::new(current_file, current_rank);

            if let Some(piece) = self.board.piece_at(current_square) {
                if piece.color == attacker {
                    let is_attacking = if diagonal {
                        piece.piece_type == PieceType::Bishop
                            || piece.piece_type == PieceType::Queen
                    } else {
                        piece.piece_type == PieceType::Rook || piece.piece_type == PieceType::Queen
                    };

                    if is_attacking {
                        return true;
                    }
                }
                break; // Piece blocks the ray
            }
        }

        false
    }

    /// Returns true if the given square is attacked by the enemy king.
    fn is_king_attacked(&self, square: Square, attacker: Color) -> bool {
        const KING_MOVES: [(i8, i8); 8] = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];

        let mut king_attacks = BitBoard::EMPTY;
        for &(df, dr) in &KING_MOVES {
            if let Some(file) = square.file().offset(df) {
                if let Some(rank) = square.rank().offset(dr) {
                    king_attacks = king_attacks.set(Square::new(file, rank));
                }
            }
        }

        let enemy_king = self.board.bitboards.pieces(PieceType::King, attacker);
        !king_attacks.intersection(enemy_king).is_empty()
    }

    /// Returns true if the current side to move is in check.
    pub fn is_in_check(&self) -> bool {
        let king_square = self.board.array_board.king_square(self.turn);
        self.is_attacked_by(king_square, self.turn.opponent())
    }

    /// Returns true if the given side is in check.
    pub fn is_side_in_check(&self, color: Color) -> bool {
        let king_square = self.board.array_board.king_square(color);
        self.is_attacked_by(king_square, color.opponent())
    }
}

/// Helper struct for counting material.
#[derive(Default, Debug)]
struct MaterialCount {
    pawns: u8,
    knights: u8,
    bishops: u8,
    rooks: u8,
    queens: u8,
}

impl MaterialCount {
    fn is_bare_king(&self) -> bool {
        self.pawns == 0
            && self.knights == 0
            && self.bishops == 0
            && self.rooks == 0
            && self.queens == 0
    }

    fn is_king_and_minor(&self) -> bool {
        self.pawns == 0 && self.rooks == 0 && self.queens == 0 && (self.knights + self.bishops) == 1
    }

    fn is_king_and_two_knights(&self) -> bool {
        self.pawns == 0
            && self.bishops == 0
            && self.rooks == 0
            && self.queens == 0
            && self.knights == 2
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starting_position() {
        let state = GameState::new();
        assert_eq!(state.turn, Color::White);
        assert_eq!(state.castling, CastlingRights::all());
        assert!(state.en_passant.is_none());
        assert_eq!(state.halfmove_clock, 0);
        assert_eq!(state.fullmove_number, 1);
    }

    #[test]
    fn test_apply_pawn_move() {
        let state = GameState::new();
        let mv = Move::new(
            Square::from_index(12).unwrap(), // e2
            Square::from_index(28).unwrap(), // e4
        );

        let new_state = state.apply_move(mv);
        assert_eq!(new_state.turn, Color::Black);
        assert_eq!(new_state.en_passant, Some(Square::from_index(20).unwrap())); // e3
        assert_eq!(new_state.halfmove_clock, 0);
        assert_eq!(new_state.fullmove_number, 1);
    }

    #[test]
    fn test_is_attacked() {
        let mut state = GameState::empty();

        // Place a white rook on e4
        state.board.array_board.set_piece(
            Square::from_index(28).unwrap(),
            Some(Piece::new(PieceType::Rook, Color::White)),
        );
        state.board.bitboards = BitBoardSet::from_board(&state.board.array_board);

        // Check that e1, e8, a4, h4 are attacked
        assert!(state.is_attacked_by(Square::from_index(4).unwrap(), Color::White)); // e1
        assert!(state.is_attacked_by(Square::from_index(60).unwrap(), Color::White)); // e8
        assert!(state.is_attacked_by(Square::from_index(24).unwrap(), Color::White)); // a4
        assert!(state.is_attacked_by(Square::from_index(31).unwrap(), Color::White)); // h4

        // Check that diagonal squares are not attacked
        assert!(!state.is_attacked_by(Square::from_index(35).unwrap(), Color::White)); // d5
    }
}
