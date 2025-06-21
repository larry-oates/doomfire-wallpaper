mod doom_fire;
mod perlin;
mod render;
mod wallpaper;

use crate::doom_fire::DoomFire;
use crate::render::render_fire_frame;
use crate::wallpaper::{set_wallpaper};
use std::{thread, time::Duration, fs, time::Instant};

fn main() -> anyhow::Result<()> {
    let screen_width = 1920;
    let screen_height = 1080;
    let scale = 8;

    let mut fire = DoomFire::new(screen_width / scale, screen_height / scale);

    let cache_dir = dirs::home_dir()
        .expect("Could not find home directory")
        .join(".cache/hyprpaper");
    fs::create_dir_all(&cache_dir)?;

    let frame_path = cache_dir.join("fire_frame.bmp");

    loop {
        let start = Instant::now();

        let t0 = Instant::now();
        fire.update();
        println!("fire.update() took: {:?}", t0.elapsed());

        let t1 = Instant::now();
        render_fire_frame(&fire, &frame_path)?;
        println!("render_fire_frame() took: {:?}", t1.elapsed());

        let t2 = Instant::now();
        set_wallpaper(&frame_path)?;
        println!("set_wallpaper() took: {:?}", t2.elapsed());
        
        let total_time = start.elapsed();
        let sleep_time = Duration::from_millis(100) - total_time;
        println!("Total frame time: {:?}", total_time);
        println!("Sleeping for: {:?}", sleep_time);
        println!();
        if sleep_time > Duration::ZERO {
            thread::sleep(sleep_time);
        }
    }
}
