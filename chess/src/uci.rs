use chess_agents::{search_with_callback_and_stop, SearchLimits, SearchProgress};
use chess_core::{GameState, Move};
use std::io::{self, BufRead, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct UciEngine {
    position: GameState,
    debug: bool,
    stop_flag: Arc<AtomicBool>,
    search_thread: Option<thread::JoinHandle<()>>,
}

impl UciEngine {
    pub fn new() -> Self {
        Self {
            position: GameState::new(),
            debug: false,
            stop_flag: Arc::new(AtomicBool::new(false)),
            search_thread: None,
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
                    self.handle_stop();
                }
                "quit" => {
                    self.handle_stop(); // Stop any ongoing search
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

    fn handle_go(&mut self, parts: &[&str]) {
        let mut limits = SearchLimits {
            max_depth: None,
            move_time: None,
            nodes: None,
            white_time: None,
            black_time: None,
            white_increment: None,
            black_increment: None,
            moves_to_go: None,
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
                "wtime" => {
                    if idx + 1 < parts.len() {
                        if let Ok(ms) = parts[idx + 1].parse::<u64>() {
                            limits.white_time = Some(Duration::from_millis(ms));
                        }
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "btime" => {
                    if idx + 1 < parts.len() {
                        if let Ok(ms) = parts[idx + 1].parse::<u64>() {
                            limits.black_time = Some(Duration::from_millis(ms));
                        }
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "winc" => {
                    if idx + 1 < parts.len() {
                        if let Ok(ms) = parts[idx + 1].parse::<u64>() {
                            limits.white_increment = Some(Duration::from_millis(ms));
                        }
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "binc" => {
                    if idx + 1 < parts.len() {
                        if let Ok(ms) = parts[idx + 1].parse::<u64>() {
                            limits.black_increment = Some(Duration::from_millis(ms));
                        }
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "movestogo" => {
                    if idx + 1 < parts.len() {
                        if let Ok(mtg) = parts[idx + 1].parse::<u32>() {
                            limits.moves_to_go = Some(mtg);
                        }
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                _ => idx += 1,
            }
        }

        // Default to depth 6 if no limits specified (including time control)
        if limits.max_depth.is_none()
            && limits.move_time.is_none()
            && limits.nodes.is_none()
            && limits.white_time.is_none()
            && limits.black_time.is_none()
        {
            limits.max_depth = Some(6);
        }

        // Wait for any previous search to finish
        if let Some(thread) = self.search_thread.take() {
            self.stop_flag.store(true, Ordering::Relaxed);
            let _ = thread.join();
        }

        // Reset stop flag for new search
        self.stop_flag.store(false, Ordering::Relaxed);

        // Clone necessary data for the search thread
        let position = self.position.clone();
        let stop_flag = Arc::clone(&self.stop_flag);

        // Spawn search thread
        let search_thread = thread::spawn(move || {
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

            let result = search_with_callback_and_stop(&position, limits, callback, stop_flag);

            // Output result
            if let Some(best_move) = result.best_move {
                println!("bestmove {}", format_move_static(best_move));
            } else {
                println!("bestmove 0000"); // Null move (no legal moves)
            }
            io::stdout().flush().unwrap();
        });

        self.search_thread = Some(search_thread);
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

    fn handle_stop(&mut self) {
        // Set the stop flag
        self.stop_flag.store(true, Ordering::Relaxed);

        // Wait for search thread to finish
        if let Some(thread) = self.search_thread.take() {
            let _ = thread.join();
        }
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
