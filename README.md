-- DOOM-style fire wallpaper for Linux --

Want to use a [fire from DOOM](https://fabiensanglard.net/doom_fire_psx/) as a dynamic wallpaper in Arch linux?  

This project generates an animated fire effect in real-time and displays it in a GTK4 window.  
It can be used as a wallpaper on various Linux desktop environments, including KDE Plasma, GNOME, and wlroots-based compositors like Hyprland and Sway.

---

## Features

- **Real-time animated fire**: Classic DOOM-style fire simulation.
- **Auto-pause**: The animation pauses when all screens (outputs) have a window (client) on them or your system is asleep to save CPU.
- **Multiple colour palettes**: Original, blue, rainbow, toxic, purple, white-hot... add your own!
- **Parallel rendering**: Uses all CPU cores for fast frame generation.
- **Configurable via TOML file**: Resolution, speed, palette, background colour, and more.

---

## Requirements
- **Linux** (Wayland or X11)
- **Rust/Cargo**
- **For X11 users (KDE Plasma on X11, GNOME on X11, XFCE, etc.):**
  - `xwinwrap`: A tool to pin a window to the desktop background.
  - `wmctrl` (optional, for scripting).
- **For Hyprland/Sway users (Wayland):**
  - No extra tools needed if compiled with the `wayland` feature.

---

## Installation

### Arch Linux (AUR)

You can install the package from the Arch User Repository using an AUR helper like `yay` or `paru`:
```sh
yay -S doomfire-wallpaper
```

### From Source

1.  **Clone the repository:**
    ```sh
    git clone https://github.com/larry-oates/doomfire-wallpaper.git
    cd doomfire-wallpaper/doomfire_wallpaper
    ```

2.  **Compile for your environment:**

    -   **For Wayland (Hyprland, Sway):**
        ```sh
        cargo build --release --features wayland
        ```
    -   **For X11 (KDE Plasma, GNOME on X11, XFCE):**
        ```sh
        cargo build --release --features x11
        ```

3.  **Run the wallpaper:**

    -   **On Hyprland/Sway (Wayland):**
        Just run the binary. It will automatically use the layer-shell protocol to become a wallpaper.
        ```sh
        ./target/release/doomfire-wallpaper &
        ```
        or on arch:
        ```sh
                doomfire-wallpaper &
        ```
        

    -   **On KDE Plasma / other X11 Desktops:**
        You need to use `xwinwrap`. First, install it from your distribution's repositories (e.g., `sudo apt install xwinwrap` or `yay -S xwinwrap-git`).
        Then, run the wallpaper through it:
        ```sh
        xwinwrap -g 1920x1080 -ov -ni -- ./target/release/doomfire-wallpaper &
        ```
        *(Replace `1920x1080` with your screen resolution)*.

    -   **On KDE Plasma / GNOME (Wayland):**
        This is experimental. There is no standard way to force a window to the background. Running the binary will create a borderless window, but it may not behave like a true wallpaper.

4.  **(Optional) Add it to your startup applications** to run it automatically when you log in.


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
- **On wlroots (Hyprland/Sway):** The app uses the `wlr-layer-shell` protocol to create a window on the background layer, turning it into a true wallpaper.
- **On X11 (KDE, XFCE):** The app is launched via `xwinwrap`, which embeds the app's window into the desktop's root X window.
- **On other Wayland compositors (KDE, GNOME):** A borderless window is created. True wallpaper integration is not possible without compositor-specific plugins.
- When all monitors contain a client window or the system is dormant, the animation will be paused to save CPU utilization.
- If `restart_on_pause` is set to `true`, the animation restarts from the beginning after a pause; if `false`, it resumes where it left off.
- If `screen_burn` is set to `true` a screen shot is taken every 100 ms using grim, converted to greyscale
and applied to the background when the last window on a screen is closed

---

## Troubleshooting

- **On X11, the wallpaper is in a normal window?**
  Make sure you are launching it through `xwinwrap`.
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
