use crate::game_state::GameState;
use crate::move_gen::generate_legal_moves;
use crate::types::Move;
use std::time::{Duration, Instant};

const INFINITY: i32 = 1_000_000;
const CHECKMATE_SCORE: i32 = 100_000;
const TIME_CHECK_INTERVAL: u64 = 1000; // Check time every 1000 nodes

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score: i32,
    pub depth: u8,
    pub nodes: u64,
    pub stopped: bool,
}

#[derive(Debug, Clone)]
pub struct SearchLimits {
    pub max_depth: Option<u8>,
    pub move_time: Option<Duration>,
    pub nodes: Option<u64>,
}

impl SearchLimits {
    pub fn depth(depth: u8) -> Self {
        Self {
            max_depth: Some(depth),
            move_time: None,
            nodes: None,
        }
    }

    pub fn move_time(millis: u64) -> Self {
        Self {
            max_depth: None,
            move_time: Some(Duration::from_millis(millis)),
            nodes: None,
        }
    }
}

struct SearchInfo {
    start_time: Instant,
    limits: SearchLimits,
    nodes: u64,
    stopped: bool,
}

impl SearchInfo {
    fn new(limits: SearchLimits) -> Self {
        Self {
            start_time: Instant::now(),
            limits,
            nodes: 0,
            stopped: false,
        }
    }

    fn should_stop(&mut self) -> bool {
        if self.stopped {
            return true;
        }

        // Check node limit
        if let Some(max_nodes) = self.limits.nodes {
            if self.nodes >= max_nodes {
                self.stopped = true;
                return true;
            }
        }

        // Check time limit periodically
        if self.nodes % TIME_CHECK_INTERVAL == 0 {
            if let Some(move_time) = self.limits.move_time {
                if self.start_time.elapsed() >= move_time {
                    self.stopped = true;
                    return true;
                }
            }
        }

        false
    }
}

pub fn search(state: &GameState, depth: u8) -> SearchResult {
    search_with_limits(state, SearchLimits::depth(depth))
}

pub fn search_with_limits(state: &GameState, limits: SearchLimits) -> SearchResult {
    let mut info = SearchInfo::new(limits.clone());

    if let Some(max_depth) = limits.max_depth {
        // Fixed depth search
        let mut result = SearchResult {
            best_move: None,
            score: 0,
            depth: max_depth,
            nodes: 0,
            stopped: false,
        };

        let (score, best_move) = alpha_beta(state, max_depth, -INFINITY, INFINITY, &mut info);

        result.score = score;
        result.best_move = best_move;
        result.nodes = info.nodes;
        result.stopped = info.stopped;
        result
    } else {
        // Iterative deepening with time control
        iterative_deepening_limits(state, &mut info)
    }
}

fn alpha_beta(
    state: &GameState,
    depth: u8,
    mut alpha: i32,
    beta: i32,
    info: &mut SearchInfo,
) -> (i32, Option<Move>) {
    info.nodes += 1;

    // Check if we should stop searching
    if info.should_stop() {
        return (0, None);
    }

    // Terminal node - return evaluation
    if depth == 0 {
        return (state.evaluate(), None);
    }

    // Generate all legal moves
    let moves = generate_legal_moves(state);

    // No legal moves - checkmate or stalemate
    if moves.is_empty() {
        if state.is_in_check() {
            // Checkmate - return negative score (we're getting mated)
            return (-CHECKMATE_SCORE + (state.fullmove_number as i32), None);
        }
        // Stalemate
        return (0, None);
    }

    // Convert to vector for sorting
    let mut moves_vec: Vec<Move> = moves.iter().copied().collect();

    // Order moves for better pruning (captures first)
    order_moves(state, &mut moves_vec);

    let mut best_move = None;
    let mut best_score = -INFINITY;

    for mv in &moves_vec {
        // Make move
        let new_state = state.apply_move(*mv);

        // Recursive search with negamax
        let (score, _) = alpha_beta(&new_state, depth - 1, -beta, -alpha, info);
        let score = -score;

        // If search was stopped, return current best
        if info.stopped {
            return (best_score, best_move);
        }

        if score > best_score {
            best_score = score;
            best_move = Some(*mv);
        }

        if score > alpha {
            alpha = score;
        }

        // Beta cutoff
        if alpha >= beta {
            break;
        }
    }

    (best_score, best_move)
}

fn order_moves(state: &GameState, moves: &mut [Move]) {
    // Simple move ordering: captures first
    // In the future, we can add:
    // - MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
    // - Killer moves
    // - History heuristic
    // - Hash move

    moves.sort_by_cached_key(|mv| {
        let mut score = 0;

        // Prioritize captures
        if state.board.piece_at(mv.to).is_some() {
            score -= 1000;
        }

        // Prioritize promotions
        if mv.promotion.is_some() {
            score -= 900;
        }

        score
    });
}

pub fn iterative_deepening(state: &GameState, max_depth: u8) -> SearchResult {
    search_with_limits(state, SearchLimits::depth(max_depth))
}

fn iterative_deepening_limits(state: &GameState, info: &mut SearchInfo) -> SearchResult {
    let mut best_result = SearchResult {
        best_move: None,
        score: 0,
        depth: 0,
        nodes: 0,
        stopped: false,
    };

    // Search to increasing depths until time runs out
    for depth in 1..=100 {
        let saved_nodes = info.nodes;
        let (score, best_move) = alpha_beta(state, depth, -INFINITY, INFINITY, info);

        // Only update result if we completed this depth
        if !info.stopped && best_move.is_some() {
            best_result.best_move = best_move;
            best_result.score = score;
            best_result.depth = depth;
            best_result.nodes = info.nodes;

            // Stop if we found checkmate
            if score.abs() >= CHECKMATE_SCORE - 100 {
                break;
            }
        } else {
            // Restore node count if search was interrupted
            info.nodes = saved_nodes;
            break;
        }
    }

    best_result.stopped = info.stopped;
    best_result
}
