# Changelog

## v0.1.3 - 2026-04-02

- Fixed the generated Homebrew formula so `cargo install` does not receive `--locked` twice during macOS installs.
- Verified the regression against a real Homebrew install path after the `v0.1.2` rollout exposed the duplicate flag.

## v0.1.2 - 2026-04-02

- Scoped external-session discovery to the configured working directory when the installed Tether supports it, reducing cross-project attach races.
- Passed explicit `codex` runner plus target directory to `tether attach`, and ensured terminal launches inherit the requested working directory even for terminals without a dedicated cwd flag.
- Expanded unit coverage for Tether table parsing, single-session detection, attach command construction, and terminal cwd handling.

## v0.1.1 - 2026-04-02

- Added `scripts/check.sh` as the single Rust quality gate for format, lint, test, and release-build verification.
- Added GitHub Actions CI so pushes and pull requests run the Rust quality gate automatically.
- Added tagged-release packaging for Debian `.deb` assets, Homebrew formula generation, and source archives.
- Added release automation that uploads packaged artifacts to GitHub Releases and mirrors the Homebrew formula into `realagiorganization/homebrew-tap`.

## v0.1.0 - 2026-04-02

- Initial public release of the Rust launcher that opens a fresh Codex CLI session in a randomly selected installed terminal.
- Added Tether-oriented prompt forwarding so the generated session can be seeded directly for Discord control flows.
- Shipped the reusable Codex skill definition, demo assets, and Dockerized UI test harness for repeatable verification.
