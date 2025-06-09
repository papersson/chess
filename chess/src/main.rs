mod uci;

use chess_agents::{iterative_deepening, search, search_with_limits, Evaluatable, SearchLimits};
use chess_core::{perft, perft_divide, positions, GameState, Color, PieceType, Square, Rank, File, Move, generate_legal_moves};
use std::env;
use std::io::{self, Write};

fn display_board(state: &GameState) {
    println!("\n  a b c d e f g h");
    println!("  ---------------");
    
    for rank_idx in (0..8).rev() {
        let rank = Rank::new(rank_idx).unwrap();
        print!("{} ", rank.to_char());
        
        for file_idx in 0..8 {
            let file = File::new(file_idx).unwrap();
            let square = Square::new(file, rank);
            
            if let Some(piece) = state.board.piece_at(square) {
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
                print!("{} ", symbol);
            } else {
                print!(". ");
            }
        }
        
        println!("| {}", rank.to_char());
    }
    
    println!("  ---------------");
    println!("  a b c d e f g h\n");
    
    // Show game state info
    println!("{} to move", if state.turn == Color::White { "White" } else { "Black" });
    
    if state.castling.white.any() || state.castling.black.any() {
        print!("Castling: ");
        if state.castling.white.kingside { print!("K"); }
        if state.castling.white.queenside { print!("Q"); }
        if state.castling.black.kingside { print!("k"); }
        if state.castling.black.queenside { print!("q"); }
        println!();
    }
    
    if let Some(ep) = state.en_passant {
        println!("En passant: {}", ep);
    }
    
    println!("Move {}", state.fullmove_number);
}

fn parse_move(state: &GameState, move_str: &str) -> Option<Move> {
    // Try to parse algebraic notation (e2e4, e7e8q)
    if move_str.len() >= 4 {
        let from_file = File::from_char(move_str.chars().nth(0)?)?;
        let from_rank = Rank::from_char(move_str.chars().nth(1)?)?;
        let to_file = File::from_char(move_str.chars().nth(2)?)?;
        let to_rank = Rank::from_char(move_str.chars().nth(3)?)?;
        
        let from = Square::new(from_file, from_rank);
        let to = Square::new(to_file, to_rank);
        
        // Check for promotion
        let promotion = if move_str.len() > 4 {
            match move_str.chars().nth(4)? {
                'q' | 'Q' => Some(PieceType::Queen),
                'r' | 'R' => Some(PieceType::Rook),
                'b' | 'B' => Some(PieceType::Bishop),
                'n' | 'N' => Some(PieceType::Knight),
                _ => None,
            }
        } else {
            None
        };
        
        let mv = if promotion.is_some() {
            Move::new_promotion(from, to, promotion.unwrap())
        } else {
            Move::new(from, to)
        };
        
        // Verify it's legal
        let legal_moves = generate_legal_moves(state);
        if legal_moves.iter().any(|&legal_mv| legal_mv == mv) {
            Some(mv)
        } else {
            None
        }
    } else {
        None
    }
}

fn play_interactive() {
    let mut state = GameState::new();
    let mut move_history = Vec::new();
    
    println!("Chess Engine - Interactive Mode");
    println!("Enter moves in algebraic notation (e.g., e2e4, e7e8q for promotion)");
    println!("Commands: 'quit', 'undo', 'new', 'help'");
    println!();
    
    loop {
        display_board(&state);
        
        // Check for game over
        let legal_moves = generate_legal_moves(&state);
        if legal_moves.is_empty() {
            if state.is_in_check() {
                println!("Checkmate! {} wins!", 
                    if state.turn == Color::White { "Black" } else { "White" });
            } else {
                println!("Stalemate!");
            }
            break;
        }
        
        if state.is_in_check() {
            println!("Check!");
        }
        
        // Get player move
        print!("Your move: ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        match input {
            "quit" => break,
            "help" => {
                println!("Enter moves like 'e2e4' or 'e7e8q' (for promotion to queen)");
                println!("Commands: quit, undo, new, help");
                continue;
            }
            "new" => {
                state = GameState::new();
                move_history.clear();
                println!("New game started!");
                continue;
            }
            "undo" => {
                if move_history.len() >= 2 {
                    move_history.pop();
                    move_history.pop();
                    state = GameState::new();
                    for mv in &move_history {
                        state = state.apply_move(*mv);
                    }
                    println!("Undid last move");
                } else {
                    println!("Nothing to undo");
                }
                continue;
            }
            _ => {}
        }
        
        // Parse and apply move
        match parse_move(&state, input) {
            Some(mv) => {
                state = state.apply_move(mv);
                move_history.push(mv);
                
                // Engine's turn
                display_board(&state);
                println!("Engine thinking...");
                
                let result = search_with_limits(&state, SearchLimits::move_time(2000));
                
                if let Some(engine_move) = result.best_move {
                    println!("Engine plays: {}", engine_move);
                    state = state.apply_move(engine_move);
                    move_history.push(engine_move);
                }
            }
            None => {
                println!("Invalid move. Try again (e.g., e2e4)");
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Check if running in UCI mode
    if args.len() > 1 && args[1] == "uci" {
        let mut engine = uci::UciEngine::new();
        engine.run();
        return;
    }

    if args.len() > 1 && args[1] == "perft" {
        if args.len() < 3 {
            println!("Usage: {} perft <depth> [fen]", args[0]);
            return;
        }

        let depth: u8 = args[2].parse().unwrap_or(1);

        // Parse optional FEN or use starting position
        let state = if args.len() > 3 {
            match GameState::from_fen(&args[3]) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error parsing FEN: {}", e);
                    return;
                }
            }
        } else {
            GameState::new()
        };

        println!("Running perft({})...", depth);
        println!("Position: {}", state.to_fen());

        if depth <= 3 {
            // Show move breakdown for shallow depths
            let results = perft_divide(&state, depth);
            let mut total = 0;

            for (mv, count) in &results {
                println!("{}: {}", mv, count);
                total += count;
            }

            println!("\nTotal: {}", total);
        } else {
            // Just show total for deeper depths
            let start = std::time::Instant::now();
            let nodes = perft(&state, depth);
            let elapsed = start.elapsed();

            println!("Nodes: {}", nodes);
            println!("Time: {:.2}s", elapsed.as_secs_f64());
            println!("NPS: {:.0}", nodes as f64 / elapsed.as_secs_f64());
        }
    } else if args.len() > 1 && args[1] == "fen" {
        // Display position from FEN
        if args.len() < 3 {
            println!("Usage: {} fen <fen_string>", args[0]);
            return;
        }

        match GameState::from_fen(&args[2]) {
            Ok(state) => {
                display_board(&state);
                println!("FEN: {}", state.to_fen());
            }
            Err(e) => eprintln!("Error parsing FEN: {}", e),
        }
    } else if args.len() > 1 && args[1] == "eval" {
        // Evaluate position
        let state = if args.len() > 2 {
            match GameState::from_fen(&args[2]) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error parsing FEN: {}", e);
                    return;
                }
            }
        } else {
            GameState::new()
        };

        display_board(&state);
        println!("Evaluation: {} cp", state.evaluate());
        println!(
            "(from {}'s perspective)",
            if state.turn == chess_core::Color::White {
                "White"
            } else {
                "Black"
            }
        );
        println!(
            "Absolute eval: {} cp (+ = White, - = Black)",
            state.evaluate_absolute()
        );
    } else if args.len() > 1 && args[1] == "search" {
        // Search for best move
        let (state, depth) = if args.len() > 2 {
            // Check if second arg is a number (depth) or FEN
            if let Ok(d) = args[2].parse::<u8>() {
                (GameState::new(), d)
            } else {
                // Try to parse as FEN
                match GameState::from_fen(&args[2]) {
                    Ok(s) => {
                        let d = if args.len() > 3 {
                            args[3].parse().unwrap_or(6)
                        } else {
                            6
                        };
                        (s, d)
                    }
                    Err(e) => {
                        eprintln!("Error parsing FEN: {}", e);
                        return;
                    }
                }
            }
        } else {
            (GameState::new(), 6)
        };

        println!("Position: {}", state.to_fen());
        println!("Searching to depth {}...", depth);

        let start = std::time::Instant::now();
        let result = if depth > 1 {
            iterative_deepening(&state, depth)
        } else {
            search(&state, depth)
        };
        let elapsed = start.elapsed();

        if let Some(best_move) = result.best_move {
            println!("\nBest move: {}", best_move);
            println!("Score: {} cp", result.score);
            println!("Depth: {}", result.depth);
            println!("Nodes: {}", result.nodes);
            println!("Time: {:.2}s", elapsed.as_secs_f64());
            println!("NPS: {:.0}", result.nodes as f64 / elapsed.as_secs_f64());
        } else {
            println!("No legal moves available");
        }
    } else if args.len() > 1 && args[1] == "movetime" {
        // Search with time limit
        let (state, millis) = if args.len() > 2 {
            // Check if second arg is a number (time) or FEN
            if let Ok(ms) = args[2].parse::<u64>() {
                (GameState::new(), ms)
            } else {
                // Try to parse as FEN
                match GameState::from_fen(&args[2]) {
                    Ok(s) => {
                        let ms = if args.len() > 3 {
                            args[3].parse().unwrap_or(1000)
                        } else {
                            1000
                        };
                        (s, ms)
                    }
                    Err(e) => {
                        eprintln!("Error parsing FEN: {}", e);
                        return;
                    }
                }
            }
        } else {
            (GameState::new(), 1000)
        };

        println!("Position: {}", state.to_fen());
        println!("Searching for {} ms...", millis);

        let start = std::time::Instant::now();
        let result = search_with_limits(&state, SearchLimits::move_time(millis));
        let elapsed = start.elapsed();

        if let Some(best_move) = result.best_move {
            println!("\nBest move: {}", best_move);
            println!("Score: {} cp", result.score);
            println!("Depth: {}", result.depth);
            println!("Nodes: {}", result.nodes);
            println!("Time: {:.2}s", elapsed.as_secs_f64());
            println!("NPS: {:.0}", result.nodes as f64 / elapsed.as_secs_f64());
            if result.stopped {
                println!("(search stopped by time limit)");
            }
        } else {
            println!("No legal moves available");
        }
    } else if args.len() > 1 && args[1] == "play" {
        play_interactive();
    } else {
        println!("Chess engine");
        println!("Commands:");
        println!("  play                 - Play against the engine");
        println!("  uci                  - Run in UCI mode for GUI compatibility");
        println!("  perft <depth> [fen]  - Run perft test");
        println!("  fen <fen_string>     - Parse and display FEN position");
        println!("  eval [fen]           - Evaluate position");
        println!("  search [depth|fen] [depth] - Search for best move");
        println!("  movetime [ms|fen] [ms] - Search with time limit (ms)");
        println!("\nExample FEN positions:");
        println!("  Starting: {}", positions::STARTING);
        println!("  Kiwipete: {}", positions::KIWIPETE);
    }
}
