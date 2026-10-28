#[cfg(unix)]
pub type IpcStream = std::os::unix::net::UnixStream;

#[cfg(windows)]
pub type IpcStream = std::fs::File;

#[cfg(unix)]
fn find_socket() -> Option<std::path::PathBuf> {
    use std::path::PathBuf;

    let uid = unsafe { libc::getuid() };
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

    let flatpak = PathBuf::from(format!("/run/user/{uid}/app/com.discordapp.Discord"));
    if flatpak.is_dir() {
        dirs.push(flatpak);
    }
    let snap = PathBuf::from(format!("/run/user/{uid}/snap.discord"));
    if snap.is_dir() {
        dirs.push(snap);
    }

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

#[cfg(unix)]
fn connect_socket(path: &std::path::Path) -> Option<IpcStream> {
    use std::time::Duration;
    let stream = IpcStream::connect(path).ok()?;
    let timeout = Some(Duration::from_secs(5));
    stream.set_read_timeout(timeout).ok()?;
    stream.set_write_timeout(timeout).ok()?;
    Some(stream)
}

#[cfg(unix)]
pub fn connect() -> Option<IpcStream> {
    let path = find_socket()?;
    connect_socket(&path)
}

#[cfg(windows)]
pub fn connect() -> Option<IpcStream> {
    use std::fs::OpenOptions;
    for i in 0..10 {
        let pipe = format!(r"\\?\pipe\discord-ipc-{i}");
        if let Ok(file) = OpenOptions::new().read(true).write(true).open(&pipe) {
            return Some(file);
        }
    }
    None
}
