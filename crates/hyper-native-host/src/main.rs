use std::{
    io::{Read, Write},
    path::PathBuf,
};
fn main() {
    if let Err(e) = run() {
        let _ = reply(&serde_json::json!({"accepted":false,"error":e.to_string()}));
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut len = [0u8; 4];
    std::io::stdin().read_exact(&mut len)?;
    let size = u32::from_le_bytes(len) as usize;
    if size > 1024 * 1024 {
        return Err("message exceeds 1 MiB".into());
    }
    let mut data = vec![0; size];
    std::io::stdin().read_exact(&mut data)?;
    let value: serde_json::Value = serde_json::from_slice(&data)?;
    let kind = value
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or("missing message type")?;
    if kind == "open_application" {
        launch_desktop()?;

        reply(&serde_json::json!({
            "accepted": true,
            "desktopAvailable": true
        }))?;

        return Ok(());
    }
    if !matches!(
        kind,
        "send_single_download" | "send_page_links" | "ping" | "get_desktop_status" | "open_application"
    ) {
        return Err("unsupported message type".into());
    }
    if matches!(kind, "send_single_download" | "send_page_links") {
        let dir = data_dir().join("browser-inbox");
        std::fs::create_dir_all(&dir)?;
        let tmp = dir.join(format!("{}.tmp", uuid::Uuid::new_v4()));
        let final_path = tmp.with_extension("json");
        std::fs::write(&tmp, &data)?;
        std::fs::rename(tmp, final_path)?;
    }
    reply(&serde_json::json!({"accepted":true,"desktopAvailable":true}))?;
    Ok(())
}
/// Starts the desktop app. Order: `HYPER_GET_APP`, then executables next to this
/// host, then names on `PATH`. The Tauri binary is called `hyper-get-desktop`.
fn launch_desktop() -> Result<(), Box<dyn std::error::Error>> {
    use std::process::{Command, Stdio};

    let spawn = |program: &std::ffi::OsStr| {
        Command::new(program)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|_| ())
    };

    if let Some(path) = std::env::var_os("HYPER_GET_APP") {
        return spawn(&path).map_err(|e| format!("cannot start HYPER_GET_APP: {e}").into());
    }

    let names: &[&str] = if cfg!(windows) {
        &["hyper-get-desktop.exe", "Hyper Get.exe", "hyper-get.exe"]
    } else {
        &["hyper-get-desktop", "hyper-get"]
    };

    if let Some(dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())) {
        for name in names {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return spawn(candidate.as_os_str()).map_err(|e| format!("cannot start desktop app: {e}").into());
            }
        }
    }

    for name in names {
        if spawn(std::ffi::OsStr::new(name)).is_ok() {
            return Ok(());
        }
    }

    Err("Hyper Get desktop app not found; set HYPER_GET_APP to its full path".into())
}
fn reply(v: &serde_json::Value) -> std::io::Result<()> {
    let data = serde_json::to_vec(v)?;
    let mut out = std::io::stdout();
    out.write_all(&(data.len() as u32).to_le_bytes())?;
    out.write_all(&data)?;
    out.flush()
}
fn data_dir() -> PathBuf {
    if let Some(v) = std::env::var_os("HYPER_GET_DATA_DIR") {
        return PathBuf::from(v);
    }
    #[cfg(windows)]
    {
        if let Some(v) = std::env::var_os("APPDATA") {
            return PathBuf::from(v).join("io.github.hyper-get");
        }
    }
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|v| PathBuf::from(v).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("io.github.hyper-get")
}
