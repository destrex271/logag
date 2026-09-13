# Using LogAg with Codex

LogAg acts as a memory backend for [Codex](https://developers.openai.com/codex/). Record user/agent sessions with Codex hooks and retrieve cached responses on similar queries over MCP.

## Prerequisites

- LogAg is built and running (see the [README](../README.md#installation))
- Codex CLI installed. Lifecycle hooks require a recent Codex build (v0.124.0+; hooks are enabled by default).

## Setup

### 1. Start LogAg

```bash
cargo run --release -- --config-path config/config.toml
```

This starts MCP endpoints at `http://127.0.0.1:8000/mcp` and `http://127.0.0.1:8000/read_mcp`, and the recording endpoint at `http://127.0.0.1:8000/record`.

### 2. Configure Codex MCP servers

Edit `~/.codex/config.toml` and add one table per server:

```toml
[mcp_servers.logag]
url = "http://127.0.0.1:8000/mcp"

[mcp_servers.logag-read]
url = "http://127.0.0.1:8000/read_mcp"
```

or register them with the CLI:

```bash
codex mcp add logag --url http://127.0.0.1:8000/mcp
codex mcp add logag-read --url http://127.0.0.1:8000/read_mcp
```

Two MCP servers are registered:

| Server | Endpoint | Purpose |
|--------|----------|---------|
| `logag` | `/mcp` | Recording events (`logag_add_event`) |
| `logag-read` | `/read_mcp` | Retrieving cached responses (`get_cached_agent_response`) |

### 3. Add AGENTS.md

Create or update an `AGENTS.md` file (referenced by Codex via project instructions) with:

```md
For every user query, first call `get_cached_agent_response` from the `logag-read`
MCP server to check if there is already cached knowledge around that prompt.
If cached content exists and is relevant, use it to inform your response.
```

### 4. Install the Auto-Recording Hook

The repo ships a Codex `Stop` hook that sends completed user/agent turns to `POST /record`.

**Option A — Install script:**

```bash
bash integrations/installer.sh
```

Select `codex` when prompted. This copies `hooks.json` to `~/.codex/hooks.json` and `store_turn.py` to `~/.codex/hooks/store_turn.py`.

**Option B — Manual install:**

```bash
mkdir -p ~/.codex/hooks
cp integrations/codex/hooks.json ~/.codex/hooks.json
cp integrations/codex/hooks/store_turn.py ~/.codex/hooks/store_turn.py
chmod +x ~/.codex/hooks/store_turn.py
```

**Trust the hooks:** Codex skips hook commands it has not reviewed yet. Run `/hooks` inside a Codex session, review the `store_turn.py` definitions (one per event: `UserPromptSubmit` and `Stop`), and trust them. The hooks then fire automatically as you chat.

Hooks are enabled by default. If `~/.codex/config.toml` already contains `[features] hooks = false`, remove it (or set it to `true`) so the hook can run.

## How the Hooks Work

Two Codex lifecycle hooks drive the integration (see `hooks.json`):

| Event | Fires | What the hook does |
|-------|-------|--------------------|
| `UserPromptSubmit` | Right before a user prompt is sent | Stores `{session_id, turn_id, prompt}` in a temp state file (`<tempdir>/logag/`) |
| `Stop` | When a turn completes | Reads the stored prompt, pairs it with `last_assistant_message`, POSTs the pair to `http://localhost:8000/record`, then removes the state file |

The recorded payload looks like:

```json
{ "userInput": "...", "agentOutput": "..." }
```

State is keyed by `session_id` + `turn_id`, so concurrent sessions and turns never collide. If a turn is interrupted before `Stop` runs, a small stale state file may remain in the temp dir — it is harmless and never affects later turns.

Both hooks are fail-open: if LogAg is offline or the state file is missing, the hook exits silently without interrupting the Codex turn.

## Data Flow

```
Codex session
    │
    ├─ UserPromptSubmit hook stores the user prompt
    │
    ├─ Stop hook pairs prompt + last_assistant_message
    │  └─ POST /record ─→ LogAg stores as LogEvent + embedding
    │
    └─ Codex reads AGENTS.md
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

The same pattern applies to other MCP-compatible tools (Claude, etc.):

1. **Record** — send user/agent pairs to `POST /record`
2. **Retrieve** — call `get_cached_agent_response` via the `logag-read` MCP server
3. **Configure** — register both MCP endpoints in that tool's MCP client config