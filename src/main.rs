mod doom_fire;
mod perlin;
mod wallpaper;

pub mod config;
pub mod fire_types;
pub mod particle;

use crate::config::{Config, ScreenBurn};
use crate::doom_fire::DoomFire;
use crate::wallpaper::{get_cursor_position, get_outputs_covered, is_system_sleeping};
use anyhow::{Context, Result};
use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Picture};
use image::{DynamicImage, GenericImageView};
use rayon::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::path::PathBuf;

const BYTES_PER_PIXEL: usize = 3; // RGB = 3 bytes

const FONT: [[[u8; 3]; 5]; 10] = [
    [[1, 1, 1], [1, 0, 1], [1, 0, 1], [1, 0, 1], [1, 1, 1]], // 0
    [[0, 1, 0], [0, 1, 0], [0, 1, 0], [0, 1, 0], [0, 1, 0]], // 1
    [[1, 1, 1], [0, 0, 1], [1, 1, 1], [1, 0, 0], [1, 1, 1]], // 2
    [[1, 1, 1], [0, 0, 1], [1, 1, 1], [0, 0, 1], [1, 1, 1]], // 3
    [[1, 0, 1], [1, 0, 1], [1, 1, 1], [0, 0, 1], [0, 0, 1]], // 4
    [[1, 1, 1], [1, 0, 0], [1, 1, 1], [1, 0, 1], [1, 1, 1]], // 5
    [[1, 1, 1], [1, 0, 0], [1, 1, 1], [1, 0, 1], [1, 1, 1]], // 6
    [[1, 1, 1], [0, 0, 1], [0, 0, 1], [0, 0, 1], [0, 0, 1]], // 7
    [[1, 1, 1], [1, 0, 1], [1, 1, 1], [1, 0, 1], [1, 1, 1]], // 8
    [[1, 1, 1], [1, 0, 1], [1, 1, 1], [0, 0, 1], [1, 1, 1]], // 9
];

fn main() -> Result<()> {
    run_wallpaper()
}

/// Creates the default config file if it doesn't exist.
fn ensure_config_exists() -> Result<()> {
    let config_path = get_config_path()?;
    if !config_path.exists() {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory at {parent:?}"))?;
        }
        let mut file = fs::File::create(&config_path)
            .with_context(|| format!("Failed to create config file at {config_path:?}"))?;
        file.write_all(DEFAULT_CONFIG.as_bytes())
            .context("Failed to write default config")?;
        println!("Created default config at {config_path:?}");
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
# screen_burn = false # true, false, or "image"
# image_path = "/path/to/image.png"
# wind_strength = 0.5
# show_fps = false
# mouse_follow = true
# mouse_fire_size = 6
"#;

/// Commands sent from the UI loop to the render worker.
enum FireCommand {
    /// Pause the fire. `clear` empties the fire (restart_on_pause),
    /// `burn` is an optional screenshot to burn into the frame. Renders once.
    Pause {
        clear: bool,
        burn: Option<DynamicImage>,
    },
    /// Advance and render one fire frame.
    Update {
        init: bool,
        burn: Option<DynamicImage>,
        fps_value: u32,
        /// Latest cursor position in fire-cell coordinates, if mouse
        /// following is enabled.
        pointer: Option<(usize, usize)>,
    },
}

/// UI-side state shared with the frame-clock tick callback (which requires a
/// `Fn` closure, so mutation is behind Rc<RefCell<_>>).
struct UiState {
    last_screenshot: HashMap<String, DynamicImage>,
    was_paused: bool,
    last_screenshot_time: Instant,
    frame_count: u32,
    current_fps: u32,
    last_fps_update: Instant,
    cached_outputs: Vec<(String, bool)>,
    last_output_check: Instant,
    sleeping: bool,
    last_sleep_check: Instant,
    vsync_credit: f64,
}

/// Runs the GTK application and the wallpaper animation loop.
fn run_wallpaper() -> Result<()> {
    ensure_config_exists()?;

    // The default GSK renderer (Vulkan-capable) cannot repaint a full-screen
    // 6MB texture per frame within the vsync budget on this setup, starving
    // the frame clock and causing stutter. Force the OpenGL renderer, which
    // handles this comfortably.
    if std::env::var_os("GSK_RENDERER").is_none() {
        // Safe: no other threads are running yet.
        let gl = if gtk::minor_version() >= 16 { "gl" } else { "ngl" };
        unsafe { std::env::set_var("GSK_RENDERER", gl) };
    }

    let app = Application::new(Some("com.leafman.doomfirewallpaper"), Default::default());
    app.connect_activate(build_ui);
    app.run();
    Ok(())
}

/// Builds the UI and sets up the animation timer.
fn build_ui(app: &Application) {
    println!("App Connected!");
    let config = Config::load();
    println!("Using config: {config:?}");

    let restart_on_pause = config.restart_on_pause.unwrap_or(false);
    let fps = config.fps.unwrap_or(10);
    let pause_on_cover = config.pause_on_cover.unwrap_or(false);
    let screen_burn_config = config.screen_burn.clone().unwrap_or(ScreenBurn::Bool(false));
    let show_fps = config.show_fps.unwrap_or(false);
    let height = config.screen_height.unwrap();
    let width = config.screen_width.unwrap();
    let scale = config.scale.unwrap_or(1);
    let background_color = config.background.unwrap_or([0, 0, 0]);
    let mouse_follow = config.mouse_follow.unwrap_or(true);
    let fire_width = width / scale;
    let fire_height = height / scale;

    let use_static_burn_image = matches!(screen_burn_config, ScreenBurn::String(ref s) if s == "image");
    let do_screen_burn = use_static_burn_image || matches!(screen_burn_config, ScreenBurn::Bool(true));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Doom Fire Wallpaper")
        .build();

    let picture = Picture::new();
    window.set_child(Some(&picture));
    window.present();

    // Load background image if provided
    let bg_image = if let Some(path_str) = &config.image_path {
        let path = resolve_path(path_str);
        match image::open(&path) {
            Ok(img) => Some(img.resize_exact(width as u32, height as u32, image::imageops::FilterType::Triangle)),
            Err(e) => {
                eprintln!("Failed to load image at {path:?}: {e}");
                None
            }
        }
    } else {
        None
    };

    // Convert bg_image to raw bytes for faster access in the render loop
    let bg_image_bytes = bg_image.as_ref().map(|img| img.to_rgb8().into_raw());
    // Keep a copy for the static burn-in screenshot handling on the UI thread
    let bg_image_main = bg_image;
    let show_bg_image = !do_screen_burn && bg_image_main.is_some();

    // Channel to receive screenshots taken on a background thread without blocking the UI
    let (screenshot_tx, screenshot_rx) = mpsc::channel::<Vec<(String, DynamicImage)>>();

    // Move fire simulation + rendering off the UI thread. The worker owns the
    // fire state; the UI loop only sends commands and displays finished frames.
    let fire = DoomFire::new(&config);

    let (frame_tx, frame_rx) = mpsc::channel::<Vec<u8>>();
    let (cmd_tx, cmd_rx) = mpsc::sync_channel::<FireCommand>(3);

    // Poll the pointer position on a background thread so the (blocking)
    // hyprctl call never stalls the UI loop; positions flow into the tick
    // callback through a channel and are forwarded to the worker command.
    let (pos_tx, pos_rx) = mpsc::channel::<(i32, i32)>();
    if mouse_follow {
        std::thread::Builder::new()
            .name("doomfire-cursor".to_owned())
            .spawn(move || loop {
                if let Some(pos) = get_cursor_position() {
                    if pos_tx.send(pos).is_err() {
                        break; // UI gone
                    }
                }
                std::thread::sleep(Duration::from_millis(50));
            })
            .context("Failed to spawn cursor polling thread")
            .expect("cursor thread");
    }

    std::thread::Builder::new()
        .name("doomfire-render".to_owned())
        .spawn(move || {
            worker_loop(
                fire,
                frame_tx,
                cmd_rx,
                width,
                height,
                scale,
                show_bg_image,
                bg_image_bytes,
                show_fps,
                background_color,
            );
        })
        .context("Failed to spawn doomfire worker thread")
        .expect("worker thread");

    let ui_state = Rc::new(RefCell::new(UiState {
        last_screenshot: HashMap::new(),
        was_paused: false,
        last_screenshot_time: Instant::now(),
        frame_count: 0,
        current_fps: 0,
        last_fps_update: Instant::now(),
        // Cache the (expensive) compositor/system state queries so they are not
        // re-spawned on every animation tick.
        cached_outputs: get_outputs_covered(),
        last_output_check: Instant::now(),
        sleeping: false,
        last_sleep_check: Instant::now(),
        // Credit pacing starts with one free command on the first tick.
        vsync_credit: 1.0,
    }));

    // Display + pacing happen on the widget's frame clock (vsync-aligned)
    // instead of a glib timeout, so frames are shown immediately when they
    // arrive and never shifted by a whole timer tick.
    let tick_ui = ui_state.clone();
    picture.add_tick_callback(move |picture, _frame_clock| {
        let ui: &mut UiState = &mut tick_ui.borrow_mut();

        // Show the most recent frame produced by the worker (drop stale ones).
        let mut latest_frame: Option<Vec<u8>> = None;
        while let Ok(frame) = frame_rx.try_recv() {
            latest_frame = Some(frame);
        }
        if let Some(pixels) = latest_frame {
            if show_fps {
                ui.frame_count += 1;
                if ui.last_fps_update.elapsed() >= Duration::from_secs(1) {
                    ui.current_fps = ui.frame_count;
                    ui.frame_count = 0;
                    ui.last_fps_update = Instant::now();
                }
            }
            // Upload the buffer directly as a texture; skips the pixbuf
            // intermediate copy + GSK pixbuf->texture conversion.
            let texture = gtk::gdk::MemoryTexture::new(
                width as i32,
                height as i32,
                gtk::gdk::MemoryFormat::R8g8b8,
                &gtk::glib::Bytes::from_owned(pixels),
                width * BYTES_PER_PIXEL,
            );
            picture.set_paintable(Some(&texture));
        }

        // Pace fire updates to min(configured fps, refresh rate) using fractional
        // credits per vsync tick, so 60fps on a 60Hz screen fires every vsync
        // instead of erratically skipping half the ticks.
        let refresh = _frame_clock.fps();
        let refresh = if refresh > 1.0 { refresh } else { 60.0 };
        let effective_fps = (fps as f64).min(refresh);
        ui.vsync_credit += effective_fps / refresh;
        if ui.vsync_credit < 1.0 {
            return gtk::glib::ControlFlow::Continue;
        }

        // Drain any screenshots that were produced by background threads
        while let Ok(batch) = screenshot_rx.try_recv() {
            for (name, img) in batch {
                ui.last_screenshot.insert(name, img);
            }
        }

        // Keep only the latest cursor position (drop stale ones) and convert
        // global screen coordinates into fire-cell coordinates.
        let mut pointer = None;
        while let Ok(pos) = pos_rx.try_recv() {
            pointer = Some(pos);
        }
        let pointer = pointer.map(|(x, y)| {
            let fx = (x.max(0) as usize / scale).min(fire_width.max(1) - 1);
            let fy = (y.max(0) as usize / scale).min(fire_height.max(1) - 1);
            (fx, fy)
        });

        // Refresh compositor/system state at a low rate to avoid spawning
        // hyprctl/systemctl processes on every animation tick.
        if ui.last_output_check.elapsed() >= Duration::from_millis(500) {
            ui.cached_outputs = get_outputs_covered();
            ui.last_output_check = Instant::now();
        }
        if ui.last_sleep_check.elapsed() >= Duration::from_secs(1) {
            ui.sleeping = is_system_sleeping();
            ui.last_sleep_check = Instant::now();
        }

        let all_covered = ui.cached_outputs.iter().all(|(_, c)| *c);
        let paused = (pause_on_cover && all_covered) || ui.sleeping;

        if paused {
            let just_paused = !ui.was_paused;
            if just_paused {
                if restart_on_pause {
                    eprintln!("[DEBUG] Fire paused and reset");
                } else {
                    eprintln!("[DEBUG] Fire paused (frozen)");
                }
                let burn =
                    take_burn_screenshot(&mut ui.last_screenshot, &ui.cached_outputs, do_screen_burn);
                let _ = cmd_tx.send(FireCommand::Pause { clear: restart_on_pause, burn });
                ui.was_paused = true;
                return gtk::glib::ControlFlow::Continue;
            }

            if use_static_burn_image {
                if let Some(img) = &bg_image_main {
                    for (name, covered) in &ui.cached_outputs {
                        if *covered && !ui.last_screenshot.contains_key(name) {
                            ui.last_screenshot.insert(name.clone(), img.clone());
                        }
                    }
                }
            }

            // While paused, take screenshots periodically.
            if do_screen_burn
                && !use_static_burn_image
                && ui.last_screenshot_time.elapsed() >= Duration::from_millis(500)
            {
                ui.last_screenshot_time = Instant::now();
                eprintln!("[DEBUG] Taking screenshot while paused");
                // Spawn screenshots on background thread
                let tx = screenshot_tx.clone();
                rayon::spawn(move || {
                    let results = take_screenshots_sync(&get_outputs_covered());
                    let _ = tx.send(results);
                });
            }

            ui.was_paused = true;
            return gtk::glib::ControlFlow::Continue;
        }

        let init = ui.was_paused && restart_on_pause;
        if ui.was_paused {
            eprintln!("[DEBUG] Fire unpaused");
        }
        let burn = take_burn_screenshot(&mut ui.last_screenshot, &ui.cached_outputs, do_screen_burn);
        ui.was_paused = false;
        // If the worker is behind, skip this tick's update instead of growing the queue.
        match cmd_tx.try_send(FireCommand::Update { init, burn, fps_value: ui.current_fps, pointer }) {
            Ok(()) => ui.vsync_credit -= 1.0,
            Err(mpsc::TrySendError::Full(_)) => {}
            Err(mpsc::TrySendError::Disconnected(_)) => return gtk::glib::ControlFlow::Break,
        }

        gtk::glib::ControlFlow::Continue
    });
}

/// Owns the fire state on a background thread: simulates and renders one frame
/// per command, sends finished pixel buffers to the UI thread.
#[allow(clippy::too_many_arguments)]
fn worker_loop(
    mut fire: DoomFire,
    frame_tx: mpsc::Sender<Vec<u8>>,
    cmd_rx: mpsc::Receiver<FireCommand>,
    width: usize,
    height: usize,
    scale: usize,
    show_bg_image: bool,
    bg_image_bytes: Option<Vec<u8>>,
    show_fps: bool,
    background_color: [u8; 3],
) {
    while let Ok(cmd) = cmd_rx.recv() {
        match cmd {
            FireCommand::Pause { clear, burn } => {
                if clear {
                    fire.pause_fire();
                }
                // Stop cursor ignition while paused so its heat doesn't
                // freeze into the frozen frame.
                fire.set_cursor(None);
                if let Some(img) = burn {
                    apply_burn(&mut fire, &img, background_color);
                }
                send_frame(&fire, &frame_tx, width, height, scale, show_bg_image, bg_image_bytes.as_deref());
            }
            FireCommand::Update { init, burn, fps_value, pointer } => {
                if init {
                    fire.initialize_fire();
                }
                fire.set_cursor(pointer);
                fire.update();
                if show_fps {
                    draw_fps(&mut fire, fps_value);
                }
                if let Some(img) = burn {
                    apply_burn(&mut fire, &img, background_color);
                }
                send_frame(&fire, &frame_tx, width, height, scale, show_bg_image, bg_image_bytes.as_deref());
            }
        }
    }
}

/// Renders the fire state into a fresh RGB pixel buffer and sends it to the UI.
fn send_frame(
    fire: &DoomFire,
    frame_tx: &mpsc::Sender<Vec<u8>>,
    width: usize,
    height: usize,
    scale: usize,
    show_bg_image: bool,
    bg_image_bytes: Option<&[u8]>,
) {
    let fire_palette = &fire.palette;
    let fire_buffer = &fire.pixel_buffer;
    let fire_width = fire.width;
    let fire_height = fire.height;

    let mut pixels = vec![0u8; width * height * BYTES_PER_PIXEL];
    pixels
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
                        if show_bg_image && idx == 0 {
                            if let Some(bg_bytes) = bg_image_bytes {
                                let bg_row_start = wy * width * BYTES_PER_PIXEL;
                                let bg_slice = &bg_bytes[bg_row_start + slice_start .. bg_row_start + slice_end];
                                row[slice_start..slice_end].copy_from_slice(bg_slice);
                            }
                        } else {
                            // Write the color pattern without allocating
                            // (the old `color.repeat()` heap-allocated per fire pixel).
                            let mut p = slice_start;
                            while p + BYTES_PER_PIXEL <= slice_end {
                                row[p] = color[0];
                                row[p + 1] = color[1];
                                row[p + 2] = color[2];
                                p += BYTES_PER_PIXEL;
                            }
                        }
                    }
                }
            }
        });

    let _ = frame_tx.send(pixels);
}

/// Pops a screenshot to burn into the fire, if burn mode is active and some
/// output is uncovered.
fn take_burn_screenshot(
    last_screenshot: &mut HashMap<String, DynamicImage>,
    covered_outputs: &[(String, bool)],
    do_screen_burn: bool,
) -> Option<DynamicImage> {
    if !do_screen_burn {
        return None;
    }
    if let Some((name, _)) = covered_outputs.iter().find(|(_, c)| !*c) {
        if let Some(img) = last_screenshot.remove(name) {
            last_screenshot.clear();
            return Some(img);
        }
    }
    None
}

/// Maxes the fire buffer with the brightness of the given screenshot so that
/// bright areas leave a burning trail.
fn apply_burn(fire: &mut DoomFire, screenshot: &DynamicImage, background_color: [u8; 3]) {
    let resized = screenshot.resize_exact(
        fire.width as u32,
        fire.height as u32,
        image::imageops::FilterType::Triangle,
    );
    for y in 0..fire.height {
        for x in 0..fire.width {
            let px = resized.get_pixel(x as u32, y as u32);
            let r_diff = (px[0] as i32 - background_color[0] as i32).abs();
            let g_diff = (px[1] as i32 - background_color[1] as i32).abs();
            let b_diff = (px[2] as i32 - background_color[2] as i32).abs();
            let distance = (r_diff + g_diff + b_diff) as f32;
            let max_dist = 255.0 * 3.0;
            let idx = ((distance / max_dist) * (fire.palette.len() as f32 - 1.0)).round() as u8;
            let current_idx = y * fire.width + x;
            let fire_idx = &mut fire.pixel_buffer[current_idx];
            *fire_idx = (*fire_idx).max(idx);
        }
    }
    eprintln!("[DEBUG] Burn-in applied from screenshot");
}

fn take_screenshots_sync(covered_outputs: &Vec<(String, bool)>) -> Vec<(String, DynamicImage)> {
    let mut results = Vec::new();
    for (name, _covered) in covered_outputs {
        eprintln!("[DEBUG] Taking screenshot for output: {name}");
        if let Ok(output) = std::process::Command::new("grim")
            .args(["-o", name, "-"])
            .output()
        {
            if output.status.success() {
                if let Ok(i) = image::load_from_memory(&output.stdout) {
                    results.push((name.clone(), i));
                } else {
                    eprintln!("[DEBUG] Failed to decode screenshot for {name}");
                }
            } else {
                eprintln!("[DEBUG] Screenshot command failed for {name}");
            }
        }
    }
    results
}

fn draw_fps(fire: &mut DoomFire, fps: u32) {
    let fps_str = fps.to_string();
    let char_width = 3;
    let char_height = 5;
    let spacing = 1;
    let padding_right = 5;
    let padding_top = 20;

    let total_width = fps_str.len() * (char_width + spacing) - spacing;

    if fire.width < total_width + padding_right {
        return;
    }

    let start_x = fire.width - total_width - padding_right;
    let start_y = padding_top;
    let max_heat = if fire.palette.is_empty() {
        255
    } else {
        (fire.palette.len() - 1) as u8
    };

    for (i, c) in fps_str.chars().enumerate() {
        if let Some(digit) = c.to_digit(10) {
            let digit = digit as usize;
            let offset_x = start_x + i * (char_width + spacing);
            for dy in 0..char_height {
                for dx in 0..char_width {
                    if FONT[digit][dy][dx] == 1 {
                        let x = offset_x + dx;
                        let y = start_y + dy;
                        if y < fire.height && x < fire.width {
                            fire.pixel_buffer[y * fire.width + x] = max_heat;
                        }
                    }
                }
            }
        }
    }
}

fn resolve_path(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }
    PathBuf::from(path)
}
