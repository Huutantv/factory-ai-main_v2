#!/bin/sh
# F.Auto installer for Linux and macOS.
#
#   curl -fsSL https://raw.githubusercontent.com/Huutantv/factory-ai/main/install.sh | sh
#
# Downloads the latest optimized `fauto` binary from GitHub Releases into
# ~/.fauto/bin (override with $FAUTO_INSTALL) and makes it executable.
# Pure static binary — no toolchain, no Node/Python.
set -eu

repo="Huutantv/factory-ai"
os="$(uname -s)"
arch="$(uname -m)"

case "$os" in
  Linux) plat="linux-x86_64" ;;
  Darwin)
    case "$arch" in
      arm64 | aarch64) plat="macos-aarch64" ;;
      x86_64) echo "fauto: Intel macs (x86_64) are no longer supported -- Apple Silicon (arm64) only." >&2; exit 1 ;;
      *) echo "fauto: unsupported macOS arch: $arch" >&2; exit 1 ;;
    esac ;;
  *) echo "fauto: unsupported OS: $os (use install.ps1 on Windows)" >&2; exit 1 ;;
esac

api="https://api.github.com/repos/$repo/releases/latest"
url="$(curl -fsSL "$api" | grep -o "https://github.com/[^\"]*fauto-[^\"]*${plat}" | head -1)"
if [ -z "$url" ]; then
  echo "fauto: no release asset for '$plat' yet (still building?) -- see $api" >&2
  exit 1
fi

dir="${FAUTO_INSTALL:-$HOME/.fauto}/bin"
mkdir -p "$dir"
dest="$dir/fauto"

echo "Downloading $(basename "$url") ..."
curl -fsSL "$url" -o "$dest"
chmod +x "$dest"
echo "fauto installed -> $dest"

# PATH hint (don't edit the user's profile silently).
case ":$PATH:" in
  *":$dir:"*) : ;;
  *)
    echo ""
    echo "Add fauto to your PATH (append to ~/.bashrc or ~/.zshrc):"
    echo "    export PATH=\"$dir:\$PATH\""
    ;;
esac
echo "Then run:  fauto config"
