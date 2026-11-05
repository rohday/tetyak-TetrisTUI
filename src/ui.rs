use crate::board::{HEIGHT, WIDTH};
use crate::game::{Difficulty, Game, GameStatus, DROP_TRAIL_MS, LOCK_FLASH_MS};
use crate::piece::Tetromino;
use crate::theme::{greyscale, mix, Theme, FLASH_CHAR, GHOST_CHAR, PARTICLE_CHAR};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Frame;

/// Renders the spacious Ratatui TUI dashboard for Tetyak.
pub fn render(f: &mut Frame, game: &Game, theme: &Theme) {
    let area = f.area();
    if area.width == 0 || area.height == 0 {
        return;
    }

    // Centered dashboard layout (approx 64x22)
    let total_width = 64.min(area.width);
    let total_height = 22.min(area.height);
    let x = area.x + (area.width.saturating_sub(total_width)) / 2;
    let y = area.y + (area.height.saturating_sub(total_height)) / 2;
    let dashboard_area = Rect::new(x, y, total_width, total_height);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(18),
            Constraint::Length(24),
            Constraint::Length(22),
        ])
        .split(dashboard_area);

    render_left_column(f, columns[0], game, theme);
    render_center_column(f, columns[1], game, theme);
    render_right_column(f, columns[2], game, theme);

    // Overlays: the board is dimmed first so the modal stands out. The game-over modal
    // waits for the grey-out animation to finish.
    let show_overlay = match game.status {
        GameStatus::Playing => false,
        GameStatus::GameOver => game.game_over_progress() >= 1.0,
        _ => true,
    };
    if show_overlay {
        dim_area(f, area, theme);
        match game.status {
            GameStatus::DifficultySelect => render_difficulty_modal(f, area, game, theme),
            GameStatus::Paused => render_paused_modal(f, area, theme),
            GameStatus::GameOver => render_game_over_modal(f, area, game, theme),
            GameStatus::Playing => {}
        }
    }
}

/// Fraction of the dim applied to everything behind a modal.
const MODAL_DIM: f32 = 0.6;

fn dim_area(f: &mut Frame, area: Rect, theme: &Theme) {
    let buf = f.buffer_mut();
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.fg = mix(cell.fg, theme.base, MODAL_DIM);
                cell.bg = mix(cell.bg, theme.base, MODAL_DIM);
            }
        }
    }
}

/// Progress (0..1) of a timed effect, or `None` once it has finished.
fn effect_progress(elapsed: std::time::Duration, total_ms: u64) -> Option<f32> {
    let p = elapsed.as_secs_f32() * 1000.0 / total_ms as f32;
    if p < 1.0 {
        Some(p)
    } else {
        None
    }
}

/// One cell of an exploding row: a wave moves outward from the center column, and each cell
/// flashes white then shrinks through shaded glyphs into the background.
fn explosion_span(theme: &Theme, color: Color, x: usize, progress: f32) -> Span<'static> {
    let center = (WIDTH as f32 - 1.0) / 2.0;
    let dist = ((x as f32 - center).abs() / center).clamp(0.0, 1.0);
    let start = 0.55 * dist;
    if progress < start {
        return theme.block_span(color);
    }
    let age = ((progress - start) / (1.0 - start)).clamp(0.0, 1.0);
    if age < 0.25 {
        Span::styled(
            FLASH_CHAR,
            Style::default().fg(theme.flash).add_modifier(Modifier::BOLD),
        )
    } else if age < 0.5 {
        Span::styled("▓▓", Style::default().fg(mix(color, theme.flash, 0.4)))
    } else if age < 0.75 {
        Span::styled(PARTICLE_CHAR, Style::default().fg(color))
    } else {
        Span::styled(GHOST_CHAR, Style::default().fg(mix(color, theme.base, 0.55)))
    }
}

fn render_left_column(f: &mut Frame, area: Rect, game: &Game, theme: &Theme) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(11)])
        .split(area);

    // [ HOLD ] Block
    let hold_block = Block::default()
        .title(Span::styled(" [HOLD] ", theme.title_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_style(false));

    let hold_inner = hold_block.inner(rows[0]);
    f.render_widget(hold_block, rows[0]);

    let hold_dimmed = game.held_piece.is_some() && !game.can_hold;
    let hold_lines = render_piece_preview(game.held_piece, hold_inner.width, theme, hold_dimmed);
    f.render_widget(Paragraph::new(hold_lines), hold_inner);

    // Controls Cheat-sheet
    let controls_block = Block::default()
        .title(Span::styled(" CONTROLS ", Style::default().fg(theme.text_muted)))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_style(false));

    let controls_inner = controls_block.inner(rows[1]);
    f.render_widget(controls_block, rows[1]);

    let controls = [
        ("A / ←", "Left"),
        ("D / →", "Right"),
        ("W / ↑", "Rot CW"),
        ("Z    ", "Rot CCW"),
        ("S / ↓", "Soft"),
        ("Space", "Hard"),
        ("C    ", "Hold"),
        ("P    ", "Pause"),
        ("Q    ", "Quit"),
    ];

    let pad = (controls_inner.width.saturating_sub(15)) / 2;
    let pad_str = " ".repeat(pad as usize);

    let controls_lines: Vec<Line> = controls
        .iter()
        .map(|(key, desc)| {
            Line::from(vec![
                Span::raw(pad_str.clone()),
                Span::styled(*key, Style::default().fg(theme.text_primary)),
                Span::styled(" : ", Style::default().fg(theme.text_muted)),
                Span::styled(*desc, Style::default().fg(theme.secondary)),
            ])
        })
        .collect();

    f.render_widget(Paragraph::new(controls_lines), controls_inner);
}

fn render_piece_preview(
    piece: Option<Tetromino>,
    inner_width: u16,
    theme: &Theme,
    dimmed: bool,
) -> Vec<Line<'static>> {
    let pad = (inner_width.saturating_sub(8)) / 2;
    let pad_str = " ".repeat(pad as usize);

    match piece {
        None => vec![
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(""),
        ],
        Some(p) => {
            let cells = p.cells(0);
            let min_x = cells.iter().map(|c| c.0).min().unwrap();
            let max_x = cells.iter().map(|c| c.0).max().unwrap();
            let min_y = cells.iter().map(|c| c.1).min().unwrap();
            let max_y = cells.iter().map(|c| c.1).max().unwrap();

            let width = max_x - min_x + 1;
            let height = max_y - min_y + 1;

            let offset_x = (4 - width) / 2 - min_x;
            let offset_y = (4 - height) / 2 - min_y;

            let mut block_color = theme.block_color(p);
            if dimmed {
                block_color = mix(block_color, theme.base, 0.65);
            }

            let mut lines = Vec::with_capacity(4);
            for py in 0..4 {
                let mut spans = Vec::with_capacity(6);
                if !pad_str.is_empty() {
                    spans.push(Span::raw(pad_str.clone()));
                }
                for px in 0..4 {
                    let orig_x = px - offset_x;
                    let orig_y = py - offset_y;
                    if cells.contains(&(orig_x, orig_y)) {
                        spans.push(theme.block_span(block_color));
                    } else {
                        spans.push(Span::raw("  "));
                    }
                }
                lines.push(Line::from(spans));
            }
            lines
        }
    }
}

fn render_center_column(f: &mut Frame, area: Rect, game: &Game, theme: &Theme) {
    let field_block = Block::default()
        .title(Span::styled(" T E T Y A K ", theme.title_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_style(true));

    let inner = field_block.inner(area);
    f.render_widget(field_block, area);

    let pad = (inner.width.saturating_sub(20)) / 2;
    let pad_str = " ".repeat(pad as usize);

    let active_cells = game.current_cells();
    let ghost_cells = game.ghost_cells();
    let ghost_active = game.ghost_y() > game.piece_y;
    let game_over = game.status == GameStatus::GameOver;
    // A pending line-clear is ignored after top-out because the game never advances past it.
    let exploding_rows = if game_over {
        &[]
    } else {
        game.board.exploding_rows().unwrap_or(&[])
    };
    let explosion_progress = game.board.explosion_progress().unwrap_or(0.0);

    let flash = game.lock_flash.as_ref().and_then(|fl| {
        effect_progress(fl.at.elapsed(), LOCK_FLASH_MS).map(|p| (fl, p))
    });
    let trail = game.drop_trail.as_ref().and_then(|tr| {
        effect_progress(tr.at.elapsed(), DROP_TRAIL_MS).map(|p| (tr, p))
    });

    // Game-over fade: the stack greys out from the bottom row upward.
    let over_progress = game.game_over_progress();
    let faded = |color: Color, y: usize| -> Color {
        if !game_over {
            return color;
        }
        let from_bottom = (HEIGHT - 1 - y) as f32 / HEIGHT as f32;
        let amount = ((over_progress * 1.25 - from_bottom) / 0.25).clamp(0.0, 1.0);
        mix(color, mix(greyscale(color), theme.base, 0.35), amount)
    };

    let mut lines = Vec::with_capacity(HEIGHT);

    for y in 0..HEIGHT {
        let mut spans = Vec::with_capacity(WIDTH + 2);
        if !pad_str.is_empty() {
            spans.push(Span::raw(pad_str.clone()));
        }

        for x in 0..WIDTH {
            let pt = (x as i32, y as i32);
            if exploding_rows.contains(&y) {
                let color = game
                    .board
                    .cell(x, y)
                    .map(|t| theme.block_color(t))
                    .unwrap_or(theme.particle);
                spans.push(explosion_span(theme, color, x, explosion_progress));
            } else if active_cells.contains(&pt) {
                spans.push(theme.block_span(faded(theme.block_color(game.current_piece), y)));
            } else if let Some((fl, p)) = flash.filter(|(fl, _)| fl.cells.contains(&pt)) {
                let color = mix(theme.flash, theme.block_color(fl.piece), p);
                spans.push(theme.block_span(color));
            } else if let Some((tr, p)) = trail.filter(|(tr, _)| tr.cells.contains(&pt)) {
                let color = mix(theme.block_color(tr.piece), theme.base, 0.25 + 0.75 * p);
                spans.push(Span::styled(PARTICLE_CHAR, Style::default().fg(color)));
            } else if ghost_active && ghost_cells.contains(&pt) {
                spans.push(Span::styled(GHOST_CHAR, Style::default().fg(theme.ghost)));
            } else if let Some(t) = game.board.cell(x, y) {
                spans.push(theme.block_span(faded(theme.block_color(t), y)));
            } else {
                spans.push(Span::raw("  "));
            }
        }

        if !pad_str.is_empty() {
            spans.push(Span::raw(pad_str.clone()));
        }
        lines.push(Line::from(spans));
    }

    f.render_widget(Paragraph::new(lines), inner);
}

fn render_right_column(f: &mut Frame, area: Rect, game: &Game, theme: &Theme) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Min(8),
        ])
        .split(area);

    // [ NEXT ] Card
    let next_block = Block::default()
        .title(Span::styled(" [NEXT] ", theme.title_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_style(false));

    let next_inner = next_block.inner(rows[0]);
    f.render_widget(next_block, rows[0]);

    let next_lines =
        render_piece_preview(Some(game.next_piece), next_inner.width, theme, false);
    f.render_widget(Paragraph::new(next_lines), next_inner);

    // [ STATS ] Card
    let stats_block = Block::default()
        .title(Span::styled(" STATS ", theme.title_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_style(false));

    let stats_inner = stats_block.inner(rows[1]);
    f.render_widget(stats_block, rows[1]);

    let mode_str = match game.difficulty {
        Difficulty::Chill => "Chill",
        Difficulty::Normal => "Normal",
        Difficulty::Intense => "Intense",
    };

    let stats_lines = vec![
        Line::from(vec![
            Span::styled("Mode:   ", Style::default().fg(theme.text_muted)),
            Span::styled(
                mode_str,
                Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Score:  ", Style::default().fg(theme.text_muted)),
            Span::styled(format!("{}", game.score), Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("Lines:  ", Style::default().fg(theme.text_muted)),
            Span::styled(format!("{}", game.lines_cleared), Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("Pieces: ", Style::default().fg(theme.text_muted)),
            Span::styled(format!("{}", game.pieces_locked), Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("Pace:   ", Style::default().fg(theme.text_muted)),
            Span::styled(
                format!("{:.1} PPM", game.pieces_per_minute()),
                Style::default().fg(theme.text_primary),
            ),
        ]),
        Line::from(vec![
            Span::styled("Speed:  ", Style::default().fg(theme.text_muted)),
            Span::styled(
                format!("{} ms", game.current_drop_interval_ms()),
                Style::default().fg(theme.text_primary),
            ),
        ]),
    ];
    f.render_widget(Paragraph::new(stats_lines), stats_inner);

    // [ BLASTS ] Card
    let blasts_block = Block::default()
        .title(Span::styled(" BLASTS ", theme.title_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_style(false));

    let blasts_inner = blasts_block.inner(rows[2]);
    f.render_widget(blasts_block, rows[2]);

    let mut blast_lines = vec![
        Line::from(vec![
            Span::styled("1x: ", Style::default().fg(theme.text_muted)),
            Span::styled(format!("{:<4}", game.blasts.singles), Style::default().fg(theme.text_primary)),
            Span::styled("2x: ", Style::default().fg(theme.text_muted)),
            Span::styled(format!("{}", game.blasts.doubles), Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("3x: ", Style::default().fg(theme.text_muted)),
            Span::styled(format!("{:<4}", game.blasts.triples), Style::default().fg(theme.text_primary)),
            Span::styled("4x: ", Style::default().fg(theme.text_muted)),
            Span::styled(format!("{}", game.blasts.tetrises), Style::default().fg(theme.text_primary)),
        ]),
    ];

    if let Some((_count, label)) = game.blasts.last_blast {
        blast_lines.push(Line::from(""));
        blast_lines.push(
            Line::from(Span::styled(
                format!(" > {} < ", label),
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center),
        );
    }
    f.render_widget(Paragraph::new(blast_lines), blasts_inner);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w, h)
}

fn render_difficulty_modal(f: &mut Frame, area: Rect, game: &Game, theme: &Theme) {
    let modal_area = centered_rect(48, 10, area);
    f.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(Span::styled(" SELECT DIFFICULTY ", theme.title_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_focused));

    let inner = block.inner(modal_area);
    f.render_widget(block, modal_area);

    let opt_style = |selected: bool| {
        if selected {
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_primary)
        }
    };

    let opt_prefix = |selected: bool| {
        if selected {
            " > "
        } else {
            "   "
        }
    };

    let c_sel = game.difficulty == Difficulty::Chill;
    let n_sel = game.difficulty == Difficulty::Normal;
    let i_sel = game.difficulty == Difficulty::Intense;

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("T E T Y A K", theme.title_style()))
            .alignment(Alignment::Center),
        Line::from(""),
        Line::from(vec![
            Span::styled(opt_prefix(c_sel), opt_style(c_sel)),
            Span::styled("[1] Chill    (800ms -> 350ms cap)", opt_style(c_sel)),
        ]),
        Line::from(vec![
            Span::styled(opt_prefix(n_sel), opt_style(n_sel)),
            Span::styled("[2] Normal   (600ms -> 200ms cap) [Default]", opt_style(n_sel)),
        ]),
        Line::from(vec![
            Span::styled(opt_prefix(i_sel), opt_style(i_sel)),
            Span::styled("[3] Intense  (350ms -> 110ms cap)", opt_style(i_sel)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press 1, 2, 3 or Enter to begin",
            Style::default().fg(theme.text_muted),
        ))
        .alignment(Alignment::Center),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn render_paused_modal(f: &mut Frame, area: Rect, theme: &Theme) {
    let modal_area = centered_rect(36, 5, area);
    f.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(Span::styled(" PAUSED ", theme.title_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.secondary));

    let inner = block.inner(modal_area);
    f.render_widget(block, modal_area);

    let lines = vec![
        Line::from(Span::styled(
            "[ PAUSED ]",
            Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        Line::from(Span::styled(
            "Press P to resume, Q to quit",
            Style::default().fg(theme.text_muted),
        ))
        .alignment(Alignment::Center),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn render_game_over_modal(f: &mut Frame, area: Rect, game: &Game, theme: &Theme) {
    let modal_area = centered_rect(38, 7, area);
    f.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(Span::styled(" GAME OVER ", theme.title_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent));

    let inner = block.inner(modal_area);
    f.render_widget(block, modal_area);

    let lines = vec![
        Line::from(Span::styled(
            "[ GAME OVER ]",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        Line::from(vec![
            Span::styled("Final Score: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                format!("{}", game.score),
                Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD),
            ),
        ])
        .alignment(Alignment::Center),
        Line::from(""),
        Line::from(Span::styled(
            "Press R to restart, Q to quit",
            Style::default().fg(theme.text_muted),
        ))
        .alignment(Alignment::Center),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}
