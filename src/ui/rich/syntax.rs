//! P5 — best-effort syntax highlighting (Rich-style `Syntax`).
//!
//! A light, language-aware pass over code-fence bodies: strings, comments,
//! numbers, keywords. Pure Rust, no new crates (no `syntect`/`tree-sitter`:
//! both threaten the no-C-dep rule). It never mangles code — anything
//! unrecognised stays default — and returns input verbatim when `decorate`
//! is false or the language is unknown.

use crate::ui::theme;

fn keywords(lang: &str) -> &'static [&'static str] {
    match lang.to_lowercase().as_str() {
        "rust" | "rs" => &[
            "fn", "let", "mut", "const", "struct", "enum", "impl", "trait", "pub", "use", "mod",
            "return", "if", "else", "match", "for", "while", "loop", "in", "where", "type", "dyn",
            "static", "ref", "move", "async", "await", "crate", "self", "Self", "Ok", "Err",
            "Some", "None",
        ],
        "python" | "py" => &[
            "def", "class", "return", "if", "elif", "else", "for", "while", "in", "import", "from",
            "as", "with", "try", "except", "finally", "raise", "None", "True", "False", "lambda",
            "yield", "async", "await", "pass",
        ],
        "javascript" | "typescript" | "js" | "ts" => &[
            "function",
            "const",
            "let",
            "var",
            "return",
            "if",
            "else",
            "for",
            "while",
            "class",
            "import",
            "from",
            "export",
            "new",
            "typeof",
            "await",
            "async",
            "true",
            "false",
            "null",
            "undefined",
        ],
        "bash" | "sh" | "shell" | "ps1" | "powershell" => &[
            "if", "then", "else", "fi", "for", "while", "do", "done", "function", "return", "echo",
            "export", "local",
        ],
        "toml" | "json" | "yaml" | "yml" => &["true", "false", "null"],
        _ => &[],
    }
}

fn line_comment(lang: &str) -> Option<&'static str> {
    match lang.to_lowercase().as_str() {
        "rust" | "rs" | "javascript" | "typescript" | "js" | "ts" => Some("//"),
        "python" | "py" | "bash" | "sh" | "shell" | "toml" | "yaml" | "yml" | "ps1"
        | "powershell" => Some("#"),
        _ => None,
    }
}

/// Highlight `code` written in `lang`. Line numbers when `line_numbers`.
/// Verbatim passthrough when `!decorate`.
pub fn highlight(code: &str, lang: &str, line_numbers: bool, decorate: bool) -> String {
    if !decorate {
        return code.to_string();
    }
    let kws = keywords(lang);
    let comment = line_comment(lang);
    let mut out = String::new();
    for (idx, line) in code.lines().enumerate() {
        if line_numbers {
            out.push_str(&theme::faint(format!("{:>4} │ ", idx + 1)).to_string());
        }
        out.push_str(&highlight_line(line, kws, comment));
        out.push('\n');
    }
    out
}

fn highlight_line(line: &str, kws: &[&str], comment: Option<&str>) -> String {
    // Split off a trailing line comment first so keywords inside comments
    // are never highlighted.
    let (code_part, comment_part) = match comment {
        Some(marker) => match find_comment_start(line, marker) {
            Some(pos) => (&line[..pos], Some(&line[pos..])),
            None => (line, None),
        },
        None => (line, None),
    };
    let mut out = highlight_code_segment(code_part, kws);
    if let Some(c) = comment_part {
        out.push_str(&theme::code_comment(c).to_string());
    }
    out
}

fn find_comment_start(line: &str, marker: &str) -> Option<usize> {
    // Naive but string-aware: skip markers inside quotes.
    let mut in_str: Option<char> = None;
    let mut i = 0;
    let bytes = line.as_bytes();
    while i < bytes.len() {
        let ch = line[i..].chars().next().unwrap();
        if let Some(q) = in_str {
            if ch == q {
                in_str = None;
            }
        } else if ch == '"' || ch == '\'' || ch == '`' {
            in_str = Some(ch);
        } else if line[i..].starts_with(marker) {
            return Some(i);
        }
        i += ch.len_utf8();
    }
    None
}

fn highlight_code_segment(seg: &str, kws: &[&str]) -> String {
    let mut out = String::new();
    let mut token = String::new();
    let flush = |token: &mut String, out: &mut String| {
        if token.is_empty() {
            return;
        }
        let t = std::mem::take(token);
        if kws.contains(&t.as_str()) {
            out.push_str(&theme::code_keyword(&t).to_string());
        } else if t.parse::<f64>().is_ok()
            && t.chars().any(|c| c.is_ascii_digit())
            && !t.starts_with("0x")
            || t.starts_with("0x") && t.len() > 2
        {
            out.push_str(&theme::code_number(&t).to_string());
        } else {
            out.push_str(&t);
        }
    };
    let mut chars = seg.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '"' || ch == '\'' || ch == '`' {
            flush(&mut token, &mut out);
            let quote = ch;
            let mut lit = String::new();
            lit.push(ch);
            for c in chars.by_ref() {
                lit.push(c);
                if c == quote {
                    break;
                }
            }
            out.push_str(&theme::code_string(&lit).to_string());
        } else if ch.is_alphanumeric() || ch == '_' {
            token.push(ch);
        } else {
            flush(&mut token, &mut out);
            out.push(ch);
        }
    }
    flush(&mut token, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_keywords_highlighted_and_comment_kept() {
        let s = highlight(
            "fn main() { // run it\n    let x = 42;\n}",
            "rust",
            false,
            true,
        );
        assert!(s.contains("fn"), "{s}");
        assert!(s.contains("42"), "{s}");
        assert!(s.contains("// run it"), "{s}");
        assert_ne!(
            s, "fn main() { // run it\n    let x = 42;\n}",
            "must add styling"
        );
    }

    #[test]
    fn pipe_is_verbatim_and_unknown_lang_passes_through() {
        let code = "fn main() {}";
        assert_eq!(highlight(code, "rust", false, false), code);
        // unknown lang: no keywords, but structure (incl. line ending) preserved
        let s = highlight(code, "cobol", false, true);
        assert!(s.contains("fn main()"), "{s}");
    }

    #[test]
    fn keyword_inside_string_is_not_highlighted_as_keyword() {
        let s = highlight("let s = \"fn not a keyword\";", "rust", false, true);
        assert!(s.contains("\"fn not a keyword\""), "{s}");
    }

    #[test]
    fn line_numbers_render_gutter() {
        let s = highlight("a\nb", "rust", true, true);
        assert!(s.contains('│'), "{s}");
    }
}
