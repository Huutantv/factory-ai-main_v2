//! P1 — upgraded tables (Rich-style).
//!
//! A `RichTable` renders with unicode box borders on a TTY, auto-shares the
//! available width across columns, wraps (never mid-codepoint) instead of
//! truncating, and falls back to stacked `key: value` records when the
//! terminal is too narrow. Pipes get tab-separated plain text.
//!
//! Display-only: callers keep whatever flat text they feed the model and
//! use this solely for human-facing CLI output.

use crate::ui::theme;
use console::measure_text_width;

const MIN_COL: usize = 8;

/// A simple human-facing table.
#[derive(Debug, Default, Clone)]
pub struct RichTable {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl RichTable {
    pub fn new(headers: &[&str]) -> Self {
        Self {
            headers: headers.iter().map(|h| h.to_string()).collect(),
            rows: Vec::new(),
        }
    }

    pub fn add_row(&mut self, cells: &[&str]) {
        self.rows
            .push(cells.iter().map(|c| c.to_string()).collect());
    }

    fn ncol(&self) -> usize {
        self.headers.len()
    }

    /// Render the table. `width` is the terminal width in cells.
    pub fn render(&self, width: usize, decorate: bool) -> String {
        if !decorate {
            return self.render_plain();
        }
        let width = width.max(20);
        if width < 40 {
            return self.render_stacked();
        }
        let ncol = self.ncol();
        if ncol == 0 {
            return String::new();
        }
        // Natural widths (header vs cells), then fair-share shrink to fit.
        let mut nat = vec![MIN_COL; ncol];
        for (i, h) in self.headers.iter().enumerate() {
            nat[i] = nat[i].max(measure_text_width(h).max(MIN_COL));
        }
        for row in &self.rows {
            for (i, c) in row.iter().enumerate().take(ncol) {
                nat[i] = nat[i].max(measure_text_width(first_line(c)).clamp(MIN_COL, 48));
            }
        }
        // chrome: "│ " + " │ " * (n-1) + " │" + borders
        let chrome = 3 * ncol + 1;
        let mut budget = width.saturating_sub(chrome);
        let mut col_w: Vec<usize> = nat.clone();
        let total_nat: usize = nat.iter().sum();
        if total_nat > budget {
            // shrink widest first, down to MIN_COL
            while col_w.iter().sum::<usize>() > budget && col_w.iter().any(|&w| w > MIN_COL) {
                if let Some(idx) = col_w
                    .iter()
                    .enumerate()
                    .filter(|(_, &w)| w > MIN_COL)
                    .max_by_key(|(_, &w)| w)
                    .map(|(i, _)| i)
                {
                    col_w[idx] -= 1;
                } else {
                    break;
                }
            }
            budget = col_w.iter().sum();
            let _ = budget;
        }
        let mut out = String::new();
        out.push_str(&self.border(&col_w, '╭', '─', '┬', '╮'));
        out.push('\n');
        out.push_str(&self.row_line(&self.headers, &col_w, true));
        out.push('\n');
        out.push_str(&self.border(&col_w, '├', '─', '┼', '┤'));
        out.push('\n');
        // Wrap tall cells into multiple physical lines.
        let wrapped: Vec<Vec<Vec<String>>> = self
            .rows
            .iter()
            .map(|row| {
                (0..ncol)
                    .map(|i| wrap_cell(row.get(i).map(String::as_str).unwrap_or(""), col_w[i]))
                    .collect()
            })
            .collect();
        for (ri, wrow) in wrapped.iter().enumerate() {
            let height = wrow.iter().map(Vec::len).max().unwrap_or(1);
            for li in 0..height {
                let cells: Vec<String> = (0..ncol)
                    .map(|i| wrow[i].get(li).cloned().unwrap_or_default())
                    .collect();
                out.push_str(&self.row_line_cells(&cells, &col_w, false));
                out.push('\n');
            }
            if ri + 1 < wrapped.len() {
                out.push_str(&self.border(&col_w, '├', '─', '┼', '┤'));
                out.push('\n');
            }
        }
        out.push_str(&self.border(&col_w, '╰', '─', '┴', '╯'));
        out
    }

    fn render_plain(&self) -> String {
        let mut out = String::new();
        out.push_str(&self.headers.join("\t"));
        out.push('\n');
        for row in &self.rows {
            out.push_str(&row.join("\t"));
            out.push('\n');
        }
        out
    }

    fn render_stacked(&self) -> String {
        // Narrow-terminal fallback: one `header: value` block per row.
        let mut out = String::new();
        for (ri, row) in self.rows.iter().enumerate() {
            out.push_str(&format!("── record {} ──\n", ri + 1));
            for (i, h) in self.headers.iter().enumerate() {
                let v = row.get(i).map(String::as_str).unwrap_or("");
                out.push_str(&format!("{h}: {v}\n"));
            }
        }
        out
    }

    fn border(&self, widths: &[usize], l: char, fill: char, mid: char, r: char) -> String {
        let mut s = String::new();
        s.push(l);
        for (i, w) in widths.iter().enumerate() {
            for _ in 0..*w + 2 {
                s.push(fill);
            }
            if i + 1 < widths.len() {
                s.push(mid);
            }
        }
        s.push(r);
        theme::faint(s).to_string()
    }

    fn row_line(&self, cells: &[String], widths: &[usize], header: bool) -> String {
        let padded: Vec<String> = cells
            .iter()
            .enumerate()
            .map(|(i, c)| pad_cell(first_line(c), widths.get(i).copied().unwrap_or(MIN_COL)))
            .collect();
        self.row_line_cells(&padded, widths, header)
    }

    fn row_line_cells(&self, cells: &[String], _widths: &[usize], header: bool) -> String {
        let bar = theme::faint("│").to_string();
        let mut s = bar.clone();
        for c in cells {
            s.push(' ');
            s.push_str(&if header {
                theme::accent(c).to_string()
            } else {
                c.clone()
            });
            s.push(' ');
            s.push_str(&bar);
        }
        s
    }
}

fn first_line(s: &str) -> &str {
    s.split('\n').next().unwrap_or("")
}

fn pad_cell(s: &str, width: usize) -> String {
    let w = measure_text_width(s);
    if w >= width {
        truncate_to(s, width)
    } else {
        format!("{s}{}", " ".repeat(width - w))
    }
}

fn truncate_to(s: &str, width: usize) -> String {
    let mut out = String::new();
    let mut w = 0;
    for ch in s.chars() {
        let cw = console::measure_text_width(&ch.to_string());
        if w + cw > width {
            break;
        }
        out.push(ch);
        w += cw;
    }
    out
}

fn wrap_cell(s: &str, width: usize) -> Vec<String> {
    let width = width.max(4);
    let mut lines = Vec::new();
    for para in s.split('\n') {
        let mut cur = String::new();
        let mut cw = 0;
        for word in para.split(' ') {
            let ww = measure_text_width(word);
            if ww > width {
                // Long token: hard-split by chars.
                if !cur.is_empty() {
                    lines.push(std::mem::take(&mut cur));
                }
                let mut chunk = String::new();
                let mut chunk_w = 0;
                for ch in word.chars() {
                    let ch_w = measure_text_width(&ch.to_string());
                    if chunk_w + ch_w > width {
                        lines.push(std::mem::take(&mut chunk));
                        chunk_w = 0;
                    }
                    chunk.push(ch);
                    chunk_w += ch_w;
                }
                cur = chunk;
                cw = chunk_w;
                continue;
            }
            let need = if cur.is_empty() { ww } else { cw + 1 + ww };
            if need > width {
                lines.push(std::mem::take(&mut cur));
                cw = 0;
            }
            if !cur.is_empty() {
                cur.push(' ');
                cw += 1;
            }
            cur.push_str(word);
            cw += ww;
        }
        lines.push(std::mem::take(&mut cur));
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines.into_iter().map(|l| pad_cell(&l, width)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> RichTable {
        let mut t = RichTable::new(&["Date", "Title", "Box Office"]);
        t.add_row(&[
            "Dec 20, 2019",
            "Star Wars: The Rise of Skywalker",
            "$375,126,118",
        ]);
        t.add_row(&[
            "Dec 15, 2017",
            "Star Wars Ep. VIII: The Last Jedi",
            "$1,332,539,889",
        ]);
        t
    }

    #[test]
    fn tty_render_has_borders_and_headers() {
        let s = sample().render(100, true);
        assert!(s.contains('╭') && s.contains('╰'), "{s}");
        assert!(s.contains("Box Office"), "{s}");
    }

    #[test]
    fn narrow_falls_back_to_stacked_records() {
        let s = sample().render(30, true);
        assert!(s.contains("record 1"), "{s}");
        assert!(!s.contains('╭'), "no box art when stacked: {s}");
    }

    #[test]
    fn pipe_output_is_plain_tabs() {
        let s = sample().render(100, false);
        assert!(s.contains('\t'), "{s}");
        assert!(!s.contains('╭'), "{s}");
    }

    #[test]
    fn long_cells_wrap_without_panicking_on_unicode() {
        let mut t = RichTable::new(&["Name", "Note"]);
        t.add_row(&[
            "ếch 🐸",
            "một chuỗi rất dài với tiếng Việt có dấu và emoji 🐸🐸🐸",
        ]);
        let s = t.render(50, true);
        assert!(s.contains("ếch"), "{s}");
    }
}
