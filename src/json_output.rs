use serde::{Serialize, Deserialize};
use std::path::PathBuf;

/// Structured JSON output for copy operations
#[derive(Serialize, Deserialize, Debug)]
pub struct OperationResult {
    /// Overall success/failure
    pub success: bool,
    /// Operation type (copy, move, etc)
    pub operation: String,
    /// Source file(s) or directory
    pub source: Vec<PathBuf>,
    /// Destination file or directory
    pub destination: PathBuf,
    /// Detailed summary
    pub summary: OperationSummary,
    /// Any error message if operation failed
    pub error: Option<String>,
}

/// Summary statistics for operation
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct OperationSummary {
    /// Total bytes transferred
    pub bytes_transferred: u64,
    /// Total files copied
    pub files_copied: usize,
    /// Total directories created
    pub directories_created: usize,
    /// Skipped count (user chose not to overwrite)
    pub files_skipped: usize,
    /// Duration in seconds
    pub duration_secs: f64,
    /// Transfer speed in MB/s
    pub speed_mbps: f64,
    /// Whether resume was used
    pub resumed: bool,
    /// Whether checksums were verified
    pub verified: bool,
}

impl OperationResult {
    /// Create successful copy result
    pub fn success(
        source: Vec<PathBuf>,
        destination: PathBuf,
        summary: OperationSummary,
    ) -> Self {
        Self {
            success: true,
            operation: "copy".to_string(),
            source,
            destination,
            summary,
            error: None,
        }
    }

    /// Create failed result with error message
    pub fn failure(
        source: Vec<PathBuf>,
        destination: PathBuf,
        error_msg: String,
    ) -> Self {
        Self {
            success: false,
            operation: "copy".to_string(),
            source,
            destination,
            summary: OperationSummary::default(),
            error: Some(error_msg),
        }
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| {
            serde_json::to_string(self).unwrap_or_default()
        })
    }

    /// Serialize to compact JSON (single line)
    pub fn to_json_compact(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_output_success() {
        let result = OperationResult::success(
            vec![PathBuf::from("/tmp/source.txt")],
            PathBuf::from("/tmp/dest.txt"),
            OperationSummary {
                bytes_transferred: 1024,
                files_copied: 1,
                directories_created: 0,
                files_skipped: 0,
                duration_secs: 1.5,
                speed_mbps: 0.68,
                resumed: false,
                verified: true,
            },
        );

        assert!(result.success);
        let json = result.to_json();
        assert!(json.contains("\"success\": true"));
        assert!(json.contains("\"bytes_transferred\": 1024"));
    }

    #[test]
    fn test_json_output_failure() {
        let result = OperationResult::failure(
            vec![PathBuf::from("/tmp/source.txt")],
            PathBuf::from("/tmp/dest.txt"),
            "File not found".to_string(),
        );

        assert!(!result.success);
        assert!(result.error.is_some());
        let json = result.to_json();
        assert!(json.contains("\"success\": false"));
        assert!(json.contains("File not found"));
    }
}
