use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Source not found: {0}")]
    SourceNotFound(String),

    #[error("Target already exists: {0}")]
    TargetExists(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("Resume state invalid or corrupted")]
    InvalidResumeState,

    #[error("Operation aborted by user")]
    UserAborted,

    #[error("Insufficient disk space")]
    DiskFull,

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("{0}")]
    Custom(String),
}

impl Error {
    /// Get a detailed message with recovery suggestions
    pub fn detailed_message(&self) -> String {
        match self {
            Error::Io(e) => {
                match e.kind() {
                    io::ErrorKind::NotFound => {
                        "File or directory not found.\n\
                         Tip: Check that the path exists and is accessible.".to_string()
                    }
                    io::ErrorKind::PermissionDenied => {
                        "Permission denied.\n\
                         Tip: You may need elevated privileges to access this file or directory.\n\
                         Try running with administrator/sudo privileges.".to_string()
                    }
                    io::ErrorKind::OutOfMemory => {
                        "Out of memory.\n\
                         Tip: Try copying smaller files or freeing up system memory.".to_string()
                    }
                    _ => format!("IO error: {}", e),
                }
            }
            Error::SourceNotFound(path) => {
                format!(
                    "Source not found: {}\n\
                     Tip: Check that the source file/directory exists and the path is correct.",
                    path
                )
            }
            Error::TargetExists(path) => {
                format!(
                    "Target already exists: {}\n\
                     Tip: Use --overwrite=always to replace, --overwrite=smart to replace if source is newer,\n\
                     or --overwrite=prompt to choose each time.",
                    path
                )
            }
            Error::PermissionDenied(path) => {
                format!(
                    "Permission denied: {}\n\
                     Tip: Try running with elevated privileges (sudo/administrator mode).\n\
                     You may not have permission to read or write this file.",
                    path
                )
            }
            Error::ChecksumMismatch { expected, actual } => {
                format!(
                    "Checksum mismatch!\n\
                     Expected: {}\n\
                     Actual:   {}\n\
                     Tip: This indicates file corruption during transfer.\n\
                     Try copying again with --resume flag or check disk integrity.",
                    expected, actual
                )
            }
            Error::InvalidResumeState => {
                "Resume state is invalid or corrupted.\n\
                 Tip: The operation will restart from the beginning.\n\
                 Use --no-resume to force a fresh copy without using saved state."
                    .to_string()
            }
            Error::UserAborted => {
                "Operation cancelled by user.".to_string()
            }
            Error::DiskFull => {
                "Insufficient disk space.\n\
                 Tip: Free up space on the destination disk and try again.\n\
                 Use --resume if you've already copied part of the file."
                    .to_string()
            }
            Error::ConfigError(msg) => {
                format!(
                    "Configuration error: {}\n\
                     Tip: Check your config file at ~/.config/better-cp/config.toml (Linux/macOS)\n\
                     or %APPDATA%\\better-cp\\config.toml (Windows)",
                    msg
                )
            }
            Error::Custom(msg) => msg.clone(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
