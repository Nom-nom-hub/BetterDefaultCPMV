use std::fs::{self, File};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::error::{Error, Result};
use crate::progress::ProgressTracker;
use crate::verify::compute_checksum;

const CHUNK_SIZE: usize = 64 * 1024 * 1024; // 64 MB chunks

/// Parallel file copier - splits large files across multiple threads
pub struct ParallelFileCopier {
    source: PathBuf,
    target: PathBuf,
    parallel_threads: usize,
    verify: bool,
}

impl ParallelFileCopier {
    pub fn new(
        source: PathBuf,
        target: PathBuf,
        parallel_threads: usize,
        verify: bool,
    ) -> Self {
        Self {
            source,
            target,
            parallel_threads,
            verify,
        }
    }

    /// Execute parallel copy
    pub async fn copy(&self) -> Result<()> {
        let src_metadata = fs::metadata(&self.source)
            .map_err(|_| Error::SourceNotFound(self.source.to_string_lossy().to_string()))?;

        if !src_metadata.is_file() {
            return Err(Error::Custom("Source is not a file".to_string()));
        }

        let total_size = src_metadata.len();

        // For small files, fall back to single-threaded copy
        if total_size < CHUNK_SIZE as u64 {
            return self.sequential_copy(total_size).await;
        }

        // Use parallel copy for large files
        self.parallel_copy(total_size).await?;

        // Verify if requested
        if self.verify {
            self.verify_copy()?;
        }

        Ok(())
    }

    /// Sequential copy for small files
    async fn sequential_copy(&self, total_size: u64) -> Result<()> {
        let tracker = ProgressTracker::new(total_size, true);

        let mut src_file = File::open(&self.source).map_err(Error::Io)?;
        let mut dst_file = File::create(&self.target).map_err(Error::Io)?;

        let mut buffer = vec![0; CHUNK_SIZE];
        loop {
            let bytes_read = src_file.read(&mut buffer).map_err(Error::Io)?;
            if bytes_read == 0 {
                break;
            }

            dst_file.write_all(&buffer[..bytes_read]).map_err(Error::Io)?;
            tracker.add_bytes(bytes_read as u64);
        }

        tracker.finish();
        Ok(())
    }

    /// Parallel copy for large files
    async fn parallel_copy(&self, total_size: u64) -> Result<()> {
        let tracker = Arc::new(ProgressTracker::new(total_size, true));

        // Pre-allocate destination file
        let dst_file = File::create(&self.target).map_err(Error::Io)?;
        dst_file
            .set_len(total_size)
            .map_err(Error::Io)?;

        // Calculate chunk boundaries
        let num_chunks = (total_size as usize + CHUNK_SIZE - 1) / CHUNK_SIZE;
        let actual_threads = std::cmp::min(self.parallel_threads, num_chunks);

        let src_path = Arc::new(self.source.clone());
        let dst_path = Arc::new(self.target.clone());

        let mut handles = Vec::new();

        for thread_id in 0..actual_threads {
            let src = Arc::clone(&src_path);
            let dst = Arc::clone(&dst_path);
            let tracker = Arc::clone(&tracker);

            let handle = tokio::spawn(async move {
                Self::copy_chunk(
                    &src,
                    &dst,
                    thread_id,
                    num_chunks,
                    actual_threads,
                    tracker,
                )
                .await
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.await.map_err(|e| Error::Custom(e.to_string()))?
                .map_err(|e| Error::Custom(format!("Thread error: {}", e)))?;
        }

        tracker.finish();
        Ok(())
    }

    /// Copy a chunk of file (used by parallel threads)
    async fn copy_chunk(
        src_path: &Path,
        dst_path: &Path,
        thread_id: usize,
        total_chunks: usize,
        num_threads: usize,
        tracker: Arc<ProgressTracker>,
    ) -> Result<()> {
        let chunk_size = CHUNK_SIZE;

        // Calculate which chunks this thread should handle
        for chunk_idx in (thread_id..total_chunks).step_by(num_threads) {
            let offset = (chunk_idx * chunk_size) as u64;

            tokio::task::block_in_place(|| {
                let mut src = File::open(src_path).map_err(Error::Io)?;
                src.seek(SeekFrom::Start(offset)).map_err(Error::Io)?;

                let mut dst = File::open(dst_path).map_err(Error::Io)?;
                dst.seek(SeekFrom::Start(offset)).map_err(Error::Io)?;

                let mut buffer = vec![0; chunk_size];
                let bytes_read = src.read(&mut buffer).map_err(Error::Io)?;

                if bytes_read > 0 {
                    dst.write_all(&buffer[..bytes_read])
                        .map_err(Error::Io)?;
                    tracker.add_bytes(bytes_read as u64);
                }

                Ok::<(), Error>(())
            })?;
        }

        Ok(())
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

/// Parallel directory copy - copies multiple files in parallel
pub async fn parallel_copy_directory(
    source: &Path,
    target: &Path,
    parallel_threads: usize,
) -> Result<()> {
    if !source.is_dir() {
        return Err(Error::Custom("Source is not a directory".to_string()));
    }

    // Create target directory
    fs::create_dir_all(target).map_err(Error::Io)?;

    // Collect all files to copy
    let mut files_to_copy = Vec::new();
    collect_files_recursive(source, target, &mut files_to_copy)?;

    if files_to_copy.is_empty() {
        return Ok(());
    }

    let tracker = Arc::new(ProgressTracker::new(
        files_to_copy.iter().map(|(_, _, size)| size).sum(),
        true,
    ));

    // Split work among threads
    let chunk_size = (files_to_copy.len() + parallel_threads - 1) / parallel_threads;
    let mut handles = Vec::new();

    for thread_idx in 0..parallel_threads {
        let start = thread_idx * chunk_size;
        let end = std::cmp::min(start + chunk_size, files_to_copy.len());

        if start >= files_to_copy.len() {
            break;
        }

        let files = files_to_copy[start..end].to_vec();
        let tracker = Arc::clone(&tracker);

        let handle = tokio::spawn(async move {
            for (src, dst, _size) in files {
                let mut src_file = tokio::task::block_in_place(|| File::open(&src))
                    .map_err(Error::Io)?;
                let mut dst_file =
                    tokio::task::block_in_place(|| File::create(&dst)).map_err(Error::Io)?;

                let mut buffer = vec![0; CHUNK_SIZE];
                loop {
                    let bytes_read =
                        tokio::task::block_in_place(|| src_file.read(&mut buffer))
                            .map_err(Error::Io)?;

                    if bytes_read == 0 {
                        break;
                    }

                    tokio::task::block_in_place(|| dst_file.write_all(&buffer[..bytes_read]))
                        .map_err(Error::Io)?;
                    tracker.add_bytes(bytes_read as u64);
                }
            }

            Ok::<(), Error>(())
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.await.map_err(|e| Error::Custom(e.to_string()))?
            .map_err(|e| Error::Custom(format!("Thread error: {}", e)))?;
    }

    tracker.finish();
    Ok(())
}

/// Collect all files to copy (recursive)
fn collect_files_recursive(
    source: &Path,
    target: &Path,
    files: &mut Vec<(PathBuf, PathBuf, u64)>,
) -> Result<()> {
    for entry in fs::read_dir(source).map_err(Error::Io)? {
        let entry = entry.map_err(Error::Io)?;
        let path = entry.path();
        let file_name = entry.file_name();
        let target_path = target.join(&file_name);

        if path.is_dir() {
            fs::create_dir_all(&target_path).map_err(Error::Io)?;
            collect_files_recursive(&path, &target_path, files)?;
        } else {
            let metadata = entry.metadata().map_err(Error::Io)?;
            files.push((path, target_path, metadata.len()));
        }
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
    async fn test_parallel_copy_small_file() {
        let temp_dir = TempDir::new().unwrap();
        let src_path = temp_dir.path().join("source.txt");
        let dst_path = temp_dir.path().join("dest.txt");

        // Create small source file
        let mut src = File::create(&src_path).unwrap();
        src.write_all(b"small content").unwrap();
        drop(src);

        let copier = ParallelFileCopier::new(src_path, dst_path.clone(), 4, false);
        assert!(copier.copy().await.is_ok());

        // Verify copy
        assert_eq!(
            fs::read(&dst_path).unwrap(),
            b"small content"
        );
    }

    #[test]
    fn test_parallel_copy_file_sequential() {
        // Test sequential copy path (for small files)
        let temp_dir = TempDir::new().unwrap();
        let src_path = temp_dir.path().join("source.bin");
        let dst_path = temp_dir.path().join("dest.bin");

        // Create small source file
        let mut src = File::create(&src_path).unwrap();
        src.write_all(&vec![42u8; 1024 * 10]).unwrap(); // 10 KB
        drop(src);

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let copier = ParallelFileCopier::new(src_path.clone(), dst_path.clone(), 4, false);
            assert!(copier.copy().await.is_ok());
        });

        // Verify sizes match
        let src_size = fs::metadata(&src_path).unwrap().len();
        let dst_size = fs::metadata(&dst_path).unwrap().len();
        assert_eq!(src_size, dst_size);
    }
}
