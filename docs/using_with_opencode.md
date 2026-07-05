# Using LogAg with Opencode

LogAg acts as a memory backend for [Opencode](https://opencode.ai). Record user/agent sessions and retrieve cached responses on similar queries.

## Prerequisites

- LogAg is built and running (see the [README](../README.md#installation))
- Opencode CLI installed

## Setup

### 1. Start LogAg

```bash
cargo run --release -- --config-path config/config.toml
```

This starts MCP endpoints at `http://127.0.0.1:8000/mcp` and `http://127.0.0.1:8000/read_mcp`.

### 2. Configure Opencode

Edit `~/.config/opencode/opencode.json`:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "instructions": ["/path/to/logag/AGENTS.md"],
  "mcp": {
    "logag": {
      "type": "remote",
      "url": "http://127.0.0.1:8000/mcp"
    },
    "logag-read": {
      "type": "remote",
      "url": "http://127.0.0.1:8000/read_mcp"
    }
  }
}
```

Two MCP servers are registered:

| Server | Endpoint | Purpose |
|--------|----------|---------|
| `logag` | `/mcp` | Recording events (`logag_add_event`) |
| `logag-read` | `/read_mcp` | Retrieving cached responses (`get_cached_agent_response`) |

### 3. Add AGENTS.md

Create or update an `AGENTS.md` file (referenced by `instructions` in the config above) with:

```md
For every user query, first call `get_cached_agent_response` from the `logag-read`
MCP server to check if there is already cached knowledge around that prompt.
If cached content exists and is relevant, use it to inform your response.
```

### 4. Install the Auto-Recording Plugin

The [opencode plugin](../js/opencode_plugin.js) hooks into chat events and automatically sends user/agent message pairs to `POST /record`.

Add it to your `opencode.json`:

```json
{
  "plugins": ["/path/to/logag/js/opencode_plugin.js"]
}
```

The plugin listens for `chat.message` and `message.part.updated` events, buffers the conversation, and POSTs completed pairs to `http://localhost:8000/record`.

## Data Flow

```
Opencode session
    │
    ├─ Plugin captures user input + agent output
    │  └─ POST /record ─→ LogAg stores as LogEvent + embedding
    │
    └─ Agent reads AGENTS.md
       └─ Calls logag-read_get_cached_agent_response(text)
          └─ MCP ─→ LogAg compares embedding via cosine distance
             └─ Returns cached agent output if similar query exists
```

## MCP Tools

| Tool | Server | Description |
|------|--------|-------------|
| `logag_add_event` | `logag` | Register a user/agent/thinking event |
| `logag-read_get_cached_agent_response` | `logag-read` | Retrieve cached agent output for a similar user query |

## Extending to Other Tools

The same pattern applies to other MCP-compatible tools (Claude, Codex, etc.):

1. **Record** — send user/agent pairs to `POST /record`
2. **Retrieve** — call `get_cached_agent_response` via the `logag-read` MCP server
3. **Configure** — register both MCP endpoints in that tool's MCP client config
