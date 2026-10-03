//! T10 — multi-task status board (Rich-style `Status`/`Live`).
//!
//! One row per fan-out task: id/role, status glyph, steps, elapsed. Rendered
//! as a completion board (final states); per-task live lines already stream
//! from the runner while tasks are in flight. Display-only: pipes get the
//! legacy flat rows (including the model, exactly like the old print).

use crate::ui::theme;

/// One finished fan-out task.
#[derive(Debug, Clone)]
pub struct BoardRow {
    pub id: String,
    pub role: String,
    pub model: String,
    pub status: String,
    pub iters: usize,
    pub elapsed_secs: f64,
}

/// Completion board for a named fan-out.
pub struct TaskBoard;

impl TaskBoard {
    pub fn render(name: &str, rows: &[BoardRow], width: usize, decorate: bool) -> String {
        if rows.is_empty() {
            return String::new();
        }
        if !decorate {
            // Legacy flat rows, byte-stable for pipes.
            let mut out = String::new();
            for r in rows {
                out.push_str(&format!(
                    "  • {} ({}/{}) — {} [{} step(s)]\n",
                    r.id, r.role, r.model, r.status, r.iters
                ));
            }
            return out;
        }
        use super::table::RichTable;
        let mut table = RichTable::new(&["Task", "Role", "Status", "Steps", "Elapsed"]);
        for r in rows {
            let status = match r.status.as_str() {
                "done" => theme::ok(&r.status).to_string(),
                "error" | "cancelled" | "deadline" => theme::err(&r.status).to_string(),
                _ => theme::muted(&r.status).to_string(),
            };
            table.add_row(&[
                &r.id,
                &r.role,
                &status,
                &r.iters.to_string(),
                &format!("{:.1}s", r.elapsed_secs),
            ]);
        }
        format!("workflow '{name}' results\n{}", table.render(width, true))
    }
}

#[cfg(test)]
mod tests {
    use super::{BoardRow, TaskBoard};

    fn rows() -> Vec<BoardRow> {
        vec![
            BoardRow {
                id: "scout".to_string(),
                role: "coder".to_string(),
                model: "mini".to_string(),
                status: "done".to_string(),
                iters: 4,
                elapsed_secs: 12.0,
            },
            BoardRow {
                id: "judge".to_string(),
                role: "reviewer".to_string(),
                model: "big".to_string(),
                status: "error".to_string(),
                iters: 9,
                elapsed_secs: 31.5,
            },
        ]
    }

    #[test]
    fn tty_board_shows_all_rows_with_status() {
        let s = TaskBoard::render("nightly", &rows(), 100, true);
        assert!(s.contains("nightly"), "{s}");
        assert!(s.contains("scout") && s.contains("judge"), "{s}");
        assert!(s.contains("done") && s.contains("error"), "{s}");
        assert!(s.contains("12s") || s.contains("12.0s"), "elapsed: {s}");
    }

    #[test]
    fn pipe_returns_legacy_flat_rows() {
        let s = TaskBoard::render("nightly", &rows(), 100, false);
        assert!(s.contains("scout (coder/mini"), "{s}");
        assert!(s.contains("done [4 step(s)]"), "{s}");
        assert!(s.contains("judge (reviewer/big"), "{s}");
    }

    #[test]
    fn empty_board_renders_nothing() {
        assert_eq!(TaskBoard::render("w", &[], 80, true), "");
        assert_eq!(TaskBoard::render("w", &[], 80, false), "");
    }
}
