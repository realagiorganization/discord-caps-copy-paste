---
name: discord-caps-copy-paste
description: Launch a clipboard or explicit Discord prompt into a new Codex CLI session in a random installed terminal, then attach the discovered Codex session to Tether with the Discord platform.
---

# Discord Caps Copy Paste

Use this skill when the user wants a prompt copied from Discord, the clipboard, or an explicit string launched into a fresh Codex CLI session and supervised through Tether/Discord.

## Workflow

1. Build the binary with `cargo build --release` if `target/release/discord-caps-copy-paste` does not exist yet.
2. Prefer an explicit `--prompt` when the task text should be stable; otherwise let the tool resolve `DCCP_PROMPT` or the clipboard.
3. Let the tool choose a random installed terminal unless the user asks for a specific terminal or you need deterministic automation.
4. The tool checks Tether health, starts it if needed, launches Codex in the terminal, waits for a new `tether list --external -r codex` session, and runs `tether attach -p discord`.
5. For deterministic verification, set `DCCP_TERMINAL_CANDIDATES=xterm`, `DCCP_RANDOM_SEED=<n>`, `DCCP_CODEX_BIN=./scripts/fake_codex.sh`, and `DCCP_TETHER_BIN=./scripts/fake_tether.sh`, then run `./scripts/run_docker_ui_test.sh`.

## Operator notes

- The launcher is reliability-first: it passes the prompt as the initial Codex CLI argument instead of relying on fragile GUI keystroke injection.
- Clipboard support prefers `wl-paste`, then `xclip`, then `xsel`, then `pbpaste`.
- If the host has no supported terminal installed, fail fast and ask the user to install one of the supported terminals or pin a working binary with `--terminal`.
- Keep `docs/discord-caps-copy-paste-demo.gif` current when the visible launcher flow changes.
