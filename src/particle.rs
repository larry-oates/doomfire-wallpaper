use crate::fire_types::FireType;
use rand::prelude::*;

#[derive(Clone, Copy)]
pub struct Particle {
    pub x: usize,
    pub y: usize,
    pub life: u8,
    pub color_idx: u8,
}

/// Spawns a new particle with fire-type-specific logic, if the random chance is met.
pub fn maybe_spawn_particle(
    particles: &mut Vec<Particle>,
    fire_type: FireType,
    palette_len: usize,
    width: usize,
    height: usize,
) {
    let mut rng = ThreadRng::default();
    if rng.random_bool(0.15) {
        let px = rng.random_range(0..width);
        let (color_idx, life) = match fire_type {
            FireType::Original | FireType::WhiteHot => {
                ((palette_len - 1) as u8, rng.random_range(10..30))
            }
            FireType::White => ((palette_len - 1) as u8, rng.random_range(10..30)),
            FireType::Blue | FireType::Ice => {
                ((palette_len as f32 * 0.7) as u8, rng.random_range(14..32))
            }
            FireType::Green | FireType::Toxic => {
                ((palette_len as f32 * 0.6) as u8, rng.random_range(12..28))
            }
            FireType::Purple => ((palette_len as f32 * 0.8) as u8, rng.random_range(12..28)),
            FireType::Rainbow | FireType::Candy => (
                rng.random_range((palette_len / 2)..palette_len) as u8,
                rng.random_range(10..30),
            ),
            FireType::FireAndIce => {
                if rng.random_bool(0.5) {
                    ((palette_len as f32 * 0.85) as u8, rng.random_range(10..28))
                } else {
                    ((palette_len as f32 * 0.15) as u8, rng.random_range(14..32))
                }
            }
            FireType::ChemicalFire => ((palette_len as f32 * 0.7) as u8, rng.random_range(10..30)),
            FireType::Cyberpunk => (
                if rng.random_bool(0.5) {
                    (palette_len as f32 * 0.85) as u8
                } else {
                    (palette_len as f32 * 0.15) as u8
                },
                rng.random_range(10..28),
            ),
            FireType::Aurora => (
                rng.random_range((palette_len / 4)..(palette_len * 3 / 4)) as u8,
                rng.random_range(16..36),
            ),
            FireType::Plasma => (
                if rng.random_bool(0.3) {
                    (palette_len - 1) as u8
                } else {
                    rng.random_range((palette_len / 3)..palette_len) as u8
                },
                rng.random_range(10..28),
            ),
            FireType::Void => (
                if rng.random_bool(0.8) {
                    0
                } else {
                    (palette_len as f32 * 0.4) as u8
                },
                rng.random_range(18..36),
            ),
        };
        particles.push(Particle {
            x: px,
            y: height - 2,
            life,
            color_idx,
        });
    }
}

/// Updates all particles, rendering them to the pixel buffer and moving them.
pub fn update_particles(
    particles: &mut Vec<Particle>,
    pixel_buffer: &mut [u8],
    width: usize,
    _height: usize,
) {
    let mut rng = ThreadRng::default();
    particles.retain_mut(|p| {
        if p.life > 0 && p.y > 0 {
            let fade = ((p.life as f32 / 30.0) * (p.color_idx as f32)).max(1.0) as u8;
            let idx = p.y * width + p.x;
            if idx < pixel_buffer.len() {
                pixel_buffer[idx] = fade;
            }
            let dx = rng.random_range(0..3) as isize - 1;
            p.x = (p.x as isize + dx).clamp(0, width as isize - 1) as usize;
            p.y -= 1;
            p.life -= 1;
            true
        } else {
            false
        }
    });
}
