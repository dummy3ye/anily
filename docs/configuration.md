# Configuration

Anily can be configured via a TOML config file. Environment variables are also supported for specific features.

## Config File

**Location:** `~/.config/anily/config.toml` (or `$XDG_CONFIG_HOME/anily/config.toml`)

The config file is optional. If it doesn't exist, Anily will use defaults.

### Example

```toml
# Discord Rich Presence Application ID
discord_client_id = "123456789012345678"

# Exclude specific genres or tags from anime feeds (e.g. "Hentai", "Ecchi", "NTR")
exclude_genres = ["Hentai", "Adult", "Ecchi", "ntr"]

# Whether to apply the exclude filter to manual search results as well (defaults to false)
filter_searches = false
```

### Options

| Option | Type | Required | Description |
| --- | --- | --- | --- |
| `discord_client_id` | `string` | No | Discord Application ID for Rich Presence. Get one from the [Discord Developer Portal](https://discord.com/developers/applications). |
| `exclude_genres` | `array of strings` | No | List of genres or tags to filter out from feeds. Case-insensitive, supports aliases like `ntr` -> `Netorare`. Defaults to `[]`. |
| `filter_searches` | `boolean` | No | Whether `exclude_genres` should also apply when manually searching for anime. Defaults to `false` (allowing users to search and watch specific titles directly). |

## Environment Variables

| Variable | Required | Description |
| --- | --- | --- |
| `XDG_CONFIG_HOME` | No | Override config directory location (defaults to `~/.config`) |
| `HOME` | No | Used to determine config directory if `XDG_CONFIG_HOME` is not set |
| `ANILY_DISCORD_CLIENT_ID` | No | *(Legacy/alternative)* Discord Application ID. If both config file and this env var are set, the config file takes precedence in the current implementation. |

**Note:** The app loads from the config file (`Config::load()` reads from TOML). Check the source for the exact precedence if needed.

## Security Notes

- Never commit your Discord Application ID to version control
- Config files may contain sensitive values - ensure appropriate file permissions (`chmod 600` on Linux/macOS)
- Discord RPC uses the local Discord IPC socket; it does not require a bot token
- Anily only reads configuration locally and never transmits it
