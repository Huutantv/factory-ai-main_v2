//! Rich-style renderables for F.Auto — a display-only layer inspired by
//! Python's `rich` (Textualize): console markup, tables, trees, progress
//! bars, syntax highlighting, tracebacks, and leveled logging.
//!
//! # Display-only contract (load-bearing)
//! Everything in here styles what the HUMAN sees. It must never change
//! what the MODEL sees:
//! - Conversation history keeps raw markdown (`llm/client.rs` pushes `shown`
//!   before styling). These helpers take already-final strings and return
//!   styled strings for `tui::emit` only.
//! - Tool results fed back to the model stay in their current flat format;
//!   tree/table renderers here are opt-in display modes for CLI output, not
//!   replacements for tool-result text.
//! - `decorate=false` (pipes/CI, `NO_COLOR`) is always a verbatim
//!   passthrough — never emit ANSI when it is false.
//! - Pure Rust only: uses `console` (already in the tree) + `theme`. No new
//!   crates, so the single-static-binary / no-C-dep posture holds.
//!
//! Sub-projects: P1 tables (`table`), P2 tree (`tree`), P3 progress
//! (`progress`), P4 markup+log (`markup`), P5 syntax (`syntax`), P6
//! traceback (`traceback`).
//!
//! NOTE: per-sub-project display wiring (using these in CLI output paths)
//! lands as follow-ups; until then `dead_code` is allowed so the build stays
//! warning-clean without touching any model-visible strings.

#![allow(dead_code)]

pub mod markup;
pub mod progress;
pub mod syntax;
pub mod table;
pub mod traceback;
pub mod tree;
