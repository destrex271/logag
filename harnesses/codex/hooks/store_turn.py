import json
import sys
from typing import Any, Iterable
from urllib.request import Request, urlopen


class EventDispatcher:
    url = "http://localhost:8000/record"

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
    def get_role_from_line(self, jsonl_entry: str) -> str | None:
        payload = json.loads(jsonl_entry).get("payload", {})
        return payload.get("role")

    def fetch_transcript(self, file_path: str) -> Iterable[str]:
        with open(file_path, encoding="utf-8") as file:
            for line in file:
                line = line.strip()
                if line:
                    yield line

    def get_first_user_input(self, file_path: str) -> str:
        for line in self.fetch_transcript(file_path):
            if self.get_role_from_line(line) != "user":
                continue

            content = json.loads(line)["payload"]["content"]
            return "\n".join(entry["text"] for entry in content if "text" in entry)

        return ""


def main(raw_payload: str) -> None:
    payload = json.loads(raw_payload)
    transcript_reader = TranscriptReader()
    EventDispatcher().send_turn_data(
        transcript_reader.get_first_user_input(payload["transcript_path"]),
        payload["last_assistant_message"],
    )


if __name__ == "__main__":
    main(sys.stdin.read())
