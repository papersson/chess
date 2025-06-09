use crate::game_state::GameState;
use crate::types::{Color, File, Move, PieceType, Rank, Square};

/// A list of moves with a fixed capacity to avoid allocations.
pub struct MoveList {
    moves: [Move; 256], // Max possible moves in any position
    count: usize,
}

impl MoveList {
    /// Creates an empty move list.
    pub const fn new() -> Self {
        Self {
            moves: [Move::new(
                Square::from_index(0).unwrap(),
                Square::from_index(0).unwrap(),
            ); 256],
            count: 0,
        }
    }

    /// Adds a move to the list.
    pub fn push(&mut self, mv: Move) {
        debug_assert!(self.count < 256, "Move list overflow");
        self.moves[self.count] = mv;
        self.count += 1;
    }

    /// Returns the number of moves.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns an iterator over the moves.
    pub fn iter(&self) -> impl Iterator<Item = &Move> {
        self.moves[..self.count].iter()
    }

    /// Clears the move list.
    pub fn clear(&mut self) {
        self.count = 0;
    }
}

/// Generates all legal moves for the current position.
pub fn generate_legal_moves(state: &GameState) -> MoveList {
    let mut moves = generate_pseudo_legal_moves(state);
    filter_legal_moves(state, &mut moves);
    moves
}

/// Generates all pseudo-legal moves (not checking for king safety).
fn generate_pseudo_legal_moves(state: &GameState) -> MoveList {
    let mut moves = MoveList::new();
    let color = state.turn;

    // Generate moves for each piece type
    generate_pawn_moves(state, color, &mut moves);
    generate_knight_moves(state, color, &mut moves);
    generate_bishop_moves(state, color, &mut moves);
    generate_rook_moves(state, color, &mut moves);
    generate_queen_moves(state, color, &mut moves);
    generate_king_moves(state, color, &mut moves);
    generate_castling_moves(state, color, &mut moves);

    moves
}

/// Filters out moves that would leave the king in check.
fn filter_legal_moves(state: &GameState, moves: &mut MoveList) {
    let mut legal_moves = MoveList::new();

    for &mv in moves.iter() {
        let new_state = state.apply_move(mv);
        if !new_state.is_side_in_check(state.turn) {
            legal_moves.push(mv);
        }
    }

    *moves = legal_moves;
}

/// Generates pawn moves for the given color.
fn generate_pawn_moves(state: &GameState, color: Color, moves: &mut MoveList) {
    let pawns = state.board.bitboards.pieces(PieceType::Pawn, color);
    let direction = color.pawn_direction();
    let start_rank = color.pawn_rank();
    let promotion_rank = color.promotion_rank();

    for from_square in pawns.iter() {
        let from_rank = from_square.rank();
        let from_file = from_square.file();

        // Single push
        if let Some(to_rank) = from_rank.offset(direction) {
            let to_square = Square::new(from_file, to_rank);
            if state.board.array_board.is_empty(to_square) {
                if to_rank == promotion_rank {
                    // Generate promotion moves
                    moves.push(Move::new_promotion(
                        from_square,
                        to_square,
                        PieceType::Queen,
                    ));
                    moves.push(Move::new_promotion(from_square, to_square, PieceType::Rook));
                    moves.push(Move::new_promotion(
                        from_square,
                        to_square,
                        PieceType::Bishop,
                    ));
                    moves.push(Move::new_promotion(
                        from_square,
                        to_square,
                        PieceType::Knight,
                    ));
                } else {
                    moves.push(Move::new(from_square, to_square));
                }

                // Double push from starting position
                if from_rank == start_rank {
                    if let Some(double_rank) = to_rank.offset(direction) {
                        let double_square = Square::new(from_file, double_rank);
                        if state.board.array_board.is_empty(double_square) {
                            moves.push(Move::new(from_square, double_square));
                        }
                    }
                }
            }
        }

        // Captures
        if let Some(to_rank) = from_rank.offset(direction) {
            // Left capture
            if let Some(left_file) = from_file.offset(-1) {
                let capture_square = Square::new(left_file, to_rank);
                if state.board.array_board.is_enemy(capture_square, color) {
                    if to_rank == promotion_rank {
                        moves.push(Move::new_promotion(
                            from_square,
                            capture_square,
                            PieceType::Queen,
                        ));
                        moves.push(Move::new_promotion(
                            from_square,
                            capture_square,
                            PieceType::Rook,
                        ));
                        moves.push(Move::new_promotion(
                            from_square,
                            capture_square,
                            PieceType::Bishop,
                        ));
                        moves.push(Move::new_promotion(
                            from_square,
                            capture_square,
                            PieceType::Knight,
                        ));
                    } else {
                        moves.push(Move::new(from_square, capture_square));
                    }
                }
            }

            // Right capture
            if let Some(right_file) = from_file.offset(1) {
                let capture_square = Square::new(right_file, to_rank);
                if state.board.array_board.is_enemy(capture_square, color) {
                    if to_rank == promotion_rank {
                        moves.push(Move::new_promotion(
                            from_square,
                            capture_square,
                            PieceType::Queen,
                        ));
                        moves.push(Move::new_promotion(
                            from_square,
                            capture_square,
                            PieceType::Rook,
                        ));
                        moves.push(Move::new_promotion(
                            from_square,
                            capture_square,
                            PieceType::Bishop,
                        ));
                        moves.push(Move::new_promotion(
                            from_square,
                            capture_square,
                            PieceType::Knight,
                        ));
                    } else {
                        moves.push(Move::new(from_square, capture_square));
                    }
                }
            }
        }

        // En passant
        if let Some(ep_square) = state.en_passant {
            if let Some(ep_rank) = from_rank.offset(direction) {
                if ep_square.rank() == ep_rank {
                    let file_diff = (ep_square.file().index() as i8) - (from_file.index() as i8);
                    if file_diff.abs() == 1 {
                        moves.push(Move::new(from_square, ep_square));
                    }
                }
            }
        }
    }
}

/// Generates knight moves for the given color.
fn generate_knight_moves(state: &GameState, color: Color, moves: &mut MoveList) {
    const KNIGHT_DELTAS: [(i8, i8); 8] = [
        (-2, -1),
        (-2, 1),
        (-1, -2),
        (-1, 2),
        (1, -2),
        (1, 2),
        (2, -1),
        (2, 1),
    ];

    let knights = state.board.bitboards.pieces(PieceType::Knight, color);

    for from_square in knights.iter() {
        for &(df, dr) in &KNIGHT_DELTAS {
            if let Some(to_file) = from_square.file().offset(df) {
                if let Some(to_rank) = from_square.rank().offset(dr) {
                    let to_square = Square::new(to_file, to_rank);
                    if !state.board.array_board.is_color(to_square, color) {
                        moves.push(Move::new(from_square, to_square));
                    }
                }
            }
        }
    }
}

/// Generates sliding piece moves along a direction.
fn generate_sliding_moves(
    state: &GameState,
    from_square: Square,
    color: Color,
    directions: &[(i8, i8)],
    moves: &mut MoveList,
) {
    for &(df, dr) in directions {
        let mut current_file = from_square.file();
        let mut current_rank = from_square.rank();

        loop {
            current_file = match current_file.offset(df) {
                Some(f) => f,
                None => break,
            };
            current_rank = match current_rank.offset(dr) {
                Some(r) => r,
                None => break,
            };

            let to_square = Square::new(current_file, current_rank);

            if state.board.array_board.is_empty(to_square) {
                moves.push(Move::new(from_square, to_square));
            } else {
                if state.board.array_board.is_enemy(to_square, color) {
                    moves.push(Move::new(from_square, to_square));
                }
                break; // Can't move past any piece
            }
        }
    }
}

/// Generates bishop moves for the given color.
fn generate_bishop_moves(state: &GameState, color: Color, moves: &mut MoveList) {
    const DIAGONAL_DIRS: [(i8, i8); 4] = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
    let bishops = state.board.bitboards.pieces(PieceType::Bishop, color);

    for from_square in bishops.iter() {
        generate_sliding_moves(state, from_square, color, &DIAGONAL_DIRS, moves);
    }
}

/// Generates rook moves for the given color.
fn generate_rook_moves(state: &GameState, color: Color, moves: &mut MoveList) {
    const STRAIGHT_DIRS: [(i8, i8); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    let rooks = state.board.bitboards.pieces(PieceType::Rook, color);

    for from_square in rooks.iter() {
        generate_sliding_moves(state, from_square, color, &STRAIGHT_DIRS, moves);
    }
}

/// Generates queen moves for the given color.
fn generate_queen_moves(state: &GameState, color: Color, moves: &mut MoveList) {
    const ALL_DIRS: [(i8, i8); 8] = [
        (-1, -1),
        (-1, 0),
        (-1, 1),
        (0, -1),
        (0, 1),
        (1, -1),
        (1, 0),
        (1, 1),
    ];
    let queens = state.board.bitboards.pieces(PieceType::Queen, color);

    for from_square in queens.iter() {
        generate_sliding_moves(state, from_square, color, &ALL_DIRS, moves);
    }
}

/// Generates king moves for the given color (excluding castling).
fn generate_king_moves(state: &GameState, color: Color, moves: &mut MoveList) {
    const KING_DELTAS: [(i8, i8); 8] = [
        (-1, -1),
        (-1, 0),
        (-1, 1),
        (0, -1),
        (0, 1),
        (1, -1),
        (1, 0),
        (1, 1),
    ];

    let king = state.board.bitboards.pieces(PieceType::King, color);

    for from_square in king.iter() {
        for &(df, dr) in &KING_DELTAS {
            if let Some(to_file) = from_square.file().offset(df) {
                if let Some(to_rank) = from_square.rank().offset(dr) {
                    let to_square = Square::new(to_file, to_rank);
                    if !state.board.array_board.is_color(to_square, color) {
                        moves.push(Move::new(from_square, to_square));
                    }
                }
            }
        }
    }
}

/// Generates castling moves for the given color.
fn generate_castling_moves(state: &GameState, color: Color, moves: &mut MoveList) {
    let rights = state.castling.get(color);
    if !rights.any() {
        return;
    }

    let king_square = state.board.array_board.king_square(color);
    let back_rank = if color == Color::White {
        Rank::new(0).unwrap()
    } else {
        Rank::new(7).unwrap()
    };

    // Check if king is in check
    if state.is_attacked_by(king_square, color.opponent()) {
        return;
    }

    // Kingside castling
    if rights.kingside {
        let f1 = Square::new(File::new(5).unwrap(), back_rank);
        let g1 = Square::new(File::new(6).unwrap(), back_rank);

        if state.board.array_board.is_empty(f1) && state.board.array_board.is_empty(g1) {
            // Check if squares king passes through are not attacked
            if !state.is_attacked_by(f1, color.opponent())
                && !state.is_attacked_by(g1, color.opponent())
            {
                moves.push(Move::new(king_square, g1));
            }
        }
    }

    // Queenside castling
    if rights.queenside {
        let d1 = Square::new(File::new(3).unwrap(), back_rank);
        let c1 = Square::new(File::new(2).unwrap(), back_rank);
        let b1 = Square::new(File::new(1).unwrap(), back_rank);

        if state.board.array_board.is_empty(d1)
            && state.board.array_board.is_empty(c1)
            && state.board.array_board.is_empty(b1)
        {
            // Check if squares king passes through are not attacked
            if !state.is_attacked_by(d1, color.opponent())
                && !state.is_attacked_by(c1, color.opponent())
            {
                moves.push(Move::new(king_square, c1));
            }
        }
    }
}

/// Checks if the current position is checkmate.
pub fn is_checkmate(state: &GameState) -> bool {
    state.is_in_check() && generate_legal_moves(state).is_empty()
}

/// Checks if the current position is stalemate.
pub fn is_stalemate(state: &GameState) -> bool {
    !state.is_in_check() && generate_legal_moves(state).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::BitBoardSet;
    use crate::types::Piece;

    #[test]
    fn test_starting_position_moves() {
        let state = GameState::new();
        let moves = generate_legal_moves(&state);

        // Starting position has exactly 20 legal moves
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn test_pawn_promotion() {
        let mut state = GameState::empty();
        state.board.array_board.set_piece(
            Square::new(File::new(0).unwrap(), Rank::new(6).unwrap()), // a7
            Some(Piece::new(PieceType::Pawn, Color::White)),
        );
        state.board.array_board.set_piece(
            Square::new(File::new(0).unwrap(), Rank::new(0).unwrap()), // a1
            Some(Piece::new(PieceType::King, Color::White)),
        );
        state.board.array_board.set_piece(
            Square::new(File::new(7).unwrap(), Rank::new(7).unwrap()), // h8
            Some(Piece::new(PieceType::King, Color::Black)),
        );
        state.board.bitboards = BitBoardSet::from_board(&state.board.array_board);

        let moves = generate_legal_moves(&state);

        // Should have 4 promotion moves + king moves
        let pawn_moves: Vec<_> = moves
            .iter()
            .filter(|m| m.from == Square::new(File::new(0).unwrap(), Rank::new(6).unwrap()))
            .collect();
        assert_eq!(pawn_moves.len(), 4); // 4 promotion choices
    }
}
