use strum_macros::EnumIter;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, EnumIter, Debug)]
pub enum FireType {
    Original,
    Blue,
    Rainbow,
    Green,
    Purple,
    White,         
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

pub fn generate_palette(fire_type: FireType, background_colour: Option<[u8; 3]>, phase: f32) -> Vec<[u8; 3]> {
    let mut pal = match fire_type {
        FireType::FireAndIce => (0..=36)
            .map(|i| {
                let t = i as f32 / 36.0;
                let r = (255.0 * t.powf(1.2) + 60.0 * t).min(255.0) as u8;
                let g = (80.0 * t + 60.0 * (1.0 - t)).min(140.0) as u8;
                let b = (255.0 * (1.0 - t) + 20.0 * t).min(255.0) as u8;
                [r, g, b]
            })
            .collect(),
        FireType::ChemicalFire => (0..=36)
            .map(|i| {
                let t = i as f32 / 36.0;
                let r = (255.0 * t).min(255.0) as u8;
                let g = (80.0 * (1.0 - t) + 120.0 * t).min(200.0) as u8;
                let b = (180.0 * (1.0 - t).powi(2)).min(180.0) as u8;
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
        FireType::WhiteHot => (0..=36)
            .map(|i| {
                let t = i as f32 / 36.0;
                // Like Original, but ends at white
                let r = (255.0 * t.sqrt()).min(255.0) as u8;
                let g = (255.0 * t.powi(3)).min(255.0) as u8;
                let blend = t; // 0 at bottom, 1 at top
                let r = ((1.0 - blend) * r as f32 + blend * 255.0) as u8;
                let g = ((1.0 - blend) * g as f32 + blend * 255.0) as u8;
                let b = (blend * 255.0) as u8;
                [r, g, b]
            })
            .collect::<Vec<[u8; 3]>>(),
        FireType::White => (0..=36)
            .map(|i| {
                let t = i as f32 / 36.0;
                let v = (255.0 * t.sqrt()).min(255.0) as u8;
                [v, v, v]
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
