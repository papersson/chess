use crate::{
    search::{search_with_limits, SearchLimits},
    Agent,
};
use chess_core::{GameState, Move};

pub struct MinimaxAgent {
    name: String,
    depth: u8,
    time_limit_ms: Option<u64>,
}

impl MinimaxAgent {
    pub fn new(depth: u8) -> Self {
        MinimaxAgent {
            name: format!("Minimax(depth={})", depth),
            depth,
            time_limit_ms: None,
        }
    }

    pub fn with_time_limit(time_ms: u64) -> Self {
        MinimaxAgent {
            name: format!("Minimax(time={}ms)", time_ms),
            depth: 99, // Will be limited by time
            time_limit_ms: Some(time_ms),
        }
    }
}

impl Agent for MinimaxAgent {
    fn best_move(&mut self, state: &GameState) -> Option<Move> {
        let limits = if let Some(time_ms) = self.time_limit_ms {
            SearchLimits::move_time(time_ms)
        } else {
            SearchLimits::depth(self.depth)
        };

        let result = search_with_limits(state, limits);
        result.best_move
    }

    fn name(&self) -> &str {
        &self.name
    }
}
