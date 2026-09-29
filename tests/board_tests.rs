use tetyak::board::{Board, HEIGHT, WIDTH};
use tetyak::piece::Tetromino;

#[test]
fn test_wall_and_floor_collision() {
    assert_eq!(WIDTH, 10);
    let board = Board::new();
    // Out of left bound
    assert!(!board.can_place(Tetromino::O, 0, -1, 0));
    // Out of right bound (O is 2 wide, x=9 puts it at 9 and 10)
    assert!(!board.can_place(Tetromino::O, 0, 9, 0));
    // Out of bottom bound (O is 2 tall, y=19 puts it at 19 and 20)
    assert!(!board.can_place(Tetromino::O, 0, 0, 19));
    // Valid in bounds
    assert!(board.can_place(Tetromino::O, 0, 0, 0));
    assert!(board.can_place(Tetromino::O, 0, 8, 18));
}

#[test]
fn test_piece_collision_and_lock() {
    let mut board = Board::new();
    // Place a piece at bottom left
    board.lock_piece(Tetromino::O, 0, 0, 18);
    // Cannot place overlapping piece
    assert!(!board.can_place(Tetromino::O, 0, 0, 18));
    assert!(!board.can_place(Tetromino::I, 0, 0, 18));
    // Can place above it
    assert!(board.can_place(Tetromino::O, 0, 0, 16));
}

#[test]
fn test_line_clear_and_collapse() {
    let mut board = Board::new();
    // Fill bottom row except column 9
    for x in 0..9 {
        board.set_cell(x, HEIGHT - 1, Some(Tetromino::I));
    }
    assert_eq!(board.check_full_lines().len(), 0);
    // Fill column 9
    board.set_cell(9, HEIGHT - 1, Some(Tetromino::I));
    let full = board.check_full_lines();
    assert_eq!(full, vec![HEIGHT - 1]);

    // Put a block above it at row HEIGHT - 2
    board.set_cell(3, HEIGHT - 2, Some(Tetromino::T));

    // Clear lines
    board.clear_lines(&full);

    // Row HEIGHT - 1 should now contain the collapsed block from row HEIGHT - 2
    assert_eq!(board.cell(3, HEIGHT - 1), Some(Tetromino::T));
    // And row HEIGHT - 2 should now be empty
    assert_eq!(board.cell(3, HEIGHT - 2), None);
}

#[test]
fn test_explosion_state_transitions() {
    let mut board = Board::new();
    assert!(!board.is_exploding());
    board.start_explosion(vec![18, 19], 50); // 50ms duration
    assert!(board.is_exploding());
    assert_eq!(board.exploding_rows(), Some(&[18, 19][..]));

    std::thread::sleep(std::time::Duration::from_millis(60));
    let finished = board.finish_explosion_if_done();
    assert!(finished);
    assert!(!board.is_exploding());
}

#[test]
fn test_multi_line_clear() {
    let mut board = Board::new();
    // Fill rows 17 and 19 completely
    for x in 0..WIDTH {
        board.set_cell(x, 17, Some(Tetromino::I));
        board.set_cell(x, 19, Some(Tetromino::I));
    }
    // Set cell at row 16 and 18
    board.set_cell(5, 16, Some(Tetromino::L));
    board.set_cell(2, 18, Some(Tetromino::S));

    let full = board.check_full_lines();
    assert_eq!(full, vec![17, 19]);

    board.clear_lines(&full);

    // Row 18 had row 19 below it cleared -> drops to row 19
    assert_eq!(board.cell(2, 19), Some(Tetromino::S));
    // Row 16 had two rows (17 and 19) below it cleared -> drops to row 18
    assert_eq!(board.cell(5, 18), Some(Tetromino::L));
    // Rows 16 and 17 should now be empty
    assert_eq!(board.cell(5, 16), None);
    assert_eq!(board.cell(2, 18), None);
}

#[test]
fn test_explosion_progress() {
    let mut board = Board::new();
    assert_eq!(board.explosion_progress(), None);
    board.start_explosion(vec![19], 100);
    assert!(board.explosion_progress().is_some());
    let prog = board.explosion_progress().unwrap();
    assert!((0.0..=1.0).contains(&prog));
}
