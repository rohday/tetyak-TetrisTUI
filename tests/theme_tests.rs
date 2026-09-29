use ratatui::style::Color;
use tetyak::piece::Tetromino;
use tetyak::theme::{
    Theme, ThemePreset, BLOCK_CHAR, FLASH_CHAR, GHOST_CHAR, PARTICLE_CHAR,
};

#[test]
fn test_default_theme_is_apple_dark() {
    let default_theme = Theme::default();
    let apple_dark = Theme::apple_dark();
    assert_eq!(default_theme, apple_dark);
}

#[test]
fn test_theme_presets() {
    let ad = ThemePreset::AppleDark.theme();
    assert_eq!(ad.accent, Color::Rgb(250, 45, 72));
    assert_eq!(ad.secondary, Color::Rgb(140, 140, 240));
    assert_eq!(ad.text_primary, Color::Rgb(240, 240, 245));
    assert_eq!(ad.text_muted, Color::Rgb(130, 130, 140));

    let cm = ThemePreset::CatppuccinMocha.theme();
    assert_eq!(cm.accent, Color::Rgb(203, 166, 247));
    assert_eq!(cm.secondary, Color::Rgb(116, 199, 236));
    assert_eq!(cm.text_primary, Color::Rgb(205, 214, 244));
    assert_eq!(cm.text_muted, Color::Rgb(147, 153, 178));

    let tn = ThemePreset::TokyoNight.theme();
    assert_eq!(tn.accent, Color::Rgb(125, 207, 255));
    assert_eq!(tn.secondary, Color::Rgb(187, 154, 247));
    assert_eq!(tn.text_primary, Color::Rgb(192, 202, 245));
    assert_eq!(tn.text_muted, Color::Rgb(86, 95, 137));
}

#[test]
fn test_preset_cycle_and_names() {
    assert_eq!(ThemePreset::AppleDark.cycle(), ThemePreset::CatppuccinMocha);
    assert_eq!(ThemePreset::CatppuccinMocha.cycle(), ThemePreset::TokyoNight);
    assert_eq!(ThemePreset::TokyoNight.cycle(), ThemePreset::AppleDark);

    assert_eq!(ThemePreset::AppleDark.name(), "Apple Dark");
    assert_eq!(ThemePreset::CatppuccinMocha.name(), "Catppuccin Mocha");
    assert_eq!(ThemePreset::TokyoNight.name(), "Tokyo Night");
}

#[test]
fn test_block_colors() {
    let theme = Theme::apple_dark();
    for piece in Tetromino::ALL {
        assert_eq!(theme.block_color(piece), piece.color());
    }

    assert_eq!(theme.block_color(Tetromino::I), Color::Rgb(100, 210, 255));
    assert_eq!(theme.block_color(Tetromino::O), Color::Rgb(255, 214, 10));
    assert_eq!(theme.block_color(Tetromino::T), Color::Rgb(175, 82, 222));
    assert_eq!(theme.block_color(Tetromino::S), Color::Rgb(48, 209, 88));
    assert_eq!(theme.block_color(Tetromino::Z), Color::Rgb(255, 69, 58));
    assert_eq!(theme.block_color(Tetromino::J), Color::Rgb(10, 132, 255));
    assert_eq!(theme.block_color(Tetromino::L), Color::Rgb(255, 159, 10));
}

#[test]
fn test_glyph_constants() {
    assert_eq!(BLOCK_CHAR, "██");
    assert_eq!(GHOST_CHAR, "░░");
    assert_eq!(FLASH_CHAR, "██");
    assert_eq!(PARTICLE_CHAR, "▒▒");
}

#[test]
fn test_style_helpers() {
    let theme = Theme::apple_dark();
    let title = theme.title_style();
    assert_eq!(title.fg, Some(theme.accent));

    let focused = theme.border_style(true);
    assert_eq!(focused.fg, Some(theme.border_focused));

    let unfocused = theme.border_style(false);
    assert_eq!(unfocused.fg, Some(theme.border_unfocused));

    let selected = theme.selected_row_style();
    assert_eq!(selected.bg, Some(theme.highlight_bg));
}
