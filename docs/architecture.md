# Architecture

Anily is a terminal user interface (TUI) application written in Rust that combines anime metadata from AniList with streaming from a playback provider and local playback via `mpv`.

## High-Level Flow

```text
User → TUI (ratatui) → App State → Providers → External Services
                            ↓
                         mpv (playback)
                         Discord IPC (optional)
```

## Core Components

### Application Layer (`src/app/`)

- **`App`** (`mod.rs`) - Main application orchestrator. Manages background tasks, mpv process lifecycle, Discord RPC, cover art loading, and key handling.
- **`AppState`** (`state.rs`) - Central state container for UI navigation, current screen, search queries, anime lists, episode lists, and playback status.

### UI Layer (`src/ui/`)

Renders the terminal interface using [ratatui](https://github.com/ratatui/ratatui). Handles all screens (home, details/episodes) with keyboard navigation and optional image rendering.

### Providers (`src/providers/`)

- **`anilist.rs`** - AniList GraphQL API client. Handles fetching trending, airing, popular, favorites, search results, anime details, and downloading/resizing cover art.
- **`hianime.rs`** - Playback provider integration. Searches for anime matches, retrieves episode lists, and resolves stream URLs and subtitles.

### discord rpc (`src/rpc/`)

- **`discord.rs`** - Discord Rich Presence integration via local IPC. Updates presence when playback starts and clears it when playback ends or the app exits.

### Configuration (`src/config.rs`)

Loads configuration from `~/.config/anily/config.toml` (or `$XDG_CONFIG_HOME/anily/config.toml`). Falls back to defaults if the file doesn't exist.

### Error Handling (`src/error.rs`)

Custom error types using `thiserror`.

## Data Flow

1. **Startup**: `App::new()` initializes HTTP client, terminal image picker, spawns home feed fetch task, and optionally connects to Discord.
2. **Home Feed**: Background task fetches from AniList (trending, airing, popular, favorites). Results are stored in `AppState` and rendered.
3. **Cover Art**: As items become visible, cover art is lazily fetched and cached. With Kitty-compatible terminals, images are rendered using `ratatui-image`.
4. **Anime Selection**: When user selects an anime, `load_episodes()` fetches full details from AniList and searches for matches on the playback provider.
5. **Playback**: When an episode is selected, `play_selected()` resolves the stream URL via the provider, spawns `mpv` as a background process, and updates Discord presence.
6. **Cleanup**: On exit or when switching away, `mpv` is killed and Discord presence is cleared.

## Async Model

- Uses `tokio` with full features for async runtime
- Background tasks for network requests and cover art fetching
- Non-blocking UI with `poll_background()` checking task completion each frame
- `mpv` runs as a separate child process (not managed by tokio)

## Terminal Image Support

Cover art rendering depends on terminal capabilities:
- Uses `ratatui-image` with `crossterm` backend
- `Picker::from_query_stdio()` auto-detects terminal capabilities (Kitty protocol, iTerm2, Sixel, etc.)
- Falls back to `Picker::halfblocks()` if auto-detection fails
- Images are cached by `(anime_id, width, height)` key

## Playback Integration

- Spawns `mpv` with `--really-quiet --no-terminal`, null stdin/stdout/stderr
- Passes `--referrer` header for stream requests
- Optionally passes `--sub-file` for subtitles
- Tracks process with `std::process::Child` and polls for completion
- On drop, kills any running `mpv` process
