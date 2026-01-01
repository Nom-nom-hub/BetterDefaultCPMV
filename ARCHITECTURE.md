# BetterDefaultCPMV Architecture

## High-Level Design

BetterDefaultCPMV is architected as a modular Rust application with clear separation of concerns:

```
┌─────────────────────────────────────────────────┐
│              CLI Layer (clap)                   │
│  Parse arguments, validate inputs               │
└────────────────────┬────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────┐
│           Operation Coordinator                │
│  - Handle copy/move dispatch                    │
│  - Manage error handling                        │
└────────────────────┬────────────────────────────┘
                     │
      ┌──────────────┼──────────────┐
      │              │              │
      ▼              ▼              ▼
  ┌────────┐  ┌──────────┐  ┌──────────────┐
  │Transfer│  │Progress  │  │Resume        │
  │Engine  │  │Reporter  │  │Manager       │
  └────────┘  └──────────┘  └──────────────┘
      │              │              │
      └──────────────┼──────────────┘
                     │
      ┌──────────────┼──────────────┐
      │              │              │
      ▼              ▼              ▼
  ┌────────┐  ┌──────────┐  ┌──────────────┐
  │Checksum│  │Config    │  │Error         │
  │Verify  │  │Manager   │  │Handler       │
  └────────┘  └──────────┘  └──────────────┘
```

## Core Modules

### 1. CLI Module (`src/cli.rs`)

**Responsibility**: Command-line argument parsing and validation

**Key Structures**:
- `Cli`: Top-level CLI parser
- `Commands`: Enum for copy/move subcommands
- `CopyArgs`: Arguments specific to copy operation
- `MoveArgs`: Arguments specific to move operation
- `OverwriteMode`: Enum for overwrite strategies
- `VerifyMode`: Enum for verification strategies
- `ReflinkMode`: Enum for reflink behavior

**Technology**: Uses `clap` derive macros for robust, auto-documented CLI.

### 2. Error Module (`src/error.rs`)

**Responsibility**: Centralized error definitions and handling

**Error Types**:
- `Io`: IO operations (files not found, permission denied)
- `SourceNotFound`: Source path doesn't exist
- `TargetExists`: Target exists (when overwrite=never)
- `PermissionDenied`: Insufficient permissions
- `ChecksumMismatch`: Verification failed
- `InvalidResumeState`: Corrupted resume metadata
- `UserAborted`: User cancelled operation
- `DiskFull`: Insufficient disk space
- `ConfigError`: Configuration file issues

**Type Alias**: `Result<T> = std::result::Result<T, Error>`

### 3. Copy Module (`src/copy.rs`)

**Responsibility**: Core file transfer logic

**Key Structures**:
- `FileCopier`: Single file copy with all options
  - `new()`: Constructor
  - `copy()`: Async copy operation
  - `perform_copy()`: Actual transfer with chunking
  - `handle_overwrite()`: Overwrite decision logic
  - `verify_copy()`: Checksum comparison

**Transfer Flow**:
1. Validate source exists and is a file
2. Check if target exists; handle per overwrite mode
3. Create parent directories
4. Open source and destination files
5. Copy in chunks (64 MB default) with progress tracking
6. Close files
7. If atomic: rename temp file to target
8. If verify: compute and compare SHA-256 checksums
9. Finish progress bar

**Key Features**:
- Chunked I/O for memory efficiency
- Real-time progress tracking
- Atomic operations via temp-then-rename pattern
- Post-transfer verification

### 4. Progress Module (`src/progress.rs`)

**Responsibility**: Real-time progress tracking and reporting

**Key Structures**:
- `ProgressTracker`: Thread-safe progress state
  - `new()`: Create tracker for total size
  - `add_bytes()`: Update progress
  - `finish()`: Finalize and display completion
  - `get_stats()`: Retrieve current statistics
- `TransferStats`: Computed statistics
  - `percent_complete()`: 0-100
  - `speed_human()`: Human-readable speed (e.g., "285 MB/s")
  - `transferred_human()`: Human-readable transferred size
  - `total_human()`: Human-readable total size

**Implementation**:
- Uses `indicatif` for progress bars
- Thread-safe via `Arc<Mutex<...>>`
- Automatic ETA calculation based on current speed
- Real-time updates as bytes are transferred

### 5. Resume Module (`src/resume.rs`)

**Responsibility**: Persist and restore transfer state for resumable copies

**Key Structures**:
- `ResumeState`: Complete transfer metadata
  - `source`: Source file path
  - `target`: Target file path
  - `total_size`: Total bytes to transfer
  - `chunks_completed`: Array of completed chunks
  - `timestamp`: When state was last updated
  - `version`: State format version
- `ChunkInfo`: Per-chunk metadata
  - `offset`: Byte offset in file
  - `length`: Chunk size in bytes
  - `checksum`: Optional SHA-256 for this chunk

**State File Format**: JSON stored at `target.better-cp.state`

**Operations**:
- `new()`: Initialize fresh state
- `save()`: Persist to disk
- `load()`: Restore from disk
- `cleanup()`: Remove state file after completion
- `mark_chunk_done()`: Record completed chunk
- `bytes_completed()`: Query total progress
- `validate()`: Check state coherence (no gaps/overlaps)

### 6. Verify Module (`src/verify.rs`)

**Responsibility**: Checksum computation and verification

**Functions**:
- `compute_checksum()`: Calculate SHA-256 for file
  - Streams file in 16 MB chunks
  - Returns hex-encoded hash string
- `verify_checksum()`: Compare computed vs expected
  - Returns boolean: match or mismatch

**Algorithm**: SHA-256 (secure hash, 256-bit output)

**Streaming**: Processes files in chunks to handle large files without loading into memory

### 7. Config Module (`src/config.rs`)

**Responsibility**: Configuration file management

**Key Structures**:
- `Config`: Root configuration object
- `Defaults`: Default behavior settings
- `Behavior`: Policy settings (follow symlinks, preserve times, etc.)
- `Performance`: Tuning parameters (buffer size, chunk size)
- `UiConfig`: Display preferences (colors, progress style)

**File Location**:
- Linux/macOS: `~/.config/better-cp/config.toml`
- Windows: `%APPDATA%\better-cp\config.toml`
- Fallback: Built-in defaults

**Loading Precedence**:
1. Environment variable `XDG_CONFIG_HOME` (if set)
2. `~/.config/better-cp/config.toml` (standard XDG location)
3. Built-in defaults

**Size Parsing**: Handles human-readable sizes (64M, 1G, etc.)

## Data Flow Diagram

### Single File Copy Flow

```
[User runs: better-cp source.txt target.txt]
                    │
                    ▼
        ┌─────────────────────┐
        │  Parse CLI Arguments│
        └──────────┬──────────┘
                   │
                   ▼
        ┌─────────────────────┐
        │  Load Configuration │
        └──────────┬──────────┘
                   │
                   ▼
        ┌─────────────────────┐
        │ Create FileCopier   │
        │ - source            │
        │ - target            │
        │ - options           │
        └──────────┬──────────┘
                   │
                   ▼
        ┌─────────────────────┐
        │ Validate Source     │
        └──────────┬──────────┘
                   │
                   ▼
        ┌─────────────────────┐
        │ Target Exists?      │
        └──────────┬──────────┘
                   │
         ┌─────────┴──────────┐
         │ Yes                │ No
         ▼                    ▼
      Handle Overwrite   Create Parent Dirs
         │                    │
         └─────────┬──────────┘
                   │
                   ▼
        ┌─────────────────────┐
        │ Open Source & Dest  │
        │ Create Progress Bar │
        └──────────┬──────────┘
                   │
                   ▼
        ┌─────────────────────┐
        │ Read 64MB Chunk     │
        │ Write Chunk         │
        │ Update Progress     │
        │ (Repeat Until EOF)  │
        └──────────┬──────────┘
                   │
                   ▼
        ┌─────────────────────┐
        │ Atomic Mode?        │
        └──────────┬──────────┘
                   │
         ┌─────────┴──────────┐
         │ Yes                │ No
         ▼                    ▼
       Rename Temp ──────────► Done
         │
         └──────────┬──────────┘
                    │
                    ▼
        ┌─────────────────────┐
        │ Verify Mode?        │
        └──────────┬──────────┘
                   │
         ┌─────────┴──────────┐
         │ Yes                │ No
         ▼                    ▼
      Compute Checksums   Finish Progress
         │                    │
         ▼                    │
      Compare ────────────────┤
         │                    │
         ├─ Match  ───────────┼──► Success
         │
         └─ Mismatch ────────► Error
```

## Concurrency Model

- **Progress Tracking**: Thread-safe via `Arc<Mutex<ProgressTrackerInner>>`
- **Async I/O**: tokio-based async/await for non-blocking operations
- **Parallel Copy**: Future support for parallel threads reading/writing simultaneously
- **File Operations**: Standard blocking I/O (efficient for most filesystems)

## Error Handling Strategy

1. **Validation Phase**: Catch errors early (source not found, invalid paths)
2. **Transfer Phase**: Graceful degradation (if check fails, quarantine and error)
3. **Cleanup Phase**: Attempt cleanup even on errors
4. **User Feedback**: Clear error messages with remediation suggestions

## Performance Considerations

### Memory Usage
- Constant 64 MB buffer (configurable)
- Progress state is O(1) in memory
- Resume state scales with number of chunks (~1 KB per 100 MB transferred)

### Disk I/O
- Sequential reads (source) and writes (target)
- No seeks or random I/O patterns
- Cache-friendly: 64 MB chunks align with typical page cache

### CPU Usage
- Minimal: mostly I/O bound
- SHA-256 uses hardware acceleration (if available)
- Progress updates batched to avoid excessive locking

## Future Enhancements

### Phase 2 Features
- **Reflink**: Instant CoW copies on supporting filesystems
- **Parallel I/O**: Multiple threads reading/writing simultaneously
- **Sparse Files**: Skip zero regions
- **Parallel Directory Copy**: Fan-out for many small files

### Phase 3+ Features
- **Cloud Storage**: S3, GCS, Azure Blob with same UX
- **Network Optimization**: Adaptive window sizing for remote filesystems
- **Deduplication**: Content-aware copying
- **Bandwidth Throttling**: Rate limiting for background transfers

## Testing Strategy

- **Unit Tests**: Per-module in `#[cfg(test)]` blocks
- **Integration Tests**: Full workflow tests
- **Benchmarks**: Performance regression tracking
- **Stress Tests**: Large files, parallel operations, edge cases
