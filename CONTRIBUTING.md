# Contributing to Anily

Thanks for contributing. Please open an issue before large changes so the
design can be discussed.

## Development setup

You need Rust stable, Cargo, and an available terminal. Run:

```sh
cargo check
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

Run the application with `cargo run`. Playback requires `mpv`; Discord Rich
Presence is optional and uses `ANILY_DISCORD_CLIENT_ID`.

## Pull requests

- Keep changes focused and explain user-visible behavior.
- Add or update tests when behavior changes.
- Keep documentation and `CHANGELOG.md` current when appropriate.
- Use a Conventional Commit title, such as `feat: add episode filters` or
  `fix: handle unavailable streams`.
- Do not commit credentials, tokens, generated build output, or personal data.

## Commit messages

Use one of these prefixes:

`feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`,
or `security`.

Imperative, focused messages make the automated changelog useful.
