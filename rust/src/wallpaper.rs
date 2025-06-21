use std::process::Command;
use std::path::Path;

pub fn get_monitor_names() -> anyhow::Result<Vec<String>> {
    let output = Command::new("hyprctl")
        .arg("monitors")
        .output()?;
    let text = String::from_utf8_lossy(&output.stdout);

    let names = text
        .lines()
        .filter(|line| line.starts_with("Monitor"))
        .filter_map(|line| line.split_whitespace().nth(1))
        .map(|s| s.to_string())
        .collect();

    Ok(names)
}

pub fn set_wallpaper(file: &Path) -> anyhow::Result<()> {
    Command::new("hyprctl")
        .args([
            "hyprpaper",
            "reload",
            &format!(",{}", file.to_str().unwrap())
        ])
        .status()?;

    Ok(())
}
