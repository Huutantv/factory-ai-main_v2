//! P3 — progress bars (Rich-style).
//!
//! Flicker-free single + multi bars for long tasks (crawl, model-download,
//! update, bench, workflow fan-out). TTY-only: `render()` returns an empty
//! string when `decorate` is false so piped/CI output stays clean, and
//! callers must skip progress while the retained TUI owns the screen
//! (same rule as `ui::spinner`).

use crate::ui::theme;

/// One bar: `label  ████████░░░░  42% (42/100)`.
#[derive(Debug, Clone)]
pub struct ProgressBar {
    pub label: String,
    pub total: u64,
    pub done: u64,
}

impl ProgressBar {
    pub fn new(label: &str, total: u64) -> Self {
        Self {
            label: label.to_string(),
            total: total.max(1),
            done: 0,
        }
    }

    pub fn set(&mut self, done: u64) {
        self.done = done.min(self.total);
    }

    pub fn inc(&mut self, by: u64) {
        self.set(self.done.saturating_add(by));
    }

    pub fn fraction(&self) -> f64 {
        self.done as f64 / self.total as f64
    }

    /// Redraw throttle: true when the bar advanced a full percent since
    /// `last_drawn` (a fraction), or when it just completed. Callers draw on
    /// `true` and store `fraction()` — per-chunk redraws would flicker and
    /// burn CPU on fast streams.
    pub fn redraw_due(&self, last_drawn: f64) -> bool {
        self.done >= self.total || self.fraction() - last_drawn >= 0.01
    }

    /// Render one line. Empty string when `!decorate`.
    pub fn render(&self, width: usize, decorate: bool) -> String {
        if !decorate {
            return String::new();
        }
        let width = width.max(20);
        // label + space + bar + space + " 100%" + space + "(d/t)"
        let counts = format!("({}/{})", self.done, self.total);
        let pct = format!("{:>3}%", (self.fraction() * 100.0).round() as u64);
        let label_w = self.label.chars().count().min(24);
        let bar_w = width
            .saturating_sub(label_w + 1 + 2 + pct.len() + 1 + counts.len() + 2)
            .clamp(8, 40);
        let filled = (self.fraction() * bar_w as f64).round() as usize;
        let mut bar = String::new();
        for _ in 0..filled {
            bar.push('█');
        }
        for _ in filled..bar_w {
            bar.push('░');
        }
        format!(
            "{} {} {} {}",
            theme::accent(truncate_pad(&self.label, label_w)),
            theme::ok(bar),
            theme::muted(pct),
            theme::faint(counts),
        )
    }
}

/// A stack of bars rendered as consecutive lines (like Rich's task table).
#[derive(Debug, Default)]
pub struct MultiProgress {
    pub bars: Vec<ProgressBar>,
}

impl MultiProgress {
    pub fn add(&mut self, bar: ProgressBar) {
        self.bars.push(bar);
    }

    pub fn render(&self, width: usize, decorate: bool) -> String {
        if !decorate {
            return String::new();
        }
        self.bars
            .iter()
            .map(|b| b.render(width, true))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn truncate_pad(s: &str, width: usize) -> String {
    let mut out = String::new();
    let mut w = 0;
    for ch in s.chars() {
        if w >= width {
            break;
        }
        out.push(ch);
        w += 1;
    }
    while w < width {
        out.push(' ');
        w += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_shows_pct_and_counts() {
        let mut b = ProgressBar::new("crawl", 100);
        b.set(42);
        let s = b.render(80, true);
        assert!(s.contains("42%"), "{s}");
        assert!(s.contains("(42/100)"), "{s}");
        assert!(s.contains('█') && s.contains('░'), "{s}");
    }

    #[test]
    fn pipe_renders_nothing() {
        let mut b = ProgressBar::new("crawl", 10);
        b.set(5);
        assert_eq!(b.render(80, false), "");
        assert_eq!(MultiProgress::default().render(80, false), "");
    }

    #[test]
    fn multi_stacks_bars_line_per_task() {
        let mut m = MultiProgress::default();
        m.add(ProgressBar::new("a", 10));
        m.add(ProgressBar::new("b", 20));
        let s = m.render(80, true);
        assert_eq!(s.lines().count(), 2, "{s}");
    }

    #[test]
    fn done_clamps_at_total() {
        let mut b = ProgressBar::new("dl", 5);
        b.set(99);
        assert!(b.render(60, true).contains("100%"));
    }

    #[test]
    fn redraw_due_throttles_to_percent_steps_and_final() {
        let mut b = ProgressBar::new("dl", 100);
        assert!(!b.redraw_due(0.0), "nothing done, nothing to draw");
        b.set(1);
        assert!(b.redraw_due(0.0), "crossed a 1% step");
        assert!(!b.redraw_due(0.01), "same step, stay quiet");
        b.set(100);
        assert!(b.redraw_due(0.99), "completion always redraws");
    }
}
