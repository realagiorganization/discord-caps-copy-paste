use anyhow::Result;
use clap::Parser;
use discord_caps_copy_paste::{
    AppConfig, DEFAULT_DISCOVERY_POLL_MS, DEFAULT_DISCOVERY_TIMEOUT_MS, DEFAULT_PLATFORM,
    DEFAULT_TITLE, LaunchResult,
};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    /// Prompt to send to the new Codex session. Falls back to DCCP_PROMPT or the clipboard.
    #[arg(long)]
    prompt: Option<String>,

    /// Working directory for the launched Codex session.
    #[arg(long, default_value = ".", env = "DCCP_CWD")]
    cwd: PathBuf,

    /// Pin one terminal instead of selecting a random installed terminal.
    #[arg(long, env = "DCCP_TERMINAL")]
    terminal: Option<String>,

    /// Comma-separated terminal allowlist used during random selection.
    #[arg(long, value_delimiter = ',', env = "DCCP_TERMINAL_CANDIDATES")]
    terminal_candidates: Vec<String>,

    /// Codex binary path.
    #[arg(long, default_value = "codex", env = "DCCP_CODEX_BIN")]
    codex_bin: String,

    /// Tether binary path.
    #[arg(long, default_value = "tether", env = "DCCP_TETHER_BIN")]
    tether_bin: String,

    /// Attach platform passed to tether attach -p.
    #[arg(long, default_value = DEFAULT_PLATFORM, env = "DCCP_PLATFORM")]
    platform: String,

    /// Window title for the launched terminal.
    #[arg(long, default_value = DEFAULT_TITLE, env = "DCCP_TITLE")]
    title: String,

    /// Optional deterministic RNG seed for repeatable terminal selection.
    #[arg(long, env = "DCCP_RANDOM_SEED")]
    random_seed: Option<u64>,

    /// Wait budget for Tether external-session discovery.
    #[arg(long, default_value_t = DEFAULT_DISCOVERY_TIMEOUT_MS, env = "DCCP_DISCOVERY_TIMEOUT_MS")]
    discovery_timeout_ms: u64,

    /// Poll interval while waiting for a new external Codex session.
    #[arg(long, default_value_t = DEFAULT_DISCOVERY_POLL_MS, env = "DCCP_DISCOVERY_POLL_MS")]
    discovery_poll_ms: u64,

    /// Skip tether start if tether status is unhealthy.
    #[arg(long, env = "DCCP_SKIP_TETHER_START", default_value_t = false)]
    skip_tether_start: bool,

    /// Print the launch plan without starting anything.
    #[arg(long, env = "DCCP_DRY_RUN", default_value_t = false)]
    dry_run: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = AppConfig {
        prompt: cli.prompt,
        cwd: cli.cwd,
        terminal: cli.terminal,
        terminal_candidates: cli.terminal_candidates,
        codex_bin: cli.codex_bin,
        tether_bin: cli.tether_bin,
        platform: cli.platform,
        title: cli.title,
        random_seed: cli.random_seed,
        discovery_timeout_ms: cli.discovery_timeout_ms,
        discovery_poll_ms: cli.discovery_poll_ms,
        skip_tether_start: cli.skip_tether_start,
        dry_run: cli.dry_run,
    };

    let result = discord_caps_copy_paste::run(&config)?;
    print_result(result);
    Ok(())
}

fn print_result(result: LaunchResult) {
    println!("terminal={}", result.terminal);
    if let Some(session_id) = result.session_id {
        println!("session_id={session_id}");
    } else {
        println!("session_id=<dry-run>");
    }
    println!("prompt_preview={}", result.prompt_preview);
}
