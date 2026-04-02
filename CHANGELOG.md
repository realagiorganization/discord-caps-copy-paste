# Changelog

## v0.1.1 - 2026-04-02

- Added `scripts/check.sh` as the single Rust quality gate for format, lint, test, and release-build verification.
- Added GitHub Actions CI so pushes and pull requests run the Rust quality gate automatically.
- Added tagged-release packaging for Debian `.deb` assets, Homebrew formula generation, and source archives.
- Added release automation that uploads packaged artifacts to GitHub Releases and mirrors the Homebrew formula into `realagiorganization/homebrew-tap`.

## v0.1.0 - 2026-04-02

- Initial public release of the Rust launcher that opens a fresh Codex CLI session in a randomly selected installed terminal.
- Added Tether-oriented prompt forwarding so the generated session can be seeded directly for Discord control flows.
- Shipped the reusable Codex skill definition, demo assets, and Dockerized UI test harness for repeatable verification.
