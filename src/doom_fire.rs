use crate::perlin::perlin_noise_1d;
use rand::Rng;
use serde::Deserialize;
use strum_macros::EnumIter;
use strum::IntoEnumIterator;

pub struct DoomFire {
    pub width: usize,
    pub height: usize,
    pub pixel_buffer: Vec<u8>,
    pub palette: Vec<[u8; 3]>,
    pub fire_type: FireType,
    t: f64,
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
                let idx = rand::random::<usize>() % variants.len();
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

        // Animate Aurora palette by mutating it each frame
        if self.fire_type == FireType::Aurora {
            self.palette = generate_palette(self.fire_type, None, self.t as f32);
        }
    }

    pub fn initialize_fire(&mut self) {
        // Clear the pixel buffer
        self.pixel_buffer.iter_mut().for_each(|x| *x = 0);

        // Draw fire type name as an overlay in the fire buffer
        let name = format!("{:?}", self.fire_type);
        let chars: Vec<char> = name.chars().collect();
        let text_width = chars.len();
        let y = self.height.saturating_sub(4); // 4 rows from the bottom
        let x_offset = (self.width.saturating_sub(text_width)) / 2;

        for (i, c) in chars.iter().enumerate() {
            if *c != ' ' {
                let idx = y * self.width + x_offset + i;
                if idx < self.pixel_buffer.len() {
                    // Use a mid-high palette value for visibility
                    self.pixel_buffer[idx] = (self.palette.len() as u8 * 3 / 4).max(1);
                }
            }
        }

        // Initialize the bottom row as usual
        for x in 0..self.width {
            if self.fire_type == FireType::Candy {
                let rand: usize = rand::thread_rng().gen_range(self.palette.len() / 2..self.palette.len());
                self.pixel_buffer[(self.height - 1) * self.width + x] = rand as u8;
            } else {
                // For other palettes, we start with the last color in the palette
                self.pixel_buffer[(self.height - 1) * self.width + x] = (self.palette.len() - 1) as u8;
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, EnumIter, Debug)]
pub enum FireType {
    Original,
    Blue,
    Rainbow,
    Green,
    Purple,
    WhiteHot,
    Ice,
    Toxic,
    FireAndIce,
    ChemicalFire,
    Cyberpunk,
    Aurora,
    Plasma,
    Void,
    Candy,
}

fn generate_palette(fire_type: FireType, background_colour: Option<[u8; 3]>, phase: f32) -> Vec<[u8; 3]> {
    let mut pal = match fire_type {
        FireType::FireAndIce => (0..=36)
        .map(|i| {
            let t = i as f32 / 36.0;
            // Make the fire less purple: reduce blue and green in the fire region, keep blue strong in ice
            let r = (255.0 * t.powf(1.2) + 60.0 * t).min(255.0) as u8; // Brighter, deeper red
            let g = (80.0 * t + 60.0 * (1.0 - t)).min(140.0) as u8;    // Lower green in fire region
            let b = (255.0 * (1.0 - t) + 20.0 * t).min(255.0) as u8;   // Lower blue in fire region
            [r, g, b]
        })
        .collect(),
        FireType::ChemicalFire => (0..=36)
            .map(|i| {
                let t = i as f32 / 36.0;
                // True ChemicalFire: deep purple, magenta, orange, yellow
                let r = (255.0 * t).min(255.0) as u8;
                let g = (80.0 * (1.0 - t) + 120.0 * t).min(200.0) as u8;
                let b = (180.0 * (1.0 - t).powi(2)).min(180.0) as u8;
                // Add magenta/purple at the low end
                let r = if t < 0.3 { (180.0 + 75.0 * t / 0.3) as u8 } else { r };
                let b = if t < 0.3 { (120.0 + 135.0 * (0.3 - t) / 0.3) as u8 } else { b };
                [r, g, b]
            })
            .collect(),
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
        FireType::Candy => (0..=36)
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
        FireType::Ice => (0..=36)
            .map(|i| {
                let t = i as f32 / 36.0;
                let b = (200.0 + 55.0 * t).min(255.0) as u8;
                let g = (220.0 * t.powi(2)).min(220.0) as u8;
                let r = (180.0 * t.powi(3)).min(180.0) as u8;
                [r, g, b]
            })
            .collect(),
        FireType::Toxic => (0..=36)
            .map(|i| {
                let t = i as f32 / 36.0;
                let g = (255.0 * t.sqrt()).min(255.0) as u8;
                let r = (128.0 * t.powi(2)).min(128.0) as u8;
                let b = (64.0 * (1.0 - t)).max(0.0) as u8;
                [r, g, b]
            })
            .collect(),
        FireType::Cyberpunk => (0..=36)
            .map(|i| {
                let t = i as f32 / 36.0;
                let r = (255.0 * (0.5 + 0.5 * (t * 2.0).sin())).min(255.0) as u8;
                let g = (0.0) as u8;
                let b = (255.0 * (0.5 + 0.5 * (t * 2.0 + 2.0).sin())).min(255.0) as u8;
                [r, g, b]
            })
            .collect(),
        FireType::Aurora => {
            // Animated/mutating palette: shift hues over time using phase
            let phase = phase;
            (0..=36)
                .map(|i| {
                    let t = i as f32 / 36.0;
                    let r = (60.0 * (0.5 + 0.5 * ((t * 6.0 + 1.0 + phase).sin()))).max(0.0) as u8;
                    let g = (180.0 * (0.5 + 0.5 * ((t * 2.5 + 2.0 + phase * 0.7).sin())) + 60.0).min(255.0) as u8;
                    let b = (200.0 * (0.5 + 0.5 * ((t * 3.5 + 4.0 + phase * 1.3).cos()))).min(255.0) as u8;
                    [r, g, b]
                })
                .collect()
        },
        FireType::Plasma => (0..=36)
            .map(|i| {
                let t = i as f32 / 36.0;
                if t < 0.08 {
                    [0, 0, 0]
                } else {
                    // Add more white patches by boosting all channels at certain intervals
                    let plasma_white = ((t * 8.0).sin().abs() > 0.85) || ((t * 4.0).cos().abs() > 0.95);
                    if plasma_white {
                        [255, 255, 255]
                    } else {
                        let r = (180.0 * (0.5 + 0.5 * (t * 6.0).sin())).min(255.0) as u8;
                        let g = (80.0 * (0.5 + 0.5 * (t * 4.0 + 1.0).cos())).min(255.0) as u8;
                        let b = (200.0 + 55.0 * (0.5 + 0.5 * (t * 8.0).sin())).min(255.0) as u8;
                        [r, g, b]
                    }
                }
            })
            .collect(),
        FireType::Void => (0..=36)
            .map(|i| {
                let t = i as f32 / 36.0;
                let r = (10.0 * (1.0 - t)).max(0.0) as u8;
                let g = (10.0 * (1.0 - t)).max(0.0) as u8;
                let b = (40.0 + 80.0 * t.powi(2)).min(120.0) as u8;
                [r, g, b]
            })
            .collect(),
        FireType::Rainbow => (0..=36)
            .map(|i| {
                let t = i as f32 / 36.0;
                let r = (200.0 * (0.5 + 0.5 * (t * 8.0).sin()) + 55.0).min(255.0) as u8;
                let g = (200.0 * (0.5 + 0.5 * (t * 8.0 + 2.0).sin()) + 55.0).min(255.0) as u8;
                let b = (200.0 * (0.5 + 0.5 * (t * 8.0 + 4.0).sin()) + 55.0).min(255.0) as u8;
                [r, g, b]
            })
            .collect(),
    };

    // Set the first entry to the background color if provided
    if let Some(bg) = background_colour {
        if let Some(first) = pal.first_mut() {
            *first = bg;
        }
    }

    pal
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub screen_width: Option<usize>,
    pub screen_height: Option<usize>,
    pub scale: Option<usize>,
    pub fps: Option<u32>,
    pub output: Option<String>,
    pub fire_type: Option<String>,
    pub background: Option<[u8; 3]>,
    pub restart_on_pause: Option<bool>,
}

impl Config {
    pub fn load() -> Self {
        let config_path = dirs::home_dir()
            .expect("Could not find home directory")
            .join(".config/doom-fire-wallpaper/config.toml");
        let config_str = std::fs::read_to_string(config_path).unwrap_or_default();
        let config: Self = toml::from_str(&config_str).unwrap_or_default();
        println!("Loaded {:?}", config);
        config
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            screen_width: Some(1920),
            screen_height: Some(1080),
            scale: Some(4),
            fps: Some(23),
            output: Some(String::new()),
            fire_type: Some("Original".to_string()),
            background: None,
            restart_on_pause: Some(true),
        }
    }
}
