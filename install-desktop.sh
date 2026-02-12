#!/bin/sh
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Install icon
mkdir -p "$HOME/.local/share/icons/hicolor/64x64/apps"
cp "$SCRIPT_DIR/assets/icon-64.png" "$HOME/.local/share/icons/hicolor/64x64/apps/wdid.png"

# Install .desktop file
mkdir -p "$HOME/.local/share/applications"
cp "$SCRIPT_DIR/assets/wdid.desktop" "$HOME/.local/share/applications/wdid.desktop"

# Update icon cache (if available)
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
fi

echo "Installed wdid.desktop and icon."
echo "You may need to log out and back in for the icon to appear."

