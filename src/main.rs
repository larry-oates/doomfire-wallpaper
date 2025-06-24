mod doom_fire;
mod perlin;
mod render;
mod wallpaper;

pub mod config;
pub mod fire_types;
pub mod particle;

use crate::config::Config;
use crate::doom_fire::DoomFire;
use crate::render::render_fire_frame_to_image;
use crate::wallpaper::{get_outputs_covered, is_system_sleeping, load_wallpaper};
use image::{DynamicImage, GenericImageView, Pixel};
use std::{collections::HashMap, fs, time::{Instant}};


fn main() -> anyhow::Result<()> {
    let config = Config::load();
    println!("Using config: {:?}", config);
    let mut fire = DoomFire::new(&config);

    let cache_dir = dirs::home_dir()
        .expect("Could not find home directory")
        .join(".cache/hyprpaper");
    fs::create_dir_all(&cache_dir)?;
    let wallpaper_path = cache_dir.join("doomfire.webp");
    let mut restart_flag = false;
    let restart_on_pause = config.restart_on_pause.unwrap(); 
    let fps = config.fps.unwrap(); 
    let output = config.output.clone().unwrap();
    let pause_on_cover = config.pause_on_cover.unwrap();
    let screen_burn = config.screen_burn.unwrap();
    let mut screenshot_count = 0;
    let mut last_screenshot: HashMap<String, DynamicImage> = HashMap::new();
    
    loop {
        let start = Instant::now();
        let covered_outputs = get_outputs_covered();
        let all_covered = covered_outputs.iter().all(|(_, c)| *c);
        let sleeping = is_system_sleeping();
        let paused = (pause_on_cover && all_covered) || sleeping;

        // Take screenshots for all outputs when they become covered (for burn effect)
        if screen_burn && all_covered {
            screenshot_count += 1;
            if screenshot_count == 25 {
                screenshot_count = 0;
                for (name, _covered) in &covered_outputs {
                    if let Ok(output) = std::process::Command::new("grim")
                    .args(["-o", name, "-" ])
                    .output() {
                        if output.status.success() {
                            if let Ok(i) = image::load_from_memory(&output.stdout) {
                                last_screenshot.insert(name.clone(), i);
                            }
                        }
                    }
                }
            }
        }

        // Handle paused state
        if paused {
            std::thread::sleep(std::time::Duration::from_millis(10));
            if restart_on_pause && !restart_flag {
                restart_flag = true;
                let bg_idx = 0u8;
                fire.pixel_buffer.iter_mut().for_each(|x| *x = bg_idx);
                let img = render_fire_frame_to_image(&fire)?;
                let mut file = std::fs::File::create(&wallpaper_path)?;
                img.write_to(&mut file, image::ImageFormat::WebP)?;
                load_wallpaper(&wallpaper_path, &output)?;
            }
            continue;
        }
        if restart_flag {
            restart_flag = false;
            fire.initialize_fire();
        }
        if screen_burn {
            if let Some((name, _)) = covered_outputs.iter().find(|(_, covered)| !*covered) {
                if let Some(img) = last_screenshot.remove(name) {
                    last_screenshot.clear();
                    let resized = img.resize_exact(fire.width as u32, fire.height as u32, image::imageops::FilterType::Triangle);
                    for y in 0..fire.height {
                        for x in 0..fire.width {
                            let px = resized.get_pixel(x as u32, y as u32);
                            let luma = px.to_luma()[0] as usize;
                            let idx = ((luma as f32 / 255.0) * (fire.palette.len() as f32 - 1.0)).round() as u8;
                            let fire_idx = &mut fire.pixel_buffer[y * fire.width + x];
                            *fire_idx = (*fire_idx).max(idx);
                        }
                    }
                }
            }
        }

        let img = render_fire_frame_to_image(&fire)?;
        fire.update();

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

