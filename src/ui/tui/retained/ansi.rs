//! Text that came from somewhere else, made safe to put on a retained frame.
//!
//! Two jobs: measuring/clipping by DISPLAY columns rather than bytes or chars (a CJK glyph is two
//! cells wide), and dealing with ANSI — either stripping it, or parsing the subset we honour into
//! ratatui spans. Control sequences that could move the cursor or clear the screen are removed:
//! this backend owns the screen, and text from a tool must never be able to repaint it.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

/// Clip a plain string to `max` display columns (ellipsis when it would overflow). Width-aware so a
/// wide glyph never half-lands past the frame.
pub(super) fn clip_to(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    if console::measure_text_width(s) <= max {
        return s.to_string();
    }
    let budget = max.saturating_sub(1);
    let mut out = String::new();
    let mut w = 0usize;
    for ch in s.chars() {
        let cw = console::measure_text_width(&ch.to_string()).max(1);
        if w + cw > budget {
            break;
        }
        out.push(ch);
        w += cw;
    }
    out.push('…');
    out
}

/// Clip then right-pad a plain string to exactly `width` display columns (for a box header).
pub(super) fn pad_to(s: &str, width: usize) -> String {
    let clipped = clip_to(s, width);
    let w = console::measure_text_width(&clipped);
    format!("{clipped}{}", " ".repeat(width.saturating_sub(w)))
}

/// Word-wrap one sanitised row (SGR colour codes only, no cursor moves) to `width` display
/// cells, preserving colours across the break.
///
/// Long `emit_line` notices (endpoint fallbacks, sandbox advisories, token lines) otherwise arrive
/// as one very long row; the frame clips it at the pane edge mid-word and the eye reads a broken
/// line. Wrapped rows keep the active style: it is re-emitted at the start of every continuation
/// line and each styled line ends with a reset, because downstream every row is parsed
/// independently. Continuation lines are indented two cells so the wrapped block reads as one
/// paragraph. Rows that already fit are returned byte-identical (zero risk to short lines).
pub(super) fn wrap_sgr_row(row: &str, width: usize) -> Vec<String> {
    let width = width.max(8);
    if row.is_empty() || console::measure_text_width(row) <= width {
        return vec![row.to_string()];
    }
    // One cell per printable char, carrying the SGR style active at its position.
    let mut cells: Vec<Cell> = Vec::new();
    let mut active = String::new();
    let mut chars = row.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            let mut seq = String::from("\x1b[");
            let mut params = String::new();
            let mut final_byte = None;
            while let Some(&pc) = chars.peek() {
                chars.next();
                seq.push(pc);
                if pc.is_ascii_digit() || pc == ';' {
                    params.push(pc);
                } else {
                    final_byte = Some(pc);
                    break;
                }
            }
            if final_byte == Some('m') {
                if params.is_empty() || params == "0" {
                    active.clear();
                } else {
                    active.push_str(&seq);
                }
            }
            continue;
        }
        // Fast path: printable ASCII is one cell by definition.
        let w = if c == ' ' || c.is_ascii_graphic() {
            1
        } else {
            console::measure_text_width(&c.to_string()).max(1)
        };
        cells.push(Cell {
            ch: c,
            w,
            style: active.clone(),
        });
    }
    if cells.is_empty() {
        return vec![row.to_string()];
    }
    const INDENT: &str = "  ";
    const INDENT_W: usize = 2;
    let mut lines: Vec<String> = Vec::new();
    let mut i = 0;
    let mut first = true;
    while i < cells.len() {
        let budget = if first { width } else { width - INDENT_W };
        // Greedy fill: how many cells fit?
        let mut take = 0;
        let mut take_w = 0;
        while i + take < cells.len() && take_w + cells[i + take].w <= budget {
            take_w += cells[i + take].w;
            take += 1;
        }
        let mut end = i + take;
        if end < cells.len() {
            // Prefer breaking after the last space in the window (prose reads as prose); a
            // window with no space (one long token) hard-splits at the budget.
            let mut space = None;
            for k in (i..end).rev() {
                if cells[k].ch == ' ' {
                    space = Some(k);
                    break;
                }
            }
            if let Some(sp) = space {
                if sp > i {
                    end = sp + 1; // keep the space at the end of this line
                }
            }
            // A leading run of spaces would make a blank-looking continuation: skip them.
            let mut start = i;
            if !first {
                while start < end && cells[start].ch == ' ' {
                    start += 1;
                }
            }
            if start >= end {
                // The window was all spaces — emit nothing and advance past them.
                i = end;
                first = false;
                continue;
            }
            lines.push(emit_wrapped_line(
                &cells[start..end],
                if first { "" } else { INDENT },
            ));
            i = end;
            // The break consumed the trailing space; skip any further spaces at the new start
            // so continuations never open with a gap.
            while i < cells.len() && cells[i].ch == ' ' {
                i += 1;
            }
        } else {
            lines.push(emit_wrapped_line(
                &cells[i..],
                if first { "" } else { INDENT },
            ));
            break;
        }
        first = false;
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// One printable char plus the SGR style active at its position.
struct Cell {
    ch: char,
    w: usize,
    style: String,
}

/// Serialise wrapped cells back to an SGR string: re-emit the style whenever it changes (so the
/// first cell's style opens the line), reset at the end when any style is active.
fn emit_wrapped_line(cells: &[Cell], indent: &str) -> String {
    let mut out = String::with_capacity(indent.len() + cells.len() * 2);
    out.push_str(indent);
    let mut emitted = String::new();
    for cell in cells {
        if cell.style != emitted {
            out.push_str(&cell.style);
            emitted = cell.style.clone();
        }
        out.push(cell.ch);
    }
    if !emitted.is_empty() {
        out.push_str("\x1b[0m");
    }
    out
}

/// Strip ALL terminal control sequences INCLUDING colour/SGR, leaving printable text only. Used for
/// the intro block (rendered as one flat dim line) and where a plain-text guarantee is wanted.
pub(super) fn sanitize_text(input: &str) -> String {
    console::strip_ansi_codes(&sanitize_keep_sgr(input)).into_owned()
}

/// Strip terminal control sequences that would corrupt the retained frame (cursor moves, screen
/// erase, save/restore, carriage returns) but PRESERVE colour/SGR (`\x1b[…m`) so [`ansi_spans`] can
/// turn them into styled ratatui spans. This is what keeps the `❯` user echo in accent, the edit
/// diff green/red, and the `◆` tool anchor tinted by state — instead of one flat grey.
pub(super) fn sanitize_keep_sgr(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            match chars.peek() {
                Some('[') => {
                    chars.next();
                    let mut seq = String::from("\x1b[");
                    let mut final_byte = None;
                    while let Some(&pc) = chars.peek() {
                        chars.next();
                        seq.push(pc);
                        // A CSI sequence ends at its final byte (0x40–0x7E). We break on the first
                        // ASCII letter, which covers every sequence we actually emit: `m` (SGR),
                        // `J`/`K` (erase), `s`/`u` (save/restore cursor), `H`/`A`…`G` (moves).
                        if pc.is_ascii_alphabetic() {
                            final_byte = Some(pc);
                            break;
                        }
                    }
                    if final_byte == Some('m') {
                        out.push_str(&seq); // keep colour / bold / dim — drop everything else
                    }
                }
                // A non-CSI escape (e.g. `\x1b7` save cursor): drop the ESC and its one payload byte.
                _ => {
                    chars.next();
                }
            }
            continue;
        }
        match c {
            '\n' => out.push('\n'),
            '\t' => out.push_str("    "),
            '\r' => {}
            c if !c.is_control() => out.push(c),
            _ => {}
        }
    }
    out
}

/// Map a basic ANSI colour index (0–15, the `30`–`37` / `90`–`97` SGR families) to a ratatui colour.
fn basic_color(n: u16) -> Color {
    match n {
        0 => Color::Black,
        1 => Color::Red,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::Cyan,
        7 => Color::Gray,
        8 => Color::DarkGray,
        9 => Color::LightRed,
        10 => Color::LightGreen,
        11 => Color::LightYellow,
        12 => Color::LightBlue,
        13 => Color::LightMagenta,
        14 => Color::LightCyan,
        _ => Color::White,
    }
}

/// Fold one SGR parameter list (the `;`-separated numbers before an `m`) into the running style,
/// resetting to `base` on `0`/`39`. Supports the codes the app actually emits: reset, bold, dim,
/// italic, underline, the 8+8 basic-colour families, and 256-colour (`38;5;n`) / truecolor (`38;2`).
fn apply_sgr(base: Style, cur: Style, params: &str) -> Style {
    let codes: Vec<u16> = params
        .split(';')
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse::<u16>().ok())
        .collect();
    if codes.is_empty() {
        return base; // a bare `\x1b[m` is `\x1b[0m` — a full reset
    }
    let mut style = cur;
    let mut i = 0;
    while i < codes.len() {
        match codes[i] {
            0 => style = base,
            1 => style = style.add_modifier(Modifier::BOLD),
            2 => style = style.add_modifier(Modifier::DIM),
            3 => style = style.add_modifier(Modifier::ITALIC),
            4 => style = style.add_modifier(Modifier::UNDERLINED),
            22 => {
                style = style
                    .remove_modifier(Modifier::BOLD)
                    .remove_modifier(Modifier::DIM)
            }
            30..=37 => style = style.fg(basic_color(codes[i] - 30)),
            39 => style = style.fg(base.fg.unwrap_or(Color::Gray)),
            90..=97 => style = style.fg(basic_color(codes[i] - 90 + 8)),
            38 => match codes.get(i + 1) {
                Some(5) => {
                    if let Some(&n) = codes.get(i + 2) {
                        style = style.fg(Color::Indexed(n as u8));
                    }
                    i += 2;
                }
                Some(2) => {
                    if let (Some(&r), Some(&g), Some(&b)) =
                        (codes.get(i + 2), codes.get(i + 3), codes.get(i + 4))
                    {
                        style = style.fg(Color::Rgb(r as u8, g as u8, b as u8));
                    }
                    i += 4;
                }
                _ => {}
            },
            48 => match codes.get(i + 1) {
                Some(5) => i += 2, // background colour — measured/skipped, foreground is what reads
                Some(2) => i += 4,
                _ => {}
            },
            _ => {}
        }
        i += 1;
    }
    style
}

/// Parse a single already-sanitised row (SGR only, no cursor moves) into ratatui spans over `base`.
/// A row with no colour codes collapses to one span in the base style, so uncoloured output looks
/// exactly as before.
pub(crate) fn ansi_spans(row: &str, base: Style) -> Vec<Span<'static>> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut cur = base;
    let mut buf = String::new();
    let mut chars = row.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            let mut params = String::new();
            let mut final_byte = None;
            while let Some(&pc) = chars.peek() {
                chars.next();
                if pc.is_ascii_digit() || pc == ';' {
                    params.push(pc);
                } else {
                    final_byte = Some(pc);
                    break;
                }
            }
            if final_byte == Some('m') {
                if !buf.is_empty() {
                    spans.push(Span::styled(std::mem::take(&mut buf), cur));
                }
                cur = apply_sgr(base, cur, &params);
            }
            continue;
        }
        buf.push(c);
    }
    if !buf.is_empty() {
        spans.push(Span::styled(buf, cur));
    }
    if spans.is_empty() {
        spans.push(Span::styled(String::new(), base));
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(rows: &[String]) -> Vec<String> {
        rows.iter()
            .map(|r| console::strip_ansi_codes(r).into_owned())
            .collect()
    }

    #[test]
    fn short_rows_pass_through_byte_identical() {
        let row = "\u{1b}[38;5;213mhi\u{1b}[0m";
        assert_eq!(wrap_sgr_row(row, 40), vec![row.to_string()]);
        assert_eq!(wrap_sgr_row("", 40), vec![String::new()]);
    }

    #[test]
    fn long_prose_wraps_at_spaces_with_indented_continuation() {
        let rows = wrap_sgr_row("aaa bbb ccc ddd eee fff", 10);
        assert_eq!(plain(&rows), vec!["aaa bbb ", "  ccc ddd ", "  eee fff"]);
        for r in &rows {
            assert!(
                console::measure_text_width(r) <= 10,
                "row exceeds width: {r:?}"
            );
        }
    }

    #[test]
    fn styled_rows_keep_colour_and_reset_per_line() {
        let row = "\u{1b}[38;5;213malpha beta gamma delta epsilon\u{1b}[0m";
        let rows = wrap_sgr_row(row, 12);
        assert!(rows.len() >= 2, "must wrap: {rows:?}");
        // Every wrapped line re-opens the colour and closes it, because downstream each row is
        // parsed independently.
        for r in &rows {
            assert!(
                r.contains("\u{1b}[38;5;213m"),
                "continuation lost its style: {r:?}"
            );
            assert!(r.ends_with("\u{1b}[0m"), "missing reset: {r:?}");
        }
        // Reassembled text (continuation indents stripped) is lossless.
        let mut iter = plain(&rows).into_iter();
        let mut joined = iter.next().unwrap_or_default();
        for r in iter {
            joined.push_str(r.strip_prefix("  ").unwrap_or(&r));
        }
        assert!(joined.contains("alpha beta gamma delta epsilon"));
    }

    #[test]
    fn long_word_without_spaces_hard_splits() {
        let rows = wrap_sgr_row("abcdefghijklmnop", 8);
        assert!(rows.len() >= 2);
        for r in &rows {
            assert!(console::measure_text_width(r) <= 8, "row too wide: {r:?}");
        }
        let mut iter = plain(&rows).into_iter();
        let mut joined = iter.next().unwrap_or_default();
        for r in iter {
            joined.push_str(r.strip_prefix("  ").unwrap_or(&r));
        }
        assert_eq!(joined, "abcdefghijklmnop");
    }

    #[test]
    fn wide_glyphs_count_two_cells() {
        // Each CJK char is 2 cells: width 8 fits 4 of them.
        let rows = wrap_sgr_row("一二三四五六七八九", 8);
        assert!(rows.len() >= 2, "must wrap: {rows:?}");
        for r in &rows {
            assert!(console::measure_text_width(r) <= 8, "row too wide: {r:?}");
        }
    }
}
