#!/bin/bash

set -e

# Build the project
echo "Building doom-fire-wallpaper..."
cargo build --release

# Install binary to ~/.local/bin
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"
cp target/release/doom-fire-wallpaper "$INSTALL_DIR/doom-fire-wallpaper"
echo "Installed binary to $INSTALL_DIR/doom-fire-wallpaper"

# Create config directory if it doesn't exist
CONFIG_DIR="$HOME/.config/doom-fire-wallpaper"
mkdir -p "$CONFIG_DIR"
if [ ! -f "$CONFIG_DIR/config.toml" ]; then
    cat > "$CONFIG_DIR/config.toml" <<EOF
# Example config.toml for doom-fire-wallpaper
screen_width = 1920
screen_height = 1080
scale = 4
fps = 23
fire_type = "Original"
background = [0, 0, 0]
restart_on_pause = true
EOF
    echo "Created example config at $CONFIG_DIR/config.toml"
fi

# Create systemd user service
SYSTEMD_USER_DIR="$HOME/.config/systemd/user"
mkdir -p "$SYSTEMD_USER_DIR"
cat > "$SYSTEMD_USER_DIR/doom-fire-wallpaper.service" <<EOF
[Unit]
Description=DOOM Fire Wallpaper for Hyprpaper
After=graphical-session.target hyprpaper.service

[Service]
Type=simple
ExecStartPre=/bin/sh -c 'until pgrep hyprpaper; do sleep 1; done'
ExecStart=%h/.local/bin/doom-fire-wallpaper
Restart=on-failure

[Install]
WantedBy=default.target
EOF

echo "Created systemd user service at $SYSTEMD_USER_DIR/doom-fire-wallpaper.service"

echo "To enable and start the wallpaper service, run:"
echo "  systemctl --user daemon-reload"
echo "  systemctl --user enable --now doom-fire-wallpaper.service"
echo "  systemctl --user start --now doom-fire-wallpaper.service"
echo 

if ! command -v hyprpaper >/dev/null 2>&1; then
    echo "Warning: Hyprpaper is not installed or not in your PATH."
    echo "Please install Hyprpaper: https://github.com/hyprwm/hyprpaper"
else
    echo "Hyprpaper is installed. You make sure it's service is enabled."
fi