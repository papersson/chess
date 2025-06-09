use chess_agents::{search_with_callback, SearchLimits, SearchProgress};
use chess_core::{GameState, Move};
use std::io::{self, BufRead, Write};
use std::time::Duration;

pub struct UciEngine {
    position: GameState,
    debug: bool,
}

impl UciEngine {
    pub fn new() -> Self {
        Self {
            position: GameState::new(),
            debug: false,
        }
    }

    pub fn run(&mut self) {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line in stdin.lock().lines() {
            let line = line.unwrap();
            let parts: Vec<&str> = line.trim().split_whitespace().collect();

            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "uci" => {
                    println!("id name Rust Chess Engine");
                    println!("id author Claude Code");
                    println!("uciok");
                    stdout.flush().unwrap();
                }
                "debug" => {
                    if parts.len() > 1 {
                        self.debug = parts[1] == "on";
                    }
                }
                "isready" => {
                    println!("readyok");
                    stdout.flush().unwrap();
                }
                "setoption" => {
                    // Handle options in the future
                }
                "ucinewgame" => {
                    self.position = GameState::new();
                }
                "position" => {
                    self.handle_position(&parts);
                }
                "go" => {
                    self.handle_go(&parts);
                }
                "stop" => {
                    // In the future, implement stopping ongoing search
                }
                "quit" => {
                    break;
                }
                _ => {
                    if self.debug {
                        eprintln!("Unknown command: {}", parts[0]);
                    }
                }
            }
        }
    }

    fn handle_position(&mut self, parts: &[&str]) {
        if parts.len() < 2 {
            return;
        }

        let mut idx = 1;

        // Parse starting position
        match parts[idx] {
            "startpos" => {
                self.position = GameState::new();
                idx += 1;
            }
            "fen" => {
                // Collect FEN string (6 parts)
                let mut fen_parts = Vec::new();
                idx += 1;

                while idx < parts.len() && parts[idx] != "moves" {
                    fen_parts.push(parts[idx]);
                    idx += 1;
                }

                if fen_parts.len() >= 6 {
                    let fen = fen_parts.join(" ");
                    match GameState::from_fen(&fen) {
                        Ok(pos) => self.position = pos,
                        Err(e) => {
                            if self.debug {
                                eprintln!("Invalid FEN: {}", e);
                            }
                            return;
                        }
                    }
                }
            }
            _ => return,
        }

        // Apply moves if present
        if idx < parts.len() && parts[idx] == "moves" {
            idx += 1;
            while idx < parts.len() {
                if let Some(mv) = self.parse_move(parts[idx]) {
                    self.position = self.position.apply_move(mv);
                } else if self.debug {
                    eprintln!("Invalid move: {}", parts[idx]);
                }
                idx += 1;
            }
        }
    }

    fn handle_go(&self, parts: &[&str]) {
        let mut limits = SearchLimits {
            max_depth: None,
            move_time: None,
            nodes: None,
        };

        let mut idx = 1;
        while idx < parts.len() {
            match parts[idx] {
                "depth" => {
                    if idx + 1 < parts.len() {
                        if let Ok(d) = parts[idx + 1].parse::<u8>() {
                            limits.max_depth = Some(d);
                        }
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "movetime" => {
                    if idx + 1 < parts.len() {
                        if let Ok(ms) = parts[idx + 1].parse::<u64>() {
                            limits.move_time = Some(Duration::from_millis(ms));
                        }
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "nodes" => {
                    if idx + 1 < parts.len() {
                        if let Ok(n) = parts[idx + 1].parse::<u64>() {
                            limits.nodes = Some(n);
                        }
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "infinite" => {
                    // Search until stopped
                    limits.max_depth = Some(100);
                    idx += 1;
                }
                "wtime" | "btime" | "winc" | "binc" | "movestogo" => {
                    // TODO: Implement time control
                    idx += 2;
                }
                _ => idx += 1,
            }
        }

        // Default to depth 6 if no limits specified
        if limits.max_depth.is_none() && limits.move_time.is_none() && limits.nodes.is_none() {
            limits.max_depth = Some(6);
        }

        // Run search with info callback
        let position = self.position.clone();
        let _start_time = std::time::Instant::now();

        let callback = Box::new(move |info: &SearchProgress| {
            print!(
                "info depth {} score cp {} nodes {} time {} nps {} pv",
                info.depth,
                info.score,
                info.nodes,
                info.time_ms,
                if info.time_ms > 0 {
                    (info.nodes * 1000) / info.time_ms
                } else {
                    0
                }
            );

            // Print principal variation
            for mv in &info.pv {
                print!(" {}", format_move_static(*mv));
            }
            println!();
            io::stdout().flush().unwrap();
        });

        let result = search_with_callback(&position, limits, callback);

        // Output result
        if let Some(best_move) = result.best_move {
            println!("bestmove {}", self.format_move(best_move));
        } else {
            println!("bestmove 0000"); // Null move (no legal moves)
        }
        io::stdout().flush().unwrap();
    }

    fn parse_move(&self, move_str: &str) -> Option<Move> {
        if move_str.len() < 4 {
            return None;
        }

        let bytes = move_str.as_bytes();

        // Parse from square
        let from_file = (bytes[0] as i8 - b'a' as i8) as u8;
        let from_rank = (bytes[1] as i8 - b'1' as i8) as u8;
        let from = chess_core::Square::new(
            chess_core::File::new(from_file)?,
            chess_core::Rank::new(from_rank)?,
        );

        // Parse to square
        let to_file = (bytes[2] as i8 - b'a' as i8) as u8;
        let to_rank = (bytes[3] as i8 - b'1' as i8) as u8;
        let to = chess_core::Square::new(
            chess_core::File::new(to_file)?,
            chess_core::Rank::new(to_rank)?,
        );

        // Parse promotion if present
        let promotion = if move_str.len() > 4 {
            match bytes[4] {
                b'q' => Some(chess_core::PieceType::Queen),
                b'r' => Some(chess_core::PieceType::Rook),
                b'b' => Some(chess_core::PieceType::Bishop),
                b'n' => Some(chess_core::PieceType::Knight),
                _ => None,
            }
        } else {
            None
        };

        // Create move and check if it's legal
        let mv = Move {
            from,
            to,
            promotion,
        };

        // Verify move is legal
        let moves = chess_core::generate_legal_moves(&self.position);
        if moves.iter().any(|&legal_mv| legal_mv == mv) {
            Some(mv)
        } else {
            None
        }
    }

    fn format_move(&self, mv: Move) -> String {
        format_move_static(mv)
    }
}

fn format_move_static(mv: Move) -> String {
    let mut result = format!("{}{}", mv.from, mv.to);
    if let Some(promo) = mv.promotion {
        result.push(match promo {
            chess_core::PieceType::Queen => 'q',
            chess_core::PieceType::Rook => 'r',
            chess_core::PieceType::Bishop => 'b',
            chess_core::PieceType::Knight => 'n',
            _ => unreachable!(),
        });
    }
    result
}
