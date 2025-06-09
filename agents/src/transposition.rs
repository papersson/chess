use chess_core::Move;
use std::sync::atomic::{AtomicU64, Ordering};

/// Type of node in the search tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    /// Exact score (PV-node)
    Exact,
    /// Lower bound (fail-high node)
    LowerBound,
    /// Upper bound (fail-low node)
    UpperBound,
}

/// Entry in the transposition table.
#[derive(Debug, Clone, Copy)]
pub struct TranspositionEntry {
    /// Zobrist hash of the position (for collision detection)
    pub hash: u64,
    /// Best move found at this position
    pub best_move: Option<Move>,
    /// Evaluation score
    pub score: i32,
    /// Search depth
    pub depth: u8,
    /// Type of node (exact, lower bound, upper bound)
    pub node_type: NodeType,
    /// Age of the entry (for replacement)
    pub age: u8,
}

impl Default for TranspositionEntry {
    fn default() -> Self {
        Self {
            hash: 0,
            best_move: None,
            score: 0,
            depth: 0,
            node_type: NodeType::Exact,
            age: 0,
        }
    }
}

/// Transposition table for caching search results.
pub struct TranspositionTable {
    /// Table entries
    entries: Vec<AtomicU64>,
    /// Size mask (size must be power of 2)
    size_mask: usize,
    /// Current search generation
    generation: u8,
}

impl TranspositionTable {
    /// Creates a new transposition table with the given size in MB.
    pub fn new(size_mb: usize) -> Self {
        // Each entry is 16 bytes (packed into 2 u64s)
        let entries_per_mb = (1024 * 1024) / 16;
        let num_entries = size_mb * entries_per_mb;

        // Round down to nearest power of 2
        let size = num_entries.next_power_of_two() / 2;
        let size_mask = size - 1;

        let mut entries = Vec::with_capacity(size * 2);
        for _ in 0..size * 2 {
            entries.push(AtomicU64::new(0));
        }

        Self {
            entries,
            size_mask,
            generation: 0,
        }
    }

    /// Stores an entry in the transposition table.
    pub fn store(
        &self,
        hash: u64,
        best_move: Option<Move>,
        score: i32,
        depth: u8,
        node_type: NodeType,
    ) {
        let index = (hash as usize & self.size_mask) * 2;

        // Pack the entry into two u64 values
        let entry = TranspositionEntry {
            hash,
            best_move,
            score,
            depth,
            node_type,
            age: self.generation,
        };

        let (packed1, packed2) = Self::pack_entry(&entry);

        // Atomic store
        self.entries[index].store(packed1, Ordering::Relaxed);
        self.entries[index + 1].store(packed2, Ordering::Relaxed);
    }

    /// Probes the transposition table for a position.
    pub fn probe(&self, hash: u64) -> Option<TranspositionEntry> {
        let index = (hash as usize & self.size_mask) * 2;

        // Atomic load
        let packed1 = self.entries[index].load(Ordering::Relaxed);
        let packed2 = self.entries[index + 1].load(Ordering::Relaxed);

        let entry = Self::unpack_entry(packed1, packed2);

        // Verify hash matches (collision detection)
        if entry.hash == hash {
            Some(entry)
        } else {
            None
        }
    }

    /// Clears the transposition table.
    pub fn clear(&mut self) {
        for entry in &self.entries {
            entry.store(0, Ordering::Relaxed);
        }
        self.generation = 0;
    }

    /// Advances to the next search generation.
    pub fn new_search(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    /// Packs an entry into two u64 values.
    fn pack_entry(entry: &TranspositionEntry) -> (u64, u64) {
        // First u64: full hash
        let packed1 = entry.hash;

        // Second u64: move (16 bits) + score (16 bits) + depth (8 bits) +
        //              node_type (2 bits) + age (8 bits) + reserved (14 bits)
        let mut packed2 = 0u64;

        // Pack move (16 bits)
        if let Some(mv) = entry.best_move {
            let from = mv.from.index() as u64;
            let to = mv.to.index() as u64;
            let promo = match mv.promotion {
                None => 0,
                Some(chess_core::PieceType::Queen) => 1,
                Some(chess_core::PieceType::Rook) => 2,
                Some(chess_core::PieceType::Bishop) => 3,
                Some(chess_core::PieceType::Knight) => 4,
                _ => 0,
            };
            packed2 |= (from << 10) | (to << 4) | promo;
        }

        // Pack score (16 bits, offset by 32768 to handle negative values)
        let score_bits = ((entry.score + 32768) as u64) & 0xFFFF;
        packed2 |= score_bits << 16;

        // Pack depth (8 bits)
        packed2 |= (entry.depth as u64) << 32;

        // Pack node type (2 bits)
        let node_type_bits = match entry.node_type {
            NodeType::Exact => 0,
            NodeType::LowerBound => 1,
            NodeType::UpperBound => 2,
        };
        packed2 |= node_type_bits << 40;

        // Pack age (8 bits)
        packed2 |= (entry.age as u64) << 42;

        (packed1, packed2)
    }

    /// Unpacks two u64 values into an entry.
    fn unpack_entry(packed1: u64, packed2: u64) -> TranspositionEntry {
        let hash = packed1;

        // Unpack move
        let move_bits = packed2 & 0xFFFF;
        let best_move = if move_bits != 0 {
            let from = chess_core::Square::from_index(((move_bits >> 10) & 0x3F) as u8).unwrap();
            let to = chess_core::Square::from_index(((move_bits >> 4) & 0x3F) as u8).unwrap();
            let promo = match move_bits & 0xF {
                1 => Some(chess_core::PieceType::Queen),
                2 => Some(chess_core::PieceType::Rook),
                3 => Some(chess_core::PieceType::Bishop),
                4 => Some(chess_core::PieceType::Knight),
                _ => None,
            };
            if let Some(p) = promo {
                Some(Move::new_promotion(from, to, p))
            } else {
                Some(Move::new(from, to))
            }
        } else {
            None
        };

        // Unpack score
        let score_bits = (packed2 >> 16) & 0xFFFF;
        let score = (score_bits as i32) - 32768;

        // Unpack depth
        let depth = ((packed2 >> 32) & 0xFF) as u8;

        // Unpack node type
        let node_type = match (packed2 >> 40) & 0x3 {
            0 => NodeType::Exact,
            1 => NodeType::LowerBound,
            2 => NodeType::UpperBound,
            _ => NodeType::Exact,
        };

        // Unpack age
        let age = ((packed2 >> 42) & 0xFF) as u8;

        TranspositionEntry {
            hash,
            best_move,
            score,
            depth,
            node_type,
            age,
        }
    }
}
