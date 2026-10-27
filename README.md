# Anily

Anily is a terminal anime browser and player built with Rust. It uses AniList
for metadata and HiAnime for playback resolution, with mpv handling playback.

## Run

```sh
cargo run
```

Install `mpv` for playback. Kitty and compatible terminals can render covers
directly; other terminals use a text fallback.

Optional Discord Rich Presence requires a Discord application ID:

```sh
export ANILY_DISCORD_CLIENT_ID=your_application_id
```

## Development

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines and
[SECURITY.md](SECURITY.md) for private vulnerability reporting.