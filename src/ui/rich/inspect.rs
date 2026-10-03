//! T6 — inspect (Rich-style `Inspect`).
//!
//! Pretty-prints key/value sections with indentation and highlighted keys,
//! wrapping long values instead of truncating them. Display-only: pipes get
//! flat `key: value` lines in the same order.

use crate::ui::theme;
use console::measure_text_width;

/// Render titled key/value fields.
pub struct Inspect;

impl Inspect {
    pub fn render(
        title: &str,
        fields: &[(String, String)],
        width: usize,
        decorate: bool,
    ) -> String {
        if fields.is_empty() {
            return String::new();
        }
        if !decorate {
            let mut out = String::new();
            for (k, v) in fields {
                out.push_str(k);
                out.push_str(": ");
                out.push_str(v);
                out.push('\n');
            }
            return out;
        }
        let width = width.max(20);
        let mut out = String::new();
        out.push_str(&theme::accent(title).to_string());
        out.push('\n');
        for (k, v) in fields {
            let head_len = k.chars().count() + 4; // "  " + ": "
            let vw = width.saturating_sub(head_len + 1).max(10);
            let wrapped = wrap_words(v, vw);
            out.push_str("  ");
            out.push_str(&theme::accent(k).to_string());
            out.push_str(": ");
            out.push_str(&wrapped[0]);
            out.push('\n');
            let indent = " ".repeat(head_len + 1);
            for rest in wrapped.iter().skip(1) {
                out.push_str(&indent);
                out.push_str(rest);
                out.push('\n');
            }
        }
        out
    }
}

fn wrap_words(s: &str, width: usize) -> Vec<String> {
    let mut lines = vec![String::new()];
    for word in s.split(' ') {
        let cur = lines.last().map(String::len).unwrap_or(0);
        let ww = measure_text_width(word);
        if cur > 0 && cur + 1 + ww > width {
            lines.push(String::new());
        }
        let last = lines.last_mut().unwrap();
        if !last.is_empty() {
            last.push(' ');
        }
        last.push_str(word);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::Inspect;

    fn sample() -> Vec<(String, String)> {
        vec![
            ("id".to_string(), "pnpm-over-npm".to_string()),
            (
                "body".to_string(),
                "the user prefers pnpm over npm for every javascript project".to_string(),
            ),
        ]
    }

    #[test]
    fn tty_shows_sections_with_keys_and_wraps_long_values() {
        let s = Inspect::render("fact", &sample(), 40, true);
        assert!(s.contains("fact"), "{s}");
        assert!(s.contains("pnpm-over-npm"), "{s}");
        assert!(s.contains("prefers pnpm"), "{s}");
    }

    #[test]
    fn pipe_returns_flat_key_value_lines_in_order() {
        let s = Inspect::render("fact", &sample(), 40, false);
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines.len(), 2, "{s}");
        assert!(lines[0].starts_with("id: pnpm-over-npm"), "{s}");
        assert!(lines[1].starts_with("body: "), "{s}");
    }

    #[test]
    fn empty_fields_render_nothing() {
        assert_eq!(Inspect::render("fact", &[], 40, true), "");
        assert_eq!(Inspect::render("fact", &[], 40, false), "");
    }
}
