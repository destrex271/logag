#!/bin/bash

echo "Please select the harness with which you want to use logag(Codex, OpenCode):"

# Keep option variables lowercase to prevent typos
options=("Codex" "OpenCode")

install_for_codex() {
    cp -R ./codex/* ~/.codex/
}

install_for_opencode() {
    DIR="~/.opencode/plugins"

    if [ -d "$DIR" ]; then
        echo "Plugins directory already exists"
    else
        echo "Creating plugins directory"
        mkdir -p ~/.opencode/plugins
    fi

    cp ./opencode/opencode_plugin.ts  ~/.opencode/plugins/forward.ts
}

select choice in "${options[@]}"; do
    case $choice in
        "Codex")
            install_for_codex
            echo "Installed for Codex"
            # Place your Codex logag code here
            break
            ;;
        "OpenCode")
            install_for_opencode
            echo "Installed for OpenCode"
            # Place your OpenCode logag code here
            break
            ;;
        *)
            # This triggers if they type 1, 2, or an invalid number
            # $REPLY contains the actual raw text/number they entered
            if [[ "$REPLY" == "1" ]]; then
                echo "You selected Codex"
                break
            elif [[ "$REPLY" == "2" ]]; then
                echo "You selected OpenCode"
                break
            else
                echo "Unknown harness: $REPLY. Please enter 1 or 2."
            fi
            ;;
    esac
done

