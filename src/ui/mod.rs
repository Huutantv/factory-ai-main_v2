//! Terminal presentation & interaction: the REPL (`tui`), `theme`/color, `markdown`
//! rendering, the `spinner`, the `splash`/landing screen, `icons`, and clipboard
//! `image_input`. Everything the user sees or types lives here.

/// Upper-case the first alphabetic character of a status/digest string (`read 310 lines` →
/// `Read 310 lines`) — a display-boundary polish only: the stored digests and format strings stay
/// lowercase, so Discord/Telegram output and the `summarize_result` tests are untouched. A no-op
/// when the first character is not a lowercase letter (digits — every search digest starts with a
/// count — punctuation, already-capital, or non-alphabetic Unicode), so it is safe on any digest.
pub(crate) fn capitalize_first(s: &str) -> String {
    match s.chars().next() {
        Some(c) if c.is_lowercase() => {
            let up: String = c.to_uppercase().collect();
            format!("{up}{}", &s[c.len_utf8()..])
        }
        _ => s.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::capitalize_first;

    #[test]
    fn capitalize_first_polishes_letters_and_noops_everything_else() {
        assert_eq!(capitalize_first("read 310 lines"), "Read 310 lines");
        assert_eq!(capitalize_first("16 match(es)"), "16 match(es)");
        assert_eq!(capitalize_first("Read already"), "Read already");
        assert_eq!(capitalize_first(""), "");
        assert_eq!(capitalize_first("ế abc"), "Ế abc");
        assert_eq!(capitalize_first("· dim"), "· dim");
    }
}


pub mod cards;
pub mod channel_markdown;
pub mod config_ui;
pub mod context_report;
pub mod effort_ui;
pub mod icons;
pub mod image_input;
pub mod layout;
pub mod links;
pub mod markdown;
pub mod menus;
pub mod mermaid;
pub mod moonscape;
pub mod plain_input;
pub mod provider_ui;
pub mod spinner;
pub mod splash;
pub mod theme;
pub mod tui;
