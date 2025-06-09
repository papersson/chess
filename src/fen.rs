use crate::board::BoardState;
use crate::game_state::GameState;
use crate::types::{
    CastlingRights, Color, File, Piece, PieceType, Rank, SideCastlingRights, Square,
};
use std::fmt;

/// FEN (Forsyth-Edwards Notation) parsing and serialization.
/// Standard notation for describing chess positions.
/// FEN parsing error types.
#[derive(Debug, Clone, PartialEq)]
pub enum FenError {
    InvalidFormat(String),
    InvalidPiece(char),
    InvalidSquare(String),
    InvalidColor(String),
    InvalidCastling(String),
    InvalidEnPassant(String),
    InvalidNumber(String),
}

impl fmt::Display for FenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FenError::InvalidFormat(s) => write!(f, "Invalid FEN format: {s}"),
            FenError::InvalidPiece(c) => write!(f, "Invalid piece character: '{c}'"),
            FenError::InvalidSquare(s) => write!(f, "Invalid square: {s}"),
            FenError::InvalidColor(s) => write!(f, "Invalid color: {s}"),
            FenError::InvalidCastling(s) => write!(f, "Invalid castling rights: {s}"),
            FenError::InvalidEnPassant(s) => write!(f, "Invalid en passant square: {s}"),
            FenError::InvalidNumber(s) => write!(f, "Invalid number: {s}"),
        }
    }
}

impl std::error::Error for FenError {}

impl GameState {
    /// Parses a FEN string into a game state.
    /// Standard starting position: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    pub fn from_fen(fen: &str) -> Result<Self, FenError> {
        let parts: Vec<&str> = fen.split_whitespace().collect();

        if parts.len() != 6 {
            return Err(FenError::InvalidFormat(format!(
                "Expected 6 fields, got {}",
                parts.len()
            )));
        }

        // Parse board position
        let board = parse_board(parts[0])?;

        // Parse side to move
        let turn = match parts[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err(FenError::InvalidColor(parts[1].to_string())),
        };

        // Parse castling rights
        let castling = parse_castling(parts[2])?;

        // Parse en passant square
        let en_passant = parse_en_passant(parts[3])?;

        // Parse halfmove clock
        let halfmove_clock = parts[4]
            .parse::<u16>()
            .map_err(|_| FenError::InvalidNumber(parts[4].to_string()))?;

        // Parse fullmove number
        let fullmove_number = parts[5]
            .parse::<u16>()
            .map_err(|_| FenError::InvalidNumber(parts[5].to_string()))?;

        Ok(GameState {
            board,
            turn,
            castling,
            en_passant,
            halfmove_clock,
            fullmove_number,
        })
    }

    /// Converts the game state to a FEN string.
    pub fn to_fen(&self) -> String {
        format!(
            "{} {} {} {} {} {}",
            board_to_fen(&self.board),
            if self.turn == Color::White { "w" } else { "b" },
            castling_to_fen(self.castling),
            en_passant_to_fen(self.en_passant),
            self.halfmove_clock,
            self.fullmove_number
        )
    }
}

/// Parses the board portion of a FEN string.
fn parse_board(board_str: &str) -> Result<BoardState, FenError> {
    let mut board = BoardState::empty();
    let ranks: Vec<&str> = board_str.split('/').collect();

    if ranks.len() != 8 {
        return Err(FenError::InvalidFormat(format!(
            "Expected 8 ranks, got {}",
            ranks.len()
        )));
    }

    for (rank_idx, rank_str) in ranks.iter().enumerate() {
        // FEN starts from rank 8 (index 7) down to rank 1 (index 0)
        let rank = Rank::new(7 - rank_idx as u8).unwrap();
        let mut file_idx = 0u8;

        for ch in rank_str.chars() {
            if file_idx >= 8 {
                return Err(FenError::InvalidFormat(format!(
                    "Too many squares in rank {}",
                    8 - rank_idx
                )));
            }

            if ch.is_numeric() {
                // Empty squares
                let empty_count = ch.to_digit(10).unwrap() as u8;
                file_idx += empty_count;
            } else {
                // Piece
                let file = File::new(file_idx).unwrap();
                let square = Square::new(file, rank);
                let piece = piece_from_char(ch)?;
                board.set_square(square, Some(piece));
                file_idx += 1;
            }
        }

        if file_idx != 8 {
            return Err(FenError::InvalidFormat(format!(
                "Rank {} has {} squares, expected 8",
                8 - rank_idx,
                file_idx
            )));
        }
    }

    Ok(board)
}

/// Converts a board to FEN notation.
fn board_to_fen(board: &BoardState) -> String {
    let mut fen = String::new();

    // Iterate from rank 8 down to rank 1
    for rank_idx in (0..8).rev() {
        let rank = Rank::new(rank_idx).unwrap();
        let mut empty_count = 0;

        for file_idx in 0..8 {
            let file = File::new(file_idx).unwrap();
            let square = Square::new(file, rank);

            match board.piece_at(square) {
                Some(piece) => {
                    if empty_count > 0 {
                        fen.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }
                    fen.push(piece_to_char(piece));
                }
                None => {
                    empty_count += 1;
                }
            }
        }

        if empty_count > 0 {
            fen.push_str(&empty_count.to_string());
        }

        if rank_idx > 0 {
            fen.push('/');
        }
    }

    fen
}

/// Converts a piece to its FEN character.
fn piece_to_char(piece: Piece) -> char {
    let ch = match piece.piece_type {
        PieceType::Pawn => 'p',
        PieceType::Knight => 'n',
        PieceType::Bishop => 'b',
        PieceType::Rook => 'r',
        PieceType::Queen => 'q',
        PieceType::King => 'k',
    };

    if piece.color == Color::White {
        ch.to_ascii_uppercase()
    } else {
        ch
    }
}

/// Parses a FEN character into a piece.
fn piece_from_char(ch: char) -> Result<Piece, FenError> {
    let color = if ch.is_uppercase() {
        Color::White
    } else {
        Color::Black
    };

    let piece_type = match ch.to_ascii_lowercase() {
        'p' => PieceType::Pawn,
        'n' => PieceType::Knight,
        'b' => PieceType::Bishop,
        'r' => PieceType::Rook,
        'q' => PieceType::Queen,
        'k' => PieceType::King,
        _ => return Err(FenError::InvalidPiece(ch)),
    };

    Ok(Piece { piece_type, color })
}

/// Parses castling rights from FEN notation.
fn parse_castling(castling_str: &str) -> Result<CastlingRights, FenError> {
    if castling_str == "-" {
        return Ok(CastlingRights::none());
    }

    let mut white = SideCastlingRights {
        kingside: false,
        queenside: false,
    };
    let mut black = SideCastlingRights {
        kingside: false,
        queenside: false,
    };

    for ch in castling_str.chars() {
        match ch {
            'K' => white.kingside = true,
            'Q' => white.queenside = true,
            'k' => black.kingside = true,
            'q' => black.queenside = true,
            _ => return Err(FenError::InvalidCastling(castling_str.to_string())),
        }
    }

    Ok(CastlingRights { white, black })
}

/// Converts castling rights to FEN notation.
fn castling_to_fen(castling: CastlingRights) -> String {
    let mut s = String::new();

    if castling.white.kingside {
        s.push('K');
    }
    if castling.white.queenside {
        s.push('Q');
    }
    if castling.black.kingside {
        s.push('k');
    }
    if castling.black.queenside {
        s.push('q');
    }

    if s.is_empty() { "-".to_string() } else { s }
}

/// Parses en passant square from FEN notation.
fn parse_en_passant(ep_str: &str) -> Result<Option<Square>, FenError> {
    if ep_str == "-" {
        return Ok(None);
    }

    if ep_str.len() != 2 {
        return Err(FenError::InvalidEnPassant(ep_str.to_string()));
    }

    let chars: Vec<char> = ep_str.chars().collect();
    let file =
        File::from_char(chars[0]).ok_or_else(|| FenError::InvalidEnPassant(ep_str.to_string()))?;
    let rank =
        Rank::from_char(chars[1]).ok_or_else(|| FenError::InvalidEnPassant(ep_str.to_string()))?;

    Ok(Some(Square::new(file, rank)))
}

/// Converts en passant square to FEN notation.
fn en_passant_to_fen(en_passant: Option<Square>) -> String {
    match en_passant {
        Some(square) => square.to_string(),
        None => "-".to_string(),
    }
}

/// Standard FEN positions for testing.
pub mod positions {
    /// Starting position.
    pub const STARTING: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    /// Kiwipete position - good for testing complex positions.
    pub const KIWIPETE: &str =
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

    /// Position after 1.e4 e5.
    pub const AFTER_E4_E5: &str = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_starting_position() {
        let state = GameState::from_fen(positions::STARTING).unwrap();
        assert_eq!(state.turn, Color::White);
        assert_eq!(state.fullmove_number, 1);
        assert_eq!(state.halfmove_clock, 0);
        assert!(state.en_passant.is_none());
    }

    #[test]
    fn test_round_trip() {
        let original_fen = positions::STARTING;
        let state = GameState::from_fen(original_fen).unwrap();
        let new_fen = state.to_fen();
        assert_eq!(original_fen, new_fen);
    }

    #[test]
    fn test_parse_kiwipete() {
        let state = GameState::from_fen(positions::KIWIPETE).unwrap();
        assert_eq!(state.turn, Color::White);

        // Verify a few key pieces
        let e1 = Square::new(File::from_char('e').unwrap(), Rank::FIRST);
        let piece = state.board.piece_at(e1).unwrap();
        assert_eq!(piece.piece_type, PieceType::King);
        assert_eq!(piece.color, Color::White);
    }

    #[test]
    fn test_parse_en_passant() {
        let state = GameState::from_fen(positions::AFTER_E4_E5).unwrap();
        assert!(state.en_passant.is_some());
        let ep = state.en_passant.unwrap();
        assert_eq!(ep.file().to_char(), 'e');
        assert_eq!(ep.rank().to_char(), '6');
    }

    #[test]
    fn test_invalid_fen() {
        assert!(GameState::from_fen("invalid").is_err());
        assert!(GameState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR").is_err());
        assert!(
            GameState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR x KQkq - 0 1")
                .is_err()
        );
    }
}
