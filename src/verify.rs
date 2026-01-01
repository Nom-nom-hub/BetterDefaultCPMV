use sha2::{Sha256, Digest};
use std::fs::File;
use std::io::{Read, Result as IoResult};
use std::path::Path;

const CHUNK_SIZE: usize = 16 * 1024 * 1024; // 16 MB chunks for hashing

/// Compute SHA-256 checksum of a file
pub fn compute_checksum<P: AsRef<Path>>(path: P) -> IoResult<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; CHUNK_SIZE];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Verify file matches expected checksum
pub fn verify_checksum<P: AsRef<Path>>(path: P, expected: &str) -> IoResult<bool> {
    let actual = compute_checksum(path)?;
    Ok(actual == expected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_checksum_computation() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"hello world").unwrap();
        file.flush().unwrap();

        let checksum = compute_checksum(file.path()).unwrap();
        assert!(!checksum.is_empty());
        assert_eq!(checksum.len(), 64); // SHA-256 hex is 64 chars
    }

    #[test]
    fn test_checksum_verification() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test data").unwrap();
        file.flush().unwrap();

        let checksum = compute_checksum(file.path()).unwrap();
        assert!(verify_checksum(file.path(), &checksum).unwrap());
        assert!(!verify_checksum(file.path(), "wronghash").unwrap());
    }
}
