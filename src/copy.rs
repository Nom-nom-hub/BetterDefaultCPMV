use std::fs::{self, File, Metadata};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use crate::error::{Error, Result};
use crate::progress::ProgressTracker;
use crate::verify::compute_checksum;
use crate::cli::OverwriteMode;
use crate::prompt::{self, OverwriteChoice};
use crate::resume::ResumeState;

const CHUNK_SIZE: usize = 64 * 1024 * 1024; // 64 MB chunks

/// Copy a single file with progress tracking and resume support
pub struct FileCopier {
    source: PathBuf,
    target: PathBuf,
    overwrite_mode: OverwriteMode,
    verify: bool,
    resume: bool,
    atomic: bool,
}

impl FileCopier {
    pub fn new(
        source: PathBuf,
        target: PathBuf,
        overwrite_mode: OverwriteMode,
        verify: bool,
        resume: bool,
        atomic: bool,
    ) -> Self {
        Self {
            source,
            target,
            overwrite_mode,
            verify,
            resume,
            atomic,
        }
    }

    /// Execute the copy operation
    pub async fn copy(&self) -> Result<()> {
        // Validate source exists
        let src_metadata = fs::metadata(&self.source)
            .map_err(|_| Error::SourceNotFound(self.source.to_string_lossy().to_string()))?;

        if !src_metadata.is_file() {
            return Err(Error::Custom("Source is not a file".to_string()));
        }

        let total_size = src_metadata.len();

        // Check for existing resume state
        let mut resume_state = if self.resume {
            ResumeState::load(&self.target)?
        } else {
            None
        };

        // Handle resume validation
        if let Some(ref state) = resume_state {
            // Validate state is still valid
            state.validate()?;
            
            // If resume state exists, ask user if they want to resume
            let bytes_done = state.bytes_completed();
            if bytes_done > 0 && bytes_done < total_size {
                let should_resume = prompt::prompt_resume(
                    &self.source,
                    &self.target,
                    total_size,
                    bytes_done,
                )?;
                
                if !should_resume {
                    // User chose not to resume, start fresh
                    resume_state = None;
                    if self.target.exists() {
                        fs::remove_file(&self.target)
                            .map_err(Error::Io)?;
                    }
                }
            }
        }

        // Check if target exists and handle overwrite logic
        if self.target.exists() && resume_state.is_none() {
            self.handle_overwrite(&src_metadata)?;
        }

        // Create parent directories if needed
        if let Some(parent) = self.target.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| Error::Custom(format!("Failed to create parent directory: {}", e)))?;
        }

        // Perform the copy
        self.perform_copy(&src_metadata, resume_state).await?;

        Ok(())
    }

    async fn perform_copy(&self, src_metadata: &Metadata, mut resume_state: Option<ResumeState>) -> Result<()> {
        let total_size = src_metadata.len();
        let tracker = ProgressTracker::new(total_size, true);

        // Use temporary file if atomic mode
        let write_target = if self.atomic {
            let mut temp_path = self.target.clone();
            temp_path.set_extension("tmp");
            temp_path
        } else {
            self.target.clone()
        };

        // Open files
        let mut src_file = File::open(&self.source)
            .map_err(Error::Io)?;
        
        // For resume, open in append mode; otherwise create/truncate
        let mut dst_file = if resume_state.is_some() {
            File::options()
                .write(true)
                .create(true)
                .truncate(false)
                .open(&write_target)
                .map_err(Error::Io)?
        } else {
            File::create(&write_target)
                .map_err(Error::Io)?
        };

        // If resuming, seek to the last completed position
        let mut current_offset: u64 = 0;
        if let Some(ref state) = resume_state {
            let bytes_done = state.bytes_completed();
            src_file.seek(SeekFrom::Start(bytes_done))
                .map_err(Error::Io)?;
            dst_file.seek(SeekFrom::End(0))
                .map_err(Error::Io)?;
            current_offset = bytes_done;
            
            // Report already-transferred bytes
            tracker.add_bytes(bytes_done);
        }

        // Create or update resume state
        if resume_state.is_none() {
            resume_state = Some(ResumeState::new(
                self.source.clone(),
                self.target.clone(),
                total_size,
            ));
        }

        // Copy in chunks
        let mut buffer = vec![0; CHUNK_SIZE];
        loop {
            let bytes_read = src_file.read(&mut buffer)
                .map_err(Error::Io)?;

            if bytes_read == 0 {
                break;
            }

            dst_file.write_all(&buffer[..bytes_read])
                .map_err(Error::Io)?;

            tracker.add_bytes(bytes_read as u64);
            current_offset += bytes_read as u64;

            // Update resume state periodically (every 100MB)
            if let Some(ref mut state) = resume_state {
                if current_offset.is_multiple_of(100 * 1024 * 1024) || bytes_read == 0 {
                    state.mark_chunk_done(
                        current_offset.saturating_sub(bytes_read as u64),
                        bytes_read as u64,
                        None, // Skip per-chunk checksums for speed
                    );
                    state.save().ok(); // Best effort save, don't fail if it fails
                }
            }
        }

        drop(src_file);
        drop(dst_file);

        // If atomic, rename temp file to target
        if self.atomic {
            fs::rename(&write_target, &self.target)
                .map_err(Error::Io)?;
        }

        // Verify checksum if requested
        if self.verify {
            self.verify_copy()?;
        }

        // Clean up resume state on success
        if let Some(ref state) = resume_state {
            state.cleanup().ok();
        }

        tracker.finish();
        Ok(())
    }

    fn handle_overwrite(&self, src_metadata: &Metadata) -> Result<()> {
        match self.overwrite_mode {
            OverwriteMode::Never => {
                Err(Error::TargetExists(self.target.to_string_lossy().to_string()))
            }
            OverwriteMode::Always => Ok(()),
            OverwriteMode::Prompt => {
                let tgt_metadata = fs::metadata(&self.target)
                    .map_err(Error::Io)?;
                
                match prompt::prompt_overwrite(&self.target, src_metadata, &tgt_metadata)? {
                    OverwriteChoice::Overwrite => Ok(()),
                    OverwriteChoice::Skip => Err(Error::Custom("Skipped by user".to_string())),
                    OverwriteChoice::Rename => Err(Error::Custom("Rename not yet implemented".to_string())),
                    OverwriteChoice::Abort => Err(Error::UserAborted),
                }
            }
            OverwriteMode::Smart => {
                let tgt_metadata = fs::metadata(&self.target)
                    .map_err(Error::Io)?;
                
                let src_modified = src_metadata.modified()
                    .map_err(Error::Io)?;
                let tgt_modified = tgt_metadata.modified()
                    .map_err(Error::Io)?;

                if src_modified > tgt_modified {
                    Ok(())
                } else {
                    Err(Error::TargetExists(self.target.to_string_lossy().to_string()))
                }
            }
        }
    }

    fn verify_copy(&self) -> Result<()> {
        let src_checksum = compute_checksum(&self.source)
            .map_err(Error::Io)?;
        let tgt_checksum = compute_checksum(&self.target)
            .map_err(Error::Io)?;

        if src_checksum == tgt_checksum {
            Ok(())
        } else {
            Err(Error::ChecksumMismatch {
                expected: src_checksum,
                actual: tgt_checksum,
            })
        }
    }
}

/// Copy a directory recursively (async version with proper boxing for recursion)
pub async fn copy_directory(
    source: &Path,
    target: &Path,
    overwrite_mode: OverwriteMode,
    verify: bool,
) -> Result<()> {
    copy_directory_impl(source, target, overwrite_mode, verify).await
}

/// Internal async implementation using a helper to allow recursion
async fn copy_directory_impl(
    source: &Path,
    target: &Path,
    overwrite_mode: OverwriteMode,
    verify: bool,
) -> Result<()> {
    if !source.is_dir() {
        return Err(Error::Custom("Source is not a directory".to_string()));
    }

    // Create target directory
    fs::create_dir_all(target)
        .map_err(Error::Io)?;

    // Walk source directory
    for entry in fs::read_dir(source)
        .map_err(Error::Io)?
    {
        let entry = entry.map_err(Error::Io)?;
        let path = entry.path();
        let file_name = entry.file_name();
        let target_path = target.join(&file_name);

        if path.is_dir() {
            // Use Box::pin to allow recursion without requiring infinite-sized future
            Box::pin(copy_directory_impl(&path, &target_path, overwrite_mode.clone(), verify)).await?;
        } else {
            let copier = FileCopier::new(
                path,
                target_path,
                overwrite_mode.clone(),
                verify,
                false,
                false,
            );
            copier.copy().await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    #[tokio::test]
    async fn test_file_copy() {
        let mut src = NamedTempFile::new().unwrap();
        src.write_all(b"test content").unwrap();
        src.flush().unwrap();

        let dst = NamedTempFile::new().unwrap();
        let dst_path = dst.path().to_path_buf();
        drop(dst);

        let copier = FileCopier::new(
            src.path().to_path_buf(),
            dst_path.clone(),
            OverwriteMode::Always,
            false,
            false,
            false,
        );

        assert!(copier.copy().await.is_ok());
        let content = fs::read(&dst_path).unwrap();
        assert_eq!(content, b"test content");
    }

    #[tokio::test]
    async fn test_directory_copy() {
        // Create source directory with files
        let src_dir = TempDir::new().unwrap();
        let src_path = src_dir.path();
        
        // Create test files
        let mut file1 = fs::File::create(src_path.join("file1.txt")).unwrap();
        file1.write_all(b"content1").unwrap();
        
        let mut file2 = fs::File::create(src_path.join("file2.txt")).unwrap();
        file2.write_all(b"content2").unwrap();
        
        // Create subdirectory with file
        fs::create_dir(src_path.join("subdir")).unwrap();
        let mut file3 = fs::File::create(src_path.join("subdir/file3.txt")).unwrap();
        file3.write_all(b"content3").unwrap();
        
        // Create destination directory
        let dst_dir = TempDir::new().unwrap();
        let dst_path = dst_dir.path().join("copy");
        
        // Copy directory
        let result = copy_directory(src_path, &dst_path, OverwriteMode::Always, false).await;
        assert!(result.is_ok(), "Directory copy failed: {:?}", result.err());
        
        // Verify files were copied
        assert_eq!(fs::read(dst_path.join("file1.txt")).unwrap(), b"content1");
        assert_eq!(fs::read(dst_path.join("file2.txt")).unwrap(), b"content2");
        assert_eq!(fs::read(dst_path.join("subdir/file3.txt")).unwrap(), b"content3");
    }
}
