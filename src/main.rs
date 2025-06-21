mod doom_fire;
mod perlin;
mod render;
mod wallpaper;

use crate::doom_fire::DoomFire;
use crate::render::render_fire_frame_to_image;
use crate::wallpaper::{is_all_screens_covered, is_system_sleeping, load_wallpaper};
use std::{fs, time::Instant};
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    screen_width: Option<usize>,
    screen_height: Option<usize>,
    scale: Option<usize>,
    fps: Option<u32>,
    output: Option<String>,
    palette: Option<String>,
    background: Option<[u8; 3]>,
    restart_on_pause: Option<bool>, 
}

impl Config {
    fn load() -> Self {
        let config_path = dirs::home_dir()
            .expect("Could not find home directory")
            .join(".config/doomfire_wallpaper/config.toml");
        let config_str = fs::read_to_string(config_path).unwrap_or_default();
        toml::from_str(&config_str).unwrap_or_default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            screen_width: Some(1920),
            screen_height: Some(1080),
            scale: Some(4),
            fps: Some(23),
            output: Some(String::new()),
            palette: Some("Original".to_string()),
            background: None,
            restart_on_pause: Some(true),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let config = Config::load();

    let screen_width = config.screen_width.unwrap_or(1920);
    let screen_height = config.screen_height.unwrap_or(1080);
    let scale = config.scale.unwrap_or(4);
    let fps = config.fps.unwrap_or(23);
    let output = config.output.unwrap_or_else(String::new);

    let palette = match config.palette.as_deref() {
        Some("Blue") => doom_fire::FirePalette::Blue,
        Some("Rainbow") => doom_fire::FirePalette::Rainbow,
        Some("Green") => doom_fire::FirePalette::Green,
        Some("Purple") => doom_fire::FirePalette::Purple,
        Some("WhiteHot") => doom_fire::FirePalette::WhiteHot,
        _ => doom_fire::FirePalette::Original,
    };

    let mut fire = DoomFire::new(
        screen_width / scale,
        screen_height / scale,
        palette,
        config.background,
    );

    let cache_dir = dirs::home_dir()
        .expect("Could not find home directory")
        .join(".cache/hyprpaper");
    fs::create_dir_all(&cache_dir)?;

    let wallpaper_path = cache_dir.join("doomfire.webp");
    let mut restart_flag = false;
    let restart_on_pause = config.restart_on_pause.unwrap_or(true); 
    loop {
        let covered = is_all_screens_covered();
        let sleeping = is_system_sleeping();

        if covered || sleeping {
            std::thread::sleep(std::time::Duration::from_millis(10));
            if restart_on_pause && !restart_flag {
                restart_flag = true;
                fire.initialize_fire();
                let img = render_fire_frame_to_image(&fire, scale)?;
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
        let img = render_fire_frame_to_image(&fire, scale)?;

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

