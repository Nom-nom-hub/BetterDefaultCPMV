# Testing Strategy for BetterDefaultCPMV

## Test Levels

### Unit Tests
Located in each module as `#[cfg(test)]` blocks.

**Current Tests**:
- `src/verify.rs`: Checksum computation and verification
- `src/resume.rs`: State file management
- `src/config.rs`: Size parsing and config defaults
- `src/copy.rs`: Single file copy operations

**Run**:
```bash
cargo test --lib
```

### Integration Tests
Full end-to-end workflow tests in `tests/` directory.

**To Create**:
```bash
mkdir -p tests/
# Create tests/integration_tests.rs
```

**Test Scenarios**:
1. Copy small file (10 MB)
2. Copy large file (1 GB+)
3. Copy directory with mixed file sizes
4. Overwrite with different modes
5. Resume interrupted transfer
6. Verify with checksum mismatch
7. Handle permission denied
8. Handle disk full

**Run**:
```bash
cargo test --test '*'
```

### Manual Testing

#### Test Data Setup

```bash
# Create test directory structure
mkdir -p test_data/source test_data/dest

# Create test files
dd if=/dev/zero of=test_data/source/small.bin bs=1M count=10
dd if=/dev/zero of=test_data/source/medium.bin bs=1M count=100
dd if=/dev/zero of=test_data/source/large.bin bs=1M count=1000

# Create text files
echo "Hello World" > test_data/source/hello.txt
for i in {1..1000}; do echo "File $i" > test_data/source/file_$i.txt; done

# Create directories
mkdir -p test_data/source/nested/deep/structure
cp test_data/source/*.txt test_data/source/nested/
```

#### Basic Functionality Tests

```bash
# Test 1: Simple copy
./target/debug/better-cp test_data/source/hello.txt test_data/dest/
# Verify: ls test_data/dest/hello.txt

# Test 2: Copy with progress
./target/debug/better-cp test_data/source/medium.bin test_data/dest/
# Expected: Shows progress bar

# Test 3: Overwrite prompt
./target/debug/better-cp --overwrite=prompt test_data/source/hello.txt test_data/dest/hello.txt
# Expected: Prompts user (when implemented)

# Test 4: Dry run
./target/debug/better-cp --dry-run test_data/source/large.bin test_data/dest/
# Expected: No files copied, shows what would happen

# Test 5: Verify checksums
./target/debug/better-cp --verify=full test_data/source/large.bin test_data/dest/
# Expected: Computes and compares SHA-256
```

#### Edge Case Tests

```bash
# Test: Source doesn't exist
./target/debug/better-cp /nonexistent /dest
# Expected: Error "Source not found"

# Test: Target exists (overwrite=never)
./target/debug/better-cp --overwrite=never test_data/source/hello.txt test_data/dest/hello.txt
# Expected: Error "Target already exists"

# Test: Multiple sources to directory
./target/debug/better-cp test_data/source/file_1.txt test_data/source/file_2.txt test_data/dest/
# Expected: Copies both files

# Test: Directory copy (recursive)
./target/debug/better-cp test_data/source/nested test_data/dest/nested
# Expected: Recursively copies all files and directories
```

## Performance Testing

### Benchmarking Setup

```bash
# Create large test files
dd if=/dev/zero of=/tmp/test_1gb.bin bs=1M count=1024

# Time the operation
time ./target/release/better-cp /tmp/test_1gb.bin /tmp/test_1gb.bin.copy
```

### Expected Performance

| Scenario | Target | Measurement |
|----------|--------|-------------|
| 1 GB file | 280 MB/s | throughput |
| 1000 small files | 120 MB/s | aggregate throughput |
| Progress overhead | <1% | latency increase |

### Profiling

```bash
# CPU flamegraph (Linux)
cargo install flamegraph
cargo flamegraph --release -- test_data/source/large.bin test_data/dest/

# Memory usage
/usr/bin/time -v ./target/release/better-cp source dest
```

## Regression Testing

### Test Script

Create `tests/regression.sh`:

```bash
#!/bin/bash
set -e

BETTER_CP="./target/release/better-cp"

# Compile
cargo build --release

# Create test directory
rm -rf test_regression
mkdir -p test_regression/{source,dest}

# Test 1: Small file
echo "test" > test_regression/source/file.txt
$BETTER_CP test_regression/source/file.txt test_regression/dest/
[ -f test_regression/dest/file.txt ] || exit 1

# Test 2: Overwrite modes
$BETTER_CP --overwrite=always test_regression/source/file.txt test_regression/dest/file.txt || true

# Test 3: Checksum verification
dd if=/dev/zero of=test_regression/source/1mb.bin bs=1M count=1
$BETTER_CP --verify=full test_regression/source/1mb.bin test_regression/dest/

# Cleanup
rm -rf test_regression

echo "✓ All regression tests passed"
```

Run:
```bash
chmod +x tests/regression.sh
./tests/regression.sh
```

## Continuous Integration

### GitHub Actions Workflow

Create `.github/workflows/test.yml`:

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --all
      - run: cargo clippy -- -D warnings
      - run: cargo fmt -- --check
```

## Test Coverage

### Generate Coverage Report

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate HTML report
cargo tarpaulin --out Html --output-dir coverage/

# Generate LCOV report for CI
cargo tarpaulin --out Lcov > coverage.lcov
```

### Coverage Goals

- **Phase 1**: 40% overall coverage
- **Phase 2**: 60% overall coverage
- **Phase 3+**: 75%+ overall coverage

Key areas to cover:
- Error paths (100%)
- Happy path for each operation (100%)
- Edge cases (edge case dependent)

## Test Categories

### Happy Path ✨
- Successful copy
- Successful move (when implemented)
- Progress bar updates
- Completion summary

### Error Cases 🚫
- Source not found
- Target exists (various modes)
- Permission denied
- Disk full
- Checksum mismatch
- Invalid resume state
- Corrupted config

### Edge Cases 🔍
- Empty files
- Very large files (>1 TB)
- Many small files (>100K)
- Symlinks
- Special files (devices, sockets)
- Paths with spaces/unicode
- Concurrent operations
- Interrupted operations

### Platform-Specific 🖥️
- **Linux**: ext4, Btrfs, XFS behaviors
- **macOS**: HFS+ sparse files, reflink
- **Windows**: NTFS permissions, path handling

## Debugging Failed Tests

### Enable Logging

```bash
RUST_LOG=debug cargo test test_name -- --nocapture
```

### Use Debugger

```bash
# GDB (Linux/macOS)
rust-gdb ./target/debug/better-cp

# LLDB (macOS)
lldb ./target/debug/better-cp

# WinDbg (Windows)
# Use Visual Studio or WinDbg standalone
```

### Inspect Intermediate State

```rust
#[test]
fn test_with_inspection() {
    let state = ResumeState::new(...);
    dbg!(&state);  // Print state
    assert_eq!(state.bytes_completed(), 0);
}
```

## Test Data Files

### Managed Test Data

Location: `test_data/`

```
test_data/
├── source/
│   ├── empty.txt           # 0 bytes
│   ├── small.txt           # 1 KB
│   ├── medium.bin          # 100 MB
│   ├── large.bin           # 1 GB (generated on demand)
│   └── nested/
│       └── deep/           # Directory structure
└── dest/                   # Target directory for copies
```

### Generate Large Test Files

Script: `tests/generate_test_data.sh`

```bash
#!/bin/bash
mkdir -p test_data/source

# Create files of various sizes
dd if=/dev/zero of=test_data/source/medium.bin bs=1M count=100
dd if=/dev/zero of=test_data/source/large.bin bs=1M count=1024

echo "Created test data in test_data/"
```

## Acceptance Criteria

### For Phase 1 Release
- [x] Project builds without errors
- [ ] All unit tests pass
- [ ] Integration tests cover 5+ scenarios
- [ ] Manual testing successful on Linux
- [ ] No critical clippy warnings
- [ ] Code formatted with rustfmt

### For Production Release
- [ ] 75%+ test coverage
- [ ] Performance benchmarks established
- [ ] Cross-platform testing (Linux, macOS, Windows)
- [ ] Stress tests with large files
- [ ] Concurrent operation tests
- [ ] Load/stress testing with thousands of small files

## Resources

- Rust Testing: https://doc.rust-lang.org/book/ch11-00-testing.html
- Cargo Test: https://doc.rust-lang.org/cargo/commands/cargo-test.html
- Criterion (benchmarking): https://bheisler.github.io/criterion.rs/book/
- Proptest (property testing): https://docs.rs/proptest/
