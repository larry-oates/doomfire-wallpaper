# DOOM-style fire wallpaper for Hyprpaper

Animated [Doom fire effect](https://fabiensanglard.net/doom_fire_psx/) as a dynamic wallpaper for [Hyprpaper](https://github.com/hyprwm/hyprpaper) on Linux built with Rust.

This project generates a real-time animated fire effect, saves each frame as a WebP image, and updates your wallpaper using Hyprpaper for a seamless, living flame desktop background.

---

## Features

- **Real-time animated fire**: Classic DOOM-style fire simulation.
- **Multiple colour palettes**: Original, blue, rainbow, toxic, purple, white-hot... add your own!
- **Parallel rendering**: Uses all CPU cores for fast frame generation.
- **Configurable via TOML file**: Resolution, speed, palette, background colour, etc.
- **Auto-pause**: The animation pauses when all screens (outputs) have a window (client) on them or your system is asleep to save CPU.

---

## Requirements

- **Linux** (Wayland, with Hyprland and Hyprpaper)
- [Rust/cargo](https://rust-lang.org/) (edition 2021)
- [Hyprpaper](https://github.com/hyprwm/hyprpaper) - running and configured

---

## Installation

1. **Install with Yay:**

  ```sh
  yay install doomfire-wallpaper
  ```

  **Or Make package manually**
  
  ```sh
  git clone https://github.com/Leafmun-certii/doom_fire_wallpaper.git
  cd doomfire_wallpaper/aur
  makepkg -Cfsri
  ```

2. **Enable hyprpaper**

  **Make sure [Hyprpaper](https://github.com/hyprwm/hyprpaper) is enabled and running in your Hyprland session.**  
  You can enable it as a systemd user service for automatic startup:

  ```sh
  systemctl --user enable --now hyprpaper.service
  systemctl --user start --now hyprpaper.service
  ```

  If you do not have a `hyprpaper.service` file, you can create one in `~/.config/systemd/user/hyprpaper.service`:

  ```ini
  [Unit]
  Description=Fast, IPC-controlled wallpaper utility for Hyprland.
  Documentation=<https://wiki.hyprland.org/Hypr-Ecosystem/hyprpaper/>
  PartOf=graphical-session.target
  Requires=graphical-session.target
  After=graphical-session.target
  ConditionEnvironment=WAYLAND_DISPLAY

  [Service]
  Type=simple
  ExecStart=/usr/bin/hyprpaper
  Slice=session.slice
  Restart=on-failure

  [Install]
  WantedBy=graphical-session.target
   ```

3. **Run the wallpaper!**

  Generate the config file, enable and start the service with:

  ```sh
  dfpaper setup
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
fire_type = "Original"    # See fire type section below for options
background = [0, 0, 0]  # Optional: RGB array, e.g. [20, 20, 20] for dark grey
restart_on_pause = true # Optional: true (default) or false, controls if animation restarts after pause. 
pause_on_cover = true   # Optional: true (default) pauses animation when all screens contain a window; set to false to keep animating even when covered
screen_burn = false # Optional: false (default). If true, when a screen is uncovered, that screen will turn to fire

```

**All fields are optional**; defaults will be used if not set.

### Applying Config Changes

After you change the configuration **you must restart the wallpaper service for changes to take effect**:

```sh
dfpaper refresh
```

### Fire Types

- **Original:** Classic DOOM fire
- **WhiteHot:** Classic DOOM palette, but blends to white at the top (hotter white flames)
- **White:** White-hot fire (all shades of white)
- **Blue:** Blue flame
- **Rainbow:** Animated rainbow
- **Green:** Toxic green
- **Purple:** Purple flame
- **Ice:** Cold blue/white
- **Toxic:** Neon green/yellow
- **FireAndIce:** Cold blue ice blending into hot fire
- **ChemicalFire:** Fire at the chemical plant
- **Cyberpunk:** Neon magenta/cyan
- **Aurora:** Animated Northern lights effect
- **Plasma:** Electric blue/purple/white bolts, fading to black
- **Void:** Deep blue/black cosmic
- **Candy:** Pastel rainbow stripes
- **Random:** Randomly selects a fire type on startup

---

## How it Works

- The program generates a new frame and then saves it as a WebP image in `~/.cache/hyprpaper/doomfire.webp`.
- It  tells Hyprpaper to reload the wallpaper using `hyprctl`.
- This loop runs at your chosen FPS, creating a smooth animated effect.
- When all monitors contain a client window or the system is dormant, the animation will be paused to save CPU utilization.
- If `restart_on_pause` is set to `true`, the animation restarts from the beginning after a pause; if `false`, it resumes where it left off.
- If `screen_burn` is set to `true` a screen shot is taken every 100 ms using grim, converted to greyscale
and applied to the background when the last window on a screen is closed

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

0BSD  
Enjoy your everliving flame!
