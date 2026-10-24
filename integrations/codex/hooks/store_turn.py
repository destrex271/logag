#!/usr/bin/env python3
"""Record Codex turns into LogAg via Codex lifecycle hooks.

Two hooks drive this script (see ``hooks.json``):

* ``UserPromptSubmit`` - fires right before a user prompt is sent. The
  script stores ``{session_id, turn_id, prompt}`` in a temp file so the
  matching ``Stop`` event can pair the prompt with the assistant's answer.

* ``Stop`` - fires when a turn completes. The script reads the stored
  prompt, pairs it with ``last_assistant_message``, POSTs the pair to
  LogAg's ``/record`` endpoint, and removes the temp file.

State files live in ``<tempdir>/logag/``. Each one is tagged with a
random UUID (``{session_id}_{turn_id}_{uuid}.json``), so multiple agents
or processes using the plugin at once never overwrite each other's
pending prompts. ``Stop`` consumes the newest pending file for its
session/turn. Failures are swallowed: a LogAg outage or a missing state
file must never interrupt or block a Codex turn.
"""

import json
import os
import sys
import tempfile
import uuid
from pathlib import Path
from typing import Any
from urllib.request import Request, urlopen

LOGAG_URL = "http://localhost:8000/record"
CREATE_PROJECT_URL = "http://localhost:8000/create_project"
LOGAG_CONFIG_FILE = "logag_config.toml"

STATE_DIR = Path(tempfile.gettempdir()) / "logag"

_cached_project_id = None


def _get_cwd():
    try:
        return os.getcwd()
    except Exception:
        return "."


def _parse_project_id_from_toml(content: str):
    # Try to find project-id or project_id
    for line in content.splitlines():
        # Skip comments
        stripped = line.strip()
        if not stripped or stripped.startswith('#'):
            continue
        if 'project' in stripped.lower() and '=' in stripped:
            # Extract value
            parts = stripped.split('=', 1)
            if len(parts) != 2:
                continue
            value = parts[1].strip()
            # Remove comments
            if '#' in value:
                value = value.split('#')[0].strip()
            # Remove quotes
            value = value.strip('"').strip("'")
            if value:
                return value
    return None


def get_or_create_project_id():
    global _cached_project_id
    if _cached_project_id:
        return _cached_project_id

    cwd = _get_cwd()
    config_path = Path(cwd) / LOGAG_CONFIG_FILE

    # Check if config exists
    if config_path.exists():
        try:
            content = config_path.read_text(encoding='utf-8')
            project_id = _parse_project_id_from_toml(content)
            if project_id:
                _cached_project_id = project_id
                return _cached_project_id
        except Exception:
            pass

    # Create project
    project_name = Path(cwd).name or "project"
    try:
        payload = json.dumps({
            "projectName": project_name,
            "timestamp": str(int(__import__('time').time() * 1000))
        }).encode('utf-8')
        req = Request(CREATE_PROJECT_URL, data=payload, headers={"Content-Type": "application/json"}, method="POST")
        try:
            with urlopen(req, timeout=10) as response:
                body = response.read()
                project_id = body.decode('utf-8').strip().strip('"')
        except Exception as e:
            project_id = ""

        if project_id:
            _cached_project_id = project_id
            try:
                config_content = f'project-id = "{project_id}"\n'
                config_path.write_text(config_content, encoding='utf-8')
                try:
                    os.chmod(config_path, 0o600)
                except Exception:
                    pass
            except Exception:
                pass
            return _cached_project_id
    except Exception:
        pass

    # Try reading again in case created elsewhere
    if config_path.exists():
        try:
            content = config_path.read_text(encoding='utf-8')
            project_id = _parse_project_id_from_toml(content)
            if project_id:
                _cached_project_id = project_id
                return _cached_project_id
        except Exception:
            pass

    return _cached_project_id or ""


class EventDispatcher:
    """Sends recorded turns to LogAg's HTTP endpoint."""

    def __init__(self, url: str = LOGAG_URL) -> None:
        self.url = url

    def send_turn_data(self, user_input: str, agent_output: str) -> None:
        self._post({"userInput": user_input, "agentOutput": agent_output})

    def _post(self, payload: dict[str, Any]) -> dict[str, Any]:
        # Add project id
        project_id = get_or_create_project_id()
        if project_id:
            payload["projectLane"] = project_id
            payload["projectId"] = project_id
            payload["project_id"] = project_id
        request = Request(
            self.url,
            data=json.dumps(payload).encode("utf-8"),
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        with urlopen(request, timeout=10) as response:
            body = response.read()
        return json.loads(body) if body else {}


def _pending_state_files(session_id: str, turn_id: str) -> list[Path]:
    """Pending state files for a session/turn, newest first.

    Each UserPromptSubmit writes a uniquely named file, so several agents
    can be mid-turn on the same session/turn without overwriting each
    other. Stop consumes the newest pending prompt; the others stay for
    their own Stop events.
    """
    return sorted(
        (
            path
            for path in STATE_DIR.glob(f"{session_id}_{turn_id}_*.json")
            if path.is_file()
        ),
        key=lambda path: path.stat().st_mtime_ns,
        reverse=True,
    )


def handle_user_prompt_submit(payload: dict[str, Any]) -> None:
    session_id = payload.get("session_id")
    turn_id = payload.get("turn_id")
    prompt = payload.get("prompt")
    if not session_id or not turn_id or not prompt:
        return

    # Initialize/get project id on init/interaction
    get_or_create_project_id()

    STATE_DIR.mkdir(parents=True, exist_ok=True, mode=0o700)
    # Unique per submission: concurrent agents sharing this session/turn
    # never collide on the same file.
    path = STATE_DIR / f"{session_id}_{turn_id}_{uuid.uuid4().hex}.json"
    path.write_text(
        json.dumps({"session_id": session_id, "turn_id": turn_id, "prompt": prompt}),
        encoding="utf-8",
    )
    os.chmod(path, 0o600)


def handle_stop(payload: dict[str, Any]) -> None:
    session_id = payload.get("session_id")
    turn_id = payload.get("turn_id")
    last_assistant_message = payload.get("last_assistant_message")
    if not session_id or not turn_id or not last_assistant_message:
        return

    # Ensure project id exists before sending
    get_or_create_project_id()

    pending = _pending_state_files(session_id, turn_id)
    if not pending:
        return

    path = pending[0]
    try:
        user_input = json.loads(path.read_text(encoding="utf-8")).get("prompt", "")
    finally:
        path.unlink(missing_ok=True)

    if not user_input:
        return

    EventDispatcher().send_turn_data(user_input, last_assistant_message)


def main(raw_payload: str) -> None:
    try:
        payload = json.loads(raw_payload)
        event = payload.get("hook_event_name")
        if event == "UserPromptSubmit":
            handle_user_prompt_submit(payload)
        elif event == "Stop":
            handle_stop(payload)
    except Exception:
        # Fail open: a hook failure or LogAg outage must not affect the turn.
        return


if __name__ == "__main__":
    main(sys.stdin.read())
