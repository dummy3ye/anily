# Changelog

All notable changes to Anily are documented here.

## 0.1.0 - 2026-10-27

### Added

- AniList-powered anime discovery, metadata, recommendations, and high-resolution cover art.
- Lazy-loading dashboard with portrait cover rendering and Kitty terminal image support.
- Search, keyboard navigation, anime details, episode selection, and sub/dub switching.
- HiAnime playback integration with episode lookup, stream resolution, subtitles, and mpv playback.
- Persistent TUI playback with a `Now playing...` episode indicator.
- Optional Discord Rich Presence through local Discord IPC.
- Cross-platform CI, release packaging, issue templates, and contributor documentation.

### Fixed

- Corrected HiAnime episode and server parsing for JSON-wrapped API responses.
- Corrected playback provider ID handling for numeric anime IDs.
- Prevented mpv output from overwriting the Anily interface.
- Kept metadata loading independent from playback-provider availability.