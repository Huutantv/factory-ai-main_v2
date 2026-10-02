//! P4 â€” console markup + leveled log (Rich-style).
//!
//! Supports a small tag subset: `[bold]`, `[dim]`, `[italic]`, `[u]`,
//! `[red]`, `[green]`, `[yellow]`, `[cyan]`, `[magenta]`, `[blue]`,
//! `[link]`, and `[/]` to pop. Unknown tags are stripped (never leaked).
//! A tiny `:emoji:` map covers the names Rich users reach for most; unknown
//! `:names:` pass through untouched.
//!
//! Display-only: `decorate=false` strips tags and leaves text verbatim.

use crate::ui::theme;
use console::style;

fn style_for(tag: &str, text: String) -> String {
    match tag {
        "bold" => style(text).bold().to_string(),
        "dim" => theme::muted(text).to_string(),
        "italic" => style(text).italic().to_string(),
        "u" | "underline" => style(text).underlined().to_string(),
        "red" | "err" | "error" => theme::err(text).to_string(),
        "green" | "ok" => theme::ok(text).to_string(),
        "yellow" | "warn" | "warning" => theme::warn(text).to_string(),
        "cyan" => theme::link(text).to_string(),
        "magenta" | "accent" => theme::accent(text).to_string(),
        "blue" | "link" => theme::link(text).to_string(),
        _ => text,
    }
}

fn emoji_for(name: &str) -> Option<&'static str> {
    match name {
        "smiley" | "smile" => Some("\u{1F603}"),
        "vampire" => Some("\u{1F9DB}"),
        "thumbs_up" | "+1" => Some("\u{1F44D}"),
        "check" | "heavy_check_mark" => Some("\u{2705}"),
        "cross_mark" | "x" => Some("\u{274C}"),
        "warning" => Some("\u{26A0}\u{FE0F}"),
        "sparkles" => Some("\u{2728}"),
        "rocket" => Some("\u{1F680}"),
        "brain" => Some("\u{1F9E0}"),
        "gear" => Some("\u{2699}\u{FE0F}"),
        "mag" => Some("\u{1F50D}"),
        "file_folder" => Some("\u{1F4C1}"),
        _ => None,
    }
}

/// Strip `[tags]` and `:emoji:` to plain text (pipe/CI path).
pub fn strip_markup(input: &str) -> String {
    render_markup_inner(input, false)
}

/// Render markup for a TTY. Falls back to stripped text when `decorate` is false.
pub fn render_markup(input: &str, decorate: bool) -> String {
    render_markup_inner(input, decorate)
}

fn flush_run(out: &mut String, run: &mut String, stack: &[String], decorate: bool) {
    if run.is_empty() {
        return;
    }
    let text = std::mem::take(run);
    if decorate {
        match stack.last() {
            Some(tag) => out.push_str(&style_for(tag, text)),
            None => out.push_str(&text),
        }
    } else {
        out.push_str(&text);
    }
}

fn render_markup_inner(input: &str, decorate: bool) -> String {
    let mut out = String::with_capacity(input.len());
    let mut run = String::new();
    let mut stack: Vec<String> = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            if let Some(end) = input[i..].find(']') {
                let tag = &input[i + 1..i + end];
                if tag == "/" {
                    flush_run(&mut out, &mut run, &stack, decorate);
                    stack.pop();
                } else if !tag.is_empty()
                    && tag
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                {
                    flush_run(&mut out, &mut run, &stack, decorate);
                    stack.push(tag.to_lowercase());
                } else {
                    run.push_str(&input[i..i + end + 1]);
                }
                i += end + 1;
                continue;
            }
            run.push('[');
            i += 1;
        } else if bytes[i] == b':' {
            let rest = &input[i + 1..];
            if let Some(end) = rest.find(':') {
                let name = &rest[..end];
                if !name.is_empty()
                    && name
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '+' || c == '-')
                {
                    if let Some(e) = emoji_for(name) {
                        flush_run(&mut out, &mut run, &stack, decorate);
                        out.push_str(e);
                    } else {
                        run.push(':');
                        run.push_str(name);
                        run.push(':');
                    }
                    i += end + 2;
                    continue;
                }
            }
            run.push(':');
            i += 1;
        } else {
            // Take one char (Unicode-safe); styling applies per run on flush.
            let ch = input[i..].chars().next().unwrap();
            run.push(ch);
            i += ch.len_utf8();
        }
    }
    flush_run(&mut out, &mut run, &stack, decorate);
    out
}

/// Rich-style `log()`: `HH:MM:SS message  file:line`. Display-only.
/// Returns plain text when `decorate` is false.
pub fn rich_log(message: &str, file: &str, line: u32, decorate: bool) -> String {
    let now = chrono::Local::now().format("%H:%M:%S").to_string();
    let loc = format!("{file}:{line}");
    if decorate {
        format!(
            "{} {}  {}",
            theme::faint(now),
            render_markup(message, true),
            theme::faint(loc)
        )
    } else {
        format!("{now} {}  {loc}", strip_markup(message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bold_tag_styles_on_tty_and_strips_on_pipe() {
        let tty = render_markup("Hello, [bold]World[/]!", true);
        assert!(tty.contains("World"), "text must survive: {tty}");
        // NOTE: under `cargo test` stdout is not a TTY so `console` strips
        // ANSI (same caveat as `theme::tests::helpers_render_to_nonempty`);
        // on a real terminal this differs from the plain text by styling.
        assert_eq!(strip_markup("Hello, [bold]World[/]!"), "Hello, World!");
        assert_eq!(
            render_markup("Hello, [bold]World[/]!", false),
            "Hello, World!"
        );
    }

    #[test]
    fn unknown_tags_never_leak_and_emoji_resolves() {
        assert_eq!(strip_markup("[bogus]hi[/]"), "hi");
        assert!(strip_markup(":rocket: go").contains("\u{1F680}"));
        assert!(strip_markup(":not_a_real_emoji_xyz:").contains(":not_a_real_emoji_xyz:"));
    }

    #[test]
    fn log_line_has_timestamp_and_location() {
        let line = rich_log("server [bold]up[/]", "serve.rs", 42, false);
        assert!(line.contains("serve.rs:42"), "{line}");
        assert!(line.contains("server up"), "{line}");
    }
}
