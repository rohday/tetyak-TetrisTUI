use ratatui::style::{Color, Modifier, Style};
use crate::piece::Tetromino;

pub const BLOCK_CHAR: &str = "██";
pub const GHOST_CHAR: &str = "░░";
pub const FLASH_CHAR: &str = "██";
pub const PARTICLE_CHAR: &str = "▒▒";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemePreset {
    #[default]
    AppleDark,
    CatppuccinMocha,
    TokyoNight,
}

impl ThemePreset {
    pub const ALL: [ThemePreset; 3] = [
        ThemePreset::AppleDark,
        ThemePreset::CatppuccinMocha,
        ThemePreset::TokyoNight,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::AppleDark => "Apple Dark",
            Self::CatppuccinMocha => "Catppuccin Mocha",
            Self::TokyoNight => "Tokyo Night",
        }
    }

    pub fn cycle(&self) -> Self {
        match self {
            Self::AppleDark => Self::CatppuccinMocha,
            Self::CatppuccinMocha => Self::TokyoNight,
            Self::TokyoNight => Self::AppleDark,
        }
    }

    pub fn theme(&self) -> Theme {
        match self {
            Self::AppleDark => Theme::apple_dark(),
            Self::CatppuccinMocha => Theme::catppuccin_mocha(),
            Self::TokyoNight => Theme::tokyo_night(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub accent: Color,
    pub secondary: Color,
    pub text_primary: Color,
    pub text_muted: Color,
    pub border_unfocused: Color,
    pub border_focused: Color,
    pub highlight_bg: Color,
    pub ghost: Color,
    pub flash: Color,
    pub particle: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::apple_dark()
    }
}

impl Theme {
    pub fn apple_dark() -> Self {
        Self {
            accent: Color::Rgb(250, 45, 72),      // Apple Red
            secondary: Color::Rgb(140, 140, 240), // Soft Purple
            text_primary: Color::Rgb(240, 240, 245),
            text_muted: Color::Rgb(130, 130, 140),
            border_unfocused: Color::Rgb(60, 60, 70),
            border_focused: Color::Rgb(250, 45, 72),
            highlight_bg: Color::Rgb(40, 40, 50),
            ghost: Color::Rgb(70, 70, 80),
            flash: Color::Rgb(255, 255, 255),
            particle: Color::Rgb(180, 180, 200),
        }
    }

    pub fn catppuccin_mocha() -> Self {
        Self {
            accent: Color::Rgb(203, 166, 247),    // Mauve
            secondary: Color::Rgb(116, 199, 236), // Sapphire
            text_primary: Color::Rgb(205, 214, 244),
            text_muted: Color::Rgb(147, 153, 178),
            border_unfocused: Color::Rgb(69, 71, 90),
            border_focused: Color::Rgb(203, 166, 247),
            highlight_bg: Color::Rgb(49, 50, 68),
            ghost: Color::Rgb(88, 91, 112),
            flash: Color::Rgb(255, 255, 255),
            particle: Color::Rgb(186, 194, 222),
        }
    }

    pub fn tokyo_night() -> Self {
        Self {
            accent: Color::Rgb(125, 207, 255),    // Electric Blue
            secondary: Color::Rgb(187, 154, 247), // Magenta Purple
            text_primary: Color::Rgb(192, 202, 245),
            text_muted: Color::Rgb(86, 95, 137),
            border_unfocused: Color::Rgb(41, 46, 66),
            border_focused: Color::Rgb(125, 207, 255),
            highlight_bg: Color::Rgb(47, 56, 93),
            ghost: Color::Rgb(65, 72, 104),
            flash: Color::Rgb(255, 255, 255),
            particle: Color::Rgb(154, 165, 206),
        }
    }

    pub fn block_color(&self, t: Tetromino) -> Color {
        t.color()
    }

    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn border_style(&self, focused: bool) -> Style {
        if focused {
            Style::default().fg(self.border_focused)
        } else {
            Style::default().fg(self.border_unfocused)
        }
    }

    pub fn selected_row_style(&self) -> Style {
        Style::default()
            .bg(self.highlight_bg)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    }
}
