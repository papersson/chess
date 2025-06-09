mod board;
mod game_state;
mod move_gen;
mod perft;
mod types;

use game_state::GameState;
use perft::{perft, perft_divide};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "perft" {
        if args.len() < 3 {
            println!("Usage: {} perft <depth>", args[0]);
            return;
        }

        let depth: u8 = args[2].parse().unwrap_or(1);
        let state = GameState::new();

        println!("Running perft({}) on starting position...", depth);

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
    } else {
        println!("Chess engine");
        println!("Commands:");
        println!("  perft <depth>  - Run perft test");
    }
}
