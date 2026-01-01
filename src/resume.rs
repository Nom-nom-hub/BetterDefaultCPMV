use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use crate::error::{Error, Result};

/// Resume state for interrupted transfers
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResumeState {
    pub source: PathBuf,
    pub target: PathBuf,
    pub total_size: u64,
    pub chunks_completed: Vec<ChunkInfo>,
    pub timestamp: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChunkInfo {
    pub offset: u64,
    pub length: u64,
    pub checksum: Option<String>,
}

impl ResumeState {
    pub fn new(source: PathBuf, target: PathBuf, total_size: u64) -> Self {
        Self {
            source,
            target,
            total_size,
            chunks_completed: Vec::new(),
            timestamp: get_timestamp(),
            version: "1.0".to_string(),
        }
    }

    /// Get state file path (stored alongside target with .better-cp.state suffix)
    pub fn state_file_path(target: &Path) -> PathBuf {
        let mut state_path = target.to_path_buf();
        let filename = format!(
            "{}.better-cp.state",
            target
                .file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_default()
        );
        state_path.set_file_name(filename);
        state_path
    }

    /// Save state to disk
    pub fn save(&self) -> Result<()> {
        let state_file = Self::state_file_path(&self.target);
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| Error::Custom(format!("Failed to serialize state: {}", e)))?;
        fs::write(&state_file, json)
            .map_err(Error::Io)?;
        Ok(())
    }

    /// Load state from disk
    pub fn load(target: &Path) -> Result<Option<ResumeState>> {
        let state_file = Self::state_file_path(target);
        if !state_file.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(&state_file)
            .map_err(Error::Io)?;
        let state: ResumeState = serde_json::from_str(&json)
            .map_err(|_| Error::InvalidResumeState)?;
        Ok(Some(state))
    }

    /// Delete state file
    pub fn cleanup(&self) -> Result<()> {
        let state_file = Self::state_file_path(&self.target);
        if state_file.exists() {
            fs::remove_file(state_file).map_err(Error::Io)?;
        }
        Ok(())
    }

    /// Add a completed chunk
    pub fn mark_chunk_done(&mut self, offset: u64, length: u64, checksum: Option<String>) {
        self.chunks_completed.push(ChunkInfo {
            offset,
            length,
            checksum,
        });
    }

    /// Get total bytes completed
    pub fn bytes_completed(&self) -> u64 {
        self.chunks_completed.iter().map(|c| c.length).sum()
    }

    /// Validate resume state is coherent
    pub fn validate(&self) -> Result<()> {
        let mut sorted_chunks = self.chunks_completed.clone();
        sorted_chunks.sort_by_key(|c| c.offset);

        // Check no overlaps and no gaps
        let mut expected_offset = 0;
        for chunk in &sorted_chunks {
            if chunk.offset != expected_offset {
                return Err(Error::InvalidResumeState);
            }
            expected_offset = chunk.offset + chunk.length;
        }

        Ok(())
    }
}

// Simple timestamp generation
fn get_timestamp() -> String {
    // Simplified ISO 8601 format
    // In production, use chrono crate: chrono::Local::now().to_rfc3339()
    "2025-01-01T12:00:00Z".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_state_file_path() {
        let target = PathBuf::from("/backup/file.iso");
        let state_path = ResumeState::state_file_path(&target);
        assert!(state_path.to_string_lossy().contains(".better-cp.state"));
    }

    #[test]
    fn test_resume_state_creation() {
        let state = ResumeState::new(
            PathBuf::from("/src/file.iso"),
            PathBuf::from("/dst/file.iso"),
            1024 * 1024 * 1024,
        );
        assert_eq!(state.total_size, 1024 * 1024 * 1024);
        assert_eq!(state.bytes_completed(), 0);
    }
}
