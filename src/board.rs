/// Board representation using both array-based and bitboard approaches.
/// This provides flexibility and performance for different operations.
use crate::types::*;

/// Array-based board representation.
/// Simple and intuitive for piece lookup and modification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Board {
    /// 64 squares, indexed by Square::index()
    squares: [Option<Piece>; 64],
}

impl Board {
    /// Creates an empty board.
    pub const fn empty() -> Self {
        Self {
            squares: [None; 64],
        }
    }

    /// Creates the standard starting position.
    pub fn starting_position() -> Self {
        let mut board = Self::empty();

        // White pieces
        board.set_piece(
            Square::from_index(0).unwrap(),
            Some(Piece::new(PieceType::Rook, Color::White)),
        );
        board.set_piece(
            Square::from_index(1).unwrap(),
            Some(Piece::new(PieceType::Knight, Color::White)),
        );
        board.set_piece(
            Square::from_index(2).unwrap(),
            Some(Piece::new(PieceType::Bishop, Color::White)),
        );
        board.set_piece(
            Square::from_index(3).unwrap(),
            Some(Piece::new(PieceType::Queen, Color::White)),
        );
        board.set_piece(
            Square::from_index(4).unwrap(),
            Some(Piece::new(PieceType::King, Color::White)),
        );
        board.set_piece(
            Square::from_index(5).unwrap(),
            Some(Piece::new(PieceType::Bishop, Color::White)),
        );
        board.set_piece(
            Square::from_index(6).unwrap(),
            Some(Piece::new(PieceType::Knight, Color::White)),
        );
        board.set_piece(
            Square::from_index(7).unwrap(),
            Some(Piece::new(PieceType::Rook, Color::White)),
        );

        // White pawns
        for i in 8..16 {
            board.set_piece(
                Square::from_index(i).unwrap(),
                Some(Piece::new(PieceType::Pawn, Color::White)),
            );
        }

        // Black pawns
        for i in 48..56 {
            board.set_piece(
                Square::from_index(i).unwrap(),
                Some(Piece::new(PieceType::Pawn, Color::Black)),
            );
        }

        // Black pieces
        board.set_piece(
            Square::from_index(56).unwrap(),
            Some(Piece::new(PieceType::Rook, Color::Black)),
        );
        board.set_piece(
            Square::from_index(57).unwrap(),
            Some(Piece::new(PieceType::Knight, Color::Black)),
        );
        board.set_piece(
            Square::from_index(58).unwrap(),
            Some(Piece::new(PieceType::Bishop, Color::Black)),
        );
        board.set_piece(
            Square::from_index(59).unwrap(),
            Some(Piece::new(PieceType::Queen, Color::Black)),
        );
        board.set_piece(
            Square::from_index(60).unwrap(),
            Some(Piece::new(PieceType::King, Color::Black)),
        );
        board.set_piece(
            Square::from_index(61).unwrap(),
            Some(Piece::new(PieceType::Bishop, Color::Black)),
        );
        board.set_piece(
            Square::from_index(62).unwrap(),
            Some(Piece::new(PieceType::Knight, Color::Black)),
        );
        board.set_piece(
            Square::from_index(63).unwrap(),
            Some(Piece::new(PieceType::Rook, Color::Black)),
        );

        board
    }

    /// Gets the piece at the given square.
    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        self.squares[square.index() as usize]
    }

    /// Sets the piece at the given square.
    pub fn set_piece(&mut self, square: Square, piece: Option<Piece>) {
        self.squares[square.index() as usize] = piece;
    }

    /// Moves a piece from one square to another.
    /// Returns the captured piece, if any.
    pub fn move_piece(&mut self, from: Square, to: Square) -> Option<Piece> {
        let piece = self.squares[from.index() as usize];
        let captured = self.squares[to.index() as usize];

        self.squares[from.index() as usize] = None;
        self.squares[to.index() as usize] = piece;

        captured
    }

    /// Returns true if the given square is empty.
    pub fn is_empty(&self, square: Square) -> bool {
        self.piece_at(square).is_none()
    }

    /// Returns true if the given square contains a piece of the given color.
    pub fn is_color(&self, square: Square, color: Color) -> bool {
        self.piece_at(square).map_or(false, |p| p.color == color)
    }

    /// Returns true if the given square contains an enemy piece.
    pub fn is_enemy(&self, square: Square, color: Color) -> bool {
        self.piece_at(square)
            .map_or(false, |p| p.color == color.opponent())
    }

    /// Finds the king square for the given color.
    /// Panics if no king is found (invalid board state).
    pub fn king_square(&self, color: Color) -> Square {
        for i in 0..64 {
            if let Some(square) = Square::from_index(i) {
                if let Some(piece) = self.piece_at(square) {
                    if piece.piece_type == PieceType::King && piece.color == color {
                        return square;
                    }
                }
            }
        }
        panic!("No king found for color {:?}", color);
    }
}

/// Bitboard-based board representation.
/// Efficient for move generation and attack detection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BitBoardSet {
    /// Bitboards by piece type and color
    pieces: [[BitBoard; 6]; 2],
    /// Combined occupancy by color
    color_occupancy: [BitBoard; 2],
    /// Total occupancy
    all_occupancy: BitBoard,
}

impl BitBoardSet {
    /// Creates an empty bitboard set.
    pub const fn empty() -> Self {
        Self {
            pieces: [[BitBoard::EMPTY; 6]; 2],
            color_occupancy: [BitBoard::EMPTY; 2],
            all_occupancy: BitBoard::EMPTY,
        }
    }

    /// Creates bitboards from an array-based board.
    pub fn from_board(board: &Board) -> Self {
        let mut bitboards = Self::empty();

        for i in 0..64 {
            if let Some(square) = Square::from_index(i) {
                if let Some(piece) = board.piece_at(square) {
                    bitboards.set_piece(square, piece);
                }
            }
        }

        bitboards
    }

    /// Gets the bitboard for pieces of a specific type and color.
    pub fn pieces(&self, piece_type: PieceType, color: Color) -> BitBoard {
        self.pieces[color as usize][piece_type as usize]
    }

    /// Gets all pieces of a specific color.
    pub fn color_occupancy(&self, color: Color) -> BitBoard {
        self.color_occupancy[color as usize]
    }

    /// Gets all occupied squares.
    pub fn all_occupancy(&self) -> BitBoard {
        self.all_occupancy
    }

    /// Gets all empty squares.
    pub fn empty_squares(&self) -> BitBoard {
        self.all_occupancy.complement()
    }

    /// Sets a piece on the given square.
    fn set_piece(&mut self, square: Square, piece: Piece) {
        let bb = BitBoard::from_square(square);
        self.pieces[piece.color as usize][piece.piece_type as usize] =
            self.pieces[piece.color as usize][piece.piece_type as usize].union(bb);
        self.color_occupancy[piece.color as usize] =
            self.color_occupancy[piece.color as usize].union(bb);
        self.all_occupancy = self.all_occupancy.union(bb);
    }

    /// Removes a piece from the given square.
    fn clear_square(&mut self, square: Square) {
        let bb = BitBoard::from_square(square);
        let complement = bb.complement();

        for color in 0..2 {
            for piece_type in 0..6 {
                self.pieces[color][piece_type] =
                    self.pieces[color][piece_type].intersection(complement);
            }
            self.color_occupancy[color] = self.color_occupancy[color].intersection(complement);
        }
        self.all_occupancy = self.all_occupancy.intersection(complement);
    }

    /// Moves a piece from one square to another.
    pub fn move_piece(&mut self, from: Square, to: Square, piece: Piece) {
        self.clear_square(from);
        self.clear_square(to); // Remove any piece on destination
        self.set_piece(to, piece);
    }
}

/// Complete board state combining both representations.
/// This allows us to use the most appropriate representation for each operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoardState {
    /// Array-based representation for simple lookups
    pub array_board: Board,
    /// Bitboard representation for efficient operations
    pub bitboards: BitBoardSet,
}

impl BoardState {
    /// Creates an empty board state.
    pub fn empty() -> Self {
        Self {
            array_board: Board::empty(),
            bitboards: BitBoardSet::empty(),
        }
    }

    /// Creates the standard starting position.
    pub fn starting_position() -> Self {
        let array_board = Board::starting_position();
        let bitboards = BitBoardSet::from_board(&array_board);
        Self {
            array_board,
            bitboards,
        }
    }

    /// Gets the piece at the given square.
    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        self.array_board.piece_at(square)
    }

    /// Moves a piece from one square to another.
    /// Returns the captured piece, if any.
    pub fn move_piece(&mut self, from: Square, to: Square) -> Option<Piece> {
        let piece = self.piece_at(from).expect("No piece at source square");
        let captured = self.array_board.move_piece(from, to);
        self.bitboards.move_piece(from, to, piece);
        captured
    }

    /// Returns true if the representations are consistent.
    /// Useful for debugging and testing.
    #[cfg(debug_assertions)]
    pub fn is_consistent(&self) -> bool {
        let reconstructed = BitBoardSet::from_board(&self.array_board);
        self.bitboards == reconstructed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starting_position() {
        let board = BoardState::starting_position();

        // Check white pieces
        assert_eq!(
            board.piece_at(Square::from_index(0).unwrap()),
            Some(Piece::new(PieceType::Rook, Color::White))
        );
        assert_eq!(
            board.piece_at(Square::from_index(4).unwrap()),
            Some(Piece::new(PieceType::King, Color::White))
        );

        // Check black pieces
        assert_eq!(
            board.piece_at(Square::from_index(60).unwrap()),
            Some(Piece::new(PieceType::King, Color::Black))
        );

        // Check empty squares
        assert!(board.piece_at(Square::from_index(35).unwrap()).is_none());

        #[cfg(debug_assertions)]
        assert!(board.is_consistent());
    }

    #[test]
    fn test_move_piece() {
        let mut board = BoardState::starting_position();

        // Move white pawn e2-e4
        let from = Square::from_index(12).unwrap(); // e2
        let to = Square::from_index(28).unwrap(); // e4

        let captured = board.move_piece(from, to);
        assert!(captured.is_none());
        assert!(board.piece_at(from).is_none());
        assert_eq!(
            board.piece_at(to),
            Some(Piece::new(PieceType::Pawn, Color::White))
        );

        #[cfg(debug_assertions)]
        assert!(board.is_consistent());
    }

    #[test]
    fn test_bitboard_occupancy() {
        let board = BoardState::starting_position();

        // Check white occupancy
        let white_occ = board.bitboards.color_occupancy(Color::White);
        assert_eq!(white_occ.count(), 16);

        // Check black occupancy
        let black_occ = board.bitboards.color_occupancy(Color::Black);
        assert_eq!(black_occ.count(), 16);

        // Check total occupancy
        assert_eq!(board.bitboards.all_occupancy().count(), 32);

        // Check empty squares
        assert_eq!(board.bitboards.empty_squares().count(), 32);
    }
}
