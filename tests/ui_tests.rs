use ratatui::backend::TestBackend;
use ratatui::Terminal;
use tetyak::game::{Difficulty, Game, GameStatus};
use tetyak::theme::{Theme, ThemePreset};
use tetyak::ui;

#[test]
fn test_render_all_game_states_no_panic() {
    let backend = TestBackend::new(100, 35);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = Theme::apple_dark();

    // 1. Difficulty select screen
    let mut game = Game::new(Difficulty::Normal);
    game.status = GameStatus::DifficultySelect;
    terminal.draw(|f| ui::render(f, &game, &theme)).unwrap();

    // 2. Playing screen
    game.status = GameStatus::Playing;
    terminal.draw(|f| ui::render(f, &game, &theme)).unwrap();

    // 3. Paused screen
    game.status = GameStatus::Paused;
    terminal.draw(|f| ui::render(f, &game, &theme)).unwrap();

    // 4. Game Over screen
    game.status = GameStatus::GameOver;
    terminal.draw(|f| ui::render(f, &game, &theme)).unwrap();
}

#[test]
fn test_render_with_exploding_rows() {
    let backend = TestBackend::new(100, 35);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = Theme::apple_dark();

    let mut game = Game::new(Difficulty::Normal);
    game.status = GameStatus::Playing;
    game.board.start_explosion(vec![18, 19], 120);

    terminal.draw(|f| ui::render(f, &game, &theme)).unwrap();
}

#[test]
fn test_render_with_held_piece_and_blasts() {
    let backend = TestBackend::new(100, 35);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = Theme::apple_dark();

    let mut game = Game::new(Difficulty::Chill);
    game.status = GameStatus::Playing;
    game.hold_piece();
    game.record_blast(4);

    terminal.draw(|f| ui::render(f, &game, &theme)).unwrap();
}

#[test]
fn test_render_all_themes_and_difficulties() {
    let backend = TestBackend::new(100, 35);
    let mut terminal = Terminal::new(backend).unwrap();

    for preset in ThemePreset::ALL {
        let theme = preset.theme();
        for diff in [Difficulty::Chill, Difficulty::Normal, Difficulty::Intense] {
            let mut game = Game::new(diff);
            game.status = GameStatus::DifficultySelect;
            terminal.draw(|f| ui::render(f, &game, &theme)).unwrap();
        }
    }
}

#[test]
fn test_render_small_and_zero_terminals() {
    let theme = Theme::apple_dark();
    let game = Game::new(Difficulty::Normal);

    // Zero dimension
    let backend = TestBackend::new(0, 0);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| ui::render(f, &game, &theme)).unwrap();

    // Very small terminal
    let backend = TestBackend::new(20, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| ui::render(f, &game, &theme)).unwrap();

    // Standard 80x24 terminal
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| ui::render(f, &game, &theme)).unwrap();
}

fn fg_sum(c: ratatui::style::Color) -> u32 {
    match c {
        ratatui::style::Color::Rgb(r, g, b) => r as u32 + g as u32 + b as u32,
        _ => panic!("expected an RGB color, got {c:?}"),
    }
}

#[test]
fn test_modal_dims_board_behind_it() {
    let theme = Theme::apple_dark();
    let mut game = Game::new(Difficulty::Normal);
    game.status = GameStatus::Playing;

    let mut terminal = Terminal::new(TestBackend::new(100, 35)).unwrap();
    terminal.draw(|f| ui::render(f, &game, &theme)).unwrap();
    // Top-left corner of the 64x22 dashboard centered in 100x35 is (18, 6): a HOLD border cell.
    let playing = terminal.backend().buffer().cell((18, 6)).unwrap().fg;

    game.status = GameStatus::Paused;
    terminal.draw(|f| ui::render(f, &game, &theme)).unwrap();
    let paused = terminal.backend().buffer().cell((18, 6)).unwrap().fg;

    assert!(fg_sum(paused) < fg_sum(playing));
}

#[test]
fn test_game_over_modal_waits_for_fade() {
    let theme = Theme::apple_dark();
    let mut game = Game::new(Difficulty::Normal);
    game.status = GameStatus::GameOver;
    game.game_over_at = Some(std::time::Instant::now());

    let text = |game: &Game| {
        let mut terminal = Terminal::new(TestBackend::new(100, 35)).unwrap();
        terminal.draw(|f| ui::render(f, game, &theme)).unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect::<String>()
    };

    assert!(!text(&game).contains("Final Score"));
    // No timestamp means the animation is treated as finished.
    game.game_over_at = None;
    assert!(text(&game).contains("Final Score"));
}
