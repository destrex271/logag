DIR="~/.opencode/plugins"

if [ -d "$DIR" ]; then
    echo "Plugins directory already exists"
else
    echo "Creating plugins directory"
    mkdir -p ~/opencode/plugins
fi

cp ./opencode_plugin.ts  ~/.opencode/plugins/forward.ts
