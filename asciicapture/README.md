
# About

Uses xdg-desktop-portal to request a screen capture, and then renders what it sees in the terminal using ansi escape sequences

# Usage

`asciicapture`

# Install

`cargo install --path .`

## Requirements

`libpipewire-dev libdbus-1-dev`
If pipewire fails to compile, try installing clang

# Bugs

I have only tested with hyprland so far, and this caused a few crashes so be careful.

If you have issues with a panic to do with screenshare, the xdg-desktop-portal for your DE may not be running or may be conflicting with another.
