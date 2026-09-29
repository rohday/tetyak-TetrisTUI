use crate::piece::Tetromino;
use std::time::Instant;

pub const WIDTH: usize = 10;
pub const HEIGHT: usize = 20;

#[derive(Debug, Clone)]
pub struct ExplosionState {
    pub rows: Vec<usize>,
    pub start_time: Instant,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct Board {
    grid: [[Option<Tetromino>; WIDTH]; HEIGHT],
    explosion: Option<ExplosionState>,
}

impl Board {
    pub fn new() -> Self {
        Self {
            grid: [[None; WIDTH]; HEIGHT],
            explosion: None,
        }
    }

    pub fn grid(&self) -> &[[Option<Tetromino>; WIDTH]; HEIGHT] {
        &self.grid
    }

    pub fn cell(&self, x: usize, y: usize) -> Option<Tetromino> {
        if x < WIDTH && y < HEIGHT {
            self.grid[y][x]
        } else {
            None
        }
    }

    pub fn set_cell(&mut self, x: usize, y: usize, val: Option<Tetromino>) {
        if x < WIDTH && y < HEIGHT {
            self.grid[y][x] = val;
        }
    }

    pub fn can_place(&self, piece: Tetromino, rotation: usize, x: i32, y: i32) -> bool {
        for (cx, cy) in piece.cells(rotation) {
            let wx = x + cx;
            let wy = y + cy;
            if wx < 0 || wx >= WIDTH as i32 || wy < 0 || wy >= HEIGHT as i32 {
                return false;
            }
            if self.grid[wy as usize][wx as usize].is_some() {
                return false;
            }
        }
        true
    }

    pub fn lock_piece(&mut self, piece: Tetromino, rotation: usize, x: i32, y: i32) {
        for (cx, cy) in piece.cells(rotation) {
            let wx = x + cx;
            let wy = y + cy;
            if wx >= 0 && wx < WIDTH as i32 && wy >= 0 && wy < HEIGHT as i32 {
                self.grid[wy as usize][wx as usize] = Some(piece);
            }
        }
    }

    pub fn check_full_lines(&self) -> Vec<usize> {
        let mut full = Vec::new();
        for y in 0..HEIGHT {
            if self.grid[y].iter().all(|cell| cell.is_some()) {
                full.push(y);
            }
        }
        full
    }

    pub fn clear_lines(&mut self, lines: &[usize]) {
        if lines.is_empty() {
            return;
        }
        let mut new_grid = [[None; WIDTH]; HEIGHT];
        let mut target_y = HEIGHT;
        for y in (0..HEIGHT).rev() {
            if !lines.contains(&y) {
                target_y -= 1;
                new_grid[target_y] = self.grid[y];
            }
        }
        self.grid = new_grid;
    }

    pub fn start_explosion(&mut self, rows: Vec<usize>, duration_ms: u64) {
        self.explosion = Some(ExplosionState {
            rows,
            start_time: Instant::now(),
            duration_ms,
        });
    }

    pub fn is_exploding(&self) -> bool {
        self.explosion.is_some()
    }

    pub fn exploding_rows(&self) -> Option<&[usize]> {
        self.explosion.as_ref().map(|e| e.rows.as_slice())
    }

    pub fn explosion_progress(&self) -> Option<f32> {
        self.explosion.as_ref().map(|e| {
            if e.duration_ms == 0 {
                1.0
            } else {
                let elapsed = e.start_time.elapsed().as_secs_f32() * 1000.0;
                (elapsed / e.duration_ms as f32).clamp(0.0, 1.0)
            }
        })
    }

    pub fn finish_explosion_if_done(&mut self) -> bool {
        if let Some(ref e) = self.explosion {
            if e.start_time.elapsed().as_millis() >= e.duration_ms as u128 {
                let rows = e.rows.clone();
                self.clear_lines(&rows);
                self.explosion = None;
                return true;
            }
        }
        false
    }

    pub fn clear(&mut self) {
        self.grid = [[None; WIDTH]; HEIGHT];
        self.explosion = None;
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}
