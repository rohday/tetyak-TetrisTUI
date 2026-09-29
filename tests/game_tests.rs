use tetyak::board::WIDTH;
use tetyak::game::{Game, GameStatus};
use tetyak::piece::{Difficulty, Tetromino};

#[test]
fn test_speed_capping_never_exceeds_cap() {
    let mut game = Game::new(Difficulty::Normal);
    assert_eq!(game.current_drop_interval_ms(), 600);
    // Simulate clearing 200 lines
    game.add_lines_cleared_for_test(200);
    // In Normal mode, speed must strictly cap at 200ms
    assert_eq!(game.current_drop_interval_ms(), 200);

    // Test Chill mode cap (350ms)
    let mut chill_game = Game::new(Difficulty::Chill);
    assert_eq!(chill_game.current_drop_interval_ms(), 800);
    chill_game.add_lines_cleared_for_test(200);
    assert_eq!(chill_game.current_drop_interval_ms(), 350);

    // Test Intense mode cap (110ms)
    let mut intense_game = Game::new(Difficulty::Intense);
    assert_eq!(intense_game.current_drop_interval_ms(), 350);
    intense_game.add_lines_cleared_for_test(200);
    assert_eq!(intense_game.current_drop_interval_ms(), 110);
}

#[test]
fn test_hold_piece_swapping_and_turn_lock() {
    let mut game = Game::new(Difficulty::Normal);
    game.status = GameStatus::Playing;
    let initial_piece = game.current_piece;
    assert_eq!(game.held_piece, None);
    assert!(game.can_hold);

    // First hold succeeds
    let held = game.hold();
    assert!(held);
    assert_eq!(game.held_piece, Some(initial_piece));
    assert!(!game.can_hold);

    // Second hold in same turn fails
    let held_again = game.hold();
    assert!(!held_again);
}

#[test]
fn test_ghost_piece_projection() {
    let mut game = Game::new(Difficulty::Normal);
    game.status = GameStatus::Playing;
    let ghost_y = game.ghost_y();
    assert!(ghost_y >= game.piece_y);
    assert!(ghost_y <= 19);
}

#[test]
fn test_hard_drop_locks_piece_and_increments_count() {
    let mut game = Game::new(Difficulty::Normal);
    game.status = GameStatus::Playing;
    let initial_count = game.pieces_locked;
    game.hard_drop();
    assert_eq!(game.pieces_locked, initial_count + 1);
    assert!(game.can_hold, "Hold should reset after locking a piece");
}

#[test]
fn test_blast_stats_recording() {
    let mut game = Game::new(Difficulty::Normal);
    game.record_blast(1);
    assert_eq!(game.blasts.singles, 1);
    assert_eq!(game.blasts.last_blast, Some((1, "SINGLE")));

    game.record_blast(4);
    assert_eq!(game.blasts.tetrises, 1);
    assert_eq!(game.blasts.last_blast, Some((4, "TETRIS")));
}

#[test]
fn test_movement_and_wall_kicks() {
    let mut game = Game::new(Difficulty::Normal);
    game.status = GameStatus::Playing;

    // Move left to wall
    for _ in 0..10 {
        game.move_left();
    }
    assert!(!game.move_left());

    // Rotate CW with wall kick
    assert!(game.rotate_cw());

    // Move right to opposite wall
    for _ in 0..15 {
        game.move_right();
    }
    assert!(!game.move_right());
    assert!(game.rotate_ccw());
}

#[test]
fn test_soft_drop_increments_score() {
    let mut game = Game::new(Difficulty::Normal);
    game.status = GameStatus::Playing;
    let initial_score = game.score;
    let initial_y = game.piece_y;

    let dropped = game.soft_drop();
    assert!(dropped);
    assert_eq!(game.piece_y, initial_y + 1);
    assert_eq!(game.score, initial_score + 1);
}

#[test]
fn test_select_difficulty_resets_game() {
    let mut game = Game::new(Difficulty::Normal);
    game.status = GameStatus::Playing;
    game.score = 5000;
    game.lines_cleared = 20;
    game.pieces_locked = 30;

    game.select_difficulty(Difficulty::Intense);
    assert_eq!(game.difficulty, Difficulty::Intense);
    assert_eq!(game.status, GameStatus::Playing);
    assert_eq!(game.score, 0);
    assert_eq!(game.lines_cleared, 0);
    assert_eq!(game.pieces_locked, 0);
    assert_eq!(game.held_piece, None);
    assert!(game.can_hold);
}

#[test]
fn test_pieces_per_minute_calculation() {
    let mut game = Game::new(Difficulty::Normal);
    game.status = GameStatus::Playing;
    game.pieces_locked = 30;
    // elapsed is very small right after creation (<1.0s), so elapsed.max(1.0) is 1.0
    let ppm = game.pieces_per_minute();
    assert!(ppm >= 0.0);
}

#[test]
fn test_line_clear_and_explosion_trigger() {
    let mut game = Game::new(Difficulty::Normal);
    game.status = GameStatus::Playing;

    // Fill bottom row 19 except where I tetromino drops
    for col in 0..WIDTH {
        if (3..=6).contains(&col) {
            continue;
        }
        game.board.set_cell(col, 19, Some(Tetromino::O));
    }

    // Force current piece to be horizontal I piece
    game.current_piece = Tetromino::I;
    game.current_rotation = 0;
    game.piece_x = 3;
    game.piece_y = 18; // I piece cells at rot 0 are (cx, 1), so piece_y=18 puts cells at row 19

    game.hard_drop();
    // 1 row should be cleared immediately (single blast, no explosion because lines <= 2)
    assert_eq!(game.blasts.singles, 1);
    assert!(!game.board.is_exploding());
}

#[test]
fn test_tetris_clear_triggers_explosion() {
    let mut game = Game::new(Difficulty::Normal);
    game.status = GameStatus::Playing;

    // Fill rows 16..=19 except column 5
    for row in 16..=19 {
        for col in 0..WIDTH {
            if col == 5 {
                continue;
            }
            game.board.set_cell(col, row, Some(Tetromino::O));
        }
    }

    // Force current piece to be vertical I piece (rotation 1: cells at cx=2, cy=0..=3)
    // piece_x = 3 so 3 + 2 = col 5
    // piece_y = 16 so 16 + (0..=3) = rows 16..=19
    game.current_piece = Tetromino::I;
    game.current_rotation = 1;
    game.piece_x = 3;
    game.piece_y = 16;

    game.hard_drop();
    // 4 rows should trigger Tetris and start explosion
    assert_eq!(game.blasts.tetrises, 1);
    assert_eq!(game.blasts.last_blast, Some((4, "TETRIS")));
    assert!(game.board.is_exploding());
}

#[test]
fn test_tick_and_lock_delay() {
    let mut game = Game::new(Difficulty::Intense);
    game.status = GameStatus::Playing;

    // Drop piece to bottom
    game.piece_y = game.ghost_y();
    assert_eq!(game.lock_delay_timer, None);

    // First tick on floor initiates lock delay timer
    game.tick();
    assert!(game.lock_delay_timer.is_some());
    let initial_locked = game.pieces_locked;

    // Simulate elapsed time past lock delay (Intense is 250ms)
    game.lock_delay_timer = Some(std::time::Instant::now() - std::time::Duration::from_millis(300));
    game.tick();
    assert_eq!(game.pieces_locked, initial_locked + 1);
}

#[test]
fn test_pause_toggle() {
    let mut game = Game::new(Difficulty::Normal);
    game.status = GameStatus::Playing;

    game.toggle_pause();
    assert_eq!(game.status, GameStatus::Paused);

    // While paused, tick and moves are blocked
    assert!(!game.move_left());
    game.toggle_pause();
    assert_eq!(game.status, GameStatus::Playing);
}


