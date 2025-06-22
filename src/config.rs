use serde::Deserialize;

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
