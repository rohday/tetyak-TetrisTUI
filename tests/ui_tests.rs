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
