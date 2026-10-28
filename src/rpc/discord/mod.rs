pub mod socket;

use anyhow::Result;
use serde_json::json;
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct DiscordRpc {
    stream: std::os::unix::net::UnixStream,
}

impl DiscordRpc {
    pub fn connect(client_id: &str) -> Option<Self> {
        let path = socket::find_socket()?;
        let mut stream = socket::connect_socket(&path)?;

        let handshake = json!({ "v": 1, "client_id": client_id }).to_string();
        write_frame(&mut stream, 0, &handshake).ok()?;

        let (op, _body) = read_frame(&mut stream).ok()?;
        if op != 1 {
            return None;
        }

        Some(Self { stream })
    }

    pub fn set_episode(
        &mut self,
        title: &str,
        episode: &str,
        cover_url: Option<&str>,
    ) -> Result<()> {
        let pid = std::process::id();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut activity = json!({
            "details": title,
            "state": format!("Episode {}", episode),
            "timestamps": { "start": now }
        });

        if let Some(img) = cover_url.filter(|s| !s.is_empty()) {
            activity["assets"] = json!({
                "large_image": img,
                "large_text": title
            });
        }

        let payload = json!({
            "cmd": "SET_ACTIVITY",
            "args": {
                "pid": pid,
                "activity": activity
            },
            "nonce": format!("{pid}-{now}")
        });

        write_frame(&mut self.stream, 1, &payload.to_string())?;
        let _ = read_frame(&mut self.stream);
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        let payload = json!({
            "cmd": "SET_ACTIVITY",
            "args": { "pid": std::process::id() },
            "nonce": "clear"
        });
        let _ = write_frame(&mut self.stream, 1, &payload.to_string());
        let _ = read_frame(&mut self.stream);
        Ok(())
    }
}

impl Drop for DiscordRpc {
    fn drop(&mut self) {
        let _ = self.clear();
    }
}

fn write_frame(
    stream: &mut std::os::unix::net::UnixStream,
    opcode: u32,
    payload: &str,
) -> std::io::Result<()> {
    let data = payload.as_bytes();
    let mut frame = Vec::with_capacity(8 + data.len());
    frame.extend_from_slice(&opcode.to_le_bytes());
    frame.extend_from_slice(&(data.len() as u32).to_le_bytes());
    frame.extend_from_slice(data);
    stream.write_all(&frame)?;
    stream.flush()
}

fn read_frame(stream: &mut std::os::unix::net::UnixStream) -> std::io::Result<(u32, String)> {
    let mut header = [0u8; 8];
    stream.read_exact(&mut header)?;
    let opcode = u32::from_le_bytes(header[0..4].try_into().unwrap());
    let len = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
    let mut buf = vec![0u8; len.min(1 << 20)];
    stream.read_exact(&mut buf)?;
    Ok((opcode, String::from_utf8_lossy(&buf).into_owned()))
}
