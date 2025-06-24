use std::process::Command;
use std::path::PathBuf;

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

/// Returns Vec<(output_name, is_covered)> for all outputs.
pub fn get_outputs_covered() -> Vec<(String, bool)> {
    let monitors = std::process::Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output()
        .ok()
        .and_then(|o| serde_json::from_slice::<serde_json::Value>(&o.stdout).ok());
    let clients = std::process::Command::new("hyprctl")
        .args(["clients", "-j"])
        .output()
        .ok()
        .and_then(|o| serde_json::from_slice::<serde_json::Value>(&o.stdout).ok());
    let (monitors, clients) = match (monitors, clients) {
        (Some(m), Some(c)) => (m, c),
        _ => return vec![],
    };
    let mut result = vec![];
    for monitor in monitors.as_array().unwrap_or(&vec![]) {
        let name = monitor.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
        let ws_id = monitor.get("activeWorkspace")
            .and_then(|ws| ws.get("id").or_else(|| ws.get("name")))
            .cloned();
        let mut found = false;
        for client in clients.as_array().unwrap_or(&vec![]) {
            let client_ws_id = client.get("workspace")
                .and_then(|ws| ws.get("id").or_else(|| ws.get("name")))
                .cloned();
            if ws_id == client_ws_id {
                found = true;
                break;
            }
        }
        result.push((name, found));
    }
    result
}