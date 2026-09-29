pub mod board;
pub mod game;
pub mod piece;
pub mod theme;
pub mod ui;

use std::error::Error;
use std::io::stdout;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::game::{Difficulty, Game, GameStatus};
use crate::theme::{Theme, ThemePreset};

struct TerminalCleanup;

impl Drop for TerminalCleanup {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, crossterm::cursor::Show);
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    // Set up panic hook to always restore terminal
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, crossterm::cursor::Show);
        default_hook(info);
    }));

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, crossterm::cursor::Hide)?;
    let _cleanup = TerminalCleanup;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut game = Game::new(Difficulty::Normal);
    game.status = GameStatus::DifficultySelect;

    let mut theme_preset = ThemePreset::AppleDark;
    let mut theme: Theme = theme_preset.theme();

    let mut _last_render = Instant::now();

    loop {
        // Draw
        terminal.draw(|f| ui::render(f, &game, &theme))?;

        // Determine poll timeout
        let timeout = if game.board.is_exploding() {
            Duration::from_millis(15)
        } else {
            Duration::from_millis(30)
        };

        // Poll for events
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match game.status {
                        GameStatus::DifficultySelect => match key.code {
                            KeyCode::Char('1') | KeyCode::Char('c') => {
                                game.select_difficulty(Difficulty::Chill);
                            }
                            KeyCode::Char('2') | KeyCode::Char('n') | KeyCode::Enter => {
                                game.select_difficulty(Difficulty::Normal);
                            }
                            KeyCode::Char('3') | KeyCode::Char('i') => {
                                game.select_difficulty(Difficulty::Intense);
                            }
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            _ => {}
                        },
                        GameStatus::Playing => match key.code {
                            KeyCode::Left | KeyCode::Char('a') => {
                                game.move_left();
                            }
                            KeyCode::Right | KeyCode::Char('d') => {
                                game.move_right();
                            }
                            KeyCode::Up | KeyCode::Char('w') => {
                                game.rotate_cw();
                            }
                            KeyCode::Char('z') => {
                                game.rotate_ccw();
                            }
                            KeyCode::Down | KeyCode::Char('s') => {
                                game.soft_drop();
                            }
                            KeyCode::Char(' ') => {
                                game.hard_drop();
                            }
                            KeyCode::Char('c') | KeyCode::Char('C') => {
                                game.hold();
                            }
                            KeyCode::Char('p') | KeyCode::Char('P') => {
                                game.toggle_pause();
                            }
                            KeyCode::Char('t') | KeyCode::Char('T') => {
                                theme_preset = theme_preset.cycle();
                                theme = theme_preset.theme();
                            }
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            _ => {}
                        },
                        GameStatus::Paused => match key.code {
                            KeyCode::Char('p') | KeyCode::Char('P') => {
                                game.toggle_pause();
                            }
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            _ => {}
                        },
                        GameStatus::GameOver => match key.code {
                            KeyCode::Char('r') | KeyCode::Char('R') | KeyCode::Enter => {
                                game.status = GameStatus::DifficultySelect;
                            }
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            _ => {}
                        },
                    }
                }
            }
        }

        // Advance game tick
        game.tick();
    }

    Ok(())
}
