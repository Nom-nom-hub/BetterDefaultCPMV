use console::style;
use std::fs::Metadata;
use std::path::Path;
#[allow(unused_imports)]
use std::io::{self, Write};
use crate::error::Result;
use chrono::{DateTime, Local};

/// User prompt for overwrite confirmation
#[derive(Debug, Clone, Copy)]
pub enum OverwriteChoice {
    Overwrite,
    Skip,
    Rename,
    Abort,
}

/// Prompt user before overwriting a file
pub fn prompt_overwrite(
    _target: &Path,
    src_metadata: &Metadata,
    tgt_metadata: &Metadata,
) -> Result<OverwriteChoice> {
    println!("\n{}", style("⚠️  Target exists").yellow().bold());
    
    // Show file details
    println!("{}", format_file_details("Source", src_metadata));
    println!("{}", format_file_details("Target", tgt_metadata));
    
    // Show confirmation dialog
    println!();
    let options = "[o]verwrite  [s]kip  [r]ename  [a]bort";
    
    // For automation testing, default to skip
    #[cfg(test)]
    return {
        println!("{} {}: skip (test default)", style("Your choice").cyan(), options);
        Ok(OverwriteChoice::Skip)
    };
    
    // Get user input (non-test code)
    #[cfg(not(test))]
    loop {
        print!("{} {}: ", style("Your choice").cyan(), options);
        io::stdout().flush().ok();
        
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return Ok(OverwriteChoice::Abort);
        }
        
        match input.trim().to_lowercase().chars().next() {
            Some('o') => return Ok(OverwriteChoice::Overwrite),
            Some('s') => return Ok(OverwriteChoice::Skip),
            Some('r') => return Ok(OverwriteChoice::Rename),
            Some('a') => return Ok(OverwriteChoice::Abort),
            _ => {
                println!("  Invalid choice. Please enter o, s, r, or a.");
            }
        }
    }
}

/// Prompt user before resuming an interrupted transfer
pub fn prompt_resume(
    source: &Path,
    target: &Path,
    total_size: u64,
    bytes_completed: u64,
) -> Result<bool> {
    let percent = (bytes_completed as f64 / total_size as f64 * 100.0) as u32;
    
    println!("\n{}", style("📋 Incomplete transfer found:").yellow().bold());
    println!("  Source: {} ({} bytes)", source.display(), total_size);
    println!("  Target: {} ({} bytes, {}% complete)", 
        target.display(), 
        bytes_completed, 
        percent
    );
    
    // For automation testing
    #[cfg(test)]
    return {
        println!("{} [y/n]: n (test default)", style("Resume from").cyan());
        Ok(false)
    };
    
    // Get user input (non-test code)
    #[cfg(not(test))]
    loop {
        print!("{} [y/n]: ", style("Resume from").cyan());
        io::stdout().flush().ok();
        
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return Ok(false);
        }
        
        match input.trim().to_lowercase().chars().next() {
            Some('y') => return Ok(true),
            Some('n') => return Ok(false),
            _ => {
                println!("  Please enter y or n.");
            }
        }
    }
}

/// Prompt for confirmation before destructive operation
pub fn confirm_action(message: &str) -> Result<bool> {
    // For automation testing
    #[cfg(test)]
    return {
        println!("{}: n (test default)", style(message).cyan());
        Ok(false)
    };
    
    // Get user input (non-test code)
    #[cfg(not(test))]
    loop {
        print!("{} [y/n]: ", style(message).cyan());
        io::stdout().flush().ok();
        
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return Ok(false);
        }
        
        match input.trim().to_lowercase().chars().next() {
            Some('y') => return Ok(true),
            Some('n') => return Ok(false),
            _ => {
                println!("  Please enter y or n.");
            }
        }
    }
}

/// Format file details for display
fn format_file_details(label: &str, metadata: &Metadata) -> String {
    let size = humansize::format_size(metadata.len(), humansize::BINARY);
    
    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| {
            let duration = t
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?;
            let datetime = DateTime::<Local>::from(
                std::time::SystemTime::UNIX_EPOCH + duration
            );
            Some(datetime.format("%Y-%m-%d %H:%M:%S").to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());
    
    format!(
        "  {}: size {}, modified {}",
        style(label).bold(),
        size,
        modified
    )
}

/// Show dry-run preview without executing
pub fn preview_operation(
    source: &Path,
    target: &Path,
    total_size: u64,
    target_exists: bool,
) {
    println!("\n{}", style("📋 Dry Run Preview").cyan().bold());
    println!("  Source: {}", source.display());
    println!("  Target: {}", target.display());
    println!("  Size: {}", humansize::format_size(total_size, humansize::BINARY));
    
    if target_exists {
        println!("  Action: {} (file already exists)", style("overwrite").red());
    } else {
        println!("  Action: {} (new file)", style("copy").green());
    }
    
    println!("\n{}", style("No files were modified (--dry-run)").green());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_details() {
        // Test that format_file_details doesn't panic
        use std::fs;
        use tempfile::NamedTempFile;
        
        let temp = NamedTempFile::new().unwrap();
        let path = temp.path();
        let metadata = fs::metadata(path).unwrap();
        
        let result = format_file_details("Test", &metadata);
        assert!(result.contains("Test"));
        assert!(result.contains("modified"));
    }
}
