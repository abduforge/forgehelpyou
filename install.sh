#!/usr/bin/env sh
set -eu

PROJECT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
BIN_DIR=${INSTALL_DIR:-"${HOME}/.local/bin"}

if [ -x "$PROJECT_DIR/target/release/forgehelpyou" ]; then
    SOURCE_BIN="$PROJECT_DIR/target/release/forgehelpyou"
elif [ -x "$PROJECT_DIR/forgehelpyou" ]; then
    SOURCE_BIN="$PROJECT_DIR/forgehelpyou"
else
    if ! command -v cargo >/dev/null 2>&1; then
        echo "Error: no prebuilt binary found and Cargo is not installed." >&2
        echo "Download the Linux binary archive or install Rust/Cargo to build from source." >&2
        exit 1
    fi
    (cd "$PROJECT_DIR" && cargo build --release --locked)
    SOURCE_BIN="$PROJECT_DIR/target/release/forgehelpyou"
fi

mkdir -p "$BIN_DIR"
install -m 0755 "$SOURCE_BIN" "$BIN_DIR/forgehelpyou"
echo "ForgeHelpYou installed to: $BIN_DIR/forgehelpyou"
case ":${PATH}:" in
    *":$BIN_DIR:"*) ;;
    *) echo "Add this to your shell profile if needed: export PATH=\"\$HOME/.local/bin:\$PATH\"" ;;
esac
