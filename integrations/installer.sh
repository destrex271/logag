
# ------ Constants ------
HARNESS_SELECTION_PORMPT="Select the harness to install \`logag\` for:";

SUPPORTED_HARNESSES=("opencode" "codex" "claude");
# -----------------------


# -------- Execution functions --------
function install_for_opencode() {
    OPEN_CODE_DIR="~/.opencode/plugins"
    if [ -d "$DIR" ]; then
        echo "Plugins directory already exists"
    else
        echo "Creating plugins directory"
        mkdir -p ~/.opencode/plugins
    fi

    cp ./opencode/opencode_plugin.ts  ~/.opencode/plugins/forward.ts

    echo "Installed plugin for logag."
}

function install_for_codex() {
    echo "Codex support not implemented yet."
}

function install_for_claude() {
    echo "Claude support not implemented yet."
}
# --------------------------------------


# ------ Harness selection logic -------
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
    esac
    break
done
# --------------------------------------
