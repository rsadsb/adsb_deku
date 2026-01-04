use ratatui_core::style::Color;

mod ayu_dark {
    use super::Color;
    use ratatui_style_ayu::ayu_dark_heretek as ayu_dark;

    pub const FG: Color = ayu_dark::FOREGROUND;
    pub const GUIDE: Color = ayu_dark::COMMENT;
    pub const ORANGE: Color = ayu_dark::ORANGE;
    pub const YELLOW: Color = ayu_dark::YELLOW;
    pub const BLUE: Color = ayu_dark::BLUE;
    pub const GREEN: Color = ayu_dark::GREEN;
    pub const PURPLE: Color = ayu_dark::PURPLE;
    pub const CYAN: Color = ayu_dark::CYAN;
    pub const UI_BORDER: Color = ayu_dark::SELECTION;
    pub const SELECTION: Color = ayu_dark::BLUE;
    pub const CYAN_BRIGHT: Color = ayu_dark::CYAN;
    pub const PINK: Color = Color::Rgb(0xF2, 0x8F, 0xB0);

    pub const COV_0: Color = Color::Rgb(0x73, 0xB8, 0x73);
    pub const COV_1: Color = Color::Rgb(0x95, 0xD5, 0x73);
    pub const COV_2: Color = Color::Rgb(0xB8, 0xE6, 0x73);
    pub const COV_3: Color = Color::Rgb(0xE6, 0xCC, 0x73);
    pub const COV_4: Color = Color::Rgb(0xFF, 0xB8, 0x73);
    pub const COV_5: Color = Color::Rgb(0xFF, 0xA7, 0x59);
    pub const COV_6: Color = Color::Rgb(0xE6, 0x95, 0xB8);
    pub const COV_7: Color = Color::Rgb(0xD2, 0x95, 0xCC);
    pub const COV_8: Color = Color::Rgb(0xD2, 0xA6, 0xE6);
    pub const COV_9: Color = Color::Rgb(0xD2, 0xA6, 0xFF);
}

mod ayu_light {
    use super::Color;
    use ratatui_style_ayu::ayu_light;

    pub const FG: Color = ayu_light::FOREGROUND;
    pub const GUIDE: Color = ayu_light::COMMENT;
    pub const ORANGE: Color = ayu_light::ORANGE;
    pub const YELLOW: Color = ayu_light::YELLOW;
    pub const BLUE: Color = ayu_light::BLUE;
    pub const GREEN: Color = ayu_light::GREEN;
    pub const PURPLE: Color = ayu_light::PURPLE;
    pub const CYAN: Color = ayu_light::CYAN;
    pub const UI_BORDER: Color = ayu_light::SELECTION;
    pub const SELECTION: Color = ayu_light::BLUE;
    pub const RED: Color = ayu_light::RED;

    pub const COV_0: Color = Color::Rgb(0x86, 0xB3, 0x00);
    pub const COV_1: Color = Color::Rgb(0xA0, 0xC6, 0x20);
    pub const COV_2: Color = Color::Rgb(0xB8, 0xCC, 0x52);
    pub const COV_3: Color = Color::Rgb(0xD5, 0xB3, 0x60);
    pub const COV_4: Color = Color::Rgb(0xF2, 0xAE, 0x49);
    pub const COV_5: Color = Color::Rgb(0xFF, 0x91, 0x40);
    pub const COV_6: Color = Color::Rgb(0xE6, 0x84, 0x8C);
    pub const COV_7: Color = Color::Rgb(0xCC, 0x7A, 0xA3);
    pub const COV_8: Color = Color::Rgb(0xB8, 0x7A, 0xCC);
    pub const COV_9: Color = Color::Rgb(0xA3, 0x7A, 0xCC);
}

/// Color themes for the radar application
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorTheme {
    AyuDark,
    AyuLight,
}

impl ColorTheme {
    /// Get colors for this theme
    pub fn colors(&self) -> ThemeColors {
        match self {
            Self::AyuDark => ThemeColors::ayu_dark(),
            Self::AyuLight => ThemeColors::ayu_light(),
        }
    }
}

impl std::str::FromStr for ColorTheme {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ayu-dark" | "ayu_dark" | "ayudark" => Ok(Self::AyuDark),
            "ayu-light" | "ayu_light" | "ayulight" => Ok(Self::AyuLight),
            _ => Err(format!("Unknown color theme: {}. Available: ayu-dark, ayu-light", s)),
        }
    }
}

/// Color palette for a theme
#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    /// Primary text color
    pub text: Color,
    /// Secondary text color (dimmer)
    pub text_dim: Color,
    /// Accent/highlight color
    pub accent: Color,
    /// Secondary accent color
    pub accent_secondary: Color,
    /// Aircraft position color
    pub aircraft: Color,
    /// Location marker color
    pub location: Color,
    /// Track/trail color
    pub track: Color,
    /// Heading indicator color
    pub heading: Color,
    /// Grid lines color
    pub grid: Color,
    /// Range circles color
    pub range_circles: Color,
    /// Range circle labels color
    pub range_labels: Color,
    /// Tab border color
    pub border: Color,
    /// Tab title color
    pub title: Color,
    /// Table header color
    pub table_header: Color,
    /// Airport marker color
    pub airport: Color,
    /// Coverage heatmap base color (RGB will be computed based on density)
    pub coverage_base: u8,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self::ayu_dark()
    }
}

impl ThemeColors {
    pub fn ayu_dark() -> Self {
        Self {
            text: Color::White,
            text_dim: ayu_dark::GUIDE,
            accent: ayu_dark::ORANGE,
            accent_secondary: ayu_dark::YELLOW,
            aircraft: ayu_dark::BLUE,
            location: ayu_dark::GREEN,
            track: ayu_dark::PURPLE,
            heading: ayu_dark::CYAN,
            grid: ayu_dark::UI_BORDER,
            range_circles: ayu_dark::SELECTION,
            range_labels: ayu_dark::CYAN_BRIGHT,
            border: ayu_dark::GREEN,
            title: ayu_dark::PURPLE,
            table_header: ayu_dark::ORANGE,
            airport: ayu_dark::PINK,
            coverage_base: 90,
        }
    }

    pub fn ayu_light() -> Self {
        Self {
            text: ayu_light::FG,
            text_dim: ayu_light::GUIDE,
            accent: ayu_light::ORANGE,
            accent_secondary: ayu_light::YELLOW,
            aircraft: ayu_light::BLUE,
            location: ayu_light::GREEN,
            track: ayu_light::PURPLE,
            heading: ayu_light::CYAN,
            grid: ayu_light::UI_BORDER,
            range_circles: ayu_light::SELECTION,
            range_labels: ayu_light::CYAN,
            border: ayu_light::GREEN,
            title: ayu_light::PURPLE,
            table_header: ayu_light::ORANGE,
            airport: ayu_light::RED,
            coverage_base: 40,
        }
    }

    pub fn coverage_color(&self, seen_number: u32) -> Color {
        match self {
            _ if matches!(self, Self { text, .. } if *text == ayu_dark::FG) => match seen_number {
                0 => ayu_dark::COV_0,
                1 => ayu_dark::COV_1,
                2 => ayu_dark::COV_2,
                3 => ayu_dark::COV_3,
                4 => ayu_dark::COV_4,
                5 => ayu_dark::COV_5,
                6 => ayu_dark::COV_6,
                7 => ayu_dark::COV_7,
                8 => ayu_dark::COV_8,
                _ => ayu_dark::COV_9,
            },
            _ if matches!(self, Self { text, .. } if *text == ayu_light::FG) => match seen_number {
                0 => ayu_light::COV_0,
                1 => ayu_light::COV_1,
                2 => ayu_light::COV_2,
                3 => ayu_light::COV_3,
                4 => ayu_light::COV_4,
                5 => ayu_light::COV_5,
                6 => ayu_light::COV_6,
                7 => ayu_light::COV_7,
                8 => ayu_light::COV_8,
                _ => ayu_light::COV_9,
            },
            _ => {
                let number: u32 = u32::from(self.coverage_base) + seen_number * 50;
                let color_number: u8 =
                    if number > u32::from(u8::MAX) { u8::MAX } else { number as u8 };
                Color::Rgb(color_number, color_number, color_number)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_theme_from_str() {
        assert_eq!("ayu-dark".parse::<ColorTheme>().unwrap(), ColorTheme::AyuDark);
        assert_eq!("ayu_dark".parse::<ColorTheme>().unwrap(), ColorTheme::AyuDark);
        assert_eq!("ayudark".parse::<ColorTheme>().unwrap(), ColorTheme::AyuDark);
        assert_eq!("ayu-light".parse::<ColorTheme>().unwrap(), ColorTheme::AyuLight);
        assert_eq!("ayu_light".parse::<ColorTheme>().unwrap(), ColorTheme::AyuLight);
        assert_eq!("ayulight".parse::<ColorTheme>().unwrap(), ColorTheme::AyuLight);
        assert!("invalid".parse::<ColorTheme>().is_err());
        assert!("default".parse::<ColorTheme>().is_err());
    }

    #[test]
    fn test_ayu_dark_colors() {
        let colors = ColorTheme::AyuDark.colors();
        assert_eq!(colors.text, Color::White);
        assert_eq!(colors.accent, ayu_dark::ORANGE);
        assert_eq!(colors.border, ayu_dark::GREEN);
    }

    #[test]
    fn test_ayu_light_colors() {
        let colors = ColorTheme::AyuLight.colors();
        assert_eq!(colors.text, ayu_light::FG);
        assert_eq!(colors.accent, ayu_light::ORANGE);
        assert_eq!(colors.border, ayu_light::GREEN);

        let dark_colors = ColorTheme::AyuDark.colors();
        assert_ne!(colors.text, dark_colors.text);
    }
}
