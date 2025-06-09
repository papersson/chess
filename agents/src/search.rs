use crate::evaluation::Evaluatable;
use crate::transposition::{NodeType, TranspositionTable};
use chess_core::{generate_legal_moves, GameState, Move};
use std::sync::Arc;
use std::time::{Duration, Instant};

const INFINITY: i32 = 1_000_000;
const CHECKMATE_SCORE: i32 = 100_000;
const TIME_CHECK_INTERVAL: u64 = 1000; // Check time every 1000 nodes
const QUIESCENCE_DEPTH: i8 = 4; // Maximum depth for quiescence search

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
    pub white_time: Option<Duration>,
    pub black_time: Option<Duration>,
    pub white_increment: Option<Duration>,
    pub black_increment: Option<Duration>,
    pub moves_to_go: Option<u32>,
}

impl SearchLimits {
    pub fn depth(depth: u8) -> Self {
        Self {
            max_depth: Some(depth),
            move_time: None,
            nodes: None,
            white_time: None,
            black_time: None,
            white_increment: None,
            black_increment: None,
            moves_to_go: None,
        }
    }

    pub fn move_time(millis: u64) -> Self {
        Self {
            max_depth: None,
            move_time: Some(Duration::from_millis(millis)),
            nodes: None,
            white_time: None,
            black_time: None,
            white_increment: None,
            black_increment: None,
            moves_to_go: None,
        }
    }

    pub fn time_control(
        white_time: Duration,
        black_time: Duration,
        white_inc: Duration,
        black_inc: Duration,
        moves_to_go: Option<u32>,
    ) -> Self {
        Self {
            max_depth: None,
            move_time: None,
            nodes: None,
            white_time: Some(white_time),
            black_time: Some(black_time),
            white_increment: Some(white_inc),
            black_increment: Some(black_inc),
            moves_to_go,
        }
    }
}

struct SearchInfo {
    start_time: Instant,
    limits: SearchLimits,
    nodes: u64,
    stopped: bool,
    info_callback: Option<InfoCallback>,
    tt: Arc<TranspositionTable>,
    quiescence_depth: i8,
}

impl SearchInfo {
    fn new(limits: SearchLimits, tt: Arc<TranspositionTable>) -> Self {
        Self {
            start_time: Instant::now(),
            limits,
            nodes: 0,
            stopped: false,
            info_callback: None,
            tt,
            quiescence_depth: QUIESCENCE_DEPTH,
        }
    }

    fn with_callback(
        limits: SearchLimits,
        callback: InfoCallback,
        tt: Arc<TranspositionTable>,
    ) -> Self {
        Self {
            start_time: Instant::now(),
            limits,
            nodes: 0,
            stopped: false,
            info_callback: Some(callback),
            tt,
            quiescence_depth: QUIESCENCE_DEPTH,
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

pub fn search_with_tt_size(
    state: &GameState,
    limits: SearchLimits,
    tt_size_mb: usize,
) -> SearchResult {
    let tt = Arc::new(TranspositionTable::new(tt_size_mb));
    let mut info = SearchInfo::new(limits, tt);
    search_internal(state, &mut info)
}

pub fn search_with_limits(state: &GameState, limits: SearchLimits) -> SearchResult {
    let tt = Arc::new(TranspositionTable::new(16)); // 16 MB default
    let mut info = SearchInfo::new(limits, tt);
    search_internal(state, &mut info)
}

pub fn search_with_callback(
    state: &GameState,
    limits: SearchLimits,
    callback: InfoCallback,
) -> SearchResult {
    let tt = Arc::new(TranspositionTable::new(16)); // 16 MB default
    let mut info = SearchInfo::with_callback(limits, callback, tt);
    search_internal(state, &mut info)
}

pub fn search_with_options(
    state: &GameState,
    limits: SearchLimits,
    tt_size_mb: usize,
    quiescence_depth: i8,
) -> SearchResult {
    let tt = Arc::new(TranspositionTable::new(tt_size_mb));
    let mut info = SearchInfo::new(limits, tt);
    info.quiescence_depth = quiescence_depth;
    search_internal(state, &mut info)
}

fn allocate_time(limits: &SearchLimits, state: &GameState) -> Option<Duration> {
    // If explicit move time is set, use it
    if let Some(move_time) = limits.move_time {
        return Some(move_time);
    }

    // Get time for the side to move
    let (our_time, our_inc) = match state.side_to_move() {
        chess_core::Color::White => (
            limits.white_time?,
            limits.white_increment.unwrap_or(Duration::from_millis(0)),
        ),
        chess_core::Color::Black => (
            limits.black_time?,
            limits.black_increment.unwrap_or(Duration::from_millis(0)),
        ),
    };

    let our_time_ms = our_time.as_millis() as u64;
    let our_inc_ms = our_inc.as_millis() as u64;

    // Estimate moves remaining in the game
    let moves_left = if let Some(mtg) = limits.moves_to_go {
        mtg as u64
    } else {
        // Estimate based on game phase (40 moves total, 20 per side on average)
        let phase_moves = match state.fullmove_number {
            1..=10 => 30,  // Opening: expect 30 more moves
            11..=30 => 20, // Middle game: expect 20 more moves
            _ => 10,       // Endgame: expect 10 more moves
        };
        phase_moves
    };

    // Basic time allocation formula
    // Use more time if we have increment, less if in time pressure
    let base_time = our_time_ms / moves_left;
    let increment_bonus = our_inc_ms * 8 / 10; // Use 80% of increment

    // Add safety margin - never use more than 95% of remaining time
    let max_time = our_time_ms * 95 / 100;
    let allocated = (base_time + increment_bonus).min(max_time);

    // Minimum time (don't think less than 50ms)
    let final_time = allocated.max(50);

    Some(Duration::from_millis(final_time))
}

fn search_internal(state: &GameState, info: &mut SearchInfo) -> SearchResult {
    // Calculate time allocation if using time control
    if info.limits.white_time.is_some() && info.limits.black_time.is_some() {
        if let Some(allocated_time) = allocate_time(&info.limits, state) {
            info.limits.move_time = Some(allocated_time);
        }
    }
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

    let original_alpha = alpha;
    let hash = state.zobrist_hash();
    let mut tt_move = None;

    // Probe transposition table
    if let Some(entry) = info.tt.probe(hash) {
        if entry.depth >= depth {
            // Can we use the stored score?
            match entry.node_type {
                NodeType::Exact => {
                    // Exact score - we can return immediately
                    return (
                        entry.score,
                        entry.best_move,
                        vec![entry.best_move.unwrap_or(Move::new(
                            chess_core::Square::from_index(0).unwrap(),
                            chess_core::Square::from_index(0).unwrap(),
                        ))],
                    );
                }
                NodeType::LowerBound => {
                    // Score is at least entry.score
                    alpha = alpha.max(entry.score);
                }
                NodeType::UpperBound => {
                    // Score is at most entry.score
                    // We can use beta cutoff instead of modifying beta
                    if entry.score <= alpha {
                        return (entry.score, entry.best_move, vec![]);
                    }
                }
            }

            // Alpha-beta cutoff
            if alpha >= beta {
                return (entry.score, entry.best_move, vec![]);
            }
        }
        // Save the best move from TT for move ordering
        tt_move = entry.best_move;
    }

    // Terminal node - enter quiescence search
    if depth == 0 {
        let score = quiescence(state, info.quiescence_depth, alpha, beta, info);
        info.tt.store(hash, None, score, 0, NodeType::Exact);
        return (score, None, vec![]);
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

    // Order moves for better pruning (TT move first, then captures)
    order_moves_with_tt(state, &mut moves_vec, tt_move);

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

    // Store in transposition table
    let node_type = if best_score <= original_alpha {
        NodeType::UpperBound
    } else if best_score >= beta {
        NodeType::LowerBound
    } else {
        NodeType::Exact
    };

    info.tt.store(hash, best_move, best_score, depth, node_type);

    (best_score, best_move, best_pv)
}

fn quiescence(
    state: &GameState,
    depth: i8,
    mut alpha: i32,
    beta: i32,
    info: &mut SearchInfo,
) -> i32 {
    info.nodes += 1;

    // Check if we should stop searching
    if info.should_stop() {
        return 0;
    }

    // Stand pat evaluation - can we beat alpha without searching?
    let stand_pat = state.evaluate();

    if stand_pat >= beta {
        return beta;
    }

    if alpha < stand_pat {
        alpha = stand_pat;
    }

    // Depth limit for quiescence search
    if depth <= 0 {
        return stand_pat;
    }

    // Generate all legal moves
    let moves = generate_legal_moves(state);

    // Filter to only captures and promotions
    let mut capture_moves: Vec<Move> = moves
        .iter()
        .copied()
        .filter(|mv| {
            // Is this a capture?
            state.board.piece_at(mv.to).is_some() ||
            // Is this a promotion?
            mv.promotion.is_some() ||
            // Is this an en passant capture?
            (state.board.piece_at(mv.from).map(|p| p.piece_type == chess_core::PieceType::Pawn).unwrap_or(false) &&
             Some(mv.to) == state.en_passant)
        })
        .collect();

    // If no captures, return stand pat
    if capture_moves.is_empty() {
        return stand_pat;
    }

    // Order captures by MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
    order_captures(state, &mut capture_moves);

    for mv in capture_moves {
        let new_state = state.apply_move(mv);
        let score = -quiescence(&new_state, depth - 1, -beta, -alpha, info);

        if info.stopped {
            return alpha;
        }

        if score >= beta {
            return beta;
        }

        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

fn order_captures(state: &GameState, moves: &mut [Move]) {
    moves.sort_by_cached_key(|mv| {
        let mut score = 0;

        // Get victim value (what we're capturing)
        if let Some(victim) = state.board.piece_at(mv.to) {
            score -= victim.piece_type.value() as i32 * 10;
        }

        // Get attacker value (prefer capturing with less valuable pieces)
        if let Some(attacker) = state.board.piece_at(mv.from) {
            score += attacker.piece_type.value() as i32;
        }

        // Promotions are also valuable
        if let Some(promo) = mv.promotion {
            score -= promo.value() as i32 * 10;
        }

        score
    });
}

fn order_moves(state: &GameState, moves: &mut [Move]) {
    order_moves_with_tt(state, moves, None);
}

fn order_moves_with_tt(state: &GameState, moves: &mut [Move], tt_move: Option<Move>) {
    // Move ordering: TT move first, then captures, then promotions
    moves.sort_by_cached_key(|mv| {
        let mut score = 0;

        // TT move gets highest priority
        if tt_move == Some(*mv) {
            return -10000;
        }

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
