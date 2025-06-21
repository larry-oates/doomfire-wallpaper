mod doom_fire;
mod perlin;
mod render;
mod wallpaper;

use crate::doom_fire::DoomFire;
use crate::render::render_fire_frame_to_image;
use crate::wallpaper::{is_all_screens_covered, is_system_sleeping, load_wallpaper};
use std::{fs, time::Instant};


fn main() -> anyhow::Result<()> {
    let screen_width = 1920;
    let screen_height = 1080;
    let scale = 4;  // scale factor to change the size of the fire effect
    let fps = 14;
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
            println!("Waiting for screens to be uncovered or system to wake up...");
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
       load_wallpaper(&wallpaper_path, output)?;

        // Wait for the next frame
        let elapsed = start.elapsed();
        let sleep_time = std::time::Duration::from_millis(1000 / fps as u64).saturating_sub(elapsed);
        if sleep_time > std::time::Duration::ZERO {
            println!("Sleeping for {:?}", sleep_time);
            std::thread::sleep(sleep_time);
        }
        else {
            let elapsed_secs = elapsed.as_secs_f64();
            println!("Frame took too long: {:?} fps", 1.0 / elapsed_secs);
        }
    }
}

