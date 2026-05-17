#!/bin/bash
set -e

echo "Cleaning up any PDS data stored in bsky cli..."

if security find-internet-password -s "bskycli" -a "user" >/dev/null 2>&1; then
	echo "Note: Remember to delete the app password you created for bskycli from your PDS server records."
	echo "Wiping the local keyring entry..."
    security delete-internet-password -s "bskycli" -a "user" 2>/dev/null || true
fi

CONFIG_PATH="${XDG_CONFIG_HOME:-$HOME/.config}/bskycli/config.json"
if [ -f "$CONFIG_PATH" ]; then
    echo "Wiping any XRPC session data..."
    rm -f "$CONFIG_PATH"
    rmdir "$(dirname "$CONFIG_PATH")" 2>/dev/null || true
fi
# Uninstall binary
echo "Uninstalling bsky cli..."
cargo uninstall bsky
echo "Done!"
