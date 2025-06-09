use crate::types::{CastlingRights, Color, Piece, PieceType, Square};

/// Zobrist hashing for chess positions.
/// Uses pre-computed random numbers for each piece-square combination.
#[derive(Debug, Clone)]
pub struct ZobristKeys {
    /// Random values for each piece type, color, and square
    piece_square: [[[u64; 64]; 6]; 2],
    /// Random value for side to move (XOR when black to move)
    black_to_move: u64,
    /// Random values for castling rights
    castling: [u64; 16],
    /// Random values for en passant files
    en_passant: [u64; 8],
}

impl ZobristKeys {
    /// Creates a new set of Zobrist keys with deterministic random values.
    /// Uses a fixed seed for reproducibility.
    pub fn new() -> Self {
        // Use a simple linear congruential generator for deterministic randomness
        let mut rng = 0x123456789ABCDEFu64;
        let mut next_random = || {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            rng
        };

        let mut piece_square = [[[0u64; 64]; 6]; 2];
        for color in 0..2 {
            for piece_type in 0..6 {
                for square in 0..64 {
                    piece_square[color][piece_type][square] = next_random();
                }
            }
        }

        let black_to_move = next_random();

        let mut castling = [0u64; 16];
        for i in 0..16 {
            castling[i] = next_random();
        }

        let mut en_passant = [0u64; 8];
        for i in 0..8 {
            en_passant[i] = next_random();
        }

        Self {
            piece_square,
            black_to_move,
            castling,
            en_passant,
        }
    }

    /// Gets the Zobrist key for a piece on a square.
    pub fn piece_square_key(&self, piece: Piece, square: Square) -> u64 {
        let color_idx = piece.color as usize;
        let piece_idx = match piece.piece_type {
            PieceType::Pawn => 0,
            PieceType::Knight => 1,
            PieceType::Bishop => 2,
            PieceType::Rook => 3,
            PieceType::Queen => 4,
            PieceType::King => 5,
        };
        self.piece_square[color_idx][piece_idx][square.index() as usize]
    }

    /// Gets the Zobrist key for the side to move.
    pub fn side_to_move_key(&self, color: Color) -> u64 {
        match color {
            Color::White => 0,
            Color::Black => self.black_to_move,
        }
    }

    /// Gets the Zobrist key for castling rights.
    pub fn castling_key(&self, rights: CastlingRights) -> u64 {
        let mut index = 0;
        if rights.white.kingside {
            index |= 1;
        }
        if rights.white.queenside {
            index |= 2;
        }
        if rights.black.kingside {
            index |= 4;
        }
        if rights.black.queenside {
            index |= 8;
        }
        self.castling[index]
    }

    /// Gets the Zobrist key for en passant square.
    pub fn en_passant_key(&self, square: Option<Square>) -> u64 {
        match square {
            Some(sq) => self.en_passant[sq.file().index() as usize],
            None => 0,
        }
    }
}

impl Default for ZobristKeys {
    fn default() -> Self {
        Self::new()
    }
}

/// Global Zobrist keys instance.
/// Initialized once and shared across the application.
pub static ZOBRIST: std::sync::LazyLock<ZobristKeys> = std::sync::LazyLock::new(ZobristKeys::new);
