use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "better-cp")]
#[command(about = "Modern cp/mv with progress, safety, and resume", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Copy files with progress and safety
    Copy(CopyArgs),
    /// Move files with progress and safety
    Move(MoveArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct CopyArgs {
    /// Source file or directory (one or more)
    #[arg(required = true)]
    pub source: Vec<PathBuf>,

    /// Destination file or directory
    #[arg(required = true)]
    pub destination: PathBuf,

    /// Overwrite behavior: never|prompt|always|smart
    #[arg(long, value_name = "MODE", default_value = "prompt")]
    pub overwrite: OverwriteMode,

    /// Resume interrupted transfers
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub resume: bool,

    /// Do not resume interrupted transfers
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub no_resume: bool,

    /// Verify checksums after transfer
    #[arg(long, value_name = "MODE", default_value = "fast")]
    pub verify: VerifyMode,

    /// Skip checksum verification
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub no_verify: bool,

    /// Use atomic operations
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub atomic: bool,

    /// Number of parallel threads (0 = auto)
    #[arg(long, value_name = "N", default_value = "0")]
    pub parallel: usize,

    /// Internal buffer size (e.g. 64M, 1G)
    #[arg(long, value_name = "SIZE", default_value = "64M")]
    pub buffer: String,

    /// Enable sparse file detection
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub sparse: bool,

    /// Reflink strategy: auto|always|never
    #[arg(long, value_name = "MODE", default_value = "auto")]
    pub reflink: ReflinkMode,

    /// Show what would happen without executing
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub dry_run: bool,

    /// Verbose output (per-file operations)
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub verbose: bool,

    /// Minimal output
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub quiet: bool,

    /// Machine-readable JSON output
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub json: bool,

    /// Write operation log to file
    #[arg(long, value_name = "FILE")]
    pub log: Option<PathBuf>,

    /// Follow symlinks
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub follow_symlinks: bool,

    /// Preserve timestamps
    #[arg(long, action = clap::ArgAction::SetTrue, default_value_t = true)]
    pub preserve_times: bool,

    /// Exclude files matching pattern
    #[arg(long, value_name = "PATTERN")]
    pub exclude: Vec<String>,

    /// Interactive mode (prompt for each action)
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub interactive: bool,
}

#[derive(Parser, Debug, Clone)]
pub struct MoveArgs {
    /// Source file or directory
    pub source: Vec<PathBuf>,

    /// Destination file or directory
    pub destination: PathBuf,

    /// Overwrite behavior: never|prompt|always|smart
    #[arg(long, value_name = "MODE", default_value = "prompt")]
    pub overwrite: OverwriteMode,

    /// Number of parallel threads (0 = auto)
    #[arg(long, value_name = "N", default_value = "0")]
    pub parallel: usize,

    /// Show what would happen without executing
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub dry_run: bool,

    /// Verbose output
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub verbose: bool,

    /// Minimal output
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub quiet: bool,

    /// Machine-readable JSON output
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub json: bool,

    /// Write operation log to file
    #[arg(long, value_name = "FILE")]
    pub log: Option<PathBuf>,

    /// Interactive mode
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub interactive: bool,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OverwriteMode {
    /// Fail if target exists
    #[value(name = "never")]
    Never,
    /// Prompt before overwriting
    #[value(name = "prompt")]
    Prompt,
    /// Always overwrite
    #[value(name = "always")]
    Always,
    /// Overwrite if source is newer
    #[value(name = "smart")]
    Smart,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum VerifyMode {
    /// No verification
    #[value(name = "none")]
    None,
    /// Verify per-chunk (fast)
    #[value(name = "fast")]
    Fast,
    /// Verify full file (slow but thorough)
    #[value(name = "full")]
    Full,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ReflinkMode {
    /// Try reflink, fall back to copy
    #[value(name = "auto")]
    Auto,
    /// Always use reflink (fail if unavailable)
    #[value(name = "always")]
    Always,
    /// Never use reflink
    #[value(name = "never")]
    Never,
}
