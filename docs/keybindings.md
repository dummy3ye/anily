# Keybindings

Anily is currently fully keyboard-driven. All navigation is done via the keyboard.

## Global

| Key | Action |
| --- | --- |
| `q` | Show exit confirmation popup (when not searching) |
| `Esc` | Go back to Home from Details/Episodes screen |
| `y`/`Y`/`Enter` | Confirm exit (in exit popup) |
| `n`/`N`/`Esc` | Cancel exit (in exit popup) |

## Home Screen

| Key | Action |
| --- | --- |
| `h`, `j`, `k`, `l` or Arrow Keys | Navigate through the dashboard |
| `PageUp` / `PageDown` | Move through the trending grid |
| `Enter` | Open selected anime (go to details/episodes) |
| `/` | Start search mode |
| `Esc` | Exit application (shows confirmation) |

## Search Mode (on Home Screen)

| Key | Action |
| --- | --- |
| Type characters | Build search query |
| `Backspace` | Delete character from query |
| `Enter` | Execute search |
| `Esc` | Cancel search (return to normal home navigation) |

## Details/Episodes Screen

| Key | Action |
| --- | --- |
| `Up` / `Down` or `k` / `j` | Select episode |
| `Enter` | Start or replace playback of selected episode |
| `d` | Toggle between sub and dub |
| `Esc` | Return to Home screen |

## Notes

- When `mpv` is playing, Anily remains visible in the background. The currently playing episode is marked with `Now playing...`.
- Playback replaces any existing `mpv` instance started by Anily.
- The exit confirmation popup appears when pressing `q` outside of search mode to prevent accidental quits.
