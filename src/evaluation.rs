use crate::game_state::GameState;
use crate::types::*;

/// Evaluates a chess position from the perspective of the side to move.
/// Returns a score in centipawns where positive values favor the side to move.
pub fn evaluate(state: &GameState) -> i32 {
    let white_eval = evaluate_color(state, Color::White);
    let black_eval = evaluate_color(state, Color::Black);

    let raw_eval = white_eval - black_eval;

    // Return from perspective of side to move
    match state.turn {
        Color::White => raw_eval,
        Color::Black => -raw_eval,
    }
}

/// Evaluates a position from White's perspective.
/// Positive scores favor White, negative favor Black.
pub fn evaluate_absolute(state: &GameState) -> i32 {
    let white_eval = evaluate_color(state, Color::White);
    let black_eval = evaluate_color(state, Color::Black);

    white_eval - black_eval
}

/// Evaluates all factors for a single color.
fn evaluate_color(state: &GameState, color: Color) -> i32 {
    let mut score = 0;

    // Material evaluation
    score += evaluate_material(state, color);

    // Positional evaluation
    score += evaluate_position(state, color);

    score
}

/// Counts material value for a color.
fn evaluate_material(state: &GameState, color: Color) -> i32 {
    let mut material = 0;

    for i in 0..64 {
        if let Some(square) = Square::from_index(i) {
            if let Some(piece) = state.board.piece_at(square) {
                if piece.color == color {
                    material += i32::from(piece.piece_type.value());
                }
            }
        }
    }

    material
}

/// Evaluates positional factors for a color.
fn evaluate_position(state: &GameState, color: Color) -> i32 {
    let mut score = 0;

    // Center control bonus
    score += evaluate_center_control(state, color);

    // Piece-specific positional bonuses
    score += evaluate_piece_positions(state, color);

    score
}

/// Awards bonuses for controlling central squares.
fn evaluate_center_control(state: &GameState, color: Color) -> i32 {
    let mut score = 0;

    // Central squares (e4, d4, e5, d5)
    const CENTER_SQUARES: [u8; 4] = [27, 28, 35, 36]; // d4, e4, d5, e5

    // Extended center (c3-f3, c4-f4, c5-f5, c6-f6)
    const EXTENDED_CENTER: [u8; 16] = [
        18, 19, 20, 21, // c3, d3, e3, f3
        26, 27, 28, 29, // c4, d4, e4, f4
        34, 35, 36, 37, // c5, d5, e5, f5
        42, 43, 44, 45, // c6, d6, e6, f6
    ];

    // Bonus for pieces in the center
    for &idx in &CENTER_SQUARES {
        if let Some(square) = Square::from_index(idx) {
            if let Some(piece) = state.board.piece_at(square) {
                if piece.color == color {
                    score += match piece.piece_type {
                        PieceType::Pawn => 15,
                        PieceType::Knight => 20,
                        PieceType::Bishop => 10,
                        _ => 5,
                    };
                }
            }
        }
    }

    // Smaller bonus for extended center
    for &idx in &EXTENDED_CENTER {
        if let Some(square) = Square::from_index(idx) {
            if let Some(piece) = state.board.piece_at(square) {
                if piece.color == color && piece.piece_type == PieceType::Pawn {
                    score += 5;
                }
            }
        }
    }

    score
}

/// Piece-square tables for positional evaluation.
fn evaluate_piece_positions(state: &GameState, color: Color) -> i32 {
    let mut score = 0;

    for i in 0..64 {
        if let Some(square) = Square::from_index(i) {
            if let Some(piece) = state.board.piece_at(square) {
                if piece.color == color {
                    score += piece_square_value(piece.piece_type, square, color);
                }
            }
        }
    }

    score
}

/// Returns positional value for a piece on a given square.
fn piece_square_value(piece_type: PieceType, square: Square, color: Color) -> i32 {
    let rank = square.rank().index();
    let file = square.file().index();

    // Mirror the rank for black pieces
    let rank_idx = match color {
        Color::White => rank,
        Color::Black => 7 - rank,
    };

    match piece_type {
        PieceType::Pawn => PAWN_TABLE[rank_idx as usize][file as usize],
        PieceType::Knight => KNIGHT_TABLE[rank_idx as usize][file as usize],
        PieceType::Bishop => BISHOP_TABLE[rank_idx as usize][file as usize],
        PieceType::Rook => ROOK_TABLE[rank_idx as usize][file as usize],
        PieceType::Queen => QUEEN_TABLE[rank_idx as usize][file as usize],
        PieceType::King => {
            // Simple king safety: prefer corners in middlegame
            // TODO: Separate endgame king table
            KING_TABLE[rank_idx as usize][file as usize]
        }
    }
}

// Piece-square tables (from White's perspective, rank 0 = 1st rank)
// Values are in centipawns

const PAWN_TABLE: [[i32; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],         // 1st rank
    [50, 50, 50, 50, 50, 50, 50, 50], // 2nd rank
    [10, 10, 20, 30, 30, 20, 10, 10], // 3rd rank
    [5, 5, 10, 25, 25, 10, 5, 5],     // 4th rank
    [0, 0, 0, 20, 20, 0, 0, 0],       // 5th rank
    [5, -5, -10, 0, 0, -10, -5, 5],   // 6th rank
    [5, 10, 10, -20, -20, 10, 10, 5], // 7th rank
    [0, 0, 0, 0, 0, 0, 0, 0],         // 8th rank
];

const KNIGHT_TABLE: [[i32; 8]; 8] = [
    [-50, -40, -30, -30, -30, -30, -40, -50],
    [-40, -20, 0, 0, 0, 0, -20, -40],
    [-30, 0, 10, 15, 15, 10, 0, -30],
    [-30, 5, 15, 20, 20, 15, 5, -30],
    [-30, 0, 15, 20, 20, 15, 0, -30],
    [-30, 5, 10, 15, 15, 10, 5, -30],
    [-40, -20, 0, 5, 5, 0, -20, -40],
    [-50, -40, -30, -30, -30, -30, -40, -50],
];

const BISHOP_TABLE: [[i32; 8]; 8] = [
    [-20, -10, -10, -10, -10, -10, -10, -20],
    [-10, 0, 0, 0, 0, 0, 0, -10],
    [-10, 0, 5, 10, 10, 5, 0, -10],
    [-10, 5, 5, 10, 10, 5, 5, -10],
    [-10, 0, 10, 10, 10, 10, 0, -10],
    [-10, 10, 10, 10, 10, 10, 10, -10],
    [-10, 5, 0, 0, 0, 0, 5, -10],
    [-20, -10, -10, -10, -10, -10, -10, -20],
];

const ROOK_TABLE: [[i32; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [5, 10, 10, 10, 10, 10, 10, 5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [0, 0, 0, 5, 5, 0, 0, 0],
];

const QUEEN_TABLE: [[i32; 8]; 8] = [
    [-20, -10, -10, -5, -5, -10, -10, -20],
    [-10, 0, 0, 0, 0, 0, 0, -10],
    [-10, 0, 5, 5, 5, 5, 0, -10],
    [-5, 0, 5, 5, 5, 5, 0, -5],
    [0, 0, 5, 5, 5, 5, 0, -5],
    [-10, 5, 5, 5, 5, 5, 0, -10],
    [-10, 0, 5, 0, 0, 0, 0, -10],
    [-20, -10, -10, -5, -5, -10, -10, -20],
];

const KING_TABLE: [[i32; 8]; 8] = [
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-20, -30, -30, -40, -40, -30, -30, -20],
    [-10, -20, -20, -20, -20, -20, -20, -10],
    [20, 20, 0, 0, 0, 0, 20, 20],
    [20, 30, 10, 0, 0, 10, 30, 20],
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{File, Rank, Square};

    #[test]
    fn test_material_count() {
        let state = GameState::new();

        // Starting position should have equal material
        let white_material = evaluate_material(&state, Color::White);
        let black_material = evaluate_material(&state, Color::Black);

        assert_eq!(white_material, black_material);

        // Each side starts with:
        // 8 pawns (100 each) = 800
        // 2 knights (320 each) = 640
        // 2 bishops (330 each) = 660
        // 2 rooks (500 each) = 1000
        // 1 queen (900) = 900
        // Total = 4000
        assert_eq!(white_material, 4000);
    }

    #[test]
    fn test_starting_position_evaluation() {
        let state = GameState::new();

        // Starting position should be roughly equal
        let eval = evaluate_absolute(&state);

        // Allow small imbalance due to tempo
        assert!(eval.abs() < 50, "Starting position eval: {}", eval);
    }

    #[test]
    fn test_perspective_evaluation() {
        let state = GameState::new();

        // Evaluation from white's perspective
        let white_eval = evaluate(&state);

        // Same position from black's perspective
        let mut black_state = state.clone();
        black_state.turn = Color::Black;
        let black_eval = evaluate(&black_state);

        // Should be opposite values
        assert_eq!(white_eval, -black_eval);
    }

    #[test]
    fn test_material_advantage() {
        let mut state = GameState::empty();

        // White has a queen, Black has a rook
        let e4 = Square::new(File::new(4).unwrap(), Rank::new(3).unwrap());
        let e5 = Square::new(File::new(4).unwrap(), Rank::new(4).unwrap());
        let e1 = Square::new(File::new(4).unwrap(), Rank::new(0).unwrap());
        let e8 = Square::new(File::new(4).unwrap(), Rank::new(7).unwrap());

        state
            .board
            .set_square(e4, Some(Piece::new(PieceType::Queen, Color::White)));
        state
            .board
            .set_square(e5, Some(Piece::new(PieceType::Rook, Color::Black)));
        state
            .board
            .set_square(e1, Some(Piece::new(PieceType::King, Color::White)));
        state
            .board
            .set_square(e8, Some(Piece::new(PieceType::King, Color::Black)));

        let eval = evaluate_absolute(&state);

        // White should be ahead by 400 centipawns (900 - 500)
        // Plus some positional factors
        assert!(eval > 350, "White material advantage eval: {}", eval);
        assert!(eval < 450, "White material advantage eval: {}", eval);
    }

    #[test]
    fn test_center_control() {
        // Test with a more straightforward position
        let fen = "8/8/8/3p4/4P3/8/8/8 w - - 0 1";
        let mut state = GameState::from_fen(fen).unwrap();

        // Add kings to make it a legal position
        state.board.set_square(
            Square::new(File::new(0).unwrap(), Rank::new(0).unwrap()),
            Some(Piece::new(PieceType::King, Color::White)),
        );
        state.board.set_square(
            Square::new(File::new(7).unwrap(), Rank::new(7).unwrap()),
            Some(Piece::new(PieceType::King, Color::Black)),
        );

        let eval = evaluate_absolute(&state);

        // Both sides have a center pawn, position should be roughly equal
        // Small differences due to piece-square tables are acceptable
        assert!(eval.abs() < 30, "Center pawns eval: {}", eval);
    }

    #[test]
    fn test_known_position() {
        // Test a position where White is up a knight
        let fen = "rnbqkb1r/pppppppp/5n2/8/8/5N2/PPPPPPPP/RNBQKB1R w KQkq - 0 1";
        let state = GameState::from_fen(fen).unwrap();

        let eval = evaluate_absolute(&state);

        // Position should be roughly equal (both sides have developed knights)
        assert!(
            eval.abs() < 50,
            "Symmetric knight development eval: {}",
            eval
        );
    }

    #[test]
    fn test_endgame_position() {
        // K+Q vs K+R endgame
        let fen = "4k3/8/8/8/8/8/4Q3/4K3 w - - 0 1";
        let state = GameState::from_fen(fen).unwrap();

        let white_material = evaluate_material(&state, Color::White);
        let black_material = evaluate_material(&state, Color::Black);

        assert_eq!(white_material, 900); // Queen
        assert_eq!(black_material, 0); // Just king

        let eval = evaluate_absolute(&state);
        assert!(eval > 800, "K+Q vs K eval: {}", eval);
    }
}
