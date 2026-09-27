# Installing F.Auto

Customer-facing setup guide. **One static binary, no runtime** — no Node, no Python, no Docker, no
toolchain, no admin rights. A fresh install takes about five minutes, and almost all of it is the
one-line install plus `fauto config`.

> This guide covers **installing and first-run configuration**. For every command and setting, see
> the [full reference](REFERENCE.md). For how the permission layers work, see [SANDBOX.md](SANDBOX.md).

---

## 1. What you need

| | |
|---|---|
| **OS** | Windows 10/11 (x86_64) · Linux (x86_64) · macOS (Apple Silicon; Intel Macs are no longer supported) |
| **Terminal** | A real interactive terminal. The REPL needs a TTY — piped/CI stdin prints a hint and exits |
| **Model access** | Any OpenAI-compatible `/chat/completions` endpoint and its API key — OpenAI, OpenRouter, a local llama.cpp/vLLM, an Anthropic gateway, … |

Unsigned-binary warnings and other first-run notes are in [§6 Known behaviors](#6-known-behaviors).

## 2. Install

### Windows (PowerShell 5+)

```powershell
irm https://raw.githubusercontent.com/Huutantv/factory-ai-main_v2/main/install.ps1 | iex
```

The installer downloads the latest `fauto.exe` from GitHub Releases, drops it in
`%LOCALAPPDATA%\F.Auto`, and adds that folder to your **user PATH**. No admin rights.

Override the install directory with `$env:FAUTO_INSTALL` before running the line above.

### Linux and macOS

```bash
curl -fsSL https://raw.githubusercontent.com/Huutantv/factory-ai-main_v2/main/install.sh | sh
```

Installs to `~/.fauto/bin/fauto` (override with `$FAUTO_INSTALL`). The script does **not** edit your
shell profile; it prints the one `export PATH="…"` line to add to `~/.bashrc` or `~/.zshrc`.

`fauto update` replaces the binary in the install directory it was launched from; if that directory
is not writable, set `AIZEN_INSTALL` to an install dir you own.

### Then, in a **new** terminal

```bash
fauto --version     # should print: F.Auto 1.7.7
```

If the command is not found, the new shell hasn't picked up PATH yet — open a fresh terminal (on
Linux/macOS add the `export PATH` line from the previous step first).

<details>
<summary>Manual install (no installer script)</summary>

Download the archive for your platform from the
[latest release](https://github.com/Huutantv/factory-ai-main_v2/releases/latest), unpack it, and put
the `fauto` binary somewhere on your PATH.

Build it yourself instead — an existing Rust toolchain is required:

```bash
cargo install --git https://github.com/Huutantv/factory-ai-main_v2
```

</details>

## 3. Configure a model — `fauto config`

```bash
fauto config
```

This is the only required setup step. It walks a short guided wizard and **saves at the end**, so
`Ctrl-C` before the last step cancels cleanly:

1. **Connection** — pick a provider preset or enter a base URL, then paste the API key. The key is
   verified against the endpoint before you continue (a bad key fails here, not on your first prompt).
2. **Model & context** — pick from the model list the endpoint reports, or type an id manually. The
   context window is pre-filled when the provider reports one, else estimated by model name.
3. **Web search** *(optional)* — a search provider key. `web_search` is keyed-only, so without it the
   agent can still fetch URLs but cannot search. Skip with Enter and add it later.
4. **Behavior** — auto-compact threshold, and the two passive learners (skills and durable memory).
5. **Display** — icon style: `emoji` (works on any font), `nerd` (needs a Nerd Font), or `off`.

Config is stored in `~/.aizen/cli-config.json` (`%USERPROFILE%\.aizen\cli-config.json` on Windows),
written owner-only. On Windows, `~/.aizen/` resolves under your user profile. You never have to hand-edit it.

### Or configure non-interactively

```bash
fauto config set --base-url https://api.openai.com/v1 --api-key sk-... --model gpt-4o-mini
```

Precedence for every connection field is **CLI flag → environment variable → saved config**. So a
per-shell override works without touching the file:

| env var | flag |
|---|---|
| `AIZEN_BASE_URL` | `--base-url` |
| `AIZEN_API_KEY` | `--api-key` |
| `AIZEN_MODEL` | `-m, --model` |

Inspect what is actually in effect with `fauto config show`; `fauto config path` prints the file
location. `fauto config set --help` lists every settable field.

### Verify the connection

```bash
fauto models                       # lists the models the endpoint reports
fauto chat -p "say hello in one line"
```

If `fauto chat` answers, you are done.

## 4. Start working

```bash
fauto
```

Lands in the unified chat + agent REPL — no mode switch, just type. A plain message is answered; a
task that needs tools uses them. Try:

```bash
fauto agent "run the tests and fix what fails"
```

Useful first commands:

| command | does |
|---|---|
| `fauto config` | re-run the settings wizard at any time |
| `fauto --help` | top-level command list |
| `/help` in the REPL | the slash-command meta layer |
| `fauto update` | upgrade **or roll back** to any published version |

## 5. Keep it current

```bash
fauto update          # list every published version, install the one you pick
```

`fauto update` takes the running version as already marked in its list and defaults the picker to the
newest stable release, so it doubles as the version check. `fauto --version` confirms what you run;
`fauto config show` includes the update-check setting.

## 6. Known behaviors

These are honest current limitations, not bugs you need to work around.

- **Windows SmartScreen.** The `.exe` is **unsigned**, so the first launch warns: *More info → Run
  anyway*. This is expected on every release until the binary is code-signed.
- **macOS is not notarized.** Gatekeeper will need a manual allow (`System Settings → Privacy &
  Security → Open Anyway`) the first time. Apple Silicon only.
- **No package managers yet.** F.Auto is not published to winget / scoop / Homebrew / crates.io / AUR.
  The install scripts above are the supported path.
- **Auto-update check.** The REPL checks for a newer release once per day in the background (cached,
  at most one line). Disable it with `fauto config set --update-check false` or set
  `AIZEN_NO_UPDATE_CHECK=1`.
- **A real terminal is required.** The REPL exits with a hint on piped/CI stdin. For scripted,
  non-interactive use, use the one-shot `fauto chat -p "…"` or `fauto agent "…"` forms; unattended
  runs fail closed on approval.
- **API keys live in the config file.** They are written owner-only, and are scrubbed from the
  environment of every child process F.Auto spawns. Do not paste keys into chats you save or share.

## 7. Getting help

- Full manual: [`docs/REFERENCE.md`](REFERENCE.md) — REPL surface, every command, MCP, browser
  tools, self-hosting (Telegram/Discord/Docker/Kubernetes), memory, skills.
- Safety model in detail: [`docs/SANDBOX.md`](SANDBOX.md) — the hard command floor, the approval
  layer, and the OS sandbox (`fauto sandbox status` shows what *your* machine enforces).
- Bugs and feature requests: open an issue on
  [`Huutantv/factory-ai-main_v2`](https://github.com/Huutantv/factory-ai-main_v2/issues).

---

## Uninstall

Remove the binary and, if you want a clean slate, the data directory.

| platform | binary | data |
|---|---|---|
| Windows | delete `%LOCALAPPDATA%\F.Auto` and its PATH entry | delete `%USERPROFILE%\.aizen` |
| Linux / macOS | `rm -rf ~/.fauto` and remove the `export PATH` line | `rm -rf ~/.aizen` |

`~/.aizen` holds your config, saved sessions, memory, and skills — deleting it is irreversible, so
back it up first if you may reinstall.
