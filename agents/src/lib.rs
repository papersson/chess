pub mod evaluation;
pub mod minimax;
pub mod random;
pub mod search;

use chess_core::{GameState, Move};

/// Core trait for chess agents
pub trait Agent {
    /// Get the best move for the current position
    fn best_move(&mut self, state: &GameState) -> Option<Move>;

    /// Get the agent's name
    fn name(&self) -> &str;
}

/// Position evaluation trait for agents that need it
pub trait Position {
    fn evaluate(&self) -> i32;
    fn generate_moves(&self) -> Vec<Move>;
    fn make_move(&self, m: Move) -> Self;
}

pub use evaluation::*;
pub use minimax::MinimaxAgent;
pub use random::RandomAgent;
pub use search::*;
