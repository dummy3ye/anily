# Development Guide

## Prerequisites

- [Rust](https://rustup.rs/) 1.85 or later
- [mpv](https://mpv.io/) for testing playback
- A terminal emulator (Kitty recommended for image rendering tests)

## Getting Started

```bash
# Clone the repository
git clone https://github.com/dummy3ye/anily.git
cd anily

# Build and run
cargo run --release

# Or just build
cargo build --release
```

## Project Structure

```text
anily/
├── src/
│   ├── app/          # Application logic and state
│   │   ├── mod.rs    # App struct and orchestration
│   │   └── state.rs  # AppState and UI state
│   ├── providers/    # External API clients
│   │   ├── anilist.rs
│   │   └── hianime.rs
│   ├── rpc/          # Discord Rich Presence
│   │   └── discord.rs
│   ├── ui/           # Terminal UI rendering
│   ├── config.rs     # Configuration loading
│   ├── error.rs      # Error types
│   ├── lib.rs        # Library exports
│   └── main.rs       # Binary entry point
├── docs/             # Documentation
├── assets/           # Screenshots and media
└── Cargo.toml        # Dependencies
```

## Code Quality

CI enforces the following checks. Run them locally before submitting:

```bash
# Format check
cargo fmt --all -- --check

# Linting
cargo clippy --all-targets --all-features -- -D warnings

# Tests
cargo test --all-targets
```

To auto-format:

```bash
cargo fmt --all
```

## Nix Development

If you use Nix with flakes:

```bash
# Enter development shell
nix develop

# Run all checks
nix flake check

# Build
nix build
```

The development shell includes Rust toolchain, mpv, and other dependencies.

## Adding Features

### Adding a New Screen

1. Add screen variant to `Screen` enum in `src/app/state.rs`
2. Add state fields to `AppState` as needed
3. Handle key events in `AppState::handle_key()` or `App::handle_key()`
4. Add rendering logic in `src/ui/`
5. Update navigation logic in `App`

### Adding a New Provider

1. Create a new module in `src/providers/`
2. Define async functions returning `Result<T>` with appropriate types
3. Add to `src/providers/mod.rs`
4. Integrate in `App` or as needed

### Modifying Configuration

1. Update `Config` struct in `src/config.rs`
2. Update `docs/configuration.md`
3. Handle loading/usage in appropriate places

## Debugging

### Logging

The app doesn't have built-in file logging. For debugging:
- Use `eprintln!` temporarily (will go to stderr, visible in some terminals)
- Check mpv output by removing `--really-quiet` flags in `play_selected()` 
- Inspect network requests by adding debug prints in provider code

### Common Issues

- **Cover art not showing**: Check terminal support, network connectivity to AniList CDN
- **mpv not starting**: Verify `mpv` is in PATH, check process spawning
- **Discord not connecting**: Ensure Discord desktop is running, check Application ID format

## Release Process

Releases are built by CI for Linux, macOS (Intel/ARM), and Windows. See `.github/workflows/` for details. Changelog is generated using `git-cliff` (see `cliff.toml`).
