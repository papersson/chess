use crate::game_state::GameState;
use crate::move_gen::generate_legal_moves;
use crate::types::{Move, PieceType};

/// Perft (performance test) results at each depth.
#[derive(Debug, Default)]
pub struct PerftResults {
    pub nodes: u64,
    pub captures: u64,
    pub en_passants: u64,
    pub castles: u64,
    pub promotions: u64,
    pub checks: u64,
    pub checkmates: u64,
}

impl PerftResults {
    /// Combines results from child nodes.
    pub fn add(&mut self, other: &Self) {
        self.nodes += other.nodes;
        self.captures += other.captures;
        self.en_passants += other.en_passants;
        self.castles += other.castles;
        self.promotions += other.promotions;
        self.checks += other.checks;
        self.checkmates += other.checkmates;
    }
}

/// Performs perft test to given depth and returns node count.
pub fn perft(state: &GameState, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = generate_legal_moves(state);

    if depth == 1 {
        return moves.len() as u64;
    }

    let mut nodes = 0;
    for mv in moves.iter() {
        let new_state = state.apply_move(*mv);
        nodes += perft(&new_state, depth - 1);
    }

    nodes
}

/// Performs detailed perft test with move breakdown.
pub fn perft_divide(state: &GameState, depth: u8) -> Vec<(Move, u64)> {
    let moves = generate_legal_moves(state);
    let mut results = Vec::new();

    for mv in moves.iter() {
        let new_state = state.apply_move(*mv);
        let nodes = if depth == 1 {
            1
        } else {
            perft(&new_state, depth - 1)
        };
        results.push((*mv, nodes));
    }

    results
}

/// Performs perft test with detailed statistics.
pub fn perft_detailed(state: &GameState, depth: u8) -> PerftResults {
    let mut results = PerftResults::default();

    if depth == 0 {
        results.nodes = 1;
        return results;
    }

    let moves = generate_legal_moves(state);

    for mv in moves.iter() {
        let new_state = state.apply_move(*mv);

        if depth == 1 {
            results.nodes += 1;

            // Classify move types
            let from_piece = state.board.piece_at(mv.from);
            let to_piece = state.board.piece_at(mv.to);

            // Capture detection
            if to_piece.is_some() {
                results.captures += 1;
            }

            // En passant detection
            if let Some(piece) = from_piece {
                if piece.piece_type == PieceType::Pawn {
                    // Check if it's a diagonal pawn move to empty square
                    if mv.from.file() != mv.to.file() && to_piece.is_none() {
                        results.en_passants += 1;
                        results.captures += 1; // En passant is also a capture
                    }
                }

                // Castle detection
                if piece.piece_type == PieceType::King && mv.from.distance(mv.to) == 2 {
                    results.castles += 1;
                }
            }

            // Promotion detection
            if mv.is_promotion() {
                results.promotions += 1;
            }
            if new_state.is_in_check() {
                results.checks += 1;
                if is_checkmate(&new_state) {
                    results.checkmates += 1;
                }
            }
        } else {
            let child_results = perft_detailed(&new_state, depth - 1);
            results.add(&child_results);
        }
    }

    results
}

/// Standard perft positions with expected results.
pub mod positions {

    /// Starting position perft values.
    pub const STARTING_POSITION: &[(u8, u64)] = &[
        (1, 20),
        (2, 400),
        (3, 8902),
        (4, 197_281),
        (5, 4_865_609),
        (6, 119_060_324),
    ];

    /// Position after 1.e4 (Kiwipete).
    pub const KIWIPETE: &str =
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    pub const KIWIPETE_PERFT: &[(u8, u64)] = &[
        (1, 48),
        (2, 2039),
        (3, 97_862),
        (4, 4_085_603),
        (5, 193_690_690),
    ];

    /// Position 3 from CPW.
    pub const POSITION_3: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
    pub const POSITION_3_PERFT: &[(u8, u64)] = &[
        (1, 14),
        (2, 191),
        (3, 2812),
        (4, 43238),
        (5, 674_624),
        (6, 11_030_083),
    ];

    /// Position 4 from CPW.
    pub const POSITION_4: &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
    pub const POSITION_4_PERFT: &[(u8, u64)] =
        &[(1, 6), (2, 264), (3, 9467), (4, 422_333), (5, 15_833_292)];

    /// Position 5 from CPW.
    pub const POSITION_5: &str = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
    pub const POSITION_5_PERFT: &[(u8, u64)] = &[
        (1, 44),
        (2, 1486),
        (3, 62_379),
        (4, 2_103_487),
        (5, 89_941_194),
    ];
}

// Import functions from move_gen that are needed
use crate::move_gen::is_checkmate;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perft_starting_position() {
        let state = GameState::new();

        // Only test depths 1-3 to avoid timeout
        let test_positions = &[(1, 20), (2, 400), (3, 8902)];

        for &(depth, expected) in test_positions {
            let result = perft(&state, depth);
            assert_eq!(
                result, expected,
                "Perft({}) failed: expected {}, got {}",
                depth, expected, result
            );
        }
    }

    #[test]
    fn test_perft_divide() {
        let state = GameState::new();
        let results = perft_divide(&state, 1);

        assert_eq!(results.len(), 20);
        assert_eq!(results.iter().map(|(_, n)| n).sum::<u64>(), 20);
    }
}
