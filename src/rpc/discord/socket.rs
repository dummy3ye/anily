// src/rpc/discord/socket.rs
// Locate and connect to the Discord IPC socket.

/// Search common locations for the Discord IPC socket.
/// Tries `discord-ipc-{0..9}` under `XDG_RUNTIME_DIR`, `TMPDIR`, `/run/user/{uid}`,
/// Flatpak paths, and `/tmp`.
#[cfg(unix)]
pub fn find_socket() -> Option<std::path::PathBuf> {
    use std::path::PathBuf;

    let uid = unsafe { libc::getuid() };

    // Candidate parent directories, ordered by preference.
    let mut dirs: Vec<PathBuf> = Vec::new();

    if let Some(xdg) = std::env::var_os("XDG_RUNTIME_DIR") {
        dirs.push(PathBuf::from(xdg));
    }
    dirs.push(PathBuf::from(format!("/run/user/{uid}")));
    if let Some(tmpdir) = std::env::var_os("TMPDIR") {
        dirs.push(PathBuf::from(tmpdir));
    }
    if let Some(temp) = std::env::var_os("TEMP") {
        dirs.push(PathBuf::from(temp));
    }
    dirs.push(PathBuf::from("/tmp"));

    // Also check Flatpak and Snap prefixed paths.
    let flatpak = PathBuf::from(format!("/run/user/{uid}/app/com.discordapp.Discord"));
    if flatpak.is_dir() {
        dirs.push(flatpak);
    }
    let snap = PathBuf::from(format!("/run/user/{uid}/snap.discord"));
    if snap.is_dir() {
        dirs.push(snap);
    }

    // Discord creates sockets named discord-ipc-0 through discord-ipc-9.
    for dir in &dirs {
        for i in 0..10 {
            let path = dir.join(format!("discord-ipc-{i}"));
            if path.exists() {
                return Some(path);
            }
        }
    }
    None
}

/// Connect to the Unix socket at `path` with a 5-second read/write timeout.
#[cfg(unix)]
pub fn connect_socket(path: &std::path::Path) -> Option<std::os::unix::net::UnixStream> {
    use std::os::unix::net::UnixStream;
    use std::time::Duration;

    let stream = UnixStream::connect(path).ok()?;
    let timeout = Some(Duration::from_secs(5));
    stream.set_read_timeout(timeout).ok()?;
    stream.set_write_timeout(timeout).ok()?;
    Some(stream)
}

#[cfg(windows)]
pub fn find_socket() -> Option<std::path::PathBuf> {
    None // Windows uses named pipes; not yet supported
}

#[cfg(windows)]
pub fn connect_socket(_path: &std::path::Path) -> Option<std::os::unix::net::UnixStream> {
    None
}
