#!/bin/bash

# ------ Constants ------
HARNESS_SELECTION_PROMPT="Select the harness to install \`logag\` for:";

SUPPORTED_HARNESSES=("opencode" "codex" "claude");

# Directory of this script, so paths work no matter where the installer is run from.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)";
# -----------------------


# -------- Execution functions --------
function install_for_opencode() {
    OPEN_CODE_PLUGIN_DIR="$HOME/.opencode/plugins"
    if [ -d "$OPEN_CODE_PLUGIN_DIR" ]; then
        echo "Plugins directory already exists"
    else
        echo "Creating plugins directory"
        mkdir -p "$OPEN_CODE_PLUGIN_DIR"
    fi

    cp "$SCRIPT_DIR/opencode/opencode_plugin.ts" "$OPEN_CODE_PLUGIN_DIR/forward.ts"

    echo "Installed plugin for logag."

    echo "Installing logag MCP servers..."
    if command -v opencode >/dev/null 2>&1; then
        opencode mcp add logag --url http://127.0.0.1:8000/mcp 2>&1 || true
        opencode mcp add logag-read --url http://127.0.0.1:8000/read_mcp 2>&1 || true
        echo "MCP servers installed."
    else
        echo "Warning: 'opencode' command not found. Skipping MCP installation."
        echo "You can manually add them with:"
        echo "  opencode mcp add logag --url http://127.0.0.1:8000/mcp"
        echo "  opencode mcp add logag-read --url http://127.0.0.1:8000/read_mcp"
    fi
}

function install_for_codex() {
    CODEX_HOOKS_DIR="$HOME/.codex"
    mkdir -p "$CODEX_HOOKS_DIR/hooks"

    if [ -f "$CODEX_HOOKS_DIR/hooks.json" ]; then
        echo "Note: $CODEX_HOOKS_DIR/hooks.json already exists and will be replaced."
        echo "Back it up first if it contains hooks you want to keep."
    fi

    cp -f "$SCRIPT_DIR/codex/hooks.json" "$CODEX_HOOKS_DIR/hooks.json"
    cp -f "$SCRIPT_DIR/codex/hooks/store_turn.py" "$CODEX_HOOKS_DIR/hooks/store_turn.py"
    chmod +x "$CODEX_HOOKS_DIR/hooks/store_turn.py"

    echo "Installed hooks for logag."
    echo "Next step: run \`/hooks\` inside Codex and trust the store_turn.py hooks so they are allowed to run."
}

function install_for_claude() {
    echo "Claude support not implemented yet."
}
# --------------------------------------


# ------ Harness selection logic -------
echo "$HARNESS_SELECTION_PROMPT"

select harness in "${SUPPORTED_HARNESSES[@]}"; do
    case $harness in
        "opencode")
            install_for_opencode
            ;;
        "codex")
            install_for_codex
            ;;
        "claude")
            install_for_claude
            ;;
        *)
            echo "Unknown harness: $REPLY. Please enter 1, 2 or 3."
            continue
            ;;
    esac
    break
done
# --------------------------------------
