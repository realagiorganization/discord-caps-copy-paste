use anyhow::{Context, Result, anyhow, bail};
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use std::collections::BTreeSet;
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub const DEFAULT_TITLE: &str = "Discord Caps Copy Paste";
pub const DEFAULT_PLATFORM: &str = "discord";
pub const DEFAULT_DISCOVERY_TIMEOUT_MS: u64 = 30_000;
pub const DEFAULT_DISCOVERY_POLL_MS: u64 = 1_000;
pub const DEFAULT_TERMINAL_CANDIDATES: &[&str] = &[
    "kitty",
    "alacritty",
    "konsole",
    "gnome-terminal",
    "xterm",
    "foot",
    "qterminal",
    "xfce4-terminal",
    "tilix",
];

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub prompt: Option<String>,
    pub cwd: PathBuf,
    pub terminal: Option<String>,
    pub terminal_candidates: Vec<String>,
    pub codex_bin: String,
    pub tether_bin: String,
    pub platform: String,
    pub title: String,
    pub random_seed: Option<u64>,
    pub discovery_timeout_ms: u64,
    pub discovery_poll_ms: u64,
    pub skip_tether_start: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchResult {
    pub terminal: String,
    pub session_id: Option<String>,
    pub prompt_preview: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptSelection {
    pub value: String,
    pub source: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalChoice {
    pub name: String,
    pub program: PathBuf,
    kind: TerminalKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TerminalKind {
    Kitty,
    Alacritty,
    Konsole,
    GnomeTerminal,
    Xterm,
    Foot,
    Qterminal,
    Xfce4Terminal,
    Tilix,
}

impl TerminalKind {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "kitty" => Some(Self::Kitty),
            "alacritty" => Some(Self::Alacritty),
            "konsole" => Some(Self::Konsole),
            "gnome-terminal" => Some(Self::GnomeTerminal),
            "xterm" => Some(Self::Xterm),
            "foot" => Some(Self::Foot),
            "qterminal" => Some(Self::Qterminal),
            "xfce4-terminal" => Some(Self::Xfce4Terminal),
            "tilix" => Some(Self::Tilix),
            _ => None,
        }
    }
}

impl TerminalChoice {
    pub fn discover(name: &str) -> Option<Self> {
        let kind = TerminalKind::from_name(name)?;
        let program = find_executable(name)?;
        Some(Self {
            name: name.to_string(),
            program,
            kind,
        })
    }

    pub fn command(&self, cwd: &Path, title: &str, shell_script: &str) -> Command {
        let cwd_text = cwd.to_string_lossy().to_string();
        let mut command = Command::new(&self.program);
        match self.kind {
            TerminalKind::Kitty => {
                command.args([
                    "--title",
                    title,
                    "--directory",
                    &cwd_text,
                    "bash",
                    "-lc",
                    shell_script,
                ]);
            }
            TerminalKind::Alacritty => {
                command.args([
                    "--title",
                    title,
                    "--working-directory",
                    &cwd_text,
                    "-e",
                    "bash",
                    "-lc",
                    shell_script,
                ]);
            }
            TerminalKind::Konsole => {
                command.args([
                    "--workdir",
                    &cwd_text,
                    "-p",
                    &format!("tabtitle={title}"),
                    "-e",
                    "bash",
                    "-lc",
                    shell_script,
                ]);
            }
            TerminalKind::GnomeTerminal => {
                command.args([
                    "--title",
                    title,
                    "--working-directory",
                    &cwd_text,
                    "--",
                    "bash",
                    "-lc",
                    shell_script,
                ]);
            }
            TerminalKind::Xterm => {
                command.args(["-T", title, "-e", "bash", "-lc", shell_script]);
            }
            TerminalKind::Foot => {
                command.args(["-T", title, "-D", &cwd_text, "bash", "-lc", shell_script]);
            }
            TerminalKind::Qterminal => {
                command.args([
                    "-t",
                    title,
                    "-w",
                    &cwd_text,
                    "-e",
                    &format!("bash -lc {}", shell_quote(shell_script)),
                ]);
            }
            TerminalKind::Xfce4Terminal => {
                command.args([
                    "--title",
                    title,
                    "--working-directory",
                    &cwd_text,
                    "--command",
                    &format!("bash -lc {}", shell_quote(shell_script)),
                ]);
            }
            TerminalKind::Tilix => {
                command.args([
                    "--title",
                    title,
                    "--working-directory",
                    &cwd_text,
                    "-e",
                    &format!("bash -lc {}", shell_quote(shell_script)),
                ]);
            }
        }
        command
    }
}

pub fn run(config: &AppConfig) -> Result<LaunchResult> {
    let prompt = resolve_prompt(config.prompt.clone())?;
    let cwd = canonicalize_or_clone(&config.cwd)?;
    let tether_bin = resolve_program(&config.tether_bin)?;
    let codex_bin = resolve_program(&config.codex_bin)?;
    let terminal = choose_terminal(config)?;

    let shell_script = [
        "clear",
        "printf '%s\\n' 'discord-caps-copy-paste launching Codex...'",
        "printf '%s\\n\\n' \"prompt source: ${DCCP_PROMPT_SOURCE}\"",
        "exec \"$DCCP_CODEX_BIN\" \"$DCCP_PROMPT\"",
    ]
    .join("\n");

    if config.dry_run {
        return Ok(LaunchResult {
            terminal: terminal.name,
            session_id: None,
            prompt_preview: preview_prompt(&prompt.value),
        });
    }

    ensure_tether_ready(tether_bin.as_os_str(), config.skip_tether_start)?;
    let before = list_external_sessions(tether_bin.as_os_str())?;
    launch_terminal(
        &terminal,
        &cwd,
        &config.title,
        &shell_script,
        codex_bin.as_os_str(),
        &prompt,
    )?;
    let session_id = wait_for_new_session(
        tether_bin.as_os_str(),
        &before,
        Duration::from_millis(config.discovery_timeout_ms),
        Duration::from_millis(config.discovery_poll_ms),
    )?;
    attach_session(tether_bin.as_os_str(), &session_id, &config.platform)?;

    Ok(LaunchResult {
        terminal: terminal.name,
        session_id: Some(session_id),
        prompt_preview: preview_prompt(&prompt.value),
    })
}

pub fn resolve_prompt(explicit_prompt: Option<String>) -> Result<PromptSelection> {
    if let Some(value) = explicit_prompt {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            bail!("--prompt was empty");
        }
        return Ok(PromptSelection {
            value,
            source: "--prompt",
        });
    }

    if let Ok(value) = env::var("DCCP_PROMPT")
        && !value.trim().is_empty()
    {
        return Ok(PromptSelection {
            value,
            source: "DCCP_PROMPT",
        });
    }

    for probe in [
        ClipboardProbe::new("wl-paste", &["--no-newline"]),
        ClipboardProbe::new("xclip", &["-selection", "clipboard", "-o"]),
        ClipboardProbe::new("xsel", &["--clipboard", "--output"]),
        ClipboardProbe::new("pbpaste", &[]),
    ] {
        if let Some(value) = probe.try_read()? {
            return Ok(PromptSelection {
                value,
                source: probe.program,
            });
        }
    }

    bail!("no prompt supplied and no clipboard tool returned text")
}

pub fn choose_terminal(config: &AppConfig) -> Result<TerminalChoice> {
    if let Some(explicit_terminal) = &config.terminal {
        return TerminalChoice::discover(explicit_terminal).ok_or_else(|| {
            anyhow!(
                "terminal '{}' is not installed or not supported",
                explicit_terminal
            )
        });
    }

    let candidates = if config.terminal_candidates.is_empty() {
        DEFAULT_TERMINAL_CANDIDATES
            .iter()
            .map(|entry| (*entry).to_string())
            .collect()
    } else {
        config.terminal_candidates.clone()
    };

    let mut discovered = Vec::new();
    for candidate in candidates {
        if discovered
            .iter()
            .any(|entry: &TerminalChoice| entry.name == candidate)
        {
            continue;
        }
        if let Some(choice) = TerminalChoice::discover(&candidate) {
            discovered.push(choice);
        }
    }

    if discovered.is_empty() {
        bail!("no supported terminals were found in PATH");
    }

    if let Some(seed) = config.random_seed {
        let mut rng = StdRng::seed_from_u64(seed);
        return discovered
            .choose(&mut rng)
            .cloned()
            .ok_or_else(|| anyhow!("no terminal candidates remained after seeded selection"));
    }

    let mut rng = rand::thread_rng();
    discovered
        .choose(&mut rng)
        .cloned()
        .ok_or_else(|| anyhow!("no terminal candidates remained after random selection"))
}

pub fn parse_external_sessions(stdout: &str) -> BTreeSet<String> {
    stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with("No external sessions found"))
        .filter_map(|line| line.split_whitespace().next().map(ToOwned::to_owned))
        .collect()
}

pub fn wait_for_new_session(
    tether_bin: &OsStr,
    before: &BTreeSet<String>,
    timeout: Duration,
    poll_interval: Duration,
) -> Result<String> {
    let deadline = Instant::now() + timeout;
    loop {
        let after = list_external_sessions(tether_bin)?;
        if let Some(new_session) = after.difference(before).next() {
            return Ok(new_session.to_string());
        }
        if Instant::now() >= deadline {
            bail!("timed out waiting for a new external Codex session");
        }
        thread::sleep(poll_interval);
    }
}

fn attach_session(tether_bin: &OsStr, session_id: &str, platform: &str) -> Result<()> {
    let status = Command::new(tether_bin)
        .args(["attach", session_id, "-p", platform])
        .status()
        .with_context(|| format!("failed to run tether attach for session '{session_id}'"))?;
    if !status.success() {
        bail!("tether attach failed for session '{session_id}'");
    }
    Ok(())
}

fn ensure_tether_ready(tether_bin: &OsStr, skip_tether_start: bool) -> Result<()> {
    let healthy = Command::new(tether_bin)
        .arg("status")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false);
    if healthy || skip_tether_start {
        return Ok(());
    }

    let status = Command::new(tether_bin)
        .arg("start")
        .status()
        .context("failed to run tether start")?;
    if !status.success() {
        bail!("tether start returned a non-zero exit code");
    }
    Ok(())
}

fn launch_terminal(
    terminal: &TerminalChoice,
    cwd: &Path,
    title: &str,
    shell_script: &str,
    codex_bin: &OsStr,
    prompt: &PromptSelection,
) -> Result<()> {
    let mut command = terminal.command(cwd, title, shell_script);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .env("DCCP_CODEX_BIN", codex_bin)
        .env("DCCP_PROMPT", &prompt.value)
        .env("DCCP_PROMPT_SOURCE", prompt.source)
        .env("DCCP_SELECTED_TERMINAL", &terminal.name)
        .spawn()
        .with_context(|| format!("failed to launch terminal '{}'", terminal.name))?;
    Ok(())
}

fn list_external_sessions(tether_bin: &OsStr) -> Result<BTreeSet<String>> {
    let output = Command::new(tether_bin)
        .args(["list", "--external", "-r", "codex"])
        .output()
        .context("failed to run tether list --external -r codex")?;
    if !output.status.success() {
        bail!("tether list --external -r codex returned a non-zero exit code");
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_external_sessions(&stdout))
}

fn resolve_program(program: &str) -> Result<PathBuf> {
    find_executable(program).ok_or_else(|| anyhow!("unable to resolve '{}' in PATH", program))
}

fn canonicalize_or_clone(path: &Path) -> Result<PathBuf> {
    path.canonicalize()
        .with_context(|| format!("failed to access working directory '{}'", path.display()))
}

fn preview_prompt(prompt: &str) -> String {
    let single_line = prompt.lines().collect::<Vec<_>>().join(" ");
    if single_line.len() > 96 {
        let prefix = single_line.chars().take(96).collect::<String>();
        format!("{prefix}...")
    } else {
        single_line
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn find_executable(program: &str) -> Option<PathBuf> {
    let candidate = Path::new(program);
    if candidate.components().count() > 1 && is_executable(candidate) {
        return Some(candidate.to_path_buf());
    }

    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|entry| entry.join(program))
        .find(|candidate| is_executable(candidate))
}

fn is_executable(path: &Path) -> bool {
    path.is_file()
}

struct ClipboardProbe<'a> {
    program: &'a str,
    args: &'a [&'a str],
}

impl<'a> ClipboardProbe<'a> {
    const fn new(program: &'a str, args: &'a [&'a str]) -> Self {
        Self { program, args }
    }

    fn try_read(&self) -> Result<Option<String>> {
        let Some(program_path) = find_executable(self.program) else {
            return Ok(None);
        };
        let output = Command::new(program_path)
            .args(self.args)
            .output()
            .with_context(|| format!("failed to run clipboard probe '{}'", self.program))?;
        if !output.status.success() {
            return Ok(None);
        }
        let text = String::from_utf8_lossy(&output.stdout).to_string();
        if text.trim().is_empty() {
            return Ok(None);
        }
        Ok(Some(text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_external_sessions_ignores_empty_status_lines() {
        let parsed = parse_external_sessions(
            "No external sessions found for runner codex\n\nabcd1234 codex /tmp/demo\n",
        );
        assert_eq!(parsed, BTreeSet::from(["abcd1234".to_string()]));
    }

    #[test]
    fn shell_quote_escapes_single_quotes() {
        assert_eq!(shell_quote("it's fine"), "'it'\"'\"'s fine'");
    }

    #[test]
    fn prompt_resolution_prefers_explicit_prompt() {
        let prompt = resolve_prompt(Some("from cli".to_string())).expect("prompt");
        assert_eq!(prompt.source, "--prompt");
        assert_eq!(prompt.value, "from cli");
    }

    #[test]
    fn preview_prompt_truncates_long_values() {
        let preview = preview_prompt(&"x".repeat(120));
        assert!(preview.ends_with("..."));
        assert!(preview.len() <= 99);
    }
}
