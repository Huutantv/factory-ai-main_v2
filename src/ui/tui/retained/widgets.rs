//! The discrete pieces a frame is assembled from: the overlay pane, assistant rows, the tool row,
//! the plan box, the diff box and the verify line.
//!
//! Each takes its payload and returns rendered lines, so `paint` decides placement and these decide
//! appearance.

use super::*;

/// Draw the informational overlay, applying `scroll` (rows hidden above the top) clamped so the last
/// page is the furthest you can go. Returns the CLAMPED scroll so the caller can write it back — a
/// PageDown past the end then reads as "at the bottom" rather than drifting into empty space.
pub(super) fn draw_overlay(
    frame: &mut Frame<'_>,
    area: Rect,
    overlay: &OverlaySnapshot,
    scroll: usize,
) -> (usize, Rect) {
    let width = area.width.saturating_sub(4).min(84).max(20);
    let height = (overlay.lines.len() as u16 + 4)
        .min(area.height.saturating_sub(2))
        .max(5);
    let rect = centered(area, width, height);
    frame.render_widget(Clear, rect);
    let block = FrameBlock::default()
        .borders(Borders::ALL)
        .title(format!(" ◈ {} ", overlay.title))
        .title_style(
            Style::default()
                .fg(Color::Indexed(crate::ui::theme::accent_idx()))
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(Color::Indexed(crate::ui::theme::accent_dim_idx())));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);
    // Selectable rows are clipped with an ellipsis (never wrapped — wrapping would break the
    // row↔line mapping the hit-test depends on). `inner.width - 2` leaves room for the `› `/`  `
    // selection prefix.
    let row_budget = (inner.width as usize).saturating_sub(2);
    let mut lines: Vec<Line<'static>> = overlay
        .lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let selected = overlay.selected == Some(i);
            let shown = if overlay.selected.is_some() {
                clip_to(line, row_budget)
            } else {
                line.clone()
            };
            Line::styled(
                if selected {
                    format!("› {shown}")
                } else {
                    format!("  {shown}")
                },
                if selected {
                    Style::default()
                        .fg(Color::Indexed(crate::ui::theme::accent_idx()))
                        .bg(Color::Indexed(crate::ui::theme::accent_dim_idx()))
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                },
            )
        })
        .collect();
    let hint = if overlay.hint.is_empty() {
        "↑↓ navigate · Enter select · Esc close".to_string()
    } else {
        format!(
            "{}  ·  ↑↓ navigate · Enter select · Esc close",
            overlay.hint
        )
    };
    lines.push(Line::styled(hint, Style::default().fg(Color::DarkGray)));
    // Clamp scroll so the final page is the furthest reachable position (never scroll past the end).
    let visible = inner.height as usize;
    let max_scroll = lines.len().saturating_sub(visible);
    let clamped = scroll.min(max_scroll);
    // A SELECTABLE overlay is a menu: its rows must stay one screen row each so a mouse click can be
    // mapped back to a row index (`overlay_menu_hit`). Long rows are clipped at the panel edge, not
    // wrapped — wrapping would break the row↔line mapping the hit-test depends on. Informational
    // overlays (no selection) keep wrapping and publish no geometry.
    let selectable = overlay.selected.is_some();
    let paragraph = if selectable {
        Paragraph::new(lines).scroll((clamped.min(u16::MAX as usize) as u16, 0))
    } else {
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((clamped.min(u16::MAX as usize) as u16, 0))
    };
    frame.render_widget(paragraph, inner);
    set_overlay_menu(selectable.then_some(OverlayMenuGeom {
        inner,
        scroll: clamped,
        rows: overlay.lines.len(),
    }));
    (clamped, rect)
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}

pub(super) fn render_assistant_rows(raw: &str, width: usize) -> Vec<String> {
    // ONE renderer for both surfaces. The live turn (streaming) and the replayed transcript
    // (`agent::replay_transcript` → `MarkdownStream`) must produce byte-identical output, or the same
    // message looks different when re-opened than when first shown. So feed the whole raw block through
    // `MarkdownStream` here too — the block is cached by `content_hash`, so re-parsing the full body on
    // each width/content change is the same trip `replay_transcript` makes, with none of the
    // incremental-splice risk an in-place streaming parser would carry.
    //
    // Keep SGR: the renderer emits the assistant gutter, code-box borders, and syntax highlight as
    // colour codes — `sanitize_keep_sgr` preserves them (dropping only cursor moves) and `ansi_spans`
    // turns them into styled spans at draw time.
    let mut md = crate::ui::markdown::MarkdownStream::new(true, width.max(8));
    let mut rendered = md.push(&format!("{raw}\n"));
    rendered.push_str(&md.finish());
    sanitize_keep_sgr(&rendered)
        .split('\n')
        .map(str::to_string)
        .collect()
}

/// Format a run time for the result line: `· 940ms` under a second, `· 1.2s` under a minute, then
/// `· 7m03s` / `· 2h05m`. Sub-second times keep millisecond resolution; the minute/hour tiers exist
/// because a sub-agent dispatch is a single tool call that can run for hours, and `· 7203.4s` makes
/// the reader do the division. Empty for `None` (unknown — restored transcripts / eager-adopted).
pub(crate) fn fmt_elapsed(ms: Option<u64>) -> String {
    match ms {
        None => String::new(),
        Some(ms) if ms < 1000 => format!(" · {ms}ms"),
        Some(ms) if ms < 60_000 => format!(" · {:.1}s", ms as f64 / 1000.0),
        Some(ms) => format!(" · {}", crate::agent::orchestration::fmt_secs(ms / 1000)),
    }
}

/// Lay out one tool-call line under the transcript, mockup-style but result-below: the call
/// `<icon> <name>   <target>` on its own line (icon + name in the tool green, target dim magenta),
/// then an indented `└ <digest> · <time>` line beneath it — the result digest tinted by state
/// (running = faint, ok = green, err = red) and carrying the wall-clock run time. A still-running
/// call (empty digest) is just the call line; the result line is added when the digest lands.
///
/// `compact` selects the single-line `icon name — digest · time` shape (the balanced/dense
/// layouts); the caller passes the active layout's flag so this stays pure over its inputs.
pub(crate) fn render_tool_row(t: &ToolEvent, width: usize, compact: bool) -> String {
    use crate::ui::theme;
    // COMMON_FLOOR: every transcript component shares one narrow-width policy (8 cells).
    let width = width.max(8);
    let icon = if t.icon.is_empty() {
        String::new()
    } else {
        format!("{} ", t.icon)
    };
    let name_styled = theme::tool(&t.name).to_string();
    let call_prefix = format!("{}{}", icon, t.name);
    let call_line = if t.target.is_empty() {
        theme::tool(&call_prefix).to_string()
    } else {
        let target_budget = width.saturating_sub(console::measure_text_width(&call_prefix) + 3);
        let target = clip_to(&t.target, target_budget);
        format!(
            "{}{}   {}",
            theme::tool(&icon),
            name_styled,
            theme::accent_dim(&target)
        )
    };
    if t.digest.is_empty() {
        return call_line;
    }
    // Compact layout: single line `icon name — digest · time`.
    if compact {
        let digest_budget = width.saturating_sub(console::measure_text_width(&call_prefix) + 5);
        let digest = clip_to(&crate::ui::capitalize_first(&t.digest), digest_budget);
        let digest_plain = format!("{} — {}{}", t.name, digest, fmt_elapsed(t.elapsed_ms));
        let compact = match t.state {
            ToolState::Running => theme::faint(&digest_plain).to_string(),
            ToolState::Ok => theme::ok(&digest_plain).to_string(),
            ToolState::Err => theme::err(&digest_plain).to_string(),
        };
        return if icon.trim().is_empty() {
            compact
        } else {
            format!("{}{}", theme::tool(&icon), compact)
        };
    }
    // Result on the line below: `└ <digest> · <time>`, digest tinted by outcome, time dimmed.
    // The digest is clipped to what fits (the timestamp never pushes content past the pane edge).
    let time = fmt_elapsed(t.elapsed_ms);
    let digest_budget = width.saturating_sub(2 + console::measure_text_width(&time));
    let digest = clip_to(&crate::ui::capitalize_first(&t.digest), digest_budget);
    let digest_styled = match t.state {
        ToolState::Running => theme::faint(&digest).to_string(),
        ToolState::Ok => theme::ok(&digest).to_string(),
        ToolState::Err => theme::err(&digest).to_string(),
    };
    let time_styled = if time.is_empty() {
        String::new()
    } else {
        theme::faint(&time).to_string()
    };
    format!(
        "{call_line}\n{} {digest_styled}{time_styled}",
        theme::faint("└")
    )
}

/// Render the in-place plan panel as a boxed checklist: a `☑ done/total · plan` header row, then one
/// `✓ / ▸ / ○` row per item, framed with the same rounded box the markdown renderer uses. Done rows
/// are green + dim-struck, the in-progress row is bright accent, pending rows are faint.
pub(crate) fn render_plan_box(rows: &[PlanRow], width: usize) -> Vec<String> {
    use crate::ui::theme;
    let done = rows.iter().filter(|r| r.status == 2).count();
    let header = format!("☑ {done}/{} · plan", rows.len());
    // Inner width: cap so the box doesn't sprawl on a very wide pane; leave room for `│ ` + ` │`.
    // BOX_CAP (80) is shared with the diff box so both panels speak one geometry.
    let inner = width.saturating_sub(2).min(80).max(12);
    let bar = "─".repeat(inner);
    let mut out = Vec::new();
    out.push(
        theme::accent_dim(format!(
            "╭─ {} ─╮",
            pad_to(&header, inner.saturating_sub(4))
        ))
        .to_string(),
    );
    for r in rows {
        let glyph = match r.status {
            2 => "✓",
            1 => "▸",
            _ => "○",
        };
        let g = match r.status {
            2 => theme::ok(glyph).to_string(),
            1 => theme::accent(glyph).bold().to_string(),
            _ => theme::faint(glyph).to_string(),
        };
        // Body row `│ <glyph> <text><pad> │`: the span BETWEEN the two rules must be exactly `inner`
        // cells so the right `│` lines up with the border corners. That span is
        // ` `+glyph(1)+` `+text+pad+` ` = 4 + text + pad, so pad = inner − 4 − text.
        let text_budget = inner.saturating_sub(4);
        let clipped = clip_to(&r.text, text_budget);
        let styled = match r.status {
            2 => theme::muted(&clipped).to_string(),
            1 => theme::accent(&clipped).bold().to_string(),
            _ => theme::faint(&clipped).to_string(),
        };
        let pad = inner.saturating_sub(4 + console::measure_text_width(&clipped));
        out.push(format!(
            "{} {} {}{} {}",
            theme::accent_dim("│"),
            g,
            styled,
            " ".repeat(pad),
            theme::accent_dim("│")
        ));
    }
    out.push(theme::accent_dim(format!("╰{bar}╯")).to_string());
    out
}

/// Render a boxed diff preview: a `diff · <path>  +A −D` header, then the `+`/`−` lines (green /
/// red) inside the same rounded frame. Lines are clipped to the inner width.
pub(crate) fn render_diff_box(d: &DiffPayload, width: usize) -> Vec<String> {
    use crate::ui::theme;
    // BOX_CAP (80), shared with the plan box.
    let inner = width.saturating_sub(2).min(80).max(12);
    let bar = "─".repeat(inner);
    let header = format!("diff · {}   +{} −{}", d.path, d.adds, d.dels);
    let mut out = Vec::new();
    out.push(
        theme::accent_dim(format!(
            "╭─ {} ─╮",
            pad_to(&header, inner.saturating_sub(4))
        ))
        .to_string(),
    );
    for (is_add, content) in &d.lines {
        let budget = inner.saturating_sub(4);
        let clipped = clip_to(content, budget.saturating_sub(2));
        let body = if *is_add {
            format!("+ {clipped}")
        } else {
            format!("− {clipped}")
        };
        let pad = inner.saturating_sub(2 + console::measure_text_width(&body));
        let styled = if *is_add {
            theme::ok(&body).to_string()
        } else {
            theme::err(&body).to_string()
        };
        out.push(format!(
            "{} {}{} {}",
            theme::accent_dim("│"),
            styled,
            " ".repeat(pad),
            theme::accent_dim("│")
        ));
    }
    out.push(theme::accent_dim(format!("╰{bar}╯")).to_string());
    out
}

/// Render the verify-gate success line: a green `✓ <cmd> — <detail>`, clipped to width.
pub(crate) fn render_verify_line(v: &VerifyPayload, width: usize) -> String {
    use crate::ui::theme;
    let text = if v.detail.is_empty() {
        format!("✓ {}", v.cmd)
    } else {
        format!("✓ {} — {}", v.cmd, v.detail)
    };
    theme::ok(clip_to(&text, width)).to_string()
}
