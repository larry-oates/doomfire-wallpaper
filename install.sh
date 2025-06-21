#!/bin/bash

set -e

# Build the project
echo "Building doomfire_wallpaper..."
cargo build --release

# Install binary to ~/.local/bin
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"
cp target/release/doomfire_wallpaper "$INSTALL_DIR/doomfire_wallpaper"
echo "Installed binary to $INSTALL_DIR/doomfire_wallpaper"

# Create config directory if it doesn't exist
CONFIG_DIR="$HOME/.config/doomfire_wallpaper"
mkdir -p "$CONFIG_DIR"
if [ ! -f "$CONFIG_DIR/config.toml" ]; then
    cat > "$CONFIG_DIR/config.toml" <<EOF
# Example config.toml for doomfire_wallpaper
screen_width = 1920
screen_height = 1080
scale = 4
fps = 23
palette = "Original"
background = [0, 0, 0]
restart_on_pause = true
EOF
    echo "Created example config at $CONFIG_DIR/config.toml"
fi

# Create systemd user service
SYSTEMD_USER_DIR="$HOME/.config/systemd/user"
mkdir -p "$SYSTEMD_USER_DIR"
cat > "$SYSTEMD_USER_DIR/doomfire_wallpaper.service" <<EOF
[Unit]
Description=DOOM Fire Wallpaper for Hyprpaper
After=graphical-session.target hyprpaper.service

[Service]
Type=simple
ExecStartPre=/bin/sh -c 'until pgrep hyprpaper; do sleep 1; done'
ExecStart=%h/.local/bin/doomfire_wallpaper
Restart=on-failure

[Install]
WantedBy=default.target
EOF

echo "Created systemd user service at $SYSTEMD_USER_DIR/doomfire_wallpaper.service"

echo "To enable and start the wallpaper service, run:"
echo "  systemctl --user daemon-reload"
echo "  systemctl --user enable --now doomfire_wallpaper.service"
echo "  systemctl --user start --now doomfire_wallpaper.service"
echo 

if ! command -v hyprpaper >/dev/null 2>&1; then
    echo "Warning: Hyprpaper is not installed or not in your PATH."
    echo "Please install Hyprpaper: https://github.com/hyprwm/hyprpaper"
else
    echo "Hyprpaper is installed. You make sure it's service is enabled."
fi