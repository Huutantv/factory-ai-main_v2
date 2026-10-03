//! T5 — columns layout (Rich-style `Columns`).
//!
//! Flows items top-to-bottom into as many columns as fit `width`, like `ls`.
//! Display-only: pipes get one item per line (the legacy list shape).

use console::measure_text_width;

/// Lay out `items` in columns separated by `gap` spaces.
pub struct Columns;

impl Columns {
    pub fn layout(items: &[String], width: usize, gap: usize, decorate: bool) -> String {
        if items.is_empty() {
            return String::new();
        }
        if !decorate {
            let mut out = String::new();
            for item in items {
                out.push_str(item);
                out.push('\n');
            }
            return out;
        }
        let width = width.max(20);
        let col_w = items
            .iter()
            .map(|s| measure_text_width(s))
            .max()
            .unwrap_or(0)
            .max(1);
        let ncols = ((width + gap) / (col_w + gap)).max(1).min(items.len());
        if ncols == 1 {
            return Self::layout(items, width, gap, false);
        }
        let nrows = items.len().div_ceil(ncols);
        let mut out = String::new();
        for row in 0..nrows {
            let mut line = String::new();
            for col in 0..ncols {
                let idx = col * nrows + row;
                if idx >= items.len() {
                    continue;
                }
                if col > 0 {
                    line.push_str(&" ".repeat(gap));
                }
                line.push_str(&pad_to(&items[idx], col_w));
            }
            out.push_str(line.trim_end());
            out.push('\n');
        }
        out
    }
}

fn pad_to(s: &str, width: usize) -> String {
    let w = measure_text_width(s);
    if w >= width {
        s.to_string()
    } else {
        format!("{s}{}", " ".repeat(width - w))
    }
}

#[cfg(test)]
mod tests {
    use super::Columns;

    fn items() -> Vec<String> {
        vec![
            "opus-4-8".to_string(),
            "gpt-5-mini".to_string(),
            "llama-3.3-70b".to_string(),
            "qwen-3-32b".to_string(),
        ]
    }

    #[test]
    fn tty_flows_into_columns_and_pipe_stays_flat() {
        let tty = Columns::layout(&items(), 80, 2, true);
        assert!(tty.contains("opus-4-8"), "{tty}");
        assert!(tty.contains("qwen-3-32b"), "{tty}");
        // At least two items share one physical line on a wide TTY.
        assert!(
            tty.lines()
                .any(|l| l.contains("opus-4-8") && l.contains("gpt-5-mini")),
            "multi-column: {tty}"
        );
        let pipe = Columns::layout(&items(), 80, 2, false);
        assert_eq!(pipe, "opus-4-8\ngpt-5-mini\nllama-3.3-70b\nqwen-3-32b\n");
    }

    #[test]
    fn narrow_width_falls_back_to_one_per_line() {
        let s = Columns::layout(&items(), 10, 2, true);
        assert_eq!(s.lines().count(), 4, "{s}");
    }

    #[test]
    fn empty_items_render_nothing() {
        assert_eq!(Columns::layout(&[], 80, 2, true), "");
        assert_eq!(Columns::layout(&[], 80, 2, false), "");
    }
}
