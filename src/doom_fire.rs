use crate::fire_types::{generate_palette, FireType};
use crate::config::Config;
use crate::particle::Particle;
use crate::perlin::perlin_noise_1d;
use rand::rngs::ThreadRng;
use rand::Rng;
use strum::IntoEnumIterator;

pub struct DoomFire {
    pub width: usize,
    pub height: usize,
    pub pixel_buffer: Vec<u8>,
    pub palette: Vec<[u8; 3]>,
    pub fire_type: FireType,
    t: f64,
    pub particles: Vec<Particle>, // Add this field
    wind_strength: f64,
}

impl DoomFire {
    pub fn new(config: &Config) -> Self {
        let width = config.screen_width.unwrap_or(1920) / config.scale.unwrap_or(1);
        let height = config.screen_height.unwrap_or(1080) / config.scale.unwrap_or(1);
        let fire_type = match config.fire_type.as_deref() {
            Some("Blue") => FireType::Blue,
            Some("Rainbow") => FireType::Rainbow,
            Some("Green") => FireType::Green,
            Some("Purple") => FireType::Purple,
            Some("WhiteHot") => FireType::WhiteHot,
            Some("White") => FireType::White,
            Some("Ice") => FireType::Ice,
            Some("Toxic") => FireType::Toxic,
            Some("FireAndIce") => FireType::FireAndIce, 
            Some("ChemicalFire") => FireType::ChemicalFire,
            Some("Cyberpunk") => FireType::Cyberpunk,
            Some("Aurora") => FireType::Aurora,
            Some("Plasma") => FireType::Plasma,
            Some("Void") => FireType::Void,
            Some("Candy") => FireType::Candy,
            Some("Random") => {
                let variants: Vec<FireType> = FireType::iter().collect();
                let mut rng = ThreadRng::default();
                let idx = rng.random_range(0..variants.len());
                println!("Random fire type selected: {:?}", variants[idx]);
                variants[idx]
            }
            _ => FireType::Original,
        };
        let background_colour = config.background;
        let size = width * height;
        let pixel_buffer = vec![0; size];
        let palette = generate_palette(fire_type, background_colour, 0.0);

        let mut doom_fire = Self {
            width,
            height,
            pixel_buffer,
            palette,
            fire_type,
            t: 0.0,
            particles: Vec::new(),
            wind_strength: config.wind_strength.unwrap_or(1.0),
        };
        doom_fire.initialize_fire();
        doom_fire
    }

    pub fn update(&mut self) {
        let mut rng = ThreadRng::default();
        self.t += 0.03; // Increase frequency for more rapid wind changes
        let noise_val = perlin_noise_1d(self.t * 1.5);
        let jitter: f64 = rng.random_range(-0.5..=0.5);
        let wind = ((noise_val + jitter) * self.wind_strength).round() as isize;
        let delay_chance = 0.3;
        for y in (2..self.height).rev() {
            for x in 0..self.width {
                let src = y * self.width + x;
                let decay = if rng.random_bool(delay_chance) { 1 } else { 0 }; // Random decay factor
                let x_offset = rng.random_range(0..3) as i32 as isize - 1 + wind;
                let dst_x = x as isize + x_offset;
                let dst_y = if rng.random_bool(0.3) { y - 2 } else { y - 1 };

                if dst_x >= 0 && dst_x < self.width as isize {
                    let dst = dst_y * self.width + dst_x as usize;
                    let value = self.pixel_buffer[src].saturating_sub(decay);
                    self.pixel_buffer[dst] = value;
                }
            }
        }

        // Spawn new particles randomly at the bottom
        crate::particle::maybe_spawn_particle(
            &mut self.particles,
            self.fire_type,
            self.palette.len(),
            self.width,
            self.height,
        );

        // Update and render particles
        crate::particle::update_particles(
            &mut self.particles,
            &mut self.pixel_buffer,
            self.width,
            self.height,
        );

        // Animate Aurora palette by mutating it each frame
        if self.fire_type == FireType::Aurora {
            self.palette = generate_palette(self.fire_type, None, self.t as f32);
        }
    }

    pub fn initialize_fire(&mut self) {
        // Clear the pixel buffer
        self.pixel_buffer.iter_mut().for_each(|x| *x = 0);

        // Initialize the bottom row
        for x in 0..self.width {
            if self.fire_type == FireType::Candy {
                let mut rng = ThreadRng::default();
                let rand: usize = rng.random_range(self.palette.len() / 2..self.palette.len());
                self.pixel_buffer[(self.height - 1) * self.width + x] = rand as u8;
            } else {
                // For other palettes, we start with the last color in the palette
                self.pixel_buffer[(self.height - 1) * self.width + x] = (self.palette.len() - 1) as u8;
            }
        }

        self.particles.clear(); // Clear particles on reset
    }
}
