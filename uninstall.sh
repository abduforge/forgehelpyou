#!/usr/bin/env sh
set -eu
BIN_DIR=${INSTALL_DIR:-"${HOME}/.local/bin"}
if [ -e "$BIN_DIR/forgehelpyou" ]; then
    rm -- "$BIN_DIR/forgehelpyou"
    echo "Removed $BIN_DIR/forgehelpyou"
else
    echo "ForgeHelpYou is not installed in $BIN_DIR"
fi
