# Configuration

Anily can be configured via a TOML config file. Environment variables are also supported for specific features.

## Config File

**Location:** `~/.config/anily/config.toml` (or `$XDG_CONFIG_HOME/anily/config.toml`)

The config file is optional. If it doesn't exist, Anily will use defaults.

### Example

```toml
# Discord Rich Presence Application ID
discord_client_id = "123456789012345678"
```

### Options

| Option | Type | Required | Description |
| --- | --- | --- | --- |
| `discord_client_id` | `string` | No | Discord Application ID for Rich Presence. Get one from the [Discord Developer Portal](https://discord.com/developers/applications). |

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
