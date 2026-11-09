# AGENTS.md

## What this is

Single-binary Rust crate (edition 2024) that renders an animated DOOM-style fire as a fullscreen GTK4 window, intended to be consumed as a wallpaper by Hyprland's Hyprwinwrap plugin (Wayland/Hyprland only). No workspace/workspace members, no lib target, no tests.

## Commands

```sh
cargo build            # verify changes; no test suite exists (cargo test passes with 0 tests)
cargo build --release  # used by the release workflow and AUR packaging
cargo clippy           # only linting available; keep clean (existing #[allow]s only for justified cases)
```

System deps required to build: GTK4 dev libraries (`libgtk-4-dev`), `libwayland-dev`, `libgirepository1.0-dev`; see `.github/workflows/release.yml` for the exact CI list and the `gtk4-layer-shell` from-source step (CI builds it with meson/ninja).

## Running / verifying

- `grim`, `hyprctl` (jq-style JSON output via `-j`), and `systemctl` are invoked as external processes at runtime (see `src/wallpaper.rs`, `src/main.rs`). They are not vendored; a dev machine outside a Hyprland session will fall back to empty/False states, so don't rely on runtime behavior for verification — verify with `cargo build`/`clippy`.
- Debug output goes to stderr via `eprintln!("[DEBUG ...]")` in `src/main.rs`.

## Architecture

- `src/main.rs` — entrypoint, GTK app, UI tick callback (frame pacing via vsync credit), screenshot capture (uses `grim`), burn-in application, FPS counter drawing, and the render worker thread.
- `src/wallpaper.rs` — Hyprland integration: socket2 event listener for coverage changes (`OUTPUTS_DIRTY` flag), `get_outputs_covered()` (polls `hyprctl monitors/clients -j`), `is_system_sleeping()` (polls `systemctl is-system-running`).
- `src/doom_fire.rs` — fire simulation state + palette.
- `src/fire_types.rs` — palette definitions for each fire type (`fire_type` config string → palette via strum).
- `src/config.rs` — TOML config deserialized at `~/.config/doomfire-wallpaper/config.toml` (note: README says `doom-fire-wallpaper` but code uses `doomfire-wallpaper` — trust the code).
- `src/particle.rs`, `src/perlin.rs` — helpers for fire behavior.

## Concurrency/quirks worth knowing before editing

- UI thread and animation are decoupled: the GTK frame-clock tick callback (UI thread) sends `FireCommand::{Pause, Update}` over an mpsc `sync_channel(3)` to a worker thread that owns `DoomFire` state, simulates, and sends finished RGB buffers back with `frame_tx`. Don't move fire state ownership between threads.
- Frame pacing uses fractional "vsync credits" (`vsync_credit` in `UiState`) rather than glib timeouts; effective fps is `min(config_fps, monitor_refresh)`.
- Recent commits rewrote rendering for performance: worker thread + frame-clock display + `MemoryTexture` (RGB8 upload) instead of pixbuf, and this forced `GSK_RENDERER=gl`/`ngl` (set in `run_wallpaper` unless already in env). Be careful not to reintroduce per-frame allocations or the Vulkan renderer path.
- pausing/unpause and the "screen burn" trail have subtle timing rules: a pause-time screenshot may be applied for up to `RESUME_BURN_GRACE` (250ms) after unpause; only one burn application per resume tick so the fire can carry the trail away. Read the relevant comments in `src/main.rs` before changing pause/burn logic.
- `screen_burn` config field is an untagged enum (`ScreenBurn::Bool(_)` or `ScreenBurn::String("image")`) — keep this deserialization shape if touching config.

## Release

Tag push matching `v<semver>` triggers `.github/workflows/release.yml` (builds release tarball, creates GitHub Release). AUR package (`doomfire-wallpaper-bin`) is updated separately, not from this repo — don't bump versions here expecting the AUR to follow automatically.
