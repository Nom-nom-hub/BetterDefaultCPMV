use console::style;

/// Output level for logging operations
#[derive(Debug, Clone, Copy)]
pub enum OutputLevel {
    /// Minimal output (quiet mode)
    Quiet,
    /// Normal output (default)
    Normal,
    /// Verbose output (per-file operations)
    Verbose,
}

/// Output manager for handling verbose/quiet modes
pub struct OutputManager {
    level: OutputLevel,
}

impl OutputManager {
    /// Create new output manager
    pub fn new(quiet: bool, verbose: bool) -> Self {
        let level = if quiet {
            OutputLevel::Quiet
        } else if verbose {
            OutputLevel::Verbose
        } else {
            OutputLevel::Normal
        };

        Self { level }
    }

    /// Log a file operation (only shown in verbose mode)
    pub fn file_operation(&self, source: &std::path::Path, target: &std::path::Path, action: &str) {
        if matches!(self.level, OutputLevel::Verbose) {
            println!(
                "  {} {} → {}",
                action,
                source.display(),
                target.display()
            );
        }
    }

    /// Log directory operation (only shown in verbose mode)
    pub fn dir_operation(&self, source: &std::path::Path, action: &str) {
        if matches!(self.level, OutputLevel::Verbose) {
            println!("  {} directory: {}", action, source.display());
        }
    }

    /// Log a status message (shown unless quiet)
    pub fn status(&self, msg: &str) {
        if !matches!(self.level, OutputLevel::Quiet) {
            println!("  {}", style(msg).cyan());
        }
    }

    /// Log a success message (shown unless quiet)
    pub fn success(&self, msg: &str) {
        if !matches!(self.level, OutputLevel::Quiet) {
            println!("  {} {}", style("✓").green(), msg);
        }
    }

    /// Log a warning message (always shown)
    pub fn warning(&self, msg: &str) {
        eprintln!("  {} {}", style("⚠️").yellow(), msg);
    }

    /// Log summary (shown unless quiet)
    pub fn summary(&self, msg: &str) {
        if !matches!(self.level, OutputLevel::Quiet) {
            println!("{}", msg);
        }
    }

    /// Check if in verbose mode
    pub fn is_verbose(&self) -> bool {
        matches!(self.level, OutputLevel::Verbose)
    }

    /// Check if in quiet mode
    pub fn is_quiet(&self) -> bool {
        matches!(self.level, OutputLevel::Quiet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_levels() {
        let quiet = OutputManager::new(true, false);
        assert!(quiet.is_quiet());

        let verbose = OutputManager::new(false, true);
        assert!(verbose.is_verbose());

        let normal = OutputManager::new(false, false);
        assert!(!normal.is_quiet());
        assert!(!normal.is_verbose());
    }

    #[test]
    fn test_output_level_priority() {
        // quiet takes priority over verbose
        let both = OutputManager::new(true, true);
        assert!(both.is_quiet());
    }
}
