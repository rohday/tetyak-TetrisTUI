pub mod board;
pub mod game;
pub mod piece;
pub mod theme;
pub mod ui;

use std::error::Error;
use std::io::stdout;
use std::time::{Duration, Instant};

use crossterm::event::{
    self, Event, KeyCode, KeyEventKind, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::game::{Difficulty, Game, GameStatus};
use crate::theme::{Theme, ThemePreset};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AutoRepeatDir {
    Left,
    Right,
    Down,
}

struct DasState {
    dir: Option<AutoRepeatDir>,
    pressed_at: Instant,
    last_repeat: Instant,
    last_event_seen: Instant,
}

impl DasState {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            dir: None,
            pressed_at: now,
            last_repeat: now,
            last_event_seen: now,
        }
    }

    fn press(&mut self, dir: AutoRepeatDir) {
        let now = Instant::now();
        self.dir = Some(dir);
        self.pressed_at = now;
        self.last_repeat = now;
        self.last_event_seen = now;
    }

    fn touch(&mut self, dir: AutoRepeatDir) {
        if self.dir == Some(dir) {
            self.last_event_seen = Instant::now();
        }
    }

    fn release(&mut self, dir: AutoRepeatDir) {
        if self.dir == Some(dir) {
            self.dir = None;
        }
    }

    fn cancel(&mut self) {
        self.dir = None;
    }
}

struct TerminalCleanup;

impl Drop for TerminalCleanup {
    fn drop(&mut self) {
        let _ = execute!(
            stdout(),
            PopKeyboardEnhancementFlags,
            LeaveAlternateScreen,
            crossterm::cursor::Show
        );
        let _ = disable_raw_mode();
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    // Set up panic hook to always restore terminal
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = execute!(
            stdout(),
            PopKeyboardEnhancementFlags,
            LeaveAlternateScreen,
            crossterm::cursor::Show
        );
        let _ = disable_raw_mode();
        default_hook(info);
    }));

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        crossterm::cursor::Hide,
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
    )?;
    let _cleanup = TerminalCleanup;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut game = Game::new(Difficulty::Normal);
    game.status = GameStatus::DifficultySelect;

    let mut theme_preset = ThemePreset::AppleDark;
    let mut theme: Theme = theme_preset.theme();

    let mut das = DasState::new();

    loop {
        // Draw current frame
        terminal.draw(|f| ui::render(f, &game, &theme))?;

        // Determine poll timeout
        let timeout = if game.board.is_exploding() || das.dir.is_some() {
            Duration::from_millis(10)
        } else {
            Duration::from_millis(25)
        };

        // Poll for input events
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => {
                    // Handle key releases for responsive DAS cancellation
                    if key.kind == KeyEventKind::Release {
                        match key.code {
                            KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => {
                                das.release(AutoRepeatDir::Left);
                            }
                            KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => {
                                das.release(AutoRepeatDir::Right);
                            }
                            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                                das.release(AutoRepeatDir::Down);
                            }
                            _ => {}
                        }
                    } else if key.kind == KeyEventKind::Press || key.kind == KeyEventKind::Repeat {
                        match game.status {
                            GameStatus::DifficultySelect => {
                                das.cancel();
                                if key.kind == KeyEventKind::Press {
                                    match key.code {
                                        KeyCode::Char('1') | KeyCode::Char('c') => {
                                            game.select_difficulty(Difficulty::Chill);
                                        }
                                        KeyCode::Char('2')
                                        | KeyCode::Char('n')
                                        | KeyCode::Enter => {
                                            game.select_difficulty(Difficulty::Normal);
                                        }
                                        KeyCode::Char('3') | KeyCode::Char('i') => {
                                            game.select_difficulty(Difficulty::Intense);
                                        }
                                        KeyCode::Char('q') | KeyCode::Esc => break,
                                        _ => {}
                                    }
                                }
                            }
                            GameStatus::Playing => match key.code {
                                KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => {
                                    if key.kind == KeyEventKind::Press {
                                        game.move_left();
                                        das.press(AutoRepeatDir::Left);
                                    } else {
                                        das.touch(AutoRepeatDir::Left);
                                    }
                                }
                                KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => {
                                    if key.kind == KeyEventKind::Press {
                                        game.move_right();
                                        das.press(AutoRepeatDir::Right);
                                    } else {
                                        das.touch(AutoRepeatDir::Right);
                                    }
                                }
                                KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                                    if key.kind == KeyEventKind::Press {
                                        game.soft_drop();
                                        das.press(AutoRepeatDir::Down);
                                    } else {
                                        das.touch(AutoRepeatDir::Down);
                                    }
                                }
                                KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                                    if key.kind == KeyEventKind::Press {
                                        game.rotate_cw();
                                    }
                                }
                                KeyCode::Char('z') | KeyCode::Char('Z') => {
                                    if key.kind == KeyEventKind::Press {
                                        game.rotate_ccw();
                                    }
                                }
                                KeyCode::Char(' ') => {
                                    if key.kind == KeyEventKind::Press {
                                        das.cancel();
                                        game.hard_drop();
                                    }
                                }
                                KeyCode::Char('c') | KeyCode::Char('C') => {
                                    if key.kind == KeyEventKind::Press {
                                        das.cancel();
                                        game.hold();
                                    }
                                }
                                KeyCode::Char('p') | KeyCode::Char('P') => {
                                    if key.kind == KeyEventKind::Press {
                                        das.cancel();
                                        game.toggle_pause();
                                    }
                                }
                                KeyCode::Char('t') | KeyCode::Char('T') => {
                                    if key.kind == KeyEventKind::Press {
                                        theme_preset = theme_preset.cycle();
                                        theme = theme_preset.theme();
                                    }
                                }
                                KeyCode::Char('q') | KeyCode::Esc => break,
                                _ => {}
                            },
                            GameStatus::Paused => {
                                das.cancel();
                                if key.kind == KeyEventKind::Press {
                                    match key.code {
                                        KeyCode::Char('p') | KeyCode::Char('P') => {
                                            game.toggle_pause();
                                        }
                                        KeyCode::Char('q') | KeyCode::Esc => break,
                                        _ => {}
                                    }
                                }
                            }
                            GameStatus::GameOver => {
                                das.cancel();
                                if key.kind == KeyEventKind::Press {
                                    match key.code {
                                        KeyCode::Char('r')
                                        | KeyCode::Char('R')
                                        | KeyCode::Enter => {
                                            game.status = GameStatus::DifficultySelect;
                                        }
                                        KeyCode::Char('q') | KeyCode::Esc => break,
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Process software DAS / ARR auto-repeat while direction is held
        if game.status == GameStatus::Playing && !game.board.is_exploding() {
            if let Some(dir) = das.dir {
                let now = Instant::now();
                // If on legacy terminals without release events, expire hold if no events seen
                if now.duration_since(das.last_event_seen) > Duration::from_millis(160) {
                    das.cancel();
                } else if now.duration_since(das.pressed_at) >= Duration::from_millis(125) {
                    // ARR step interval: 35ms for smooth 30Hz auto-sliding
                    let arr_interval = match dir {
                        AutoRepeatDir::Down => Duration::from_millis(45),
                        _ => Duration::from_millis(35),
                    };

                    if now.duration_since(das.last_repeat) >= arr_interval {
                        match dir {
                            AutoRepeatDir::Left => {
                                game.move_left();
                            }
                            AutoRepeatDir::Right => {
                                game.move_right();
                            }
                            AutoRepeatDir::Down => {
                                game.soft_drop();
                            }
                        }
                        das.last_repeat = now;
                    }
                }
            }
        }

        // Advance game tick
        game.tick();
    }

    Ok(())
}
