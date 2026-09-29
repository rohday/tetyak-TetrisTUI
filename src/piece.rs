use ratatui::style::Color;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static RNG_COUNTER: AtomicU64 = AtomicU64::new(0x853c49e6748fea9b);

fn generate_seed() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0xda3e39cb94b95bdb);
    let counter = RNG_COUNTER.fetch_add(0x9e3779b97f4a7c15, Ordering::Relaxed);
    let mut state = nanos ^ counter;
    if state == 0 {
        state = 0x123456789abcdef0;
    }
    // Splitmix64 mixing
    let mut z = state.wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tetromino {
    I = 0,
    O = 1,
    T = 2,
    S = 3,
    Z = 4,
    J = 5,
    L = 6,
}

impl Tetromino {
    pub const ALL: [Tetromino; 7] = [
        Tetromino::I,
        Tetromino::O,
        Tetromino::T,
        Tetromino::S,
        Tetromino::Z,
        Tetromino::J,
        Tetromino::L,
    ];

    /// Returns standard Tetris 4-cell relative coordinates for the given rotation (modulo 4).
    pub fn cells(&self, rotation: usize) -> [(i32, i32); 4] {
        let r = rotation % 4;
        match self {
            Tetromino::I => match r {
                0 => [(0, 1), (1, 1), (2, 1), (3, 1)],
                1 => [(2, 0), (2, 1), (2, 2), (2, 3)],
                2 => [(0, 2), (1, 2), (2, 2), (3, 2)],
                _ => [(1, 0), (1, 1), (1, 2), (1, 3)],
            },
            Tetromino::O => [(0, 0), (1, 0), (0, 1), (1, 1)],
            Tetromino::T => match r {
                0 => [(1, 0), (0, 1), (1, 1), (2, 1)],
                1 => [(1, 0), (1, 1), (2, 1), (1, 2)],
                2 => [(0, 1), (1, 1), (2, 1), (1, 2)],
                _ => [(1, 0), (0, 1), (1, 1), (1, 2)],
            },
            Tetromino::S => match r {
                0 => [(1, 0), (2, 0), (0, 1), (1, 1)],
                1 => [(1, 0), (1, 1), (2, 1), (2, 2)],
                2 => [(1, 1), (2, 1), (0, 2), (1, 2)],
                _ => [(0, 0), (0, 1), (1, 1), (1, 2)],
            },
            Tetromino::Z => match r {
                0 => [(0, 0), (1, 0), (1, 1), (2, 1)],
                1 => [(2, 0), (1, 1), (2, 1), (1, 2)],
                2 => [(0, 1), (1, 1), (1, 2), (2, 2)],
                _ => [(1, 0), (0, 1), (1, 1), (0, 2)],
            },
            Tetromino::J => match r {
                0 => [(0, 0), (0, 1), (1, 1), (2, 1)],
                1 => [(1, 0), (2, 0), (1, 1), (1, 2)],
                2 => [(0, 1), (1, 1), (2, 1), (2, 2)],
                _ => [(1, 0), (1, 1), (0, 2), (1, 2)],
            },
            Tetromino::L => match r {
                0 => [(2, 0), (0, 1), (1, 1), (2, 1)],
                1 => [(1, 0), (1, 1), (1, 2), (2, 2)],
                2 => [(0, 1), (1, 1), (2, 1), (0, 2)],
                _ => [(0, 0), (1, 0), (1, 1), (1, 2)],
            },
        }
    }

    /// Modern AppleTUI-inspired vibrant RGB color for the tetromino.
    pub fn color(&self) -> Color {
        match self {
            Tetromino::I => Color::Rgb(100, 210, 255), // Cyan
            Tetromino::O => Color::Rgb(255, 214, 10),  // Yellow
            Tetromino::T => Color::Rgb(175, 82, 222),  // Purple
            Tetromino::S => Color::Rgb(48, 209, 88),   // Green
            Tetromino::Z => Color::Rgb(255, 69, 58),   // Red
            Tetromino::J => Color::Rgb(10, 132, 255),  // Blue
            Tetromino::L => Color::Rgb(255, 159, 10),  // Orange
        }
    }

    pub fn from_index(index: usize) -> Self {
        Self::ALL[index % Self::ALL.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Chill,
    Normal,
    Intense,
}

impl Difficulty {
    pub fn initial_drop_ms(&self) -> u64 {
        match self {
            Difficulty::Chill => 800,
            Difficulty::Normal => 600,
            Difficulty::Intense => 350,
        }
    }

    pub fn speed_cap_ms(&self) -> u64 {
        match self {
            Difficulty::Chill => 350,
            Difficulty::Normal => 200,
            Difficulty::Intense => 110,
        }
    }

    pub fn lock_delay_ms(&self) -> u64 {
        match self {
            Difficulty::Chill => 500,
            Difficulty::Normal => 400,
            Difficulty::Intense => 250,
        }
    }

    /// (lines_per_step, delta_ms_reduction)
    pub fn acceleration_step(&self) -> (u32, u64) {
        match self {
            Difficulty::Chill => (5, 15),
            Difficulty::Normal => (4, 25),
            Difficulty::Intense => (3, 25),
        }
    }

    /// Drop interval computed from lines cleared, strictly capped at `speed_cap_ms`.
    pub fn drop_interval_ms(&self, lines_cleared: u32) -> u64 {
        let (step_lines, delta_ms) = self.acceleration_step();
        let steps = lines_cleared as u64 / step_lines as u64;
        let reduction = steps * delta_ms;
        self.initial_drop_ms()
            .saturating_sub(reduction)
            .max(self.speed_cap_ms())
    }
}

pub struct Randomizer {
    pub difficulty: Difficulty,
    pub last_piece: Option<Tetromino>,
    pub bag: Vec<Tetromino>,
    pub rng_state: u64,
}

impl Randomizer {
    pub fn new(difficulty: Difficulty) -> Self {
        let mut r = Self {
            difficulty,
            last_piece: None,
            bag: Vec::new(),
            rng_state: generate_seed(),
        };
        if difficulty == Difficulty::Chill {
            r.refill_chill_bag();
        }
        r
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.rng_state;
        if x == 0 {
            x = 0x853c49e6748fea9b;
        }
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state = x;
        x
    }

    fn random_piece(&mut self) -> Tetromino {
        let idx = (self.next_u64() % 7) as usize;
        Tetromino::ALL[idx]
    }

    fn refill_chill_bag(&mut self) {
        let mut bag = Vec::with_capacity(14);
        for piece in Tetromino::ALL {
            bag.push(piece);
            bag.push(piece);
        }

        let is_sz = |p: Tetromino| matches!(p, Tetromino::S | Tetromino::Z);

        // Try shuffling and check the consecutive S/Z constraint
        for _ in 0..200 {
            // Fisher-Yates shuffle
            for i in (1..bag.len()).rev() {
                let j = (self.next_u64() % ((i + 1) as u64)) as usize;
                bag.swap(i, j);
            }

            // Since we pop from the back, bag.last() will be the first piece drawn.
            let mut valid = true;
            if let Some(last) = self.last_piece {
                if is_sz(last) && is_sz(*bag.last().unwrap()) {
                    valid = false;
                }
            } else if is_sz(*bag.last().unwrap()) {
                // Initial game start: prefer non-S/Z first piece
                valid = false;
            }

            if valid {
                for i in 1..bag.len() {
                    if is_sz(bag[i]) && is_sz(bag[i - 1]) {
                        valid = false;
                        break;
                    }
                }
            }

            if valid {
                self.bag = bag;
                return;
            }
        }

        // Deterministic fallback: interleave non-SZ and SZ pieces
        let (sz, mut non_sz): (Vec<_>, Vec<_>) = bag.into_iter().partition(|&p| is_sz(p));
        let mut sz = sz;
        let mut resolved = Vec::with_capacity(14);
        while !non_sz.is_empty() || !sz.is_empty() {
            if let Some(n) = non_sz.pop() {
                resolved.push(n);
            }
            if let Some(n) = non_sz.pop() {
                resolved.push(n);
            }
            if let Some(s) = sz.pop() {
                resolved.push(s);
            }
        }
        self.bag = resolved;
    }

    pub fn next_piece(&mut self) -> Tetromino {
        match self.difficulty {
            Difficulty::Chill => {
                if self.bag.is_empty() {
                    self.refill_chill_bag();
                }
                let piece = self.bag.pop().unwrap();
                self.last_piece = Some(piece);
                piece
            }
            Difficulty::Normal => {
                let mut piece = self.random_piece();
                if let Some(last) = self.last_piece {
                    if piece == last {
                        piece = self.random_piece();
                    }
                }
                self.last_piece = Some(piece);
                piece
            }
            Difficulty::Intense => {
                let piece = self.random_piece();
                self.last_piece = Some(piece);
                piece
            }
        }
    }
}
