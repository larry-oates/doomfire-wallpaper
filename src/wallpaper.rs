use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Set by the compositor event listener whenever a Hyprland event may change
/// workspace/monitor coverage; consumed by the UI tick callback.
static OUTPUTS_DIRTY: AtomicBool = AtomicBool::new(false);

/// Consumes the flag set by the event listener.
pub fn take_outputs_dirty() -> bool {
    OUTPUTS_DIRTY.swap(false, Ordering::Relaxed)
}

/// Hyprland's socket2 event socket, if we are in a Hyprland session.
fn hyprland_socket2_path() -> Option<PathBuf> {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")?;
    let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
    Some(
        PathBuf::from(runtime)
            .join("hypr")
            .join(sig)
            .join(".socket2.sock"),
    )
}

/// Socket2 events that can change which outputs are covered.
const COVERAGE_EVENTS: &[&str] = &[
    "workspace>>",
    "workspacev2>>",
    "specialworkspace>>",
    "specialworkspacev2>>",
    "focusedmon>>",
    "focusedmonv2>>",
    "openwindow>>",
    "openwindowv2>>",
    "closewindow>>",
    "movewindow>>",
    "movewindowv2>>",
    "moveWorkspace>>",
    "createWorkspace>>",
    "destroyWorkspace>>",
    "monitoradded>>",
    "monitoraddedv2>>",
    "monitorremoved>>",
    "fullscreen>>",
    "changefloatingmode>>",
    "configreloaded>>",
];

/// Listens to Hyprland socket2 events in the background and flags coverage as
/// dirty when something relevant happens, so the UI reacts within a frame
/// instead of waiting for the next poll. Falls back to polling in callers.
pub fn start_compositor_event_listener() {
    if hyprland_socket2_path().is_none() {
        return;
    }
    let _ = std::thread::Builder::new()
        .name("hypr_events".to_string())
        .spawn(move || loop {
            let Some(path) = hyprland_socket2_path() else {
                return;
            };
            match UnixStream::connect(&path) {
                Ok(stream) => {
                    let reader = BufReader::new(stream);
                    for line in reader.lines().map_while(Result::ok) {
                        if COVERAGE_EVENTS.iter().any(|e| line.starts_with(e)) {
                            OUTPUTS_DIRTY.store(true, Ordering::Relaxed);
                        }
                    }
                }
                Err(_) => {}
            }
            // Socket dropped (compositor restart) - retry shortly.
            std::thread::sleep(Duration::from_secs(2));
        });
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
            if client
                .get("hidden")
                .and_then(|h| h.as_bool())
                .unwrap_or(false)
            {
                continue;
            }
            if client
                .get("mapped")
                .and_then(|m| m.as_bool())
                .unwrap_or(true) == false
            {
                continue;
            }
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