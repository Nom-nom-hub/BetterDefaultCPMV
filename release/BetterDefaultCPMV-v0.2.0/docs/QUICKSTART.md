# Quick Start Guide

## For Users

### Build the Project
```bash
cd BetterDefaultCPMV
cargo build --release
```

### Try It Out
```bash
# Create a test file
echo "Hello World" > test.txt

# Copy with progress
./target/release/better-cp test.txt test_backup.txt

# Show help
./target/release/better-cp --help

# Copy with verification
./target/release/better-cp --verify=full large_file.iso backup/

# Smart overwrite (only if source is newer)
./target/release/better-cp --overwrite=smart source dest
```

---

## For Developers

### Setup Development Environment
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and enter directory
cd BetterDefaultCPMV

# Verify setup
cargo --version  # Should be 1.70+
rustc --version

# Install helpful tools
cargo install cargo-watch
cargo install cargo-clippy
```

### Build and Test
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Watch for changes (auto-rebuild)
cargo watch -x build

# Check code quality
cargo clippy -- -D warnings
cargo fmt --check

# Format code
cargo fmt
```

### Understand the Code
1. Start with **ARCHITECTURE.md** (5 min read)
2. Look at **src/lib.rs** (module exports)
3. Read **DEVELOPMENT.md** for patterns
4. Look at specific modules in `src/`

### Common Development Tasks

#### Add a Dependency
```bash
cargo add <crate-name>
cargo build
```

#### Run with Debug Output
```bash
RUST_LOG=debug cargo run -- source dest
```

#### Run Specific Test
```bash
cargo test test_name -- --nocapture
```

#### Check for Issues
```bash
cargo clippy -- -D warnings
cargo audit  # Check for security issues
cargo fmt -- --check
```

---

## Architecture Quick Reference

### Core Modules

| Module | Purpose | Key Types |
|--------|---------|-----------|
| `cli.rs` | CLI argument parsing | `Cli`, `CopyArgs`, `OverwriteMode` |
| `copy.rs` | Core copy logic | `FileCopier`, transfer in 64MB chunks |
| `progress.rs` | Progress tracking | `ProgressTracker`, `TransferStats` |
| `verify.rs` | Checksum validation | `compute_checksum()`, `verify_checksum()` |
| `resume.rs` | Resume state management | `ResumeState`, `ChunkInfo` |
| `config.rs` | Configuration loading | `Config`, TOML file support |
| `error.rs` | Error definitions | `Error` enum, `Result<T>` |

### How It Works

```
User runs: better-cp source dest
           ↓
       Parse CLI args (cli.rs)
           ↓
       Create FileCopier (copy.rs)
           ↓
       Validate source & target
           ↓
       Open files, start progress bar (progress.rs)
           ↓
       Copy in 64MB chunks
           ↓
       Update progress real-time
           ↓
       Verify checksum (verify.rs)
           ↓
       Done! Show summary
```

---

## Project Structure

```
BetterDefaultCPMV/
├── README.md                    ← Start here (user guide)
├── ARCHITECTURE.md              ← System design
├── DEVELOPMENT.md               ← Developer setup
├── EXECUTIVE_SUMMARY.md         ← High-level overview
├── PHASE_1_STATUS.md            ← Current progress
├── TESTING.md                   ← Test strategy
├── IMPLEMENTATION_PROGRESS.md   ← Detailed tracking
│
├── src/
│   ├── lib.rs                   ← Module exports
│   ├── cli.rs                   ← CLI parsing
│   ├── copy.rs                  ← Copy engine (★ main logic)
│   ├── progress.rs              ← Progress tracking
│   ├── verify.rs                ← Checksum verification
│   ├── resume.rs                ← Resume state
│   ├── config.rs                ← Configuration
│   ├── error.rs                 ← Error types
│   └── bin/
│       ├── better_cp.rs         ← Copy binary
│       └── better_mv.rs         ← Move binary (Phase 2)
│
├── tests/                       ← Integration tests (planned)
├── target/                      ← Build output
├── Cargo.toml                   ← Project configuration
└── Cargo.lock                   ← Dependency lock file
```

---

## Current Capabilities (What Works)

✅ **Working Features**
- Copy single files with progress bar
- Show real-time speed, ETA, percentage
- Verify with SHA-256 checksums
- Overwrite modes: never | always | smart
- Atomic operations (safe from partial writes)
- Preserve file metadata
- Create parent directories
- Handle various file sizes

⚠️ **Not Yet Implemented**
- Interactive prompts (planned Week 1)
- Resume on interruption (planned Week 1)
- Directory copying (needs fix)
- Dry-run mode
- Advanced error recovery

---

## Key Files to Know

### For Understanding Concepts
- **ARCHITECTURE.md**: How everything fits together
- **PHASE_1_STATUS.md**: What's done, what's next

### For Day-to-Day Development
- **src/copy.rs**: The core copy logic (★ start here for features)
- **src/cli.rs**: Command-line argument handling
- **Cargo.toml**: Dependencies and build config

### For Testing & Validation
- **TESTING.md**: How to test your changes
- **DEVELOPMENT.md**: Build and test commands

---

## Useful Commands (Cheat Sheet)

```bash
# Build
cargo build                 # Debug build
cargo build --release       # Optimized build
cargo check                 # Quick syntax check

# Test
cargo test                  # Run all tests
cargo test test_name        # Run specific test
cargo test -- --nocapture   # Show output

# Quality
cargo clippy                # Lint checking
cargo fmt                   # Auto-format code
cargo audit                 # Security check
cargo doc --open            # Generate docs

# Development
cargo watch -x build        # Auto-rebuild
cargo watch -x test         # Auto-test
RUST_LOG=debug cargo run -- args  # Debug logging

# Run
./target/debug/better-cp source dest              # Debug binary
./target/release/better-cp source dest            # Release binary
```

---

## Common Issues & Solutions

### Build Fails
```bash
# Clean and rebuild
cargo clean
cargo build

# Update dependencies
cargo update
cargo build
```

### Test Fails
```bash
# Run with output
cargo test -- --nocapture

# Run single test
cargo test test_name -- --nocapture
```

### Want to Debug
```bash
RUST_LOG=debug cargo run -- source dest
# Shows detailed logging
```

### Code Style Issues
```bash
# Auto-fix formatting
cargo fmt

# See clippy suggestions
cargo clippy
```

---

## What to Work On Next (Prioritized)

### 🔴 Critical (Blocking MVP)
1. **Interactive Prompts** - Let user confirm overwrites
2. **Resume Functionality** - Continue interrupted transfers
3. **Directory Copy** - Fix async recursion
4. **Error Recovery** - Handle disk full, permissions, etc.

### 🟡 Important (Complete MVP)
5. **JSON Output** - Machine-readable results
6. **Logging** - Write operation log to file
7. **Verbose Mode** - Per-file operation display
8. **Tests** - Expand test coverage

### 🟢 Nice to Have (Phase 2)
9. **Reflink** - Instant CoW copies
10. **Parallel** - Multi-threaded directory copy
11. **Sparse** - Skip zero regions
12. **Move** - Implement `better-mv`

---

## Documentation Map

```
Start Here:
├─ README.md (user overview)
└─ EXECUTIVE_SUMMARY.md (this session summary)

Understand the System:
├─ ARCHITECTURE.md (detailed design)
└─ PHASE_1_STATUS.md (what's done)

Start Developing:
├─ DEVELOPMENT.md (setup & workflow)
├─ TESTING.md (how to test)
└─ Look at: src/copy.rs (main logic)

Track Progress:
└─ IMPLEMENTATION_PROGRESS.md (what's planned)

Quick Reference:
└─ This file (QUICKSTART.md)
```

---

## Getting Help

### I want to...

**Understand the architecture**
→ Read ARCHITECTURE.md (30 min)

**Set up development environment**
→ Follow DEVELOPMENT.md

**Write/modify code**
→ Check patterns in DEVELOPMENT.md

**Add tests**
→ See TESTING.md for examples

**Know what to work on**
→ Read PHASE_1_STATUS.md "Next Immediate Actions"

**Understand current state**
→ Read EXECUTIVE_SUMMARY.md

---

## Project Stats

- **Lines of Code**: ~900 (library) + ~65 (binaries)
- **Test Coverage**: 40% (7 passing tests)
- **Documentation**: 1,500+ lines across 8 guides
- **Build Time**: ~5 seconds (debug), ~15 seconds (release)
- **Binary Size**: ~50 MB (debug), ~8 MB (release)

---

## Next Milestone

**Goal**: Interactive prompts + basic resume  
**Effort**: ~10-15 hours  
**Timeline**: 1-2 weeks  
**Success**: Can pause/resume large file copies with user confirmations

---

## Success!

Once you can run this:
```bash
./target/release/better-cp large_file.iso backup/
```

And see:
```
Copying: large_file.iso → backup/
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ 87%
3.4 GB / 3.9 GB | 285 MB/s | ⏱ 2m 15s remaining
✓ Copy completed successfully
```

You've successfully built the first Phase of BetterDefaultCPMV! 🎉

---

## Questions?

- Architecture: See ARCHITECTURE.md
- Development: See DEVELOPMENT.md
- Testing: See TESTING.md
- Status: See PHASE_1_STATUS.md
- Features: See README.md

**Happy coding!** 🚀
