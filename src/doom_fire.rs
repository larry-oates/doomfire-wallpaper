use crate::perlin::perlin_noise_1d;
use rand::Rng;

pub struct DoomFire {
    pub width: usize,
    pub height: usize,
    pub pixel_buffer: Vec<u8>,
    pub palette: Vec<[u8; 3]>,
    t: f64,
}

impl DoomFire {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        let mut pixel_buffer = vec![0; size];
        let palette = generate_palette();

        for x in 0..width {
            pixel_buffer[(height - 1) * width + x] = (palette.len() - 1) as u8;
        }

        Self {
            width,
            height,
            pixel_buffer,
            palette,
            t: 0.0,
        }
    }

    pub fn update(&mut self) {
        let mut rng = rand::thread_rng();
        self.t += 0.03; // Increase frequency for more rapid wind changes
        let noise_val = perlin_noise_1d(self.t * 1.5); // Higher frequency
        let jitter: f64 = rng.gen_range(-0.5..=0.5);   // Add some randomness
        let wind = ((noise_val + jitter) * 1.0).round() as isize;
        let delay_chance = 0.3; 
        for y in (2..self.height).rev() {
            for x in 0..self.width {
                let src = y * self.width + x;
                let decay = rng.gen_bool(delay_chance) as u8; // Random decay factor
                let x_offset = rng.gen_range(0..3) as isize - 1 + wind;
                let dst_x = x as isize + x_offset;
                let dst_y = if rng.gen_bool(0.3) { y - 2 } else { y - 1 };

                if dst_x >= 0 && dst_x < self.width as isize && dst_y > 0 {
                    let dst = dst_y * self.width + dst_x as usize;
                    let value = self.pixel_buffer[src].saturating_sub(decay);
                    self.pixel_buffer[dst] = value;
                }
            }
        }
    }
}

fn generate_palette() -> Vec<[u8; 3]> {
    let mut p = Vec::new();
    for i in 0..=36 {
        let t = i as f32 / 36.0;
        let r = (255.0 * t.sqrt()).min(255.0) as u8;
        let g = (255.0 * t.powi(3)).min(255.0) as u8;
        p.push([r, g, 0]);
    }
    p
}
