# DOOM fire Wallpaper for hyprpaper

Animated [Doom fire effect](https://fabiensanglard.net/doom_fire_psx/) as a dynamic wallpaper for [Hyprpaper](https://github.com/hyprwm/hyprpaper) on Linux built with rust.

This project generates a real-time animated fire effect, saves each frame as a WebP image, and updates your wallpaper using Hyprpaper for a seamless, living flame desktop background.

---

## Features

- **Real-time animated fire**: Classic DOOM-style fire simulation.
- **Parallel rendering**: Uses all CPU cores for fast frame generation.
- **WebP output**: Efficient, high-quality image format.
- **Hyprpaper integration**: Automatically updates your wallpaper.
- **Configurable resolution and speed**.

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
   git clone https://github.com/yourusername/doomfire_wallpaper.git
   cd doomfire_wallpaper/rust
   ```

2. **Install dependencies:**
   ```sh
   cargo build --release
   ```

3. **Run Hyprpaper** (if not already running):
   ```sh
   hyprpaper &
   ```

4. **Run the wallpaper generator:**
   ```sh
   cargo run --release
   ```

---

## Configuration

Edit the following variables in [`src/main.rs`](src/main.rs) to change resolution, scale, and frame rate:

```rust
let screen_width = 1920;
let screen_height = 1080;
let scale = 6;  // Lower = more detailed fire, higher = faster
let fps = 12;   // Frames per second
```

You can also set the `output` variable to target a specific monitor, or leave it empty to update all outputs.

---

## How it Works

- The program generates a new fire frame, saves it as a WebP image in `~/.cache/hyprpaper/doomfire.webp`.
- It then tells Hyprpaper to reload the wallpaper using `hyprctl`.
- This loop runs at your chosen FPS, creating a smooth animated effect.

---

## Troubleshooting

- **Wallpaper not updating?**  
  Make sure Hyprpaper is running.
- **Performance issues?**  
  Increase the `scale` value or lower the resolution/FPS.
- **Multiple monitors?**  
  Set the `output` variable to your desired monitor name (see `wlr-randr` or `hyprctl monitors`).
- **Flickering animation**
  Disable any system animations - hyprland for example

---

## Credits

- Fire algorithm inspired by [Fabien Sanglard's DOOM fire article](https://fabiensanglard.net/doom_fire_psx/).
- [Hyprpaper](https://github.com/hyprwm/hyprpaper) for dynamic wallpaper support.
- [rayon](https://crates.io/crates/rayon) for parallel rendering.
- [Larry's DOOM fire wallpaper](https://github.com/Leafmun-certii/arch_linux_doom_fire_wallpaper)

---

## License

WTFPL

---

Enjoy your everliving flame