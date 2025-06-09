mod board;
mod evaluation;
mod fen;
mod game_state;
mod move_gen;
mod perft;
mod types;

use fen::positions;
use game_state::GameState;
use perft::{perft, perft_divide};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

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
                println!("Parsed FEN: {}", state.to_fen());
                // TODO: Add board display
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

        println!("Position: {}", state.to_fen());
        println!("Evaluation: {} cp", state.evaluate());
        println!(
            "(from {}'s perspective)",
            if state.turn == types::Color::White {
                "White"
            } else {
                "Black"
            }
        );
        println!(
            "Absolute eval: {} cp (+ = White, - = Black)",
            state.evaluate_absolute()
        );
    } else {
        println!("Chess engine");
        println!("Commands:");
        println!("  perft <depth> [fen]  - Run perft test");
        println!("  fen <fen_string>     - Parse and display FEN position");
        println!("  eval [fen]           - Evaluate position");
        println!("\nExample FEN positions:");
        println!("  Starting: {}", positions::STARTING);
        println!("  Kiwipete: {}", positions::KIWIPETE);
    }
}
