//! T7 — rule / align / pad primitives (Rich-style `Rule`, `Align`, `Padding`).
//!
//! Section dividers with optional titles, centered text, and padded boxes.
//! Display-only: pipes degrade to plain titles or `--` lines.

use crate::ui::theme;
use console::measure_text_width;

/// A full-width divider, optionally titled: `── title ──...`.
/// Exactly `width` cells wide on a TTY; plain title (or `--`) on pipes.
pub fn rule(title: Option<&str>, width: usize, decorate: bool) -> String {
    let width = width.max(8);
    if !decorate {
        return title.unwrap_or("--").to_string();
    }
    match title {
        None => "─".repeat(width),
        Some(t) => {
            let tw = measure_text_width(t);
            if tw + 4 >= width {
                return t.chars().take(width).collect();
            }
            let left = 2;
            let right = width - left - tw - 2;
            format!(
                "{} {} {}",
                "─".repeat(left),
                theme::accent(t),
                "─".repeat(right)
            )
        }
    }
}

/// Center `text` in `width` cells. Verbatim on pipes.
pub fn center(text: &str, width: usize, decorate: bool) -> String {
    if !decorate {
        return text.to_string();
    }
    let tw = measure_text_width(text);
    if tw >= width {
        return text.chars().take(width).collect();
    }
    let left = (width - tw) / 2;
    let right = width - tw - left;
    format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
}

/// Surround `text` with `n` blank lines above and below on a TTY.
pub fn pad(text: &str, n: usize, decorate: bool) -> String {
    if !decorate || n == 0 {
        return text.to_string();
    }
    format!("{}\n{text}\n{}", "\n".repeat(n - 1), "\n".repeat(n - 1))
}

#[cfg(test)]
mod tests {
    use super::{center, pad, rule};

    #[test]
    fn tty_rule_spans_full_width_with_title() {
        // Visible width, not char count: another test may enable ANSI globally,
        // and escape bytes must not count toward the 40 cells.
        use console::measure_text_width;
        let s = rule(Some("agents"), 40, true);
        assert_eq!(measure_text_width(&s), 40, "{s}");
        assert!(s.contains("agents"), "{s}");
        let bare = rule(None, 40, true);
        assert_eq!(measure_text_width(&bare), 40, "{bare}");
    }

    #[test]
    fn pipe_rule_degrades_to_plain_title_or_dashes() {
        assert_eq!(rule(Some("agents"), 40, false), "agents");
        assert_eq!(rule(None, 40, false), "--");
    }

    #[test]
    fn center_pads_both_sides_and_pipe_passes_through() {
        let s = center("hi", 10, true);
        assert_eq!(s.chars().count(), 10, "{s}");
        assert!(s.starts_with("    "), "{s}");
        assert_eq!(center("hi", 10, false), "hi");
        assert_eq!(
            center("way too long for this", 10, true).chars().count(),
            10
        );
    }

    #[test]
    fn pad_adds_blank_lines_around() {
        assert_eq!(pad("x", 1, false), "x");
        assert_eq!(pad("x", 1, true), "\nx\n");
        assert_eq!(pad("x", 0, true), "x");
    }
}
