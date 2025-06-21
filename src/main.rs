mod doom_fire;
mod perlin;
mod render;

use crate::doom_fire::DoomFire;
use crate::render::render_fire_frame_to_image;
use std::{fs, time::Instant, process::Command};

fn main() -> anyhow::Result<()> {
    let screen_width = 1920;
    let screen_height = 1080;
    let scale = 6;  // scale factor to change the size of the fire effect
    let fps = 30;
    let mut fire = DoomFire::new(screen_width / scale, screen_height / scale);

    let cache_dir = dirs::home_dir()
        .expect("Could not find home directory")
        .join(".cache/hyprpaper");
    fs::create_dir_all(&cache_dir)?;

    // Set your output name here (use `wlr-randr` or `hyprctl monitors` to find it)
    let output = ""; // leave empty for all outputs, or set to a specific output name

    let wallpaper_path = cache_dir.join("doomfire.webp");

    loop {
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

