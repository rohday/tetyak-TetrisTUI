use tetyak::piece::{Difficulty, Randomizer, Tetromino};

#[test]
fn test_all_tetrominoes_have_four_cells_across_rotations() {
    for t in [
        Tetromino::I,
        Tetromino::O,
        Tetromino::T,
        Tetromino::S,
        Tetromino::Z,
        Tetromino::J,
        Tetromino::L,
    ] {
        for r in 0..4 {
            let cells = t.cells(r);
            assert_eq!(cells.len(), 4);
        }
    }
}

#[test]
fn test_rotation_wraparound() {
    let t = Tetromino::T;
    assert_eq!(t.cells(0), t.cells(4));
    assert_eq!(t.cells(1), t.cells(5));
}

#[test]
fn test_randomizer_generates_all_pieces() {
    for diff in [Difficulty::Chill, Difficulty::Normal, Difficulty::Intense] {
        let mut rng = Randomizer::new(diff);
        let mut seen = [false; 7];
        for _ in 0..150 {
            let p = rng.next_piece();
            seen[p as usize] = true;
        }
        for (idx, &s) in seen.iter().enumerate() {
            assert!(s, "Piece {idx} should appear in {diff:?}");
        }
    }
}

#[test]
fn test_normal_mode_not_strict_seven_bag() {
    // Over multiple runs of 14 pieces, in a strict 7-bag each piece appears exactly twice.
    // In NES 1-history reroll, counts will vary from exactly 2.
    let mut non_exact_two_found = false;
    for seed_step in 0..20 {
        let mut rng = Randomizer::new(Difficulty::Normal);
        // Burn some
        for _ in 0..seed_step {
            rng.next_piece();
        }
        let mut counts = [0; 7];
        for _ in 0..14 {
            counts[rng.next_piece() as usize] += 1;
        }
        if counts.iter().any(|&c| c != 2) {
            non_exact_two_found = true;
            break;
        }
    }
    assert!(non_exact_two_found, "Normal mode should not be a rigid 7-bag");
}
