use crate::fire_types::{generate_palette, FireType};
use crate::config::Config;
use crate::particle::Particle;
use crate::perlin::perlin_noise_1d;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
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
    rng: SmallRng,
    cursor: Option<(usize, usize)>,
    mouse_fire_size: usize,
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
                let mut rng = SmallRng::from_os_rng();
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
            rng: SmallRng::from_os_rng(),
            cursor: None,
            mouse_fire_size: config.mouse_fire_size.unwrap_or(6).max(1),
        };
        doom_fire.initialize_fire();
        doom_fire
    }

    pub fn update(&mut self) {
        let rng = &mut self.rng;
        self.t += 0.03; // Increase frequency for more rapid wind changes
        let noise_val = perlin_noise_1d(self.t * 1.5);
        let jitter: f64 = rng.random_range(-0.5..=0.5);
        let wind = ((noise_val + jitter) * self.wind_strength).round() as isize;
        let delay_chance = 0.3;
        for y in (2..self.height).rev() {
            for x in 0..self.width {
                let src = y * self.width + x;
                let decay = if rng.random_bool(delay_chance) { 1 } else { 0 }; // Random decay factor
                let x_offset = rng.random_range(0..3) as isize - 1 + wind;
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

        // Re-ignite the fire directly under the mouse cursor (fire-cell
        // coordinates) so the flames follow the pointer each frame.
        if let Some((cx, cy)) = self.cursor {
            self.ignite_at(cx, cy);
        }

        // Animate Aurora palette by mutating it each frame
        if self.fire_type == FireType::Aurora {
            self.palette = generate_palette(self.fire_type, None, self.t as f32);
        }
    }

    /// Sets the fire-cell coordinates where the cursor fire should burn;
    /// None stops cursor ignition (e.g. while paused).
    pub fn set_cursor(&mut self, cell: Option<(usize, usize)>) {
        self.cursor = cell;
    }

    /// Paints a disc of heat centred on the cursor cell. The existing
    /// propagation pass then carries that heat upward, producing a plume
    /// that rises from the cursor. Heat falls off towards the disc edge,
    /// with slight random flicker for liveliness.
    fn ignite_at(&mut self, cx: usize, cy: usize) {
        let max = if self.palette.is_empty() {
            0
        } else {
            (self.palette.len() - 1) as u8
        };
        let half = (self.mouse_fire_size / 2) as i32;
        let half2 = half * half;
        for dy in -half..=half {
            for dx in -half..=half {
                let dist2 = dx * dx + dy * dy;
                if dist2 > half2 || half2 == 0 {
                    continue;
                }
                let x = cx as i32 + dx;
                let y = cy as i32 + dy;
                if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
                    continue;
                }
                let falloff = 1.0 - dist2 as f32 / half2 as f32;
                let heat = (falloff * max as f32) as u8;
                // Randomise the outer band so the disc edges flicker.
                let heat = if heat < max {
                    heat.max(self.rng.random_range(0..=max))
                } else {
                    heat
                };
                self.pixel_buffer[y as usize * self.width + x as usize] = heat;
            }
        }
    }

    pub fn initialize_fire(&mut self) {
        // Clear the pixel buffer
        self.pixel_buffer.iter_mut().for_each(|x| *x = 0);

        // Initialize the bottom row
        for x in 0..self.width {
            if self.fire_type == FireType::Candy {
                let rand: usize = self.rng.random_range(self.palette.len() / 2..self.palette.len());
                self.pixel_buffer[(self.height - 1) * self.width + x] = rand as u8;
            } else {
                // For other palettes, we start with the last color in the palette
                self.pixel_buffer[(self.height - 1) * self.width + x] = (self.palette.len() - 1) as u8;
            }
        }

        self.particles.clear(); // Clear particles on reset
    }

    // Set the paused screen for the fire - purely background color
    pub fn pause_fire(&mut self) {
        // Clear the pixel buffer
        self.pixel_buffer.iter_mut().for_each(|x| *x = 0);

        self.particles.clear(); 
    }
}
