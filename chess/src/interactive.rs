use chess_agents::{search_with_limits, SearchLimits};
use chess_core::{generate_legal_moves, Color, File, GameState, Move, PieceType, Rank, Square};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent},
    style::{Color as TermColor, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
    ExecutableCommand,
};
use std::io::{self, Write};

pub struct InteractiveGame {
    state: GameState,
    cursor_pos: (u8, u8), // (file, rank) in 0-7 range
    selected_square: Option<Square>,
    legal_moves_for_selected: Vec<Move>,
    message: String,
    move_history: Vec<Move>,
}

impl InteractiveGame {
    pub fn new() -> Self {
        Self {
            state: GameState::new(),
            cursor_pos: (4, 1), // Start at e2
            selected_square: None,
            legal_moves_for_selected: Vec::new(),
            message: String::from("Use hjkl to move, Enter to select/move, q to quit"),
            move_history: Vec::new(),
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        // Setup terminal
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(Hide)?;
        stdout.execute(Clear(ClearType::All))?;

        let result = self.game_loop();

        // Cleanup
        stdout.execute(Show)?;
        terminal::disable_raw_mode()?;
        stdout.execute(Clear(ClearType::All))?;
        stdout.execute(MoveTo(0, 0))?;

        result
    }

    fn game_loop(&mut self) -> io::Result<()> {
        loop {
            self.draw_board()?;

            // Check for game over
            let legal_moves = generate_legal_moves(&self.state);
            if legal_moves.is_empty() {
                if self.state.is_in_check() {
                    self.message = format!(
                        "Checkmate! {} wins!",
                        if self.state.turn == Color::White {
                            "Black"
                        } else {
                            "White"
                        }
                    );
                } else {
                    self.message = String::from("Stalemate!");
                }
                self.draw_board()?;
                event::read()?; // Wait for any key
                break;
            }

            if self.state.is_in_check() {
                self.message = String::from("Check!");
            }

            // Handle input
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('h') | KeyCode::Left => self.move_cursor(-1, 0),
                    KeyCode::Char('j') | KeyCode::Down => self.move_cursor(0, -1),
                    KeyCode::Char('k') | KeyCode::Up => self.move_cursor(0, 1),
                    KeyCode::Char('l') | KeyCode::Right => self.move_cursor(1, 0),
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        if self.handle_selection() {
                            // Player made a move, now engine's turn
                            self.engine_move()?;
                        }
                    }
                    KeyCode::Char('u') => self.undo_move(),
                    KeyCode::Char('n') => self.new_game(),
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn move_cursor(&mut self, dx: i8, dy: i8) {
        let new_file = self.cursor_pos.0 as i8 + dx;
        let new_rank = self.cursor_pos.1 as i8 + dy;

        if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
            self.cursor_pos = (new_file as u8, new_rank as u8);
        }
    }

    fn handle_selection(&mut self) -> bool {
        let cursor_square = Square::new(
            File::new(self.cursor_pos.0).unwrap(),
            Rank::new(self.cursor_pos.1).unwrap(),
        );

        if let Some(selected) = self.selected_square {
            // We have a piece selected, try to move it
            if let Some(mv) = self
                .legal_moves_for_selected
                .iter()
                .find(|m| m.to == cursor_square)
            {
                // Check if this is a promotion
                let mv = if mv.from.rank() == self.state.turn.pawn_rank().offset(5).unwrap()
                    && cursor_square.rank() == self.state.turn.promotion_rank()
                    && self
                        .state
                        .board
                        .piece_at(mv.from)
                        .map(|p| p.piece_type == PieceType::Pawn)
                        .unwrap_or(false)
                {
                    // Always promote to queen for simplicity
                    Move::new_promotion(mv.from, mv.to, PieceType::Queen)
                } else {
                    *mv
                };

                self.state = self.state.apply_move(mv);
                self.move_history.push(mv);
                self.selected_square = None;
                self.legal_moves_for_selected.clear();
                self.message = format!("Moved: {}", mv);
                return true;
            } else {
                // Clicked somewhere else, deselect
                self.selected_square = None;
                self.legal_moves_for_selected.clear();
            }
        }

        // Try to select a piece
        if let Some(piece) = self.state.board.piece_at(cursor_square) {
            if piece.color == self.state.turn {
                self.selected_square = Some(cursor_square);

                // Find all legal moves for this piece
                let all_moves = generate_legal_moves(&self.state);
                self.legal_moves_for_selected = all_moves
                    .iter()
                    .filter(|m| m.from == cursor_square)
                    .copied()
                    .collect();

                self.message = format!(
                    "Selected {} at {}",
                    match piece.piece_type {
                        PieceType::Pawn => "Pawn",
                        PieceType::Knight => "Knight",
                        PieceType::Bishop => "Bishop",
                        PieceType::Rook => "Rook",
                        PieceType::Queen => "Queen",
                        PieceType::King => "King",
                    },
                    cursor_square
                );
            }
        }

        false
    }

    fn engine_move(&mut self) -> io::Result<()> {
        self.message = String::from("Engine thinking...");
        self.draw_board()?;

        let result = search_with_limits(&self.state, SearchLimits::move_time(2000));

        if let Some(engine_move) = result.best_move {
            self.state = self.state.apply_move(engine_move);
            self.move_history.push(engine_move);
            self.message = format!("Engine played: {}", engine_move);
        }

        Ok(())
    }

    fn undo_move(&mut self) {
        if self.move_history.len() >= 2 {
            // Undo both player and engine moves
            self.move_history.pop();
            self.move_history.pop();

            // Rebuild position
            self.state = GameState::new();
            for mv in &self.move_history {
                self.state = self.state.apply_move(*mv);
            }

            self.selected_square = None;
            self.legal_moves_for_selected.clear();
            self.message = String::from("Undid last move");
        } else {
            self.message = String::from("Nothing to undo");
        }
    }

    fn new_game(&mut self) {
        self.state = GameState::new();
        self.move_history.clear();
        self.selected_square = None;
        self.legal_moves_for_selected.clear();
        self.cursor_pos = (4, 1); // e2
        self.message = String::from("New game started!");
    }

    fn draw_board(&self) -> io::Result<()> {
        let mut stdout = io::stdout();
        stdout.execute(MoveTo(0, 0))?;

        // Title
        println!("Chess - Interactive Mode (vim keys: hjkl)\r");
        println!("Commands: Enter=select/move, u=undo, n=new, q=quit\r");
        println!("\r");

        // Board with coordinates
        println!("  a b c d e f g h  \r");
        println!(" ┌─────────────────┐\r");

        for rank_idx in (0..8).rev() {
            print!("{}│ ", rank_idx + 1);

            for file_idx in 0..8 {
                let square =
                    Square::new(File::new(file_idx).unwrap(), Rank::new(rank_idx).unwrap());

                let is_cursor = self.cursor_pos == (file_idx, rank_idx);
                let is_selected = self.selected_square == Some(square);
                let is_legal_move = self.legal_moves_for_selected.iter().any(|m| m.to == square);

                // Set background color
                if is_cursor {
                    stdout.execute(SetBackgroundColor(TermColor::Yellow))?;
                } else if is_selected {
                    stdout.execute(SetBackgroundColor(TermColor::Green))?;
                } else if is_legal_move {
                    stdout.execute(SetBackgroundColor(TermColor::Blue))?;
                } else if (file_idx + rank_idx) % 2 == 0 {
                    stdout.execute(SetBackgroundColor(TermColor::DarkGrey))?;
                } else {
                    stdout.execute(SetBackgroundColor(TermColor::Black))?;
                }

                // Draw piece or empty square
                if let Some(piece) = self.state.board.piece_at(square) {
                    let symbol = match (piece.piece_type, piece.color) {
                        (PieceType::King, Color::White) => '♔',
                        (PieceType::Queen, Color::White) => '♕',
                        (PieceType::Rook, Color::White) => '♖',
                        (PieceType::Bishop, Color::White) => '♗',
                        (PieceType::Knight, Color::White) => '♘',
                        (PieceType::Pawn, Color::White) => '♙',
                        (PieceType::King, Color::Black) => '♚',
                        (PieceType::Queen, Color::Black) => '♛',
                        (PieceType::Rook, Color::Black) => '♜',
                        (PieceType::Bishop, Color::Black) => '♝',
                        (PieceType::Knight, Color::Black) => '♞',
                        (PieceType::Pawn, Color::Black) => '♟',
                    };

                    if piece.color == Color::White {
                        stdout.execute(SetForegroundColor(TermColor::White))?;
                    } else {
                        stdout.execute(SetForegroundColor(TermColor::Magenta))?;
                    }

                    print!("{} ", symbol);
                } else {
                    print!("  ");
                }

                stdout.execute(ResetColor)?;
            }

            println!("│{}\r", rank_idx + 1);
        }

        println!(" └─────────────────┘\r");
        println!("  a b c d e f g h  \r");
        println!("\r");

        // Game info
        println!(
            "{} to move | Move {}\r",
            if self.state.turn == Color::White {
                "White"
            } else {
                "Black"
            },
            self.state.fullmove_number
        );

        // Status message
        println!("\r");
        println!("{}\r", self.message);

        stdout.flush()?;
        Ok(())
    }
}
