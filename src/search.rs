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
pub struct SearchProgress {
    pub depth: u8,
    pub score: i32,
    pub nodes: u64,
    pub pv: Vec<Move>,
    pub time_ms: u64,
}

pub type InfoCallback = Box<dyn Fn(&SearchProgress) + Send>;

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
    info_callback: Option<InfoCallback>,
}

impl SearchInfo {
    fn new(limits: SearchLimits) -> Self {
        Self {
            start_time: Instant::now(),
            limits,
            nodes: 0,
            stopped: false,
            info_callback: None,
        }
    }

    fn with_callback(limits: SearchLimits, callback: InfoCallback) -> Self {
        Self {
            start_time: Instant::now(),
            limits,
            nodes: 0,
            stopped: false,
            info_callback: Some(callback),
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
    let mut info = SearchInfo::new(limits);
    search_internal(state, &mut info)
}

pub fn search_with_callback(
    state: &GameState,
    limits: SearchLimits,
    callback: InfoCallback,
) -> SearchResult {
    let mut info = SearchInfo::with_callback(limits, callback);
    search_internal(state, &mut info)
}

fn search_internal(state: &GameState, info: &mut SearchInfo) -> SearchResult {
    if let Some(max_depth) = info.limits.max_depth {
        // Fixed depth search
        let mut result = SearchResult {
            best_move: None,
            score: 0,
            depth: max_depth,
            nodes: 0,
            stopped: false,
        };

        let (score, best_move, _) = alpha_beta_root(state, max_depth, -INFINITY, INFINITY, info);

        result.score = score;
        result.best_move = best_move;
        result.nodes = info.nodes;
        result.stopped = info.stopped;
        result
    } else {
        // Iterative deepening with time control
        iterative_deepening_limits(state, info)
    }
}

fn alpha_beta_root(
    state: &GameState,
    depth: u8,
    mut alpha: i32,
    beta: i32,
    info: &mut SearchInfo,
) -> (i32, Option<Move>, Vec<Move>) {
    let moves = generate_legal_moves(state);
    if moves.is_empty() {
        if state.is_in_check() {
            return (
                -CHECKMATE_SCORE + i32::from(state.fullmove_number),
                None,
                vec![],
            );
        }
        return (0, None, vec![]);
    }

    let mut moves_vec: Vec<Move> = moves.iter().copied().collect();
    order_moves(state, &mut moves_vec);

    let mut best_move = None;
    let mut best_score = -INFINITY;
    let mut best_pv = vec![];

    for mv in &moves_vec {
        let new_state = state.apply_move(*mv);
        let (score, _, mut pv) = alpha_beta(&new_state, depth - 1, -beta, -alpha, info);
        let score = -score;

        if info.stopped {
            break;
        }

        if score > best_score {
            best_score = score;
            best_move = Some(*mv);
            best_pv = vec![*mv];
            best_pv.append(&mut pv);
        }

        if score > alpha {
            alpha = score;
        }

        if alpha >= beta {
            break;
        }
    }

    (best_score, best_move, best_pv)
}

fn alpha_beta(
    state: &GameState,
    depth: u8,
    mut alpha: i32,
    beta: i32,
    info: &mut SearchInfo,
) -> (i32, Option<Move>, Vec<Move>) {
    info.nodes += 1;

    // Check if we should stop searching
    if info.should_stop() {
        return (0, None, vec![]);
    }

    // Terminal node - return evaluation
    if depth == 0 {
        return (state.evaluate(), None, vec![]);
    }

    // Generate all legal moves
    let moves = generate_legal_moves(state);

    // No legal moves - checkmate or stalemate
    if moves.is_empty() {
        if state.is_in_check() {
            // Checkmate - return negative score (we're getting mated)
            return (
                -CHECKMATE_SCORE + i32::from(state.fullmove_number),
                None,
                vec![],
            );
        }
        // Stalemate
        return (0, None, vec![]);
    }

    // Convert to vector for sorting
    let mut moves_vec: Vec<Move> = moves.iter().copied().collect();

    // Order moves for better pruning (captures first)
    order_moves(state, &mut moves_vec);

    let mut best_move = None;
    let mut best_score = -INFINITY;
    let mut best_pv = vec![];

    for mv in &moves_vec {
        // Make move
        let new_state = state.apply_move(*mv);

        // Recursive search with negamax
        let (score, _, mut pv) = alpha_beta(&new_state, depth - 1, -beta, -alpha, info);
        let score = -score;

        // If search was stopped, return current best
        if info.stopped {
            return (best_score, best_move, best_pv);
        }

        if score > best_score {
            best_score = score;
            best_move = Some(*mv);
            best_pv = vec![*mv];
            best_pv.append(&mut pv);
        }

        if score > alpha {
            alpha = score;
        }

        // Beta cutoff
        if alpha >= beta {
            break;
        }
    }

    (best_score, best_move, best_pv)
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
        let _depth_start = info.start_time.elapsed();
        let (score, best_move, pv) = alpha_beta_root(state, depth, -INFINITY, INFINITY, info);

        // Only update result if we completed this depth
        if !info.stopped && best_move.is_some() {
            best_result.best_move = best_move;
            best_result.score = score;
            best_result.depth = depth;
            best_result.nodes = info.nodes;

            // Send info to callback if present
            if let Some(ref callback) = info.info_callback {
                let progress = SearchProgress {
                    depth,
                    score,
                    nodes: info.nodes,
                    pv: pv.clone(),
                    time_ms: info.start_time.elapsed().as_millis() as u64,
                };
                callback(&progress);
            }

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
