use crate::Agent;
use chess_core::{generate_legal_moves, GameState, Move};
use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct RandomAgent {
    name: String,
}

impl RandomAgent {
    pub fn new() -> Self {
        RandomAgent {
            name: "Random".to_string(),
        }
    }
}

impl Default for RandomAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl Agent for RandomAgent {
    fn best_move(&mut self, state: &GameState) -> Option<Move> {
        let moves = generate_legal_moves(state);

        if moves.is_empty() {
            None
        } else {
            let mut rng = thread_rng();
            let move_vec: Vec<Move> = moves.iter().copied().collect();
            move_vec.choose(&mut rng).copied()
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}
