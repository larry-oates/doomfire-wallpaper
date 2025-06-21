mod doom_fire;
mod perlin;
mod render;

use crate::doom_fire::DoomFire;
use crate::render::render_fire_frame_to_image;
use std::{fs, time::Instant, process::Command};
use serde_json::Value;

fn is_all_screens_covered() -> bool {
    // Get monitor info
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

fn is_system_sleeping() -> bool {
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

fn main() -> anyhow::Result<()> {
    let screen_width = 1920;
    let screen_height = 1080;
    let scale = 4;  // scale factor to change the size of the fire effect
    let fps = 12;
    let mut fire = DoomFire::new(screen_width / scale, screen_height / scale);

    let cache_dir = dirs::home_dir()
        .expect("Could not find home directory")
        .join(".cache/hyprpaper");
    fs::create_dir_all(&cache_dir)?;

    // Set your output name here (use `wlr-randr` or `hyprctl monitors` to find it)
    let output = ""; // leave empty for all outputs, or set to a specific output name

    let wallpaper_path = cache_dir.join("doomfire.webp");

    loop {
        let covered = is_all_screens_covered();
        let sleeping = is_system_sleeping();

        if covered || sleeping {
            println!("Waiting for all screens to be covered or system to wake up...");
            std::thread::sleep(std::time::Duration::from_millis(10));
            continue;
        }
        let start = Instant::now();

        // Generate a new frame
        fire.update();
        let img = render_fire_frame_to_image(&fire, scale)?;

        // Save as WebP
        {
            let mut file = std::fs::File::create(&wallpaper_path)?;
            img.write_to(&mut file, image::ImageFormat::WebP)?;
        }

        // Tell Hyprpaper to reload the wallpaper
        let status = Command::new("hyprctl")
            .args([
                "hyprpaper",
                "preload",
                wallpaper_path.to_str().unwrap(),
            ])
            .status()?;
        if !status.success() {
            eprintln!("Failed to preload wallpaper with hyprpaper");
        }

        let status = Command::new("hyprctl")
            .args([
                "hyprpaper",
                "wallpaper",
                &format!("{},{}", output, wallpaper_path.to_str().unwrap()),
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

        // Wait for the next frame
        let elapsed = start.elapsed();
        let sleep_time = std::time::Duration::from_millis(1000 / fps as u64).saturating_sub(elapsed);
        if sleep_time > std::time::Duration::ZERO {
            std::thread::sleep(sleep_time);
        }
    }
}

