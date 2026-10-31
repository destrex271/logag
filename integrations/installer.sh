#!/bin/bash

# ------ Constants ------
HARNESS_SELECTION_PROMPT="Select the harness to install \`logag\` for:";

SUPPORTED_HARNESSES=("opencode" "codex" "claude");

# Directory of this script, so paths work no matter where the installer is run from.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)";
# -----------------------


# -------- Execution functions --------
function install_mcp_servers() {
    local cli="$1"

    echo "Installing logag MCP servers..."
    if command -v "$cli" >/dev/null 2>&1; then
        "$cli" mcp add logag --url http://127.0.0.1:8000/mcp 2>&1 || true
        "$cli" mcp add logag-read --url http://127.0.0.1:8000/read_mcp 2>&1 || true
        "$cli" mcp add logag-project --url http://127.0.0.1:8000/project_management 2>&1 || true
        echo "MCP servers installed."
    else
        echo "Warning: '$cli' command not found. Skipping MCP installation."
        echo "You can manually add them with:"
        echo "  $cli mcp add logag --url http://127.0.0.1:8000/mcp"
        echo "  $cli mcp add logag-read --url http://127.0.0.1:8000/read_mcp"
        echo "  $cli mcp add logag-project --url http://127.0.0.1:8000/project_management"
    fi
}
function install_base_instructions_md() {
    local BASE_INSTRUCTIONS="${SCRIPT_DIR}/base_agent_directions.md"
    local directory="$1"
    local filename="${directory}/AGENTS.md"
    touch $filename
    
    # 1. Ensure the target file actually exists first
    if [ ! -f "$filename" ]; then
        echo "Error: $filename does not exist."
        return 1
    fi

    # 2. Corrected grep to target the specific file
    if grep -q "<!--- Log Ag instructions -->" "$filename"; then
        echo "Log Ag instructions already exist, skipping... (If you think this is a mistake, please remove the log ag header from agents.md)"
    else
        # 3. Read the original content safely
        local filedata
        filedata=$(<"$BASE_INSTRUCTIONS")
        echo $filedata
        
        # 4. Use > (overwrite) to put the new header at the very top
        echo "<!--- Log Ag instructions -->" > "$filename"
        
        # 5. Use >> (append) to put the original data back underneath it
        echo "$filedata" >> "$filename"
        echo "Successfully added instructions to $filename"
    fi
}

function install_for_opencode() {
    BASE_OPEN_CODE="$HOME/.config/opencode"
    OPEN_CODE_PLUGIN_DIR="$HOME/.opencode/plugins"
    if [ -d "$OPEN_CODE_PLUGIN_DIR" ]; then
        echo "Plugins directory already exists"
    else
        echo "Creating plugins directory"
        mkdir -p "$OPEN_CODE_PLUGIN_DIR"
    fi

    cp "$SCRIPT_DIR/opencode/opencode_plugin.ts" "$OPEN_CODE_PLUGIN_DIR/forward.ts"

    echo "Installed plugin for logag."

    install_mcp_servers opencode
    install_base_instructions_md $BASE_OPEN_CODE
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

    install_mcp_servers codex
    install_base_instructions_md $CODEX_HOOKS_DIR
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
