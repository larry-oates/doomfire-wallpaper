mod doom_fire;
mod perlin;
mod wallpaper;

pub mod config;
pub mod fire_types;
pub mod particle;

use crate::config::Config;
use crate::doom_fire::DoomFire;
use crate::wallpaper::{get_outputs_covered, is_system_sleeping};
use anyhow::{Context, Result};
use gtk4 as gtk;
use gtk::gdk_pixbuf::{Colorspace, Pixbuf};
use gtk::glib::source::timeout_add_local;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Picture};
use image::{DynamicImage, GenericImageView, Pixel};
use rayon::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

const BYTES_PER_PIXEL: usize = 3; // RGB = 3 bytes

fn main() -> Result<()> {
    run_wallpaper()
}

/// Creates the default config file if it doesn't exist.
fn ensure_config_exists() -> Result<()> {
    let config_path = get_config_path()?;
    if !config_path.exists() {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory at {:?}", parent))?;
        }
        let mut file = fs::File::create(&config_path)
            .with_context(|| format!("Failed to create config file at {:?}", config_path))?;
        file.write_all(DEFAULT_CONFIG.as_bytes())
            .context("Failed to write default config")?;
        println!("Created default config at {:?}", config_path);
    }
    Ok(())
}

/// Helper to get the path to the config file.
fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().context("Could not find config directory")?;
    Ok(config_dir.join("doomfire-wallpaper/config.toml"))
}

const DEFAULT_CONFIG: &str = r#"# Default config for doomfire-wallpaper
# For a full list of options, see the README on GitHub.
screen_width = 1920
screen_height = 1080
scale = 4
fps = 24
fire_type = "Original"
# background = [0, 0, 0]
# restart_on_pause = true
# pause_on_cover = true
# screen_burn = false
# wind_strength = 0.5
"#;

/// Runs the GTK application and the wallpaper animation loop.
fn run_wallpaper() -> Result<()> {
    ensure_config_exists()?;

    let app = Application::new(Some("com.leafman.doomfirewallpaper"), Default::default());
    app.connect_activate(build_ui);
    app.run();
    Ok(())
}

/// Builds the UI and sets up the animation timer.
fn build_ui(app: &Application) {
    println!("App Connected!");
    let config = Config::load();
    println!("Using config: {:?}", config);
    let fire = Rc::new(RefCell::new(DoomFire::new(&config)));

    let restart_on_pause = config.restart_on_pause.unwrap_or(false);
    let fps = config.fps.unwrap_or(10);
    let pause_on_cover = config.pause_on_cover.unwrap_or(false);
    let screen_burn = config.screen_burn.unwrap_or(false);
    let height = config.screen_height.unwrap();
    let width = config.screen_width.unwrap();
    let scale = config.scale.unwrap_or(1);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Doom Fire Wallpaper")
        .build();

    let picture = Picture::new();
    window.set_child(Some(&picture));
    window.present();

    let mut last_screenshot: HashMap<String, DynamicImage> = HashMap::new();
    let mut screenshot_count = -1;
    let mut was_paused = false;

    timeout_add_local(std::time::Duration::from_millis(1000 / fps as u64), {
        move || {
            let mut fire = fire.borrow_mut();

            let covered_outputs = get_outputs_covered();
            let all_covered = covered_outputs.iter().all(|(_, c)| *c);
            let sleeping = is_system_sleeping();
            let paused = (pause_on_cover && all_covered) || sleeping;

            // Transition detection: just entered paused state
            let just_paused = paused && !was_paused;
            
            if screen_burn && all_covered {
                screenshot_count += 1;
                if screenshot_count == 24 {
                    screenshot_count = 0;
                    take_screenshots(&mut last_screenshot, &covered_outputs);
                }
            }
            
            if paused {
                if just_paused {
                    eprintln!("[DEBUG] Fire paused (first frame)");
                    if restart_on_pause {
                        fire.pause_fire();
                    }
                    if screen_burn {
                        take_screenshots(&mut last_screenshot, &covered_outputs);
                        screenshot_count = 0;
                    }
                } else {
                    eprintln!("[DEBUG] Fire paused (skipping render)");
                    return glib::ControlFlow::Continue;
                }
            } else {
                if was_paused {
                    eprintln!("[DEBUG] Fire unpaused");
                    if restart_on_pause {
                        fire.initialize_fire();
                    }
                }
                else {
                    fire.update();
                }
            }
            was_paused = paused;
            
            if screen_burn {
                if let Some((name, _)) = covered_outputs.iter().find(|(_, c)| !*c) {
                    if let Some(img) = last_screenshot.remove(name) {
                        last_screenshot.clear();
                        let resized = img.resize_exact(
                            fire.width as u32,
                            fire.height as u32,
                            image::imageops::FilterType::Triangle,
                        );
                        for y in 0..fire.height {
                            for x in 0..fire.width {
                                let px = resized.get_pixel(x as u32, y as u32);
                                let luma = px.to_luma()[0] as usize;
                                let idx = ((luma as f32 / 255.0) * (fire.palette.len() as f32 - 1.0))
                                    .round() as u8;
                                let current_idx = y * fire.width + x;
                                let fire_idx = &mut fire.pixel_buffer[current_idx];
                                *fire_idx = (*fire_idx).max(idx);
                            }
                        }
                        eprintln!("[DEBUG] Burn-in applied from screenshot");
                    }
                }
            }

            // Clone fire buffer for rendering
            let fire_palette = fire.palette.to_vec();
            let fire_buffer = fire.pixel_buffer.to_vec();
            let fire_width = fire.width;
            let fire_height = fire.height;

            let pixels = {
                let mut buffer = vec![0u8; width * height * BYTES_PER_PIXEL];
                buffer
                    .par_chunks_mut(width * BYTES_PER_PIXEL)
                    .enumerate()
                    .for_each(|(wy, row)| {
                        let fy = wy / scale;
                        if fy < fire_height {
                            for fx in 0..fire_width {
                                let idx = fire_buffer[fy * fire_width + fx] as usize;
                                let color = fire_palette[idx];
                                let start_wx = fx * scale;
                                let end_wx = ((fx + 1) * scale).min(width);

                                let slice_start = start_wx * BYTES_PER_PIXEL;
                                let slice_end = end_wx * BYTES_PER_PIXEL;

                                if slice_end <= row.len() {
                                    for pixel_chunk in
                                        row[slice_start..slice_end].chunks_exact_mut(BYTES_PER_PIXEL)
                                    {
                                        pixel_chunk.copy_from_slice(&color);
                                    }
                                }
                            }
                        }
                    });
                buffer
            };

            // Now create the Pixbuf from the owned pixel vector
            let pixbuf = Pixbuf::from_bytes(
                &glib::Bytes::from_owned(pixels),
                Colorspace::Rgb,
                false, // no alpha channel
                8,     // bits per sample
                width as i32,
                height as i32,
                width as i32 * BYTES_PER_PIXEL as i32, // rowstride in bytes
            );

            // Update the GTK image widget
            picture.set_pixbuf(Some(&pixbuf));

            glib::ControlFlow::Continue
        }
    });
}

fn take_screenshots(last_screenshot: &mut HashMap<String, DynamicImage>, covered_outputs: &Vec<(String, bool)>) {
    for (name, _covered) in covered_outputs {
        eprintln!("[DEBUG] Taking screenshot for output: {}", name);
        if let Ok(output) = std::process::Command::new("grim")
        .args(["-o", name, "-"])
        .output()
        {
            if output.status.success() {
                if let Ok(i) = image::load_from_memory(&output.stdout) {
                    last_screenshot.insert(name.clone(), i);
                } else {
                    eprintln!("[DEBUG] Failed to decode screenshot for {}", name);
                }
            } else {
                eprintln!("[DEBUG] Screenshot command failed for {}", name);
            }
        }
    }
}