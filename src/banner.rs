//! The Visor banner: the app icon in terminal pixels next to the wordmark.
//!
//! Each character cell shows two icon pixels with the upper half block `▀`:
//! the foreground paints the top pixel and the background the bottom one.
//! The icon needs color, so without it the banner is the wordmark alone.

use std::fmt::Write as _;
use std::io::IsTerminal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rgb(u8, u8, u8);

/// How many colors the terminal shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Palette {
    None,
    Ansi256,
    TrueColor,
}

impl Palette {
    /// Reads `NO_COLOR` (<https://no-color.org>) and `COLORTERM`. Terminals
    /// that support 24-bit color set `COLORTERM`; Apple Terminal does not.
    #[must_use]
    pub fn detect(is_terminal: bool) -> Self {
        let set = |name| std::env::var_os(name).is_some_and(|value| !value.is_empty());
        if !is_terminal || set("NO_COLOR") || std::env::var("TERM").as_deref() == Ok("dumb") {
            Self::None
        } else if matches!(std::env::var("COLORTERM").as_deref(), Ok("truecolor" | "24bit")) {
            Self::TrueColor
        } else {
            Self::Ansi256
        }
    }

    fn fg(self, color: Rgb) -> String {
        match self {
            Self::None => String::new(),
            Self::Ansi256 => format!("\x1b[38;5;{}m", ansi256(color)),
            Self::TrueColor => format!("\x1b[38;2;{};{};{}m", color.0, color.1, color.2),
        }
    }

    fn bg(self, color: Rgb) -> String {
        match self {
            Self::None => String::new(),
            Self::Ansi256 => format!("\x1b[48;5;{}m", ansi256(color)),
            Self::TrueColor => format!("\x1b[48;2;{};{};{}m", color.0, color.1, color.2),
        }
    }
}

/// The nearest color of the xterm 6×6×6 cube (codes 16 to 231).
fn ansi256(Rgb(r, g, b): Rgb) -> u8 {
    const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];
    let nearest = |value: u8| {
        (0..6u8)
            .min_by_key(|&i| (i16::from(LEVELS[usize::from(i)]) - i16::from(value)).abs())
            .expect("six levels")
    };
    16 + 36 * nearest(r) + 6 * nearest(g) + nearest(b)
}

/// The wordmark in thin lines, like the logo's lettering.
const WORDMARK: [&str; 5] = [
    "╲        ╱   │   ╭───────╴   ╭───────╮   ╭───────╴",
    " ╲      ╱    │   │           │       │   │",
    "  ╲    ╱     │   ╰───────╮   │       │   │",
    "   ╲  ╱      │           │   │       │   │",
    "    ╲╱       │   ╶───────╯   ╰───────╯   │",
];

/// The Visor app icon, 17×17 pixels, sampled from `icon.svg`.
const ICON: [[Option<Rgb>; 17]; 17] = [
    [
        None,
        None,
        Some(Rgb(0, 138, 249)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 138, 249)),
        None,
        None,
    ],
    [
        None,
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 112, 202)),
        None,
    ],
    [
        Some(Rgb(0, 138, 249)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 138, 249)),
    ],
    [
        Some(Rgb(0, 112, 202)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(0, 112, 202)),
    ],
    [
        Some(Rgb(0, 112, 202)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(0, 112, 202)),
    ],
    [
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 112, 202)),
    ],
    [
        Some(Rgb(0, 112, 202)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(0, 10, 20)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 112, 202)),
    ],
    [
        Some(Rgb(0, 138, 249)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(67, 224, 233)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(0, 10, 20)),
        Some(Rgb(0, 10, 20)),
        Some(Rgb(0, 10, 20)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(7, 214, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 213, 226)),
        Some(Rgb(0, 137, 249)),
    ],
    [
        Some(Rgb(0, 130, 246)),
        Some(Rgb(0, 131, 247)),
        Some(Rgb(0, 130, 247)),
        Some(Rgb(1, 131, 247)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(0, 10, 20)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(1, 131, 247)),
        Some(Rgb(0, 130, 246)),
        Some(Rgb(0, 130, 246)),
        Some(Rgb(0, 131, 247)),
    ],
    [
        Some(Rgb(0, 124, 244)),
        Some(Rgb(1, 124, 245)),
        Some(Rgb(0, 124, 245)),
        Some(Rgb(0, 124, 245)),
        Some(Rgb(0, 123, 244)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(0, 124, 244)),
        Some(Rgb(0, 124, 245)),
        Some(Rgb(0, 124, 244)),
        Some(Rgb(0, 124, 244)),
        Some(Rgb(1, 124, 245)),
    ],
    [
        Some(Rgb(0, 117, 242)),
        Some(Rgb(0, 117, 242)),
        Some(Rgb(0, 117, 242)),
        Some(Rgb(0, 117, 242)),
        Some(Rgb(0, 117, 242)),
        Some(Rgb(0, 117, 242)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(255, 255, 255)),
        Some(Rgb(250, 252, 254)),
        Some(Rgb(1, 117, 242)),
        Some(Rgb(1, 117, 242)),
        Some(Rgb(0, 117, 242)),
        Some(Rgb(0, 117, 242)),
        Some(Rgb(0, 117, 242)),
        Some(Rgb(0, 117, 242)),
    ],
    [
        Some(Rgb(0, 109, 240)),
        Some(Rgb(1, 110, 240)),
        Some(Rgb(0, 109, 240)),
        Some(Rgb(1, 110, 240)),
        Some(Rgb(0, 109, 240)),
        Some(Rgb(1, 110, 240)),
        Some(Rgb(0, 110, 240)),
        Some(Rgb(1, 110, 240)),
        Some(Rgb(1, 110, 240)),
        Some(Rgb(0, 109, 240)),
        Some(Rgb(1, 110, 240)),
        Some(Rgb(0, 110, 240)),
        Some(Rgb(1, 110, 240)),
        Some(Rgb(1, 110, 240)),
        Some(Rgb(0, 110, 240)),
        Some(Rgb(0, 109, 240)),
        Some(Rgb(1, 110, 240)),
    ],
    [
        Some(Rgb(1, 103, 237)),
        Some(Rgb(0, 102, 237)),
        Some(Rgb(0, 102, 237)),
        Some(Rgb(0, 102, 237)),
        Some(Rgb(0, 102, 237)),
        Some(Rgb(1, 103, 238)),
        Some(Rgb(0, 102, 237)),
        Some(Rgb(0, 103, 237)),
        Some(Rgb(0, 102, 237)),
        Some(Rgb(0, 102, 237)),
        Some(Rgb(1, 103, 238)),
        Some(Rgb(0, 102, 237)),
        Some(Rgb(0, 103, 237)),
        Some(Rgb(0, 102, 237)),
        Some(Rgb(1, 103, 238)),
        Some(Rgb(1, 103, 237)),
        Some(Rgb(0, 102, 237)),
    ],
    [
        Some(Rgb(0, 95, 235)),
        Some(Rgb(1, 96, 235)),
        Some(Rgb(1, 96, 235)),
        Some(Rgb(1, 96, 236)),
        Some(Rgb(1, 96, 235)),
        Some(Rgb(0, 96, 235)),
        Some(Rgb(1, 96, 235)),
        Some(Rgb(0, 96, 235)),
        Some(Rgb(1, 96, 236)),
        Some(Rgb(1, 96, 235)),
        Some(Rgb(0, 96, 235)),
        Some(Rgb(1, 96, 235)),
        Some(Rgb(0, 96, 235)),
        Some(Rgb(1, 96, 236)),
        Some(Rgb(0, 95, 235)),
        Some(Rgb(0, 95, 235)),
        Some(Rgb(1, 96, 235)),
    ],
    [
        Some(Rgb(1, 89, 233)),
        Some(Rgb(1, 88, 232)),
        Some(Rgb(1, 89, 233)),
        Some(Rgb(0, 88, 232)),
        Some(Rgb(1, 88, 233)),
        Some(Rgb(0, 88, 232)),
        Some(Rgb(0, 88, 232)),
        Some(Rgb(0, 88, 232)),
        Some(Rgb(0, 88, 232)),
        Some(Rgb(1, 88, 233)),
        Some(Rgb(0, 88, 232)),
        Some(Rgb(0, 88, 232)),
        Some(Rgb(0, 88, 232)),
        Some(Rgb(0, 88, 232)),
        Some(Rgb(1, 89, 233)),
        Some(Rgb(1, 89, 233)),
        Some(Rgb(1, 88, 232)),
    ],
    [
        None,
        Some(Rgb(1, 82, 231)),
        Some(Rgb(0, 81, 230)),
        Some(Rgb(1, 81, 230)),
        Some(Rgb(1, 82, 231)),
        Some(Rgb(1, 82, 231)),
        Some(Rgb(0, 81, 230)),
        Some(Rgb(1, 81, 230)),
        Some(Rgb(1, 81, 230)),
        Some(Rgb(1, 82, 231)),
        Some(Rgb(1, 82, 231)),
        Some(Rgb(0, 81, 230)),
        Some(Rgb(1, 81, 230)),
        Some(Rgb(1, 81, 230)),
        Some(Rgb(1, 81, 230)),
        Some(Rgb(1, 82, 231)),
        None,
    ],
    [
        None,
        None,
        Some(Rgb(1, 74, 228)),
        Some(Rgb(1, 74, 228)),
        Some(Rgb(1, 74, 228)),
        Some(Rgb(1, 74, 228)),
        Some(Rgb(1, 74, 228)),
        Some(Rgb(1, 74, 228)),
        Some(Rgb(1, 74, 228)),
        Some(Rgb(1, 74, 228)),
        Some(Rgb(1, 74, 228)),
        Some(Rgb(1, 74, 228)),
        Some(Rgb(1, 74, 228)),
        Some(Rgb(1, 74, 228)),
        Some(Rgb(1, 74, 228)),
        None,
        None,
    ],
];

/// The banner for a terminal with `palette`, ending in a newline.
#[must_use]
pub fn render(palette: Palette) -> String {
    let mut out = String::from("\n");
    if palette == Palette::None {
        for line in WORDMARK {
            let _ = writeln!(out, "  {line}");
        }
        out.push('\n');
        return out;
    }

    // The wordmark sits beside the middle of the icon's rows.
    let icon_rows = ICON.len().div_ceil(2);
    let first_word_row = (icon_rows - WORDMARK.len()) / 2;
    for row in 0..icon_rows {
        out.push_str("  ");
        for col in 0..ICON.len() {
            let top = ICON[row * 2][col];
            let bottom = ICON.get(row * 2 + 1).and_then(|line| line[col]);
            match (top, bottom) {
                (None, None) => out.push(' '),
                (Some(top), None) => {
                    let _ = write!(out, "{}▀\x1b[0m", palette.fg(top));
                }
                (None, Some(bottom)) => {
                    let _ = write!(out, "{}▄\x1b[0m", palette.fg(bottom));
                }
                (Some(top), Some(bottom)) => {
                    let _ = write!(out, "{}{}▀\x1b[0m", palette.fg(top), palette.bg(bottom));
                }
            }
        }
        if let Some(line) = row.checked_sub(first_word_row).and_then(|i| WORDMARK.get(i)) {
            let _ = write!(out, "    \x1b[1m{line}\x1b[0m");
        }
        out.push('\n');
    }
    out.push('\n');
    out
}

/// Prints the banner on standard error when it is a terminal: the banner
/// decorates a conversation with a person and never reaches a pipe.
pub fn print() {
    let stderr = std::io::stderr();
    if stderr.is_terminal() {
        eprint!("{}", render(Palette::detect(true)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strip_ansi(text: &str) -> String {
        let mut out = String::new();
        let mut chars = text.chars();
        while let Some(c) = chars.next() {
            if c == '\x1b' {
                chars.by_ref().find(|&c| c == 'm');
            } else {
                out.push(c);
            }
        }
        out
    }

    #[test]
    fn without_color_the_banner_is_the_wordmark_alone() {
        let banner = render(Palette::None);

        assert!(!banner.contains('\x1b'));
        assert!(!banner.contains('▀'));
        assert!(banner.contains("╭───────╮"));
    }

    #[test]
    fn with_color_the_icon_sits_beside_the_wordmark() {
        let banner = render(Palette::TrueColor);
        let lines: Vec<String> = banner.lines().map(strip_ansi).collect();

        // 17 pixel rows make 9 text rows, between two blank lines.
        assert_eq!(lines.len(), 11);
        assert!(lines.iter().any(|line| line.contains('▀') && line.contains("╲╱")));
        assert!(banner.contains("\x1b[38;2;0;213;226m"), "the cyan band is drawn");
        assert!(
            lines.iter().all(|line| line.chars().count() <= 80),
            "fits 80 columns"
        );
    }

    #[test]
    fn ansi256_uses_cube_codes() {
        let banner = render(Palette::Ansi256);

        assert!(banner.contains("\x1b[38;5;"));
        assert!(!banner.contains("38;2;"));
        assert_eq!(ansi256(Rgb(255, 255, 255)), 231);
        assert_eq!(ansi256(Rgb(0, 0, 0)), 16);
    }
}
