use std::fmt;

/// Represents one of the two players in chess.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    /// Returns the opposite color.
    pub const fn opponent(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    /// Returns the starting rank for pawns of this color.
    pub const fn pawn_rank(self) -> Rank {
        match self {
            Color::White => Rank::Second,
            Color::Black => Rank::Seventh,
        }
    }

    /// Returns the promotion rank for pawns of this color.
    pub const fn promotion_rank(self) -> Rank {
        match self {
            Color::White => Rank::Eighth,
            Color::Black => Rank::First,
        }
    }

    /// Returns the direction pawns of this color move.
    pub const fn pawn_direction(self) -> i8 {
        match self {
            Color::White => 1,
            Color::Black => -1,
        }
    }
}

/// The six types of chess pieces.
/// Using an enum ensures only valid piece types can exist.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceType {
    /// Returns the material value of this piece type in centipawns.
    pub const fn value(self) -> u16 {
        match self {
            PieceType::Pawn => 100,
            PieceType::Knight => 320,
            PieceType::Bishop => 330,
            PieceType::Rook => 500,
            PieceType::Queen => 900,
            PieceType::King => 0, // King has no material value
        }
    }

    /// Returns true if this piece type can slide (bishop, rook, queen).
    pub const fn is_slider(self) -> bool {
        matches!(self, PieceType::Bishop | PieceType::Rook | PieceType::Queen)
    }
}

/// A chess piece with both type and color.
/// This ensures every piece has valid attributes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

impl Piece {
    /// Creates a new piece with the given type and color.
    pub const fn new(piece_type: PieceType, color: Color) -> Self {
        Self { piece_type, color }
    }
}

/// A file on the chess board (a-h).
/// Using a newtype ensures type safety and valid range.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct File(u8);

impl File {
    /// Creates a new file from index (0-7).
    /// Returns None if index is out of range.
    pub const fn new(index: u8) -> Option<Self> {
        if index < 8 { Some(File(index)) } else { None }
    }

    /// Creates a file from a character ('a'-'h').
    pub const fn from_char(c: char) -> Option<Self> {
        match c {
            'a' => Some(File(0)),
            'b' => Some(File(1)),
            'c' => Some(File(2)),
            'd' => Some(File(3)),
            'e' => Some(File(4)),
            'f' => Some(File(5)),
            'g' => Some(File(6)),
            'h' => Some(File(7)),
            _ => None,
        }
    }

    /// Returns the file as a character ('a'-'h').
    pub const fn to_char(self) -> char {
        (b'a' + self.0) as char
    }

    /// Returns the file index (0-7).
    pub const fn index(self) -> u8 {
        self.0
    }

    /// Returns the adjacent file in the given direction, if valid.
    pub const fn offset(self, delta: i8) -> Option<Self> {
        let new_file = self.0 as i8 + delta;
        if new_file >= 0 && new_file < 8 {
            Some(File(new_file as u8))
        } else {
            None
        }
    }
}

/// A rank on the chess board (1-8).
/// Using a newtype ensures type safety and valid range.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Rank(u8);

impl Rank {
    /// Creates a new rank from index (0-7).
    /// Returns None if index is out of range.
    pub const fn new(index: u8) -> Option<Self> {
        if index < 8 { Some(Rank(index)) } else { None }
    }

    /// Creates a rank from a digit ('1'-'8').
    pub const fn from_char(c: char) -> Option<Self> {
        match c {
            '1' => Some(Rank(0)),
            '2' => Some(Rank(1)),
            '3' => Some(Rank(2)),
            '4' => Some(Rank(3)),
            '5' => Some(Rank(4)),
            '6' => Some(Rank(5)),
            '7' => Some(Rank(6)),
            '8' => Some(Rank(7)),
            _ => None,
        }
    }

    /// Returns the rank as a character ('1'-'8').
    pub const fn to_char(self) -> char {
        (b'1' + self.0) as char
    }

    /// Returns the rank index (0-7).
    pub const fn index(self) -> u8 {
        self.0
    }

    /// Returns the adjacent rank in the given direction, if valid.
    pub const fn offset(self, delta: i8) -> Option<Self> {
        let new_rank = self.0 as i8 + delta;
        if new_rank >= 0 && new_rank < 8 {
            Some(Rank(new_rank as u8))
        } else {
            None
        }
    }
}

/// Rank constants for readability.
impl Rank {
    pub const First: Rank = Rank(0);
    pub const Second: Rank = Rank(1);
    pub const Third: Rank = Rank(2);
    pub const Fourth: Rank = Rank(3);
    pub const Fifth: Rank = Rank(4);
    pub const Sixth: Rank = Rank(5);
    pub const Seventh: Rank = Rank(6);
    pub const Eighth: Rank = Rank(7);
}

/// A square on the chess board.
/// Using a newtype with u8 ensures memory efficiency while maintaining type safety.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Square(u8);

impl Square {
    /// Creates a new square from file and rank.
    pub const fn new(file: File, rank: Rank) -> Self {
        Square(rank.0 * 8 + file.0)
    }

    /// Creates a square from index (0-63).
    /// Returns None if index is out of range.
    pub const fn from_index(index: u8) -> Option<Self> {
        if index < 64 {
            Some(Square(index))
        } else {
            None
        }
    }

    /// Returns the file of this square.
    pub const fn file(self) -> File {
        File(self.0 % 8)
    }

    /// Returns the rank of this square.
    pub const fn rank(self) -> Rank {
        Rank(self.0 / 8)
    }

    /// Returns the square index (0-63).
    pub const fn index(self) -> u8 {
        self.0
    }

    /// Returns the square color (light or dark).
    pub const fn color(self) -> Color {
        if (self.file().0 + self.rank().0) % 2 == 0 {
            Color::Black // Dark squares
        } else {
            Color::White // Light squares
        }
    }

    /// Calculates Manhattan distance to another square.
    pub const fn distance(self, other: Square) -> u8 {
        let file_diff = if self.file().0 > other.file().0 {
            self.file().0 - other.file().0
        } else {
            other.file().0 - self.file().0
        };

        let rank_diff = if self.rank().0 > other.rank().0 {
            self.rank().0 - other.rank().0
        } else {
            other.rank().0 - self.rank().0
        };

        if file_diff > rank_diff { file_diff } else { rank_diff }
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.file().to_char(), self.rank().to_char())
    }
}

/// Castling rights for a single side.
/// Using a struct with booleans ensures clear semantics.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct SideCastlingRights {
    pub kingside: bool,
    pub queenside: bool,
}

impl SideCastlingRights {
    /// Creates new castling rights with both sides available.
    pub const fn both() -> Self {
        Self {
            kingside: true,
            queenside: true,
        }
    }

    /// Creates castling rights with no rights available.
    pub const fn none() -> Self {
        Self {
            kingside: false,
            queenside: false,
        }
    }

    /// Returns true if any castling right is available.
    pub const fn any(self) -> bool {
        self.kingside || self.queenside
    }
}

/// Complete castling rights for both colors.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CastlingRights {
    pub white: SideCastlingRights,
    pub black: SideCastlingRights,
}

impl CastlingRights {
    /// Creates castling rights with all rights available.
    pub const fn all() -> Self {
        Self {
            white: SideCastlingRights::both(),
            black: SideCastlingRights::both(),
        }
    }

    /// Creates castling rights with no rights available.
    pub const fn none() -> Self {
        Self {
            white: SideCastlingRights::none(),
            black: SideCastlingRights::none(),
        }
    }

    /// Gets castling rights for a specific color.
    pub const fn get(self, color: Color) -> SideCastlingRights {
        match color {
            Color::White => self.white,
            Color::Black => self.black,
        }
    }

    /// Updates castling rights when a piece moves from a square.
    pub fn update_after_move(self, from: Square, to: Square) -> Self {
        let mut rights = self;

        // King moves
        if from.index() == 4 {
            // e1
            rights.white = SideCastlingRights::none();
        } else if from.index() == 60 {
            // e8
            rights.black = SideCastlingRights::none();
        }

        // Rook moves or captures
        match from.index() {
            0 => rights.white.queenside = false,  // a1
            7 => rights.white.kingside = false,   // h1
            56 => rights.black.queenside = false, // a8
            63 => rights.black.kingside = false,  // h8
            _ => {}
        }

        // Rook captures
        match to.index() {
            0 => rights.white.queenside = false,  // a1
            7 => rights.white.kingside = false,   // h1
            56 => rights.black.queenside = false, // a8
            63 => rights.black.kingside = false,  // h8
            _ => {}
        }

        rights
    }
}

/// A chess move.
/// Includes all information needed to make and unmake the move.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<PieceType>,
}

impl Move {
    /// Creates a normal move.
    pub const fn new(from: Square, to: Square) -> Self {
        Self {
            from,
            to,
            promotion: None,
        }
    }

    /// Creates a promotion move.
    pub const fn new_promotion(from: Square, to: Square, promotion: PieceType) -> Self {
        Self {
            from,
            to,
            promotion: Some(promotion),
        }
    }

    /// Returns true if this is a castling move based on king movement.
    pub fn is_castle(self, piece: Piece) -> bool {
        piece.piece_type == PieceType::King && self.from.distance(self.to) == 2
    }
}

/// A bitboard represents a set of squares using a 64-bit integer.
/// Each bit corresponds to a square on the board.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct BitBoard(pub u64);

impl BitBoard {
    /// An empty bitboard with no squares set.
    pub const EMPTY: Self = BitBoard(0);

    /// A full bitboard with all squares set.
    pub const FULL: Self = BitBoard(!0);

    /// Creates a bitboard with a single square set.
    pub const fn from_square(square: Square) -> Self {
        BitBoard(1u64 << square.0)
    }

    /// Returns true if the given square is set.
    pub const fn contains(self, square: Square) -> bool {
        (self.0 & (1u64 << square.0)) != 0
    }

    /// Sets the given square.
    pub const fn set(self, square: Square) -> Self {
        BitBoard(self.0 | (1u64 << square.0))
    }

    /// Clears the given square.
    pub const fn clear(self, square: Square) -> Self {
        BitBoard(self.0 & !(1u64 << square.0))
    }

    /// Returns the number of set bits.
    pub const fn count(self) -> u32 {
        self.0.count_ones()
    }

    /// Returns true if no squares are set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns the union of two bitboards.
    pub const fn union(self, other: Self) -> Self {
        BitBoard(self.0 | other.0)
    }

    /// Returns the intersection of two bitboards.
    pub const fn intersection(self, other: Self) -> Self {
        BitBoard(self.0 & other.0)
    }

    /// Returns the complement of this bitboard.
    pub const fn complement(self) -> Self {
        BitBoard(!self.0)
    }
}

/// Iterator over set squares in a bitboard.
pub struct BitBoardIterator {
    bits: u64,
}

impl Iterator for BitBoardIterator {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            None
        } else {
            let index = self.bits.trailing_zeros() as u8;
            self.bits &= self.bits - 1; // Clear lowest set bit
            Square::from_index(index)
        }
    }
}

impl BitBoard {
    /// Returns an iterator over all set squares.
    pub fn iter(self) -> BitBoardIterator {
        BitBoardIterator { bits: self.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_opponent() {
        assert_eq!(Color::White.opponent(), Color::Black);
        assert_eq!(Color::Black.opponent(), Color::White);
    }

    #[test]
    fn test_square_creation() {
        let e4 = Square::new(File::new(4).unwrap(), Rank::new(3).unwrap());
        assert_eq!(e4.index(), 28);
        assert_eq!(format!("{}", e4), "e4");
    }

    #[test]
    fn test_bitboard_operations() {
        let bb1 = BitBoard::from_square(Square::from_index(0).unwrap());
        let bb2 = BitBoard::from_square(Square::from_index(7).unwrap());

        assert_eq!(bb1.count(), 1);
        assert_eq!(bb1.union(bb2).count(), 2);
        assert!(bb1.intersection(bb2).is_empty());
    }
}
