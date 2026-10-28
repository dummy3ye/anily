# Anily

> A fast, keyboard-driven anime browser and player for the terminal.

Anily combines AniList metadata with a playback provider and `mpv` to provide
anime discovery, rich details, episode selection, and playback without leaving
the terminal. It is written in Rust and works in regular terminals, with
enhanced cover-art rendering in Kitty-compatible terminals.

home screenshot

## Features

- AniList-powered trending, airing, popular, favorite, search, and detail data.
- High-resolution portrait cover art with lazy loading.
- Kitty/native terminal image rendering with a text fallback.
- Keyboard-first home dashboard with multiple recommendation sections.
- Anime details including titles, synopsis, genres, score, status, duration, and
  episode count.
- Episode list navigation with sub/dub selection.
- `mpv` playback with subtitles and a visible `Now playing...` indicator.
- Optional Discord Rich Presence through the local Discord desktop IPC.
- Nix flake for reproducible development and builds.
- Linux, macOS, and Windows CI and release artifacts.

anime details and episodes list screenshot.

## Requirements

### Runtime

- Rust 1.85+ if building from source.
- `mpv` for episode playback.
- A terminal emulator with UTF-8 support.
- Discord desktop, only if Discord Rich Presence is desired.

Kitty or another terminal with image support is recommended for cover art.
Terminals without image protocols use the built-in text rendering fallback.

### Metadata and playback services

Anily uses AniList as its primary metadata source. Playback resolution is
handled separately by the configured playback provider and launched through
`mpv`. Provider availability can change independently of metadata availability.

Use the application only where the relevant content and services are lawful and
permitted in your jurisdiction. Anily does not host or redistribute media.

## Installation

### From a GitHub release

Download the archive for your operating system from the
[Releases](https://github.com/dummy3ye/anily/releases) page, extract it, and
place the `anily` executable somewhere on your `PATH`.

Release artifacts currently target:

| Platform | Artifact |
| --- | --- |
| Linux x86_64 | `x86_64-unknown-linux-gnu` |
| macOS Intel | `x86_64-apple-darwin` |
| macOS Apple Silicon | `aarch64-apple-darwin` |
| Windows x86_64 | `x86_64-pc-windows-msvc` |

Install `mpv` separately using your operating system's package manager.

### From source

```sh
git clone https://github.com/dummy3ye/anily.git
cd anily
cargo install --path .
```

Run the installed application:

```sh
anily
```

Or run it without installing:

```sh
cargo run --release
```

### Nix

Run the latest repository revision:

```sh
nix run github:dummy3ye/anily
```

Build the package locally:

```sh
nix build
./result/bin/anily
```

Enter the development environment:

```sh
nix develop
```

The flake supports `x86_64-linux`, `aarch64-linux`, `x86_64-darwin`, and
`aarch64-darwin`. The packaged application includes `mpv` on its wrapped
runtime `PATH`.

## Usage

### Home

The home screen opens with live catalog data loading in the background. Use the
arrow keys or `h`, `j`, `k`, and `l` to move through the dashboard.

| Key | Action |
| --- | --- |
| `/` | Start search |
| `Enter` | Open the selected anime |
| `Arrow keys` / `h j k l` | Navigate |
| `PageUp` / `PageDown` | Move through the trending grid |
| `Esc` | Exit from home |
| `q` | Quit |

search result screenshot.

### Details and episodes

On the details screen, episode rows are selectable and scroll within the
available terminal space.

| Key | Action |
| --- | --- |
| `Up` / `Down` | Select an episode |
| `Enter` | Start or replace playback |
| `d` | Toggle sub/dub |
| `Esc` | Return home |

Playback runs in the background while Anily remains visible. The selected row
shows `Now playing...` while `mpv` is active.

`now playing...` screenshot

## Discord Rich Presence

Discord RPC is optional and does not require a bot token. It uses a Discord
Application ID and the local Discord IPC socket.

1. Create an application in the [Discord Developer Portal](https://discord.com/developers/applications).
2. Copy its **Application ID**.
3. Export the ID before starting Anily:

   ```sh
   export ANILY_DISCORD_CLIENT_ID=your_application_id
   anily
   ```

When an episode starts, Discord shows Anily, the anime title, and the episode
number. Presence is cleared when playback ends or Anily exits. If the variable
is unset, or Discord is not running, Anily continues without RPC.

## Configuration

Anily currently uses environment variables rather than a configuration file.

| Variable | Required | Description |
| --- | --- | --- |
| `ANILY_DISCORD_CLIENT_ID` | No | Discord Application ID for Rich Presence |

Do not commit this value to the repository or place it in a public log.

## Development

Install stable Rust, then run the checks used by CI:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

With Nix:

```sh
nix develop
nix flake check
```

Useful project files:

```text
src/providers/anilist.rs   AniList metadata and cover integration
src/providers/hianime.rs   Playback search, episodes, and stream resolution
src/discord.rs             Discord Rich Presence IPC integration
src/app/                   Application state and orchestration
src/ui/                    Terminal rendering
flake.nix                  Nix package and development shell
cliff.toml                 git-cliff changelog configuration
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines and
[SECURITY.md](SECURITY.md) for private vulnerability reporting.

## Troubleshooting

### Covers are not visible

Use Kitty or a terminal with a supported image protocol. Other terminals show
the fallback representation. Confirm that the terminal is not filtering escape
sequences and that network access to AniList is available.

### Playback does not start

Confirm that `mpv` is installed and available:

```sh
mpv --version
```

Playback providers can temporarily return no match or no stream even when
AniList metadata loads correctly. Retry later or verify the selected language
with `d`.

### Discord presence is missing

Check that Discord desktop is running and that
`ANILY_DISCORD_CLIENT_ID` contains the Application ID, not a bot token.

## Project status

Anily is under active development. Provider behavior, terminal image support,
and UI details may evolve between releases.
