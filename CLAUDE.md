# CLAUDE.md — project facts for coding agents

Read this before touching anything. It records the decisions an agent is most likely to get wrong
because older files, older releases, and stale memory say otherwise.

## What this project is

**F.Auto** — a terminal-native agentic coding CLI. Pure Rust, shipped as **one static binary**, points
at any OpenAI-compatible `/chat/completions` endpoint. The command is `fauto`.

Hard constraints (a change that breaks one of these will be rejected):

- **Single static binary.** No C/native dependency, no external runtime. TLS is rustls-only — never
  reintroduce OpenSSL or anything needing a native toolchain.
- **Pure Rust.** If a crate pulls a C build, it doesn't go in.
- Startup time and binary size are features. Measured 2026-08-02: **10.8 ms** startup, **34.1 MB**
  binary. This is the project's main advantage over Claude Code (needs Node) and Hermes Agent
  (needs Python + ~2.4 GB) — do not regress it casually.

## License — Apache-2.0 (changed 2026-08-03)

**The project is Apache-2.0. It is NOT PolyForm Noncommercial anymore.** It is open source, and
commercial use is allowed.

- `LICENSE` is the verbatim Apache License 2.0 — the TERMS AND CONDITIONS section is a byte-exact
  copy of <https://www.apache.org/licenses/LICENSE-2.0.txt>. **Never reword, reformat, or "tidy" it**;
  GitHub's license detection and every corporate legal review depend on it matching exactly. Only the
  APPENDIX carries the filled-in `Copyright 2026 F.Auto authors`.
- `Cargo.toml` uses `license = "Apache-2.0"` (the SPDX id), **not** `license-file`. Using
  `license-file` makes crates.io and GitHub report an unrecognized license.
- `NOTICE` must ship with any redistribution (Apache §4d). Keep it in sync when attribution changes.
- **There IS a CLA, as of 2026-08-17.** `CLA.md` (v2.0) is enforced by
  `.github/workflows/cla.yml`; contributors agree once by commenting the sign sentence on their PR.
  **`.github/workflows/dco.yml` was deleted and the DCO sign-off is no longer asked for** — don't tell
  contributors to `git commit -s`, and don't restore that workflow.
  - History, so nobody "fixes" this back and forth: a CLA v1.0 existed under PolyForm, was deleted at
    the Apache relicense (`586b7e9`) in favour of Apache-2.0 §5 + DCO, and was reinstated as v2.0 by
    maintainer decision. v2.0 differs from v1.0 in naming Apache-2.0 as the public license.
  - The point of §3 is the **commercial relicensing grant** — it is what keeps a paid edition
    possible. It is a broader ask than Apache-2.0 §5, so user-facing text must say so plainly rather
    than describing the CLA as a formality.
- Trademark: Apache §6 grants no rights to the "F.Auto" name or logo. Forks may use the code, not the
  brand.
- Releases **up to and including v0.5.5** went out under PolyForm Noncommercial. That is history and
  cannot be retracted; everything from the relicensing commit onward is Apache-2.0. If you find a
  file still saying PolyForm, it is a leftover — fix it.

## Git remote — one canonical repo

```
origin  → https://github.com/Huutantv/factory-ai   (PRIVATE fork — full source, day-to-day work)
```

- **The canonical repo is `Huutantv/factory-ai`.** Write it in all user-facing URLs, install scripts,
  and code. The upstream lineage is `aizen-stack/aizen` (and the older `dawnofcd/Aizen_agent` /
  `dawnofcd/aizen` remotes) — those are history, not this project's target.
- The product is **F.Auto**, the CLI command is **`fauto`**. The binary/package name is `fauto`
  (see `Cargo.toml` `[[bin]]`), so **all build/test commands use `--bin fauto`** — `--bin aizen`
  does not exist and fails ("no bin target named `aizen`").
- Release binaries are published to `Huutantv/factory-ai`. `src/features/update.rs` has
  `DEFAULT_REPO` and `fauto update` reads releases from there — keep it aligned with `install.ps1`
  (`$Repo`) and `install.sh` (`repo=`).
- Never push to `main` without being asked. Branch, then push with `-u`.

## Layout worth knowing

| Path | What |
|---|---|
| `README.md` | the short landing page — install, why, what it does. **Keep it under ~110 lines**; details go to `docs/REFERENCE.md`, not here |
| `docs/REFERENCE.md` | the full manual: REPL surface, every command, self-hosting, MCP, browser, safety model |
| `src/features/update.rs` | self-update; `DEFAULT_REPO` lives here |
| `src/sandbox/` | OS sandbox under approval/cmd_guard: policy · runner · audit · per-platform backends (Landlock/seccomp · Job Object · Seatbelt). Every model/repo-influenced spawn goes through `runner::prepare_*` — never add a bare `Command::new` for those. `docs/SANDBOX.md` is the contract; keep its capability matrix honest |
| `install.ps1` / `install.sh` | one-line installers; repo slug is hardcoded in both |
| `CLA.md` + `.github/workflows/cla.yml` | the CLA and its bot (signatures land in the `cla-signatures` branch); replaced the DCO check |
| `dist/` | assets for the public download channel |
| `docs/` | design + audit notes |
| `bench-fixtures/` | fixtures for `fauto bench` |

Links inside `docs/REFERENCE.md` that point at repo-root paths need a `../` prefix — it lives one
level down.

## Build / verify

```bash
cargo check                                   # fast feedback
cargo build --release --bin fauto             # the shipped artifact
cargo test --bin fauto                        # must be green before pushing
cargo build --release --features dense --bin fauto   # semantic-retrieval tier (feature-gated)
cargo fmt && cargo clippy
```

Note: `cargo test` on Windows can be slow; the 120 s shell cap may kill it. Run long builds through a
background process, not a foreground shell call.

## Known distribution gaps (as of 2026-08-03)

Real numbers, not guesses — from the GitHub API:

- `Huutantv/factory-ai`: 25 stars, 4 forks, **0 watchers**, Discussions **off**.
- v0.5.5 downloads: Windows 3, Linux 0, macOS 0.
- Windows `.exe` is **unsigned** → SmartScreen warns. macOS is **not notarized**.
- Not published to winget / scoop / Homebrew / crates.io / AUR. Apache-2.0 now unblocks the OSI-only
  ones (crates.io, AUR, Homebrew).
- The upstream landing page (`fauto-stack.vercel.app`) belonged to the project before the fork and is
  not ours; there is no live landing page for `Huutantv/factory-ai` yet. Add one when releasing.

## Working style the maintainer expects

- Vietnamese for conversation; English for code, comments, and docs.
- Evidence over assumption: measure, cite the tool output, and say plainly when something is
  unverified. Don't claim a command passed unless you ran it.
- Say what is weak about this project honestly — no sales pitch about our own tool.
- Business/licensing decisions are the maintainer's, not the agent's. Ask before changing one.
