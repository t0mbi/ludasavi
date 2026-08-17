#!/usr/bin/env bash
# Builds Ludusavi and installs it (binary, .desktop entry, icon) for the
# current user, so it shows up in the application launcher/start menu.
# Safe to re-run any time to rebuild and reinstall.
set -euo pipefail

cd "$(dirname "$0")"

echo "Building Ludusavi (release)..."
cargo build --release

BIN_DIR="$HOME/.local/bin"
APPS_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"

mkdir -p "$BIN_DIR" "$APPS_DIR" "$ICON_DIR"

install -m 755 target/release/ludusavi "$BIN_DIR/ludusavi"
install -m 644 assets/icon.svg "$ICON_DIR/com.mtkennerly.ludusavi.svg"

sed "s|^Exec=.*|Exec=$BIN_DIR/ludusavi|" assets/linux/com.mtkennerly.ludusavi.desktop \
    > "$APPS_DIR/com.mtkennerly.ludusavi.desktop"
chmod 644 "$APPS_DIR/com.mtkennerly.ludusavi.desktop"

command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$APPS_DIR" || true
command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache "$HOME/.local/share/icons/hicolor" >/dev/null 2>&1 || true
command -v kbuildsycoca6 >/dev/null 2>&1 && kbuildsycoca6 || true
command -v kbuildsycoca5 >/dev/null 2>&1 && kbuildsycoca5 || true

echo
echo "Installed:"
echo "  binary: $BIN_DIR/ludusavi"
echo "  entry:  $APPS_DIR/com.mtkennerly.ludusavi.desktop"
echo "  icon:   $ICON_DIR/com.mtkennerly.ludusavi.svg"
echo

case ":${PATH:-}:" in
  *":$BIN_DIR:"*) ;;
  *) echo "Note: $BIN_DIR isn't on your PATH, so 'ludusavi' won't work from a terminal yet." \
          "Add this to your shell profile (~/.bashrc / ~/.zshrc):" \
          $'\n\n    export PATH="$HOME/.local/bin:$PATH"\n' ;;
esac

echo "Ludusavi should now appear in your application launcher."
