use serde_json::Value;  
use std::process::Command;
use std::path::PathBuf;

pub fn is_all_screens_covered() -> bool {
    let monitors = Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output()
        .ok()
        .and_then(|o| serde_json::from_slice::<Value>(&o.stdout).ok());
    let clients = Command::new("hyprctl")
        .args(["clients", "-j"])
        .output()
        .ok()
        .and_then(|o| serde_json::from_slice::<Value>(&o.stdout).ok());

    let (monitors, clients) = match (monitors, clients) {
        (Some(m), Some(c)) => (m, c),
        _ => return false,
    };

    // For each monitor, check if its active workspace has at least one client
    for monitor in monitors.as_array().unwrap_or(&vec![]) {
        // Get the active workspace ID or name for this monitor
        let ws_id = monitor.get("activeWorkspace")
            .and_then(|ws| ws.get("id").or_else(|| ws.get("name")))
            .cloned();

        let mut found = false;
        for client in clients.as_array().unwrap_or(&vec![]) {
            // Compare workspace id or name
            let client_ws_id = client.get("workspace")
                .and_then(|ws| ws.get("id").or_else(|| ws.get("name")))
                .cloned();
            if ws_id == client_ws_id {
                found = true;
                break;
            }
        }
        if !found {
            return false;
        }
    }
    true
}

pub fn is_system_sleeping() -> bool {
    let output = Command::new("systemctl")
        .args(["is-system-running"])
        .output();
    if let Ok(output) = output {
        let state = String::from_utf8_lossy(&output.stdout);
        state.contains("suspend") || state.contains("sleep")
    } else {
        false
    }
}
pub fn load_wallpaper(path: &PathBuf, output: &str) -> anyhow::Result<()> {
    let status = Command::new("hyprctl")
        .args([
            "hyprpaper",
            "preload",
            &format!("{}", path.to_str().unwrap()),
        ])
        .status()?;
    if !status.success() {
        eprintln!("Failed to set wallpaper with hyprpaper");
    }let status = Command::new("hyprctl")
        .args([
            "hyprpaper",
            "wallpaper",
            &format!("{},{}", output, path.to_str().unwrap()),
        ])
        .status()?;
    if !status.success() {
        eprintln!("Failed to set wallpaper with hyprpaper");
    }

    let status = Command::new("hyprctl")
        .args(["hyprpaper", "unload", "all"])
        .status()?;
    if !status.success() {
        eprintln!("Failed to reload hyprpaper");
    }

    Ok(())
}