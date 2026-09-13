#!/usr/bin/env python3
"""Record a finished Codex turn into LogAg via ``POST /record``.

This script is installed as a Codex ``Stop`` hook (see ``hooks.json``).
When a turn completes, Codex pipes one JSON object on stdin containing at
least:

* ``transcript_path`` - path to the session rollout (JSONL) file, and
* ``last_assistant_message`` - the last assistant message text.

The script extracts the user prompt of the finished turn from the transcript
and posts the user/agent pair to LogAg so it is stored as a LogEvent with its
embedding. Failures are swallowed: a LogAg outage must never interrupt or
block a Codex turn.
"""

import json
import sys
from typing import Any, Iterable
from urllib.request import Request, urlopen

LOGAG_URL = "http://localhost:8000/record"


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


class TranscriptReader:
    """Extracts the turn's user prompt from a Codex rollout transcript.

    A rollout is a JSONL file where user/assistant turns are recorded as
    ``response_item`` entries whose ``payload`` looks like::

        {"type": "message", "role": "user",
         "content": [{"type": "input_text", "text": "..."}]}

    Codex also injects synthetic ``<...>``-wrapped user blocks (plugin
    suggestions, continuation banners, etc.); those are not real prompts and
    are skipped.
    """

    @staticmethod
    def _payload(jsonl_entry: str) -> dict[str, Any]:
        return json.loads(jsonl_entry).get("payload", {})

    @staticmethod
    def _entry_text(jsonl_entry: str) -> str:
        content = TranscriptReader._payload(jsonl_entry).get("content", [])
        if not isinstance(content, list):
            return ""
        return "\n".join(
            entry.get("text", "")
            for entry in content
            if isinstance(entry, dict) and entry.get("text")
        )

    @staticmethod
    def _is_synthetic(text: str) -> bool:
        stripped = text.strip()
        return stripped.startswith("<") and stripped.endswith(">")

    def fetch_transcript(self, file_path: str) -> Iterable[str]:
        with open(file_path, encoding="utf-8") as file:
            for line in file:
                line = line.strip()
                if line:
                    yield line

    def get_user_input(self, file_path: str) -> str:
        user_input = ""
        for line in self.fetch_transcript(file_path):
            payload = self._payload(line)
            if payload.get("type") != "message" or payload.get("role") != "user":
                continue

            text = self._entry_text(line)
            if text and not self._is_synthetic(text):
                user_input = text

        return user_input


def main(raw_payload: str) -> None:
    try:
        payload = json.loads(raw_payload)
        transcript_path = payload.get("transcript_path")
        last_assistant_message = payload.get("last_assistant_message")
        if not transcript_path or not last_assistant_message:
            return

        user_input = TranscriptReader().get_user_input(transcript_path)
        if not user_input:
            return

        EventDispatcher().send_turn_data(user_input, last_assistant_message)
    except Exception:
        # Fail open: a missing/offline LogAg must not affect the Codex turn.
        return


if __name__ == "__main__":
    main(sys.stdin.read())