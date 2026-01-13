use std::fs::{self, File};
use std::path::{Path, PathBuf};
use crate::error::{Error, Result};
use crate::progress::ProgressTracker;
use crate::cli::OverwriteMode;
use crate::prompt::{self, OverwriteChoice};

/// Move a file or directory with progress tracking
pub struct FileMover {
    source: PathBuf,
    target: PathBuf,
    overwrite_mode: OverwriteMode,
    verbose: bool,
}

impl FileMover {
    pub fn new(
        source: PathBuf,
        target: PathBuf,
        overwrite_mode: OverwriteMode,
        verbose: bool,
    ) -> Self {
        Self {
            source,
            target,
            overwrite_mode,
            verbose,
        }
    }

    /// Execute the move operation
    pub async fn move_file(&self) -> Result<()> {
        // Validate source exists
        if !self.source.exists() {
            return Err(Error::SourceNotFound(
                self.source.to_string_lossy().to_string(),
            ));
        }

        let src_metadata = fs::metadata(&self.source).map_err(Error::Io)?;

        // Check if target exists and handle overwrite logic
        if self.target.exists() {
            self.handle_overwrite(&src_metadata)?;
        }

        // Create parent directories if needed
        if let Some(parent) = self.target.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| Error::Custom(format!("Failed to create parent directory: {}", e)))?;
        }

        // Try simple rename first (same filesystem)
        match fs::rename(&self.source, &self.target) {
            Ok(_) => {
                if self.verbose {
                    println!("Moved: {} → {}", self.source.display(), self.target.display());
                }
                return Ok(());
            }
            Err(e) if e.kind() == std::io::ErrorKind::InvalidInput
                || e.kind() == std::io::ErrorKind::PermissionDenied => {
                // Cross-filesystem move: copy then delete
                if self.verbose {
                    println!(
                        "Cross-filesystem move (copy+delete): {} → {}",
                        self.source.display(),
                        self.target.display()
                    );
                }
                self.move_via_copy().await?;
            }
            Err(e) => return Err(Error::Io(e)),
        }

        Ok(())
    }

    /// Move via copy and delete (for cross-filesystem moves)
    async fn move_via_copy(&self) -> Result<()> {
        let src_metadata = fs::metadata(&self.source).map_err(Error::Io)?;
        let total_size = src_metadata.len();

        let tracker = ProgressTracker::new(total_size, true);

        // Copy file in chunks
        let mut src_file =
            File::open(&self.source).map_err(Error::Io)?;
        let mut dst_file =
            File::create(&self.target).map_err(Error::Io)?;

        const CHUNK_SIZE: usize = 64 * 1024 * 1024; // 64 MB
        let mut buffer = vec![0; CHUNK_SIZE];

        use std::io::{Read, Write};

        loop {
            let bytes_read = src_file.read(&mut buffer).map_err(Error::Io)?;

            if bytes_read == 0 {
                break;
            }

            dst_file.write_all(&buffer[..bytes_read]).map_err(Error::Io)?;
            tracker.add_bytes(bytes_read as u64);
        }

        drop(src_file);
        drop(dst_file);
        tracker.finish();

        // Delete source after successful copy
        if src_metadata.is_dir() {
            fs::remove_dir_all(&self.source).map_err(Error::Io)?;
        } else {
            fs::remove_file(&self.source).map_err(Error::Io)?;
        }

        Ok(())
    }

    fn handle_overwrite(&self, src_metadata: &fs::Metadata) -> Result<()> {
        match self.overwrite_mode {
            OverwriteMode::Never => {
                Err(Error::TargetExists(
                    self.target.to_string_lossy().to_string(),
                ))
            }
            OverwriteMode::Always => Ok(()),
            OverwriteMode::Prompt => {
                let tgt_metadata = fs::metadata(&self.target).map_err(Error::Io)?;

                match prompt::prompt_overwrite(&self.target, src_metadata, &tgt_metadata)? {
                    OverwriteChoice::Overwrite => Ok(()),
                    OverwriteChoice::Skip => {
                        Err(Error::Custom("Skipped by user".to_string()))
                    }
                    OverwriteChoice::Rename => {
                        Err(Error::Custom("Rename not yet implemented".to_string()))
                    }
                    OverwriteChoice::Abort => Err(Error::UserAborted),
                }
            }
            OverwriteMode::Smart => {
                let tgt_metadata = fs::metadata(&self.target).map_err(Error::Io)?;

                let src_modified = src_metadata.modified().map_err(Error::Io)?;
                let tgt_modified = tgt_metadata.modified().map_err(Error::Io)?;

                if src_modified > tgt_modified {
                    Ok(())
                } else {
                    Err(Error::TargetExists(
                        self.target.to_string_lossy().to_string(),
                    ))
                }
            }
        }
    }
}

/// Move a directory recursively
pub async fn move_directory(
    source: &Path,
    target: &Path,
    overwrite_mode: OverwriteMode,
    verbose: bool,
) -> Result<()> {
    if !source.is_dir() {
        return Err(Error::Custom("Source is not a directory".to_string()));
    }

    // Try simple rename first
    match fs::rename(source, target) {
        Ok(_) => {
            if verbose {
                println!("Moved directory: {} → {}", source.display(), target.display());
            }
            return Ok(());
        }
        Err(e) if e.kind() == std::io::ErrorKind::InvalidInput
            || e.kind() == std::io::ErrorKind::PermissionDenied => {
            // Cross-filesystem move: copy directory then delete
            if verbose {
                println!(
                    "Cross-filesystem move (copy+delete): {} → {}",
                    source.display(),
                    target.display()
                );
            }
            move_directory_via_copy(source, target, overwrite_mode, verbose).await?;
        }
        Err(e) => return Err(Error::Io(e)),
    }

    Ok(())
}

/// Move directory via copy and delete (cross-filesystem)
async fn move_directory_via_copy(
    source: &Path,
    target: &Path,
    overwrite_mode: OverwriteMode,
    verbose: bool,
) -> Result<()> {
    use crate::copy::copy_directory;

    // Copy entire directory
    copy_directory(source, target, overwrite_mode, false).await?;

    // Delete source directory
    fs::remove_dir_all(source).map_err(Error::Io)?;

    if verbose {
        println!("Moved directory: {} → {}", source.display(), target.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_move() {
        let temp_dir = TempDir::new().unwrap();
        let src_path = temp_dir.path().join("source.txt");
        let dst_path = temp_dir.path().join("dest.txt");

        // Create source file
        let mut src = File::create(&src_path).unwrap();
        src.write_all(b"test content").unwrap();
        drop(src);

        let mover = FileMover::new(src_path.clone(), dst_path.clone(), OverwriteMode::Always, false);
        assert!(mover.move_file().await.is_ok());

        // Source should be gone
        assert!(!src_path.exists());

        // Destination should exist with correct content
        assert!(dst_path.exists());
        let content = fs::read(&dst_path).unwrap();
        assert_eq!(content, b"test content");
    }

    #[tokio::test]
    async fn test_directory_move() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("source_dir");
        let dst_dir = temp_dir.path().join("dest_dir");

        // Create source directory with file
        fs::create_dir(&src_dir).unwrap();
        let mut src_file = File::create(src_dir.join("file.txt")).unwrap();
        src_file.write_all(b"content").unwrap();
        drop(src_file);

        let result = move_directory(&src_dir, &dst_dir, OverwriteMode::Always, false).await;
        assert!(result.is_ok());

        // Source should be gone
        assert!(!src_dir.exists());

        // Destination should exist with file
        assert!(dst_dir.exists());
        assert_eq!(fs::read(dst_dir.join("file.txt")).unwrap(), b"content");
    }
}
