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

**Trust the hook:** Codex skips hook commands it has not reviewed yet. Run `/hooks` inside a Codex session, review the `store_turn.py` definition, and trust it. The hook then fires automatically when a turn stops.

Hooks are enabled by default. If `~/.codex/config.toml` already contains `[features] hooks = false`, remove it (or set it to `true`) so the hook can run.

## How the Hook Works

Codex pipes one JSON object to the hook command on stdin. The hook uses two fields:

| Field | Description |
|-------|-------------|
| `transcript_path` | Path to the session rollout (`~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl`) |
| `last_assistant_message` | The last assistant message text of the turn |

The hook reads the first real user message from the transcript roll-out and POSTs the pair to `http://localhost:8000/record`:

```json
{ "userInput": "...", "agentOutput": "..." }
```

If LogAg is offline the hook logs nothing and exits silently, so it never interrupts a Codex turn.

## Data Flow

```
Codex session
    │
    ├─ Stop hook fires when a turn completes
    │  └─ store_turn.py reads transcript_path + last_assistant_message
    │     └─ POST /record ─→ LogAg stores as LogEvent + embedding
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