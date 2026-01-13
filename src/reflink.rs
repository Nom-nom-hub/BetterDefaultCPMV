use std::fs;
use std::path::Path;
use crate::error::{Error, Result};
use crate::cli::ReflinkMode;

/// Attempt to create a copy-on-write reflink of a file
pub fn try_reflink(source: &Path, target: &Path, mode: ReflinkMode) -> Result<bool> {
    match mode {
        ReflinkMode::Never => Ok(false),
        ReflinkMode::Always => reflink_file(source, target).map(|_| true),
        ReflinkMode::Auto => reflink_file(source, target).map(|_| true).or(Ok(false)),
    }
}

/// Try platform-specific reflink operations
#[cfg(target_os = "linux")]
fn reflink_file(source: &Path, target: &Path) -> Result<()> {
    use std::fs::File;
    use std::os::unix::io::AsRawFd;

    let src = File::open(source).map_err(Error::Io)?;
    let dst = File::create(target).map_err(Error::Io)?;

    let src_fd = src.as_raw_fd();
    let dst_fd = dst.as_raw_fd();

    // FICLONE ioctl (Linux 4.5+)
    const FICLONE: u64 = 0x40049409;

    unsafe {
        let result = libc::ioctl(dst_fd, FICLONE, src_fd);
        if result == 0 {
            Ok(())
        } else {
            Err(Error::Custom(
                "Reflink not supported on this filesystem".to_string(),
            ))
        }
    }
}

#[cfg(target_os = "macos")]
fn reflink_file(source: &Path, target: &Path) -> Result<()> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let src_cstr = CString::new(source.as_os_str().as_bytes())
        .map_err(|e| Error::Custom(format!("Invalid source path: {}", e)))?;
    let dst_cstr = CString::new(target.as_os_str().as_bytes())
        .map_err(|e| Error::Custom(format!("Invalid target path: {}", e)))?;

    // clonefile() on macOS
    unsafe {
        let result = libc::clonefile(src_cstr.as_ptr(), dst_cstr.as_ptr(), 0);
        if result == 0 {
            Ok(())
        } else {
            Err(Error::Custom(
                "Reflink not supported on this filesystem".to_string(),
            ))
        }
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn reflink_file(_source: &Path, _target: &Path) -> Result<()> {
    Err(Error::Custom(
        "Reflink not supported on this platform".to_string(),
    ))
}

/// Detect if source and target are on the same filesystem (for optimization)
pub fn same_filesystem(source: &Path, target: &Path) -> Result<bool> {
    let src_metadata = fs::metadata(source).map_err(Error::Io)?;
    let tgt_metadata = fs::metadata(
        target
            .parent()
            .ok_or_else(|| Error::Custom("Invalid target path".to_string()))?,
    ).map_err(Error::Io)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(src_metadata.dev() == tgt_metadata.dev())
    }

    #[cfg(not(unix))]
    {
        // On non-Unix, assume different filesystems
        Ok(false)
    }
}

/// Detect sparse files (files with holes)
pub fn is_sparse_file(path: &Path) -> Result<bool> {
    let metadata = fs::metadata(path).map_err(Error::Io)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        // blocks * 512 < size indicates sparse file
        let block_size = 512u64;
        let allocated = metadata.blocks() * block_size;
        let actual_size = metadata.len();
        Ok(allocated < actual_size)
    }

    #[cfg(not(unix))]
    {
        // Windows doesn't have sparse file concept in the same way
        Ok(false)
    }
}

/// Skip zero regions in a file (sparse file optimization)
pub fn find_zero_regions(path: &Path, chunk_size: usize) -> Result<Vec<(u64, u64)>> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path).map_err(Error::Io)?;
    let file_size = fs::metadata(path).map_err(Error::Io)?.len();

    let mut zero_regions = Vec::new();
    let mut buffer = vec![0; chunk_size];
    let mut offset = 0u64;
    let mut region_start = None;

    loop {
        let bytes_read = file.read(&mut buffer).map_err(Error::Io)?;
        if bytes_read == 0 {
            break;
        }

        let is_all_zeros = buffer[..bytes_read].iter().all(|&b| b == 0);

        if is_all_zeros {
            if region_start.is_none() {
                region_start = Some(offset);
            }
        } else if let Some(start) = region_start.take() {
            zero_regions.push((start, offset - start));
        }

        offset += bytes_read as u64;
    }

    // Close any unclosed zero region
    if let Some(start) = region_start {
        zero_regions.push((start, file_size - start));
    }

    Ok(zero_regions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_same_filesystem() {
        let temp_dir = TempDir::new().unwrap();
        let src_path = temp_dir.path().join("source.txt");
        let tgt_path = temp_dir.path().join("target.txt");

        // Both in temp directory, should be same
        let mut src = File::create(&src_path).unwrap();
        src.write_all(b"content").unwrap();
        drop(src);

        // Since both are in temp dir, they should be on same filesystem
        assert!(same_filesystem(&src_path, &tgt_path).unwrap());
    }

    #[test]
    fn test_sparse_file_detection() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("sparse.bin");

        // Create regular (non-sparse) file
        let mut file = File::create(&path).unwrap();
        file.write_all(&vec![42u8; 1024]).unwrap();
        drop(file);

        // Regular file should not be sparse
        let is_sparse = is_sparse_file(&path).unwrap();
        assert!(!is_sparse);
    }

    #[test]
    fn test_zero_region_detection() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("file_with_zeros.bin");

        // Create file with larger zero regions to detect properly
        let mut file = File::create(&path).unwrap();
        file.write_all(&vec![42u8; 2048]).unwrap();     // Non-zero (2KB)
        file.write_all(&vec![0u8; 2048]).unwrap();      // Zeros (2KB)
        file.write_all(&vec![42u8; 2048]).unwrap();     // Non-zero (2KB)
        drop(file);

        let regions = find_zero_regions(&path, 1024).unwrap();
        assert!(!regions.is_empty());
        assert!(regions.iter().any(|(start, len)| *start == 2048 && *len == 2048),
                "Expected zero region at offset 2048 with length 2048, got: {:?}", regions);
    }
}
