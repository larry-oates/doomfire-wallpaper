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
use clap::{Parser, Subcommand};
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
use std::process::Command;
use std::rc::Rc;

const BYTES_PER_PIXEL: usize = 3; // RGB = 3 bytes
const SERVICE_NAME: &str = "doom-fire-wallpaper.service";

/// A DOOM-style fire wallpaper for Hyprland/Hyprpaper
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Set up the configuration and systemd service
    Setup,
    /// Restart the wallpaper service to apply changes
    Refresh,
    /// Stop and disable the wallpaper service
    Stop,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Setup) => setup(),
        Some(Commands::Refresh) => refresh(),
        Some(Commands::Stop) => stop(),
        None => run_wallpaper(),
    }
}

/// Sets up the default configuration and enables the systemd service.
fn setup() -> Result<()> {
    println!("Setting up doom-fire-wallpaper...");

    // 1. Create config directory and default config file
    let config_path = get_config_path()?;
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory at {:?}", parent))?;
    }

    if !config_path.exists() {
        let mut file = fs::File::create(&config_path)
            .with_context(|| format!("Failed to create config file at {:?}", config_path))?;
        file.write_all(DEFAULT_CONFIG.as_bytes())
            .context("Failed to write default config")?;
        println!("Created default config at {:?}", config_path);
    } else {
        println!("Config file already exists at {:?}", config_path);
    }

    // 2. Enable and start the systemd user service
    println!("\nEnabling and starting systemd user service...");
    run_systemctl(&["--user", "daemon-reload"])?;
    run_systemctl(&["--user", "enable", "--now", SERVICE_NAME])?;

    println!("\nSetup complete!");
    println!("The wallpaper service is now running.");
    println!("You can customize the settings in {:?}", config_path);
    println!("Remember to run 'doom-fire-wallpaper refresh' after changing the config.");

    Ok(())
}

/// Restarts the systemd service.
fn refresh() -> Result<()> {
    println!("Refreshing doom-fire-wallpaper service...");
    run_systemctl(&["--user", "restart", SERVICE_NAME])?;
    println!("Service restarted.");
    Ok(())
}

/// Stops and disables the systemd service.
fn stop() -> Result<()> {
    println!("Stopping and disabling doom-fire-wallpaper service...");
    run_systemctl(&["--user", "stop", SERVICE_NAME])?;
    run_systemctl(&["--user", "disable", SERVICE_NAME])?;
    println!("Service stopped and disabled.");
    Ok(())
}

/// Helper to run systemctl commands.
fn run_systemctl(args: &[&str]) -> Result<()> {
    let status = Command::new("systemctl")
        .args(args)
        .status()
        .context("Failed to execute systemctl command. Is systemd running?")?;

    if !status.success() {
        anyhow::bail!("systemctl command failed: `systemctl {}`", args.join(" "));
    }
    Ok(())
}

/// Helper to get the path to the config file.
fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().context("Could not find config directory")?;
    Ok(config_dir.join("doom-fire-wallpaper/config.toml"))
}

const DEFAULT_CONFIG: &str = r#"# Default config for doom-fire-wallpaper
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
"#;

/// Runs the GTK application and the wallpaper animation loop.
fn run_wallpaper() -> Result<()> {
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

    timeout_add_local(std::time::Duration::from_millis(1000 / fps as u64), {
        let picture = picture.clone();
        let fire = fire.clone();
        let mut was_paused = false;

        move || {
            let mut fire = fire.borrow_mut();

            let covered_outputs = get_outputs_covered();
            let all_covered = covered_outputs.iter().all(|(_, c)| *c);
            let sleeping = is_system_sleeping();
            let paused = (pause_on_cover && all_covered) || sleeping;

            // Transition detection: just entered paused state
            let just_paused = paused && !was_paused;
            was_paused = paused;

            if screen_burn && all_covered {
                screenshot_count += 1;
                if screenshot_count == 24 {
                    screenshot_count = 0;
                    for (name, _covered) in &covered_outputs {
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
            }

            if paused {
                if just_paused {
                    eprintln!("[DEBUG] Fire paused (first frame)");

                    if restart_on_pause {
                        fire.initialize_fire();
                    }
                    // Allow render to happen once
                } else {
                    eprintln!("[DEBUG] Fire paused (skipping render)");
                    return glib::ControlFlow::Continue;
                }
            } else {
                // Not paused, normal update
                fire.update();
            }

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
