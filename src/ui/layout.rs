//! TUI layout system — opencode-style spacing/visibility config.
//!
//! Complements `ui::theme` (colours) with structure: how much breathing room messages get,
//! whether the HUD/footer chrome shows, and how dense the transcript is. Built-ins are
//! `default`, `balanced`, `polished`, `dense`, and `spacious`; custom layouts live in
//! `~/.aizen/layout/*.{json,jsonc}` and are picked with `/layout` (persisted in cli-config).

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

/// Spacing + visibility knobs for one frame. All fields have serde defaults so a custom
/// JSON file can override just what it cares about.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayoutConfig {
    pub name: String,
    /// Blank lines inserted between transcript blocks.
    #[serde(default = "d1")]
    pub message_separation: usize,
    /// Left padding cells for transcript rows.
    #[serde(default = "d0")]
    pub message_padding_left: usize,
    /// Show the HUD status strip (model · tokens · turns) above the input box.
    #[serde(default = "t")]
    pub show_header: bool,
    /// Show the input-box framing rules + footer chrome.
    #[serde(default = "t")]
    pub show_footer: bool,
    /// Show the right-side context meter + health chip in the HUD.
    #[serde(default = "t")]
    pub show_context_meter: bool,
    /// Compact tool rows: single line `icon name — digest` instead of the two-line call/result.
    #[serde(default = "f")]
    pub compact_tools: bool,
    /// Max composer rows before scrolling (clamped by terminal height anyway).
    #[serde(default = "d10")]
    pub max_input_rows: usize,
    /// Show keyboard hints on the empty composer row.
    #[serde(default = "t")]
    pub show_input_hints: bool,
}

fn d0() -> usize {
    0
}
fn d1() -> usize {
    1
}
fn d10() -> usize {
    10
}
fn t() -> bool {
    true
}
fn f() -> bool {
    false
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            message_separation: 1,
            message_padding_left: 0,
            show_header: true,
            show_footer: true,
            show_context_meter: true,
            compact_tools: false,
            max_input_rows: 10,
            show_input_hints: true,
        }
    }
}

impl LayoutConfig {
    pub fn dense() -> Self {
        Self {
            name: "dense".to_string(),
            message_separation: 0,
            message_padding_left: 0,
            show_header: true,
            show_footer: true,
            show_context_meter: false,
            compact_tools: true,
            max_input_rows: 5,
            show_input_hints: false,
        }
    }

    /// A balanced presentation for normal terminals: a little breathing room around the transcript,
    /// compact tool activity, and a context meter that remains useful without dominating the footer.
    pub fn polished() -> Self {
        Self {
            name: "polished".to_string(),
            message_separation: 1,
            message_padding_left: 1,
            show_header: true,
            show_footer: true,
            show_context_meter: true,
            compact_tools: true,
            max_input_rows: 8,
            show_input_hints: true,
        }
    }

    pub fn spacious() -> Self {
        Self {
            name: "spacious".to_string(),
            message_separation: 2,
            message_padding_left: 2,
            show_header: true,
            show_footer: true,
            show_context_meter: true,
            compact_tools: false,
            max_input_rows: 10,
            show_input_hints: true,
        }
    }

    /// The recommendation for a normal terminal: transcript gets one blank line of air and a
    /// single-cell left pad, tool activity is compact, and the context meter stays on. Shorter
    /// composer than `polished` so the conversation keeps most of the screen.
    pub fn balanced() -> Self {
        Self {
            name: "balanced".to_string(),
            message_separation: 1,
            message_padding_left: 1,
            show_header: true,
            show_footer: true,
            show_context_meter: true,
            compact_tools: true,
            max_input_rows: 6,
            show_input_hints: true,
        }
    }
}

/// Built-in layouts in picker order.
pub fn builtin_layouts() -> Vec<LayoutConfig> {
    vec![
        LayoutConfig::default(),
        LayoutConfig::balanced(),
        LayoutConfig::polished(),
        LayoutConfig::dense(),
        LayoutConfig::spacious(),
    ]
}

fn layout_dir() -> std::path::PathBuf {
    crate::core::config::aizen_home().join("layout")
}

/// Strip `//` and `/* */` comments so `.jsonc` files parse as JSON.
fn strip_jsonc(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    let mut in_str = false;
    let mut esc = false;
    while let Some(c) = chars.next() {
        if in_str {
            out.push(c);
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == '"' {
                in_str = false;
            }
            continue;
        }
        if c == '"' {
            in_str = true;
            out.push(c);
            continue;
        }
        if c == '/' {
            match chars.peek() {
                Some('/') => {
                    for nc in chars.by_ref() {
                        if nc == '\n' {
                            out.push('\n');
                            break;
                        }
                    }
                    continue;
                }
                Some('*') => {
                    chars.next();
                    let mut prev_star = false;
                    for nc in chars.by_ref() {
                        if prev_star && nc == '/' {
                            break;
                        }
                        prev_star = nc == '*';
                    }
                    continue;
                }
                _ => {
                    out.push(c);
                    continue;
                }
            }
        }
        out.push(c);
    }
    out
}

/// All layout names: built-ins first, then custom files.
pub fn list_layouts() -> Vec<String> {
    let mut names: Vec<String> = builtin_layouts().iter().map(|l| l.name.clone()).collect();
    for custom in custom_layout_names() {
        if !names.iter().any(|n| n.eq_ignore_ascii_case(&custom)) {
            names.push(custom);
        }
    }
    names
}

fn custom_layout_names() -> Vec<String> {
    let dir = layout_dir();
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(&dir) else {
        return out;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "json" && ext != "jsonc" {
            continue;
        }
        if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
            out.push(stem.to_string());
        }
    }
    out.sort();
    out
}

/// Resolve by name: built-in → custom file → `None`.
pub fn resolve_layout(name: &str) -> Option<LayoutConfig> {
    let want = name.trim();
    if want.is_empty() {
        return None;
    }
    if let Some(b) = builtin_layouts()
        .into_iter()
        .find(|l| l.name.eq_ignore_ascii_case(want))
    {
        return Some(b);
    }
    for ext in ["json", "jsonc"] {
        let path = layout_dir().join(format!("{want}.{ext}"));
        let Ok(raw) = std::fs::read_to_string(&path) else {
            continue;
        };
        let clean = strip_jsonc(&raw);
        if let Ok(mut l) = serde_json::from_str::<LayoutConfig>(&clean) {
            if l.name.trim().is_empty() {
                l.name = want.to_string();
            }
            return Some(l);
        }
    }
    None
}

static AUTO: AtomicBool = AtomicBool::new(false);
/// The live layout. Starts as `balanced` — the same geometry an unset config resolves to — so no
/// first frame ever paints with a different geometry before `apply_saved_layout()` runs.
static CURRENT: once_cell::sync::Lazy<std::sync::RwLock<LayoutConfig>> =
    once_cell::sync::Lazy::new(|| std::sync::RwLock::new(LayoutConfig::balanced()));

/// Select the responsive built-in layout for a terminal width.
pub fn layout_for_columns(cols: u16) -> LayoutConfig {
    if cols < 80 {
        LayoutConfig::dense()
    } else if cols < 120 {
        LayoutConfig::balanced()
    } else {
        LayoutConfig::spacious()
    }
}

/// Currently active layout (cloned; cheap).
pub fn current() -> LayoutConfig {
    if AUTO.load(Ordering::Relaxed) {
        let cols = crossterm::terminal::size().map(|(w, _)| w).unwrap_or(80);
        return layout_for_columns(cols);
    }
    CURRENT.read().map(|g| g.clone()).unwrap_or_default()
}

/// Activate by name, persisting into cli-config (`layout`). Returns `None` when unknown.
pub fn set_layout(name: &str) -> Option<LayoutConfig> {
    if name.trim().eq_ignore_ascii_case("auto") {
        AUTO.store(true, Ordering::Relaxed);
        let mut cfg = crate::core::cli_config::load();
        cfg.layout = Some("auto".to_string());
        let _ = crate::core::cli_config::save(&cfg);
        return Some(current());
    }
    AUTO.store(false, Ordering::Relaxed);
    let l = resolve_layout(name)?;
    if let Ok(mut g) = CURRENT.write() {
        *g = l.clone();
    }
    let mut cfg = crate::core::cli_config::load();
    cfg.layout = Some(l.name.clone());
    let _ = crate::core::cli_config::save(&cfg);
    Some(l)
}

/// Load the persisted layout into the live cache. Called once at REPL startup.
///
/// When nothing is persisted, `balanced` is the out-of-box look: one blank line of air + a single
/// left pad + compact tool rows. `/layout default` still selects the legacy zero-pad geometry.
pub fn apply_saved_layout() {
    let name = crate::core::cli_config::load()
        .layout
        .unwrap_or_else(|| "balanced".to_string());
    if name.eq_ignore_ascii_case("auto") {
        AUTO.store(true, Ordering::Relaxed);
        return;
    }
    AUTO.store(false, Ordering::Relaxed);
    if let Some(l) = resolve_layout(&name) {
        if let Ok(mut g) = CURRENT.write() {
            *g = l;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_resolve() {
        assert!(resolve_layout("default").is_some());
        assert!(resolve_layout("dense").is_some());
        assert!(resolve_layout("DENSE").is_some());
        assert!(resolve_layout("spacious").is_some());
        assert!(resolve_layout("nope").is_none());
    }

    #[test]
    fn dense_is_compact() {
        let d = LayoutConfig::dense();
        assert_eq!(d.message_separation, 0);
        assert!(d.compact_tools);
        assert!(!d.show_context_meter);
        assert!(!d.show_input_hints);
    }

    #[test]
    fn polished_balances_spacing_and_tool_density() {
        let p = LayoutConfig::polished();
        assert_eq!(p.message_padding_left, 1);
        assert!(p.compact_tools);
        assert!(p.show_context_meter);
        assert_eq!(p.max_input_rows, 8);
        assert!(p.show_input_hints);
    }

    #[test]
    fn spacious_gives_wide_terminals_more_breathing_room() {
        let s = LayoutConfig::spacious();
        assert_eq!(s.message_separation, 2);
        assert_eq!(s.message_padding_left, 2);
        assert!(!s.compact_tools);
    }

    #[test]
    fn responsive_breakpoints_are_stable() {
        assert_eq!(layout_for_columns(79).name, "dense");
        assert_eq!(layout_for_columns(80).name, "balanced");
        assert_eq!(layout_for_columns(119).name, "balanced");
        assert_eq!(layout_for_columns(120).name, "spacious");
    }

    #[test]
    fn balanced_is_distinct_from_polished() {
        let b = LayoutConfig::balanced();
        let p = LayoutConfig::polished();
        assert!(b.compact_tools && b.show_context_meter);
        assert_eq!(b.message_padding_left, 1);
        assert!(
            b.max_input_rows < p.max_input_rows,
            "balanced keeps a shorter composer so the transcript dominates"
        );
    }

    #[test]
    fn default_keeps_the_legacy_safe_geometry() {
        let d = LayoutConfig::default();
        assert_eq!(d.message_padding_left, 0);
        assert!(!d.compact_tools);
        assert_eq!(d.max_input_rows, 10);
        assert!(d.show_input_hints);
    }

    #[test]
    fn jsonc_comments_stripped() {
        let raw = r#"{
            // a comment
            "name": "x", /* inline */
            "message_separation": 2
        }"#;
        let clean = strip_jsonc(raw);
        let v: serde_json::Value = serde_json::from_str(&clean).expect("parses");
        assert_eq!(v["message_separation"], 2);
    }
}
