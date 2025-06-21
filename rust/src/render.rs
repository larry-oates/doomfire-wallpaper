use crate::doom_fire::DoomFire;
use image::{RgbImage, ImageFormat};
use std::path::Path;
use rayon::prelude::*;

pub fn render_fire_frame(fire: &DoomFire, path: &Path) -> anyhow::Result<()> {
    let width = fire.width;
    let height = fire.height;
    let mut img = RgbImage::new(width as u32, height as u32);
    let img_buf = img.as_mut();

    // Each row in the output image is width * 3 bytes
    img_buf
        .par_chunks_mut(width * 3)
        .enumerate()
        .for_each(|(y, row)| {
            for x in 0..width {
                let idx = y * fire.width + x;
                let color = fire.palette[fire.pixel_buffer[idx] as usize];
                let out_idx = x * 3;
                row[out_idx] = color[0];
                row[out_idx + 1] = color[1];
                row[out_idx + 2] = color[2];
            }
        });

    let mut file = std::fs::File::create(path)?;
    img.write_to(&mut file, ImageFormat::Bmp)?;
    Ok(())
}
