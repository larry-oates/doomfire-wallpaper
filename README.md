# DOOM-style fire wallpaper for Hyprpaper

Want to use a [fire from DOOM](https://fabiensanglard.net/doom_fire_psx/) as a dynamic wallpaper in Hyprland?  

This project generates an animated fire effect in real-time and displays it in a GTK4 window.  
Setting this as your wallpaper depends on your compositor, currently there is only a guide for Hyprland using Hyprwinwrap  
*if you want it for another please raise an issue! [XWinWrap](https://github.com/mmhobi7/xwinwrap) could potentially work for a X11 setup...*

## Features

- **Real-time animated fire**: Classic DOOM-style fire simulation.
- **Auto-pause**: The animation pauses when all screens (outputs) have a window (client) on them or your system is asleep to save CPU.
- **Multiple colour palettes**: Original, blue, rainbow, toxic, purple, white-hot... add your own!
- **Parallel rendering**: Uses all CPU cores for fast frame generation.
- **Configurable via TOML file**: Resolution, speed, palette, background colour, and more.

## Requirements

- **Linux** (with Hyprland)
- [Rust/cargo](https://rust-lang.org/) (edition 2021)
- [Hyprwinwrap](https://aur.archlinux.org/packages/hyprland-plugin-hyprwinwrap) - running and configured (as described below)

## Installation

1.**Install with Yay:**

  ```sh
  yay -Sy doomfire-wallpaper
  ```

1b.**Or Make package manually**
  
  ```sh
  git clone https://github.com/larry-oates/doomfire-wallpaper.git
  cd doom_fire_wallpaper/doomfire-wallpaper
  makepkg -Cfsri
  ```

2.**Set up hyprwinwrap**

  **Make sure Hyprwinwrap is enabled and running in your Hyprland session.**
  Add this to your `hyprland.conf`:

```.conf
  plugin {
    hyprwinwrap {
      class = com.leafman.doomfirewallpaper
    }
  }
```

3.**Setup the wallpaper!**

 You can now run the wallpaper with:

```sh
doomfire-wallpaper
```

This will create a default config at `~/.config/doomfire-wallpaper/config.toml` and start the wallpaper.

Set up the wallpaper to run on startup by adding:

```.conf
# Anywhere after you have loaded the hyprwinwrap plugin...
exec-once = hyprpm reload 
# add...

# Doom fire wallpaper
exec-once = doomfire-wallpaper
```

to your hyprland.conf

## Configuration

The config file is created at `~/.config/doomfire-wallpaper/config.toml` after running `doomfire-wallpaper setup`.

```toml
screen_width = 1920
screen_height = 1080
scale = 4
fps = 24
fire_type = "Original"    # See fire type section below for options
# background = [0, 0, 0]  # RGB array, e.g. [20, 20, 20] for dark grey
# restart_on_pause = true # True or false, controls if animation restarts after pause.
# pause_on_cover = true   # True pauses animation when all screens contain a window; set to false to keep animating even when covered
# screen_burn = false     # If true, when a screen is uncovered, that screen will turn to fire
# wind_strength = 0.5     # Float, controls how much the fire is affected by wind. 0.0 for no wind.
```

**All fields are optional**; defaults will be used if not set.

### Applying Config Changes

After you change the configuration **you must restart the wallpaper service for changes to take effect**:

```sh
doomfire-wallpaper refresh
```

---

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

## How It Works

- The program is a GTK4 application that creates a borderless window.
- It continuously calculates and renders the fire animation onto a pixel buffer inside this window at your configured FPS.
- The `hyprwinwrap` plugin (configured in `hyprland.conf`) detects this window by its class name (`com.leafman.doomfirewallpaper`) and forces it to a background layer, effectively making it the wallpaper.
- The animation pauses when all monitors are covered by other windows or when the system is asleep, to save CPU.
- If `screen_burn` is set to `true`, when all screens are covered, the app will periodically take screenshots. When a screen becomes uncovered again, the last screenshot taken will be used to "burn" its shape into the fire animation.

## Troubleshooting

- **Wallpaper not updating?**  
  Make sure Hyprpaper is running and you have no other programs managing your wallpaper (e.g. [waypaper](https://github.com/anufrievroman/waypaper)).
- **Performance issues?**  
  Increase the `scale` value or lower the resolution/FPS.
- **Multiple monitors?**  
  Set the `output` variable to your desired monitor name (see `wlr-randr` or `hyprctl monitors`).
- **Frozen screen?**  
  Ensure you only have one instance of doomfire-wallpaper running with htop

## Credits

- Fire algorithm inspired by [Fabien Sanglard's DOOM fire article](https://fabiensanglard.net/doom_fire_psx/).
- [rayon](https://crates.io/crates/rayon) for parallel rendering.

## License

0BSD  
Enjoy your everliving flame!
