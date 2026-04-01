# discord-caps-copy-paste

`discord-caps-copy-paste` is a compiled Rust launcher for one narrow workflow: take a Discord-style prompt from `--prompt`, `DCCP_PROMPT`, or the system clipboard, open a fresh Codex CLI session in a random installed terminal, then bind that new Codex session to Tether with `tether attach -p discord`.

![discord-caps-copy-paste demo](docs/discord-caps-copy-paste-demo.gif)

## What it does

- resolves prompt text from `--prompt`, `DCCP_PROMPT`, or clipboard tools such as `wl-paste`, `xclip`, `xsel`, and `pbpaste`
- chooses one installed terminal at random from the supported set unless `--terminal` is pinned
- launches `codex` in that terminal with the prompt as the initial request
- waits for a new `tether list --external -r codex` session to appear
- attaches the discovered session to the requested platform, default `discord`

## Supported terminals

- `kitty`
- `alacritty`
- `konsole`
- `gnome-terminal`
- `xterm`
- `foot`
- `qterminal`
- `xfce4-terminal`
- `tilix`

## Build

```bash
cargo build --release
```

The binary will be written to `target/release/discord-caps-copy-paste`.

## Usage

Explicit prompt:

```bash
./target/release/discord-caps-copy-paste \
  --prompt "Investigate the failing Discord bridge and post a fix plan." \
  --cwd "$HOME/subprojects/tether"
```

Clipboard-backed prompt:

```bash
./target/release/discord-caps-copy-paste --cwd "$HOME/subprojects/tether"
```

Deterministic terminal selection for automation:

```bash
DCCP_RANDOM_SEED=7 \
DCCP_TERMINAL_CANDIDATES=xterm \
./target/release/discord-caps-copy-paste --prompt "Record the UI demo"
```

## Runtime knobs

- `DCCP_PROMPT`: prompt fallback when `--prompt` is omitted
- `DCCP_CWD`: working directory for the launched Codex session
- `DCCP_TERMINAL`: force one terminal instead of random selection
- `DCCP_TERMINAL_CANDIDATES`: comma-separated allowlist used for random selection
- `DCCP_CODEX_BIN`: Codex binary path, default `codex`
- `DCCP_TETHER_BIN`: Tether binary path, default `tether`
- `DCCP_PLATFORM`: Tether platform for attach, default `discord`
- `DCCP_RANDOM_SEED`: deterministic RNG seed for repeatable launches and tests
- `DCCP_DISCOVERY_TIMEOUT_MS`: wait budget for the new external Codex session
- `DCCP_DISCOVERY_POLL_MS`: poll interval while waiting for Tether discovery
- `DCCP_SKIP_TETHER_START`: do not call `tether start` when `tether status` is unhealthy

## Dockerized UI test with screen recording

The repository includes a containerized X11 harness that runs the launcher against fake `codex` and `tether` binaries, records the terminal window, and refreshes the tracked README demo GIF.

```bash
./scripts/run_docker_ui_test.sh
```

Outputs:

- tracked README asset: `docs/discord-caps-copy-paste-demo.gif`
- transient artifacts: `artifacts/ui/discord-caps-copy-paste-demo.mp4`
- fake session state: `artifacts/ui/state/`

The harness verifies that:

- the launcher selected a supported terminal
- the fake Codex process received the prompt
- a new external Codex session appeared to Tether
- the launcher attached that session with `-p discord`

## Skill metadata

This repository is also a Codex skill bundle:

- [`SKILL.md`](SKILL.md) defines when to use it
- [`agents/openai.yaml`](agents/openai.yaml) provides UI-facing skill metadata

Use the compiled binary when the user wants a real machine-side launch, and use the fake harness when you need deterministic local verification.
