mod doom_fire;
mod perlin;
mod render;
mod wallpaper;

use crate::doom_fire::{Config, DoomFire};
use crate::render::render_fire_frame_to_image;
use crate::wallpaper::{is_all_screens_covered, is_system_sleeping, load_wallpaper};
use std::{fs, time::Instant};


fn main() -> anyhow::Result<()> {
    let config = Config::load();

    let mut fire = DoomFire::new(&config);

    let cache_dir = dirs::home_dir()
        .expect("Could not find home directory")
        .join(".cache/hyprpaper");
    fs::create_dir_all(&cache_dir)?;

    let wallpaper_path = cache_dir.join("doomfire.webp");
    let mut restart_flag = false;
    let restart_on_pause = config.restart_on_pause.unwrap_or(true); 

    let fps = config.fps.unwrap_or(30); 
    let output = config.output.clone().unwrap_or_else(|| "".to_string());

    loop {
        let covered = is_all_screens_covered();
        let sleeping = is_system_sleeping();

        if covered || sleeping {
            std::thread::sleep(std::time::Duration::from_millis(10));
            if restart_on_pause && !restart_flag {
                restart_flag = true;
                fire.initialize_fire();
                let img = render_fire_frame_to_image(&fire)?;
                let mut file = std::fs::File::create(&wallpaper_path)?;
                img.write_to(&mut file, image::ImageFormat::WebP)?;
                load_wallpaper(&wallpaper_path, &output)?;
            }
            continue;
        }
        if restart_flag {
           restart_flag = false;
        }

        let start = Instant::now();

        fire.update();  
        let img = render_fire_frame_to_image(&fire)?;

        let mut file = std::fs::File::create(&wallpaper_path)?;
        img.write_to(&mut file, image::ImageFormat::WebP)?;

        load_wallpaper(&wallpaper_path, &output)?;

        let elapsed = start.elapsed();
        let sleep_time = std::time::Duration::from_millis(1000 / fps as u64).saturating_sub(elapsed);
        if sleep_time > std::time::Duration::ZERO {
            std::thread::sleep(sleep_time);
        }
    }
}

