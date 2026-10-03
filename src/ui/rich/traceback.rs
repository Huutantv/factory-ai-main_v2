//! P6 — beautiful error panels (Rich-style tracebacks).
//!
//! Formats an `anyhow`-style error chain as a boxed panel: headline,
//! `Caused by:` chain, optional code frame, and an actionable hint. Secrets
//! are never printed by this module — callers must mask keys before passing
//! text in (same rule as `config show`).
//!
//! Display-only for humans; the agent loop keeps feeding its own compact
//! error text to the model (verify-gate). A richer panel here does not by
//! itself lengthen model history.

use crate::ui::theme;

/// One snippet line around an error location.
#[derive(Debug, Clone)]
pub struct CodeFrame {
    pub file: String,
    pub line: u32,
    pub snippet: Vec<String>,
}

/// Render `headline` + `causes` + optional `frame` + `hint`.
/// Plain (no box art) when `decorate` is false.
pub fn render_error_chain(
    headline: &str,
    causes: &[String],
    frame: Option<&CodeFrame>,
    hint: Option<&str>,
    decorate: bool,
) -> String {
    if !decorate {
        let mut s = format!("error: {headline}\n");
        for c in causes {
            s.push_str(&format!("caused by: {c}\n"));
        }
        if let Some(h) = hint {
            s.push_str(&format!("hint: {h}\n"));
        }
        return s;
    }
    let mut inner: Vec<String> = Vec::new();
    inner.push(format!("{} {}", theme::err("×"), theme::err(headline)));
    for (i, c) in causes.iter().enumerate() {
        inner.push(format!(
            "  {} {}",
            theme::faint(format!("caused by[{i}]:").to_string()),
            c
        ));
    }
    if let Some(f) = frame {
        inner.push(format!(
            "  {} {}:{}",
            theme::faint("-->".to_string()),
            f.file,
            f.line
        ));
        for (i, snip) in f.snippet.iter().enumerate() {
            inner.push(format!(
                "  {} {}",
                theme::faint(format!("{:>4} │", f.line as usize + i).to_string()),
                snip
            ));
        }
    }
    if let Some(h) = hint {
        inner.push(format!("  {} {}", theme::warn("hint:"), h));
    }
    let width = inner
        .iter()
        .map(|l| console::measure_text_width(l))
        .max()
        .unwrap_or(0)
        .max(24);
    let top = format!("╭{}╮", "─".repeat(width + 2));
    let bottom = format!("╰{}╯", "─".repeat(width + 2));
    let mut out = format!("{}\n", theme::err(top));
    for line in inner {
        let w = console::measure_text_width(&line);
        let pad = " ".repeat(width.saturating_sub(w));
        out.push_str(&format!(
            "{} {line}{pad} {}\n",
            theme::err("│"),
            theme::err("│")
        ));
    }
    out.push_str(&theme::err(bottom).to_string());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tty_panel_has_box_and_chain() {
        let s = render_error_chain(
            "Codex HTTP 401",
            &["provider rejected the token".to_string()],
            None,
            Some("run `fauto auth login codex`"),
            true,
        );
        assert!(s.contains('╭') && s.contains('╰'), "{s}");
        assert!(s.contains("Codex HTTP 401"), "{s}");
        assert!(s.contains("caused by"), "{s}");
        assert!(s.contains("auth login"), "{s}");
    }

    #[test]
    fn pipe_is_plain_lines_with_hint() {
        let s = render_error_chain(
            "boom",
            &["bad input".to_string()],
            None,
            Some("retry"),
            false,
        );
        assert!(s.contains("error: boom"), "{s}");
        assert!(s.contains("caused by: bad input"), "{s}");
        assert!(!s.contains('╭'), "{s}");
    }

    #[test]
    fn code_frame_shows_file_and_lines() {
        let f = CodeFrame {
            file: "src/main.rs".into(),
            line: 100,
            snippet: vec!["let x = 1;".into(), "panic!();".into()],
        };
        let s = render_error_chain("panic", &[], Some(&f), None, true);
        assert!(s.contains("src/main.rs:100"), "{s}");
        assert!(s.contains("panic!();"), "{s}");
    }
}
