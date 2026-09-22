//! The single source of truth for the F.Auto TUI palette — the **vivid** identity.
//!
//! Every role has its own hue: magenta accent for structure, cyan for the user's voice, green for
//! tool calls, purple for the live/reasoning caption, sky for the HUD, plus the semantic set
//! (green ok / red err / yellow warn / cyan-blue link) and a code-syntax sub-palette. Everything is
//! 256-colour (universally supported; no truecolor dependency), routed through `console::style`
//! so `NO_COLOR` and non-TTY output are auto-stripped.
//!
//! This is the ONLY theme — it is compiled in, not selectable, so the whole CLI (transcript, tool
//! rows, splash, input box, menus, code blocks) speaks one coherent, high-contrast palette. Use the
//! helpers (`accent`, `ok`, `err`, …) instead of scattering raw `color256(..)` calls so roles stay
//! consistent; the `*_idx()` getters and `current_palette()` exist for the ratatui paint path.

use console::{style, StyledObject};
use std::fmt::Display;

// ── core palette (256-colour indices) ───────────────────────────────────────────
/// Magenta — brand + structure (prompt arrow, tool block, headings, splash wordmark).
pub const ACCENT: u8 = 213;
/// Dim magenta — secondary: targets/values, quiet rules/borders.
pub const ACCENT_DIM: u8 = 177;
/// Neutral grey for secondary text.
pub const MUTED: u8 = 245;
/// Very faint grey — separators, the code-block rule, timestamps.
pub const FAINT: u8 = 240;

// ── semantic (used ONLY where the colour carries meaning) ────────────────────────
/// Success / confirmation / added.
pub const OK: u8 = 84;
/// Error / failure / removed.
pub const ERR: u8 = 203;
/// Warning / caution — the `⚡ yolo` chip + cautions.
pub const WARN: u8 = 220;
/// Links + inline code.
pub const LINK: u8 = 81;

// ── code-syntax sub-palette (light, best-effort highlighter) ─────────────────────
pub const CODE_KEYWORD: u8 = 213; // magenta
pub const CODE_STRING: u8 = 120; // green
pub const CODE_NUMBER: u8 = 81; // cyan
pub const CODE_COMMENT: u8 = 245; // grey
pub const CODE_RULE: u8 = 240; // the left │ / box border

// ── role tokens ─────────────────────────────────────────────────────────────────
/// The user's own words in the transcript.
pub const USER: u8 = 81;
/// The assistant gutter/attribution mark.
pub const ASSISTANT: u8 = 213;
/// Tool-call names + targets.
pub const TOOL: u8 = 120;
/// The reasoning / live-status channel — the working caption's hue.
pub const REASONING: u8 = 141;
/// The HUD strip text.
pub const HEADER: u8 = 87;
/// Splash wordmark gradient (top row first).
pub const TITLE: [u8; 5] = [213, 207, 171, 141, 105];
/// Four accent tones (splash groups, command panel, spinner).
pub const TONE: [u8; 4] = [213, 81, 120, 220];

// ── helpers (return StyledObject so callers can still chain .bold()/.italic()) ───
pub fn accent<D: Display>(d: D) -> StyledObject<D> {
    style(d).color256(accent_idx())
}
pub fn accent_dim<D: Display>(d: D) -> StyledObject<D> {
    style(d).color256(accent_dim_idx())
}
pub fn muted<D: Display>(d: D) -> StyledObject<D> {
    style(d).color256(muted_idx())
}
pub fn faint<D: Display>(d: D) -> StyledObject<D> {
    style(d).color256(faint_idx())
}
pub fn ok<D: Display>(d: D) -> StyledObject<D> {
    style(d).color256(ok_idx())
}
pub fn err<D: Display>(d: D) -> StyledObject<D> {
    style(d).color256(err_idx())
}
pub fn warn<D: Display>(d: D) -> StyledObject<D> {
    style(d).color256(warn_idx())
}
pub fn link<D: Display>(d: D) -> StyledObject<D> {
    style(d).color256(link_idx())
}
pub fn code_keyword<D: Display>(d: D) -> StyledObject<D> {
    style(d).color256(code_keyword_idx())
}
pub fn code_string<D: Display>(d: D) -> StyledObject<D> {
    style(d).color256(code_string_idx())
}
pub fn code_number<D: Display>(d: D) -> StyledObject<D> {
    style(d).color256(code_number_idx())
}
pub fn code_comment<D: Display>(d: D) -> StyledObject<D> {
    style(d).color256(code_comment_idx())
}
pub fn code_rule<D: Display>(d: D) -> StyledObject<D> {
    style(d).color256(code_rule_idx())
}
pub fn assistant<D: Display>(d: D) -> StyledObject<D> {
    style(d).color256(assistant_idx())
}
pub fn tool<D: Display>(d: D) -> StyledObject<D> {
    style(d).color256(tool_idx())
}

/// The compiled-in palette. A record (rather than bare consts) so the ratatui paint path can read
/// every role from one value; `Copy`, so `current_palette()` is free.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemePalette {
    pub accent: u8,
    pub accent_dim: u8,
    pub muted: u8,
    pub faint: u8,
    pub ok: u8,
    pub err: u8,
    pub warn: u8,
    pub link: u8,
    pub code_keyword: u8,
    pub code_string: u8,
    pub code_number: u8,
    pub code_comment: u8,
    pub code_rule: u8,
    /// The user's own words in the transcript (their turn).
    pub user: u8,
    /// The assistant gutter / attribution mark.
    pub assistant: u8,
    /// Tool-call names + targets.
    pub tool: u8,
    /// The reasoning / live-status channel (the working caption).
    pub reasoning: u8,
    /// The HUD strip text.
    pub header: u8,
    /// Five-stop gradient for the splash block-art wordmark, top row first.
    pub title: [u8; 5],
    /// Four accent tones: splash group headings, command panel, spinner.
    pub tone: [u8; 4],
}

impl Default for ThemePalette {
    fn default() -> Self {
        Self {
            accent: ACCENT,
            accent_dim: ACCENT_DIM,
            muted: MUTED,
            faint: FAINT,
            ok: OK,
            err: ERR,
            warn: WARN,
            link: LINK,
            code_keyword: CODE_KEYWORD,
            code_string: CODE_STRING,
            code_number: CODE_NUMBER,
            code_comment: CODE_COMMENT,
            code_rule: CODE_RULE,
            user: USER,
            assistant: ASSISTANT,
            tool: TOOL,
            reasoning: REASONING,
            header: HEADER,
            title: TITLE,
            tone: TONE,
        }
    }
}

/// The one active palette (compiled in; no runtime switching).
pub fn current_palette() -> ThemePalette {
    ThemePalette::default()
}

// ── dynamic index getters (used by the ratatui paint path) ──────────────────────
pub fn accent_idx() -> u8 {
    current_palette().accent
}
pub fn accent_dim_idx() -> u8 {
    current_palette().accent_dim
}
pub fn muted_idx() -> u8 {
    current_palette().muted
}
pub fn faint_idx() -> u8 {
    current_palette().faint
}
pub fn ok_idx() -> u8 {
    current_palette().ok
}
pub fn err_idx() -> u8 {
    current_palette().err
}
pub fn warn_idx() -> u8 {
    current_palette().warn
}
pub fn link_idx() -> u8 {
    current_palette().link
}
pub fn code_keyword_idx() -> u8 {
    current_palette().code_keyword
}
pub fn code_string_idx() -> u8 {
    current_palette().code_string
}
pub fn code_number_idx() -> u8 {
    current_palette().code_number
}
pub fn code_comment_idx() -> u8 {
    current_palette().code_comment
}
pub fn code_rule_idx() -> u8 {
    current_palette().code_rule
}
pub fn assistant_idx() -> u8 {
    current_palette().assistant
}
pub fn tool_idx() -> u8 {
    current_palette().tool
}
/// The splash wordmark gradient (5 stops, top row first).
pub fn title_ramp() -> [u8; 5] {
    current_palette().title
}
/// One of the four accent tones (wraps).
pub fn tone_idx(i: usize) -> u8 {
    let t = current_palette().tone;
    t[i % t.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_colours_are_distinct() {
        // A regression guard: if two semantic roles collapse onto the same index the UI loses
        // meaning. Accent/ok/err/link/warn must all differ.
        let all = [ACCENT, OK, ERR, WARN, LINK];
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(a, b, "two semantic colours share index {a}");
            }
        }
    }

    #[test]
    fn role_colours_are_distinct() {
        // Vivid is the point: user / assistant / tool / reasoning / header must be different hues.
        let roles = [USER, ASSISTANT, TOOL, REASONING, HEADER];
        for (i, a) in roles.iter().enumerate() {
            for b in &roles[i + 1..] {
                assert_ne!(a, b, "two role tokens share index {a}");
            }
        }
    }

    #[test]
    fn helpers_render_to_nonempty() {
        // Under the test harness colours may be stripped (no TTY); the text must still be present.
        assert!(accent("x").to_string().contains('x'));
        assert!(ok("done").to_string().contains("done"));
    }

    #[test]
    fn indices_and_ramps_have_sane_shape() {
        let pal = current_palette();
        for c in [
            pal.accent,
            pal.accent_dim,
            pal.muted,
            pal.faint,
            pal.ok,
            pal.err,
            pal.warn,
            pal.link,
            pal.user,
            pal.assistant,
            pal.tool,
            pal.reasoning,
            pal.header,
        ] {
            // 256-colour space only (no truecolor dependency).
            assert!(c <= 255);
        }
        assert_eq!(title_ramp().len(), 5);
        // tone wraps rather than panicking on an out-of-range index.
        assert_eq!(tone_idx(0), tone_idx(4));
    }
}
