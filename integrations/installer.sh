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
    echo "Next step: run \`/hooks\` inside Codex and trust the store_turn.py hook so it is allowed to run."
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
