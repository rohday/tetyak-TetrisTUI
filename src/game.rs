use crate::board::Board;
pub use crate::piece::{Difficulty, Randomizer, Tetromino};
use std::time::Instant;

/// Tracks counts and labels for line clears ("blasts").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BlastStats {
    pub singles: u32,
    pub doubles: u32,
    pub triples: u32,
    pub tetrises: u32,
    pub last_blast: Option<(u8, &'static str)>,
}

/// Lifecycle status of the Tetris game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameStatus {
    #[default]
    DifficultySelect,
    Playing,
    Paused,
    GameOver,
}

/// Core game state managing board, pieces, pace, scoring, and input interactions.
#[derive(Debug, Clone)]
pub struct Game {
    pub board: Board,
    pub difficulty: Difficulty,
    pub randomizer: Randomizer,
    pub current_piece: Tetromino,
    pub current_rotation: usize,
    pub piece_x: i32,
    pub piece_y: i32,
    pub next_piece: Tetromino,
    pub held_piece: Option<Tetromino>,
    pub can_hold: bool,
    pub score: u64,
    pub lines_cleared: u32,
    pub pieces_locked: u32,
    pub blasts: BlastStats,
    pub status: GameStatus,
    pub start_time: Instant,
    pub last_fall_time: Instant,
    pub lock_delay_timer: Option<Instant>,
}

impl Game {
    /// Creates a new game instance centered at top spawn (x: 3, y: 0).
    pub fn new(difficulty: Difficulty) -> Self {
        let mut randomizer = Randomizer::new(difficulty);
        let current_piece = randomizer.next_piece();
        let next_piece = randomizer.next_piece();
        let now = Instant::now();

        Self {
            board: Board::new(),
            difficulty,
            randomizer,
            current_piece,
            current_rotation: 0,
            piece_x: 3,
            piece_y: 0,
            next_piece,
            held_piece: None,
            can_hold: true,
            score: 0,
            lines_cleared: 0,
            pieces_locked: 0,
            blasts: BlastStats::default(),
            status: GameStatus::DifficultySelect,
            start_time: now,
            last_fall_time: now,
            lock_delay_timer: None,
        }
    }

    /// Resets the game state with the given difficulty and starts playing.
    pub fn select_difficulty(&mut self, difficulty: Difficulty) {
        self.board.clear();
        self.difficulty = difficulty;
        self.randomizer = Randomizer::new(difficulty);
        self.current_piece = self.randomizer.next_piece();
        self.current_rotation = 0;
        self.piece_x = 3;
        self.piece_y = 0;
        self.next_piece = self.randomizer.next_piece();
        self.held_piece = None;
        self.can_hold = true;
        self.score = 0;
        self.lines_cleared = 0;
        self.pieces_locked = 0;
        self.blasts = BlastStats::default();
        self.status = GameStatus::Playing;
        let now = Instant::now();
        self.start_time = now;
        self.last_fall_time = now;
        self.lock_delay_timer = None;
    }

    /// Moves the active piece left by 1 column if unblocked.
    pub fn move_left(&mut self) -> bool {
        if self.status != GameStatus::Playing || self.board.is_exploding() {
            return false;
        }
        if self.board.can_place(
            self.current_piece,
            self.current_rotation,
            self.piece_x - 1,
            self.piece_y,
        ) {
            self.piece_x -= 1;
            self.lock_delay_timer = None;
            true
        } else {
            false
        }
    }

    /// Moves the active piece right by 1 column if unblocked.
    pub fn move_right(&mut self) -> bool {
        if self.status != GameStatus::Playing || self.board.is_exploding() {
            return false;
        }
        if self.board.can_place(
            self.current_piece,
            self.current_rotation,
            self.piece_x + 1,
            self.piece_y,
        ) {
            self.piece_x += 1;
            self.lock_delay_timer = None;
            true
        } else {
            false
        }
    }

    /// Rotates clockwise with wall kick offsets (0, -1, 1, -2, 2).
    pub fn rotate_cw(&mut self) -> bool {
        if self.status != GameStatus::Playing || self.board.is_exploding() {
            return false;
        }
        let new_rot = (self.current_rotation + 1) % 4;
        for offset_x in [0, -1, 1, -2, 2] {
            if self.board.can_place(
                self.current_piece,
                new_rot,
                self.piece_x + offset_x,
                self.piece_y,
            ) {
                self.current_rotation = new_rot;
                self.piece_x += offset_x;
                self.lock_delay_timer = None;
                return true;
            }
        }
        false
    }

    /// Rotates counter-clockwise with wall kick offsets (0, -1, 1, -2, 2).
    pub fn rotate_ccw(&mut self) -> bool {
        if self.status != GameStatus::Playing || self.board.is_exploding() {
            return false;
        }
        let new_rot = (self.current_rotation + 3) % 4;
        for offset_x in [0, -1, 1, -2, 2] {
            if self.board.can_place(
                self.current_piece,
                new_rot,
                self.piece_x + offset_x,
                self.piece_y,
            ) {
                self.current_rotation = new_rot;
                self.piece_x += offset_x;
                self.lock_delay_timer = None;
                return true;
            }
        }
        false
    }

    /// Moves the piece down by 1 row if unblocked, awarding 1 point.
    pub fn soft_drop(&mut self) -> bool {
        if self.status != GameStatus::Playing || self.board.is_exploding() {
            return false;
        }
        if self.board.can_place(
            self.current_piece,
            self.current_rotation,
            self.piece_x,
            self.piece_y + 1,
        ) {
            self.piece_y += 1;
            self.score += 1;
            self.last_fall_time = Instant::now();
            self.lock_delay_timer = None;
            true
        } else {
            false
        }
    }

    /// Hard drops the piece down to ghost_y, locks it immediately, and spawns the next piece.
    pub fn hard_drop(&mut self) {
        if self.status != GameStatus::Playing || self.board.is_exploding() {
            return;
        }
        let gy = self.ghost_y();
        let drop_dist = (gy - self.piece_y).max(0);
        self.score += (drop_dist as u64) * 2;
        self.piece_y = gy;
        self.lock_current_piece();
    }

    /// Swaps current piece and held piece (once per turn).
    pub fn hold(&mut self) -> bool {
        if self.status != GameStatus::Playing || !self.can_hold || self.board.is_exploding() {
            return false;
        }

        if let Some(prev_held) = self.held_piece {
            self.held_piece = Some(self.current_piece);
            self.current_piece = prev_held;
        } else {
            self.held_piece = Some(self.current_piece);
            self.current_piece = self.next_piece;
            self.next_piece = self.randomizer.next_piece();
        }

        self.current_rotation = 0;
        self.piece_x = 3;
        self.piece_y = 0;
        self.can_hold = false;
        self.lock_delay_timer = None;
        self.last_fall_time = Instant::now();

        if !self.board.can_place(
            self.current_piece,
            self.current_rotation,
            self.piece_x,
            self.piece_y,
        ) {
            self.status = GameStatus::GameOver;
        }

        true
    }

    /// Alias for `hold`.
    pub fn hold_piece(&mut self) -> bool {
        self.hold()
    }

    /// Calculates the lowest y coordinate the current piece can reach without collision.
    pub fn ghost_y(&self) -> i32 {
        let mut gy = self.piece_y;
        while self.board.can_place(
            self.current_piece,
            self.current_rotation,
            self.piece_x,
            gy + 1,
        ) {
            gy += 1;
        }
        gy
    }

    /// Drop interval computed strictly capped according to the current difficulty.
    pub fn current_drop_interval_ms(&self) -> u64 {
        self.difficulty.drop_interval_ms(self.lines_cleared)
    }

    /// Live pieces locked per minute pace.
    pub fn pieces_per_minute(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        (self.pieces_locked as f64 / elapsed.max(1.0)) * 60.0
    }

    /// Total pieces locked in this game session.
    pub fn total_pieces(&self) -> u32 {
        self.pieces_locked
    }

    /// Records single, double, triple, or tetris line clears with AppleTUI scores and blast badges.
    pub fn record_blast(&mut self, count: usize) {
        self.lines_cleared += count as u32;
        match count {
            1 => {
                self.blasts.singles += 1;
                self.blasts.last_blast = Some((1, "SINGLE"));
                self.score += 100;
            }
            2 => {
                self.blasts.doubles += 1;
                self.blasts.last_blast = Some((2, "DOUBLE"));
                self.score += 300;
            }
            3 => {
                self.blasts.triples += 1;
                self.blasts.last_blast = Some((3, "TRIPLE"));
                self.score += 500;
            }
            4 => {
                self.blasts.tetrises += 1;
                self.blasts.last_blast = Some((4, "TETRIS"));
                self.score += 800;
            }
            _ => {}
        }
    }

    /// Advances the game by one tick: handles line explosions, gravity fall, and lock delays.
    pub fn tick(&mut self) {
        if self.status != GameStatus::Playing {
            return;
        }

        if self.board.is_exploding() {
            self.board.finish_explosion_if_done();
            return;
        }

        // Check if piece has hit floor or obstacle below
        let can_move_down = self.board.can_place(
            self.current_piece,
            self.current_rotation,
            self.piece_x,
            self.piece_y + 1,
        );

        if !can_move_down {
            let delay_ms = self.difficulty.lock_delay_ms();
            let timer = match self.lock_delay_timer {
                Some(t) => t,
                None => {
                    let now = Instant::now();
                    self.lock_delay_timer = Some(now);
                    now
                }
            };

            if timer.elapsed().as_millis() >= delay_ms as u128 {
                self.lock_current_piece();
            }
        } else {
            self.lock_delay_timer = None;
            let interval = self.current_drop_interval_ms();
            if self.last_fall_time.elapsed().as_millis() >= interval as u128 {
                self.piece_y += 1;
                self.last_fall_time = Instant::now();
            }
        }
    }

    /// Toggles pause state between Playing and Paused.
    pub fn toggle_pause(&mut self) {
        match self.status {
            GameStatus::Playing => self.status = GameStatus::Paused,
            GameStatus::Paused => self.status = GameStatus::Playing,
            _ => {}
        }
    }

    /// Absolute board cell coordinates for the active falling piece.
    pub fn current_cells(&self) -> [(i32, i32); 4] {
        let mut cells = self.current_piece.cells(self.current_rotation);
        for cell in &mut cells {
            cell.0 += self.piece_x;
            cell.1 += self.piece_y;
        }
        cells
    }

    /// Absolute board cell coordinates for the ghost piece projection.
    pub fn ghost_cells(&self) -> [(i32, i32); 4] {
        let gy = self.ghost_y();
        let mut cells = self.current_piece.cells(self.current_rotation);
        for cell in &mut cells {
            cell.0 += self.piece_x;
            cell.1 += gy;
        }
        cells
    }

    /// Helper for testing speed curves without clearing individual lines.
    pub fn add_lines_cleared_for_test(&mut self, lines: u32) {
        self.lines_cleared += lines;
    }

    /// Helper to increment cleared line count.
    pub fn add_lines_cleared(&mut self, lines: u32) {
        self.lines_cleared += lines;
    }

    fn lock_current_piece(&mut self) {
        self.board.lock_piece(
            self.current_piece,
            self.current_rotation,
            self.piece_x,
            self.piece_y,
        );
        self.pieces_locked += 1;
        self.can_hold = true;
        self.lock_delay_timer = None;

        let full_lines = self.board.check_full_lines();
        if !full_lines.is_empty() {
            let count = full_lines.len();
            self.record_blast(count);
            if count <= 2 {
                self.board.clear_lines(&full_lines);
            } else {
                self.board.start_explosion(full_lines, 120);
            }
        }

        self.spawn_next_piece();
    }

    fn spawn_next_piece(&mut self) {
        self.current_piece = self.next_piece;
        self.next_piece = self.randomizer.next_piece();
        self.current_rotation = 0;
        self.piece_x = 3;
        self.piece_y = 0;
        self.last_fall_time = Instant::now();
        self.lock_delay_timer = None;

        if !self.board.can_place(
            self.current_piece,
            self.current_rotation,
            self.piece_x,
            self.piece_y,
        ) {
            self.status = GameStatus::GameOver;
        }
    }
}
