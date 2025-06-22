use crate::perlin::perlin_noise_1d;
use rand::Rng;

pub struct DoomFire {
    pub width: usize,
    pub height: usize,
    pub pixel_buffer: Vec<u8>,
    pub palette: Vec<[u8; 3]>,
    pub fire_type: FireType,
    t: f64,
}

impl DoomFire {
    pub fn new(
        width: usize,
        height: usize,
        fire_type: FireType,
        background_colour: Option<[u8; 3]>,
    ) -> Self {
        let size = width * height;
        let pixel_buffer = vec![0; size];
        let palette = generate_palette(fire_type, background_colour);

        let mut doom_fire = Self {
            width,
            height,
            pixel_buffer,
            palette,
            fire_type,
            t: 0.0,
        };
        doom_fire.initialize_fire();
        doom_fire
    }

    pub fn update(&mut self) {
        let mut rng = rand::thread_rng();
        self.t += 0.03; // Increase frequency for more rapid wind changes
        let noise_val = perlin_noise_1d(self.t * 1.5);
        let jitter: f64 = rng.gen_range(-0.5..=0.5);
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
    pub fn initialize_fire(&mut self) {
        for v in &mut self.pixel_buffer {
            *v = 0;
        }
        for x in 0..self.width {
            if self.fire_type == FireType::Rainbow {
                // For rainbow palette, we want to start with a random color
                let rand: usize = rand::thread_rng().gen_range(self.palette.len()/2..self.palette.len());
                self.pixel_buffer[(self.height - 1) * self.width + x] = rand as u8;
            } else {
                // For other palettes, we start with the last color in the palette
                self.pixel_buffer[(self.height - 1) * self.width + x] = (self.palette.len() - 1) as u8;
            }

        }
    }
}


#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq)]
pub enum FireType {
    Original,
    Blue,
    Rainbow,
    Green,
    Purple,
    WhiteHot,
}

fn generate_palette(palette: FireType, background_colour: Option<[u8; 3]>) -> Vec<[u8; 3]> {
    let mut pal = match palette {
        FireType::Original => (0..=36)
            .map(|i| {
                let t = i as f32 / 36.0;
                let r = (255.0 * t.sqrt()).min(255.0) as u8;
                let g = (255.0 * t.powi(3)).min(255.0) as u8;
                [r, g, 0]
            })
            .collect::<Vec<[u8; 3]>>(),
        FireType::Blue => (0..=36)
            .map(|i| {
                let t = i as f32 / 36.0;
                let b = (255.0 * t.sqrt()).min(255.0) as u8;
                let g = (128.0 * t.powi(2)).min(128.0) as u8;
                [0, g, b]
            })
            .collect::<Vec<[u8; 3]>>(),
        FireType::Rainbow => (0..=36)
            .map(|i| {
                let t = i as f32 / 36.0;
                let r = (255.0 * (0.5 + 0.5 * (t * 3.0).sin())).min(255.0) as u8;
                let g = (255.0 * (0.5 + 0.5 * (t * 3.0 + 2.0).sin())).min(255.0) as u8;
                let b = (255.0 * (0.5 + 0.5 * (t * 3.0 + 4.0).sin())).min(255.0) as u8;
                [r, g, b]
            })
            .collect::<Vec<[u8; 3]>>(),
        FireType::Green => (0..=36)
            .map(|i| {
                let t = i as f32 / 36.0;
                let g = (255.0 * t.sqrt()).min(255.0) as u8;
                let b = (128.0 * t.powi(2)).min(128.0) as u8;
                [0, g, b]
            })
            .collect::<Vec<[u8; 3]>>(),
        FireType::Purple => (0..=36)
            .map(|i| {
                let t = i as f32 / 36.0;
                let r = (128.0 * t.sqrt()).min(128.0) as u8;
                let b = (255.0 * t.powi(2)).min(255.0) as u8;
                [r, 0, b]
            })
            .collect::<Vec<[u8; 3]>>(),
        FireType::WhiteHot => (0..=36)
            .map(|i| {
                let t = i as f32 / 36.0;
                let v = (255.0 * t.sqrt()).min(255.0) as u8;
                [v, v, v]
            })
            .collect::<Vec<[u8; 3]>>(),
    };

    // Set the first entry to the background color if provided
    if let Some(bg) = background_colour {
        if let Some(first) = pal.first_mut() {
            *first = bg;
        }
    }

    pal
}
