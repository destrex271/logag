#!/usr/bin/env python3
"""Record Codex turns into LogAg via Codex lifecycle hooks.

Two hooks drive this script (see ``hooks.json``):

* ``UserPromptSubmit`` - fires right before a user prompt is sent. The
  script stores ``{session_id, turn_id, prompt}`` in a temp file so the
  matching ``Stop`` event can pair the prompt with the assistant's answer.

* ``Stop`` - fires when a turn completes. The script reads the stored
  prompt, pairs it with ``last_assistant_message``, POSTs the pair to
  LogAg's ``/record`` endpoint, and removes the temp file.

State files live in ``<tempdir>/logag/`` and are keyed by ``session_id``
and ``turn_id`` so concurrent sessions and turns never collide. Failures
are swallowed: a LogAg outage or a missing state file must never interrupt
or block a Codex turn.
"""

import json
import os
import sys
import tempfile
from pathlib import Path
from typing import Any
from urllib.request import Request, urlopen

LOGAG_URL = "http://localhost:8000/record"

STATE_DIR = Path(tempfile.gettempdir()) / "logag"


class EventDispatcher:
    """Sends recorded turns to LogAg's HTTP endpoint."""

    def __init__(self, url: str = LOGAG_URL) -> None:
        self.url = url

    def send_turn_data(self, user_input: str, agent_output: str) -> None:
        self._post({"userInput": user_input, "agentOutput": agent_output})

    def _post(self, payload: dict[str, Any]) -> dict[str, Any]:
        request = Request(
            self.url,
            data=json.dumps(payload).encode("utf-8"),
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        with urlopen(request, timeout=10) as response:
            body = response.read()
        return json.loads(body) if body else {}


def _state_file(session_id: str, turn_id: str) -> Path:
    return STATE_DIR / f"{session_id}_{turn_id}.json"


def handle_user_prompt_submit(payload: dict[str, Any]) -> None:
    session_id = payload.get("session_id")
    turn_id = payload.get("turn_id")
    prompt = payload.get("prompt")
    if not session_id or not turn_id or not prompt:
        return

    STATE_DIR.mkdir(parents=True, exist_ok=True, mode=0o700)
    path = _state_file(session_id, turn_id)
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

    path = _state_file(session_id, turn_id)
    if not path.is_file():
        return

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