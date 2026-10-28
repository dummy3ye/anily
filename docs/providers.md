# Providers

Anily uses two external providers: one for metadata and one for playback.

## AniList (Metadata)

**Module:** `src/providers/anilist.rs`  
**API:** [AniList GraphQL API](https://anilist.gitbook.io/anilist-apiv2-docs/)  
**Base URL:** `https://graphql.anilist.co`

### Data Fetched

| Function | Purpose |
| --- | --- |
| `home_feed()` | Fetches trending, airing, popular, and favorites in parallel |
| `trending()` | Fetch trending anime |
| `search()` | Search anime by query |
| `details()` | Get detailed information for a specific anime |
| `fetch_cover()` | Download and decode cover art images |

### Data Structures

- `Anime` - Basic anime info (id, titles, cover URLs, etc.)
- `AnimeDetails` - Extended info (description, genres, score, status, duration, episode count)
- `HomeFeed` - Container for all home screen sections
- `CoverArt` - Decoded image with dimensions

### Notes

- Uses GraphQL queries with custom structs
- Cover art is fetched lazily as items become visible
- Failed cover fetches trigger retry with `cover_large` URL as fallback
- Images are decoded with the `image` crate (supports JPEG, PNG, WebP)

## HiAnime (Playback)

**Module:** `src/providers/hianime.rs`  
**Purpose:** Finds streamable episodes and resolves video URLs

### Flow

1. `search()` - Find anime matches by title on the playback provider
2. `episodes()` - Get episode list for the matched anime
3. `stream()` - Resolve actual stream URL and subtitles for a specific episode

### Data Structures

- `Match` - Search result with provider-specific ID
- `Episode` - Episode info (id, number, etc.)
- `Stream` - Stream info (video URL, referer, optional subtitles)

### Notes

- Playback provider search uses the AniList anime title
- If no match is found, the details screen still opens (metadata loads from AniList independently)
- Stream resolution happens on-demand when user presses Enter to play
- The provider's availability can change without notice
- Anily does not host or redistribute any media content

## Legal Notice

Use Anily only where the relevant content and services are lawful and permitted in your jurisdiction. Anily does not host or redistribute media. Users are responsible for compliance with their local laws and terms of service.
