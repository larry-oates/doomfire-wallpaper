# DOOM fire Wallpaper for hyprpaper

Animated [Doom fire effect](https://fabiensanglard.net/doom_fire_psx/) as a dynamic wallpaper for [Hyprpaper](https://github.com/hyprwm/hyprpaper) on Linux built with Rust.

This project generates a real-time animated fire effect, saves each frame as a WebP image, and updates your wallpaper using Hyprpaper for a seamless, living flame desktop background.

---

## Features

- **Real-time animated fire**: Classic DOOM-style fire simulation.
- **Multiple color palettes**: Original, blue, rainbow, green, purple, white-hot... add your own!
- **Parallel rendering**: Uses all CPU cores for fast frame generation.
- **WebP output**: Efficient, high-quality image format.
- **Hyprpaper integration**: Automatically updates your wallpaper.
- **Configurable via TOML file**: Resolution, speed, palette, background color, etc.
- **Auto-pause**: Animation pauses when all screens are covered or system is sleeping.

---

## Requirements

- **Linux** (Wayland, with Hyprland and Hyprpaper)
- [Rust](https://rust-lang.org/) (edition 2021)
- [Hyprpaper](https://github.com/hyprwm/hyprpaper) running and configured
- [WebP support in the `image` crate](https://crates.io/crates/image) (enabled by default)
- [rayon](https://crates.io/crates/rayon) for parallel rendering

---

## Installation

1. **Clone the repository:**
   ```sh
   git clone https://github.com/Leafmun-certii/doom_fire_wallpaper.git
   cd doom_fire_wallpaper
   ```


2. **(Recommended) Use the install script:**
   ```sh
   sh ./install.sh
   ```
   Do not run this as sudo!
   This script will:
   - Build the project
   - Copy the binary to `~/.local/bin`
   - Create an example config file at `~/.config/doom-fire-wallpaper/config.toml` (if it doesn't exist)
   - Install a systemd user service for autostart at `~/.config/systemd/user/doom-fire-wallpaper.service`

   After running the script, enable and start the service with:
   ```sh
   systemctl --user daemon-reload
   systemctl --user enable --now doom-fire-wallpaper.service
   ```

**Or run manually:**
   ```sh
   hyprpaper &
   cargo run --release
   ```
   Or run the binary directly from `target/release/doom-fire-wallpaper`.

  3. **Enable hyprpaper**

---

## Enabling Hyprpaper

Make sure Hyprpaper is enabled and running in your Hyprland session.  
You can enable it as a systemd user service for automatic startup:

```sh
systemctl --user enable --now hyprpaper.service
```

Or, you can start it manually (for testing):

```sh
hyprpaper &
```

If you do not have a `hyprpaper.service` file, you can create one in `~/.config/systemd/user/hyprpaper.service`:

```ini
[Unit]
Description=Hyprpaper wallpaper daemon

[Service]
ExecStart=hyprpaper

[Install]
WantedBy=default.target
```

Then reload systemd and enable:

```sh
systemctl --user daemon-reload
systemctl --user enable --now hyprpaper.service
```

---

## Configuration

Create or edit the config file at `~/.config/doom-fire-wallpaper/config.toml`:

```toml
screen_width = 1920
screen_height = 1080
scale = 4
fps = 23
output = ""
fire_type = "Original"    # See fire type section for options
background = [0, 0, 0]  # Optional: RGB array, e.g. [20, 20, 20] for dark gray
restart_on_pause = true # Optional: true (default) or false, controls if animation restarts after pause. 
random_on_pause = false # Optional: false (default) or true, selects random fire type restart after pause if fire type is "Random"
```

**All fields are optional**; defaults will be used if not set.

## Fire Types

- **Original:** Classic DOOM fire
- **Blue:** Blue flame
- **Rainbow:** Animated rainbow
- **Green:** Toxic green
- **Purple:** Purple flame
- **WhiteHot:** White-hot fire
- **Ice:** Cold blue/white
- **Toxic:** Neon green/yellow
- **FireAndIce:** Cold blue ice blending into bright, deep red fire
- **ChemicalFire:** Fire at the chemical plant
- **Cyberpunk:** Neon magenta/cyan
- **Aurora:** Animated Northern lights effect
- **Plasma:** Electric blue/purple/white bolts, fading to black
- **Void:** Deep blue/black cosmic
- **Candy:** Pastel rainbow stripes
- **Random:** Randomly selects a fire type on startup

---

## How it Works

- The program generates a new fire frame, saves it as a WebP image in `~/.cache/hyprpaper/doomfire.webp`.
- It then tells Hyprpaper to reload the wallpaper using `hyprctl`.
- This loop runs at your chosen FPS, creating a smooth animated effect.
- When all monitors contain a client window or the system is dormant, the animation will be paused to save CPU utilization.
- If `restart_on_pause` is set to `true`, the animation restarts from the beginning after a pause; if `false`, it resumes where it left off.

---

## Troubleshooting

- **Wallpaper not updating?**  
  Make sure Hyprpaper is running and you have no other programs managing your wallpaper (e.g. [waypaper](https://github.com/anufrievroman/waypaper)).
- **Performance issues?**  
  Increase the `scale` value or lower the resolution/FPS.
- **Multiple monitors?**  
  Set the `output` variable to your desired monitor name (see `wlr-randr` or `hyprctl monitors`).
- **Flickering animation?**  
  Disable any system animations (see [Hyprland animation docs](https://wiki.hypr.land/Configuring/Animations/)).

---

## Credits

- Fire algorithm inspired by [Fabien Sanglard's DOOM fire article](https://fabiensanglard.net/doom_fire_psx/).
- [Hyprpaper](https://github.com/hyprwm/hyprpaper) for dynamic wallpaper support.
- [rayon](https://crates.io/crates/rayon) for parallel rendering.
- [Larry's DOOM fire wallpaper](https://github.com/Leafmun-certii/arch_linux_doom_fire_wallpaper)

---

## License

WTFPL  
Enjoy your everliving flame!