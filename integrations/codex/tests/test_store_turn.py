"""Unit tests for the Codex hook script (integrations/codex/hooks/store_turn.py).

Requires only the Python standard library. Run from the repo root with:

    python3 -m unittest discover -s integrations/codex/tests -v
    # or
    python3 -m unittest integrations/codex/tests/test_store_turn.py -v
"""

import importlib.util
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

_HOOKS_DIR = Path(__file__).resolve().parent.parent / "hooks"
_SPEC = importlib.util.spec_from_file_location("store_turn", _HOOKS_DIR / "store_turn.py")
assert _SPEC is not None and _SPEC.loader is not None
store_turn = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(store_turn)


class FakeDispatcher:
    """Stand-in for EventDispatcher that records what would be POSTed."""

    calls = []

    def __init__(self, url=None):
        self.url = url

    def send_turn_data(self, user_input, agent_output):
        FakeDispatcher.calls.append((user_input, agent_output))


class StoreTurnBehaviorTest(unittest.TestCase):
    """Tests the UserPromptSubmit/Stop handlers and the main dispatcher."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.state_dir = Path(self.tmp.name) / "logag"
        patcher = mock.patch.object(store_turn, "STATE_DIR", self.state_dir)
        patcher.start()
        self.addCleanup(patcher.stop)
        FakeDispatcher.calls = []

    def state_file(self, session_id, turn_id):
        return self.state_dir / f"{session_id}_{turn_id}.json"

    def stop_payload(self, session_id, turn_id, last_assistant_message="answer"):
        return {
            "hook_event_name": "Stop",
            "session_id": session_id,
            "turn_id": turn_id,
            "last_assistant_message": last_assistant_message,
        }

    def test_user_prompt_submit_writes_state_file(self):
        store_turn.handle_user_prompt_submit({
            "hook_event_name": "UserPromptSubmit",
            "session_id": "sess-1",
            "turn_id": "turn-1",
            "prompt": "Help me debug the auth panic",
        })

        path = self.state_file("sess-1", "turn-1")
        self.assertTrue(path.is_file())
        self.assertEqual(
            json.loads(path.read_text(encoding="utf-8")),
            {"session_id": "sess-1", "turn_id": "turn-1", "prompt": "Help me debug the auth panic"},
        )
        # State files can hold sensitive prompts: the file is owner-only
        # and the state dir is not group/world accessible.
        self.assertEqual(os.stat(path).st_mode & 0o777, 0o600)
        self.assertEqual(os.stat(self.state_dir).st_mode & 0o700, 0o700)

    def test_user_prompt_submit_overwrites_existing_state(self):
        self.state_dir.mkdir(parents=True)
        self.state_file("sess-1", "turn-1").write_text(
            json.dumps({"session_id": "sess-1", "turn_id": "turn-1", "prompt": "old"}),
            encoding="utf-8",
        )
        store_turn.handle_user_prompt_submit(
            {"session_id": "sess-1", "turn_id": "turn-1", "prompt": "new"}
        )
        self.assertEqual(
            json.loads(self.state_file("sess-1", "turn-1").read_text(encoding="utf-8"))["prompt"],
            "new",
        )

    def test_user_prompt_submit_skips_incomplete_payloads(self):
        for payload in (
            {"session_id": "s", "turn_id": "t"},                       # no prompt
            {"session_id": "s", "prompt": "p"},                        # no turn_id
            {"turn_id": "t", "prompt": "p"},                           # no session_id
            {"session_id": "", "turn_id": "t", "prompt": "p"},         # empty session_id
            {"session_id": "s", "turn_id": "", "prompt": "p"},         # empty turn_id
            {"session_id": "s", "turn_id": "t", "prompt": ""},         # empty prompt
        ):
            with self.subTest(payload=payload):
                store_turn.handle_user_prompt_submit(payload)
        self.assertFalse(self.state_dir.exists())

    def test_stop_pairs_stored_prompt_with_answer_and_cleans_up(self):
        self.state_dir.mkdir(parents=True)
        self.state_file("sess-1", "turn-1").write_text(
            json.dumps({"session_id": "sess-1", "turn_id": "turn-1", "prompt": "Why is it slow?"}),
            encoding="utf-8",
        )

        with mock.patch.object(store_turn, "EventDispatcher", FakeDispatcher):
            store_turn.handle_stop(self.stop_payload("sess-1", "turn-1", "Because of N+1 queries"))

        self.assertEqual(FakeDispatcher.calls, [("Why is it slow?", "Because of N+1 queries")])
        self.assertFalse(self.state_file("sess-1", "turn-1").exists())

    def test_stop_without_state_is_a_noop(self):
        with mock.patch.object(store_turn, "EventDispatcher", FakeDispatcher):
            store_turn.handle_stop(self.stop_payload("sess-9", "turn-9"))

        self.assertEqual(FakeDispatcher.calls, [])
        self.assertFalse(self.state_dir.exists())

    def test_stop_skips_when_payload_incomplete(self):
        self.state_dir.mkdir(parents=True)
        self.state_file("sess-1", "turn-1").write_text(
            json.dumps({"session_id": "sess-1", "turn_id": "turn-1", "prompt": "p"}),
            encoding="utf-8",
        )

        with mock.patch.object(store_turn, "EventDispatcher", FakeDispatcher):
            store_turn.handle_stop({"session_id": "sess-1", "turn_id": "turn-1"})          # no answer
            store_turn.handle_stop({"session_id": "sess-1", "last_assistant_message": "a"})  # no turn_id
            store_turn.handle_stop({"turn_id": "turn-1", "last_assistant_message": "a"})     # no session_id

        self.assertEqual(FakeDispatcher.calls, [])
        # Early returns leave the state file in place for a later Stop.
        self.assertTrue(self.state_file("sess-1", "turn-1").exists())

    def test_concurrent_sessions_and_turns_do_not_collide(self):
        for session, turn, prompt in (
            ("sess-A", "turn-1", "prompt A1"),
            ("sess-B", "turn-1", "prompt B1"),
            ("sess-A", "turn-2", "prompt A2"),
        ):
            store_turn.handle_user_prompt_submit(
                {"session_id": session, "turn_id": turn, "prompt": prompt}
            )

        self.assertEqual(
            sorted(p.name for p in self.state_dir.iterdir()),
            ["sess-A_turn-1.json", "sess-A_turn-2.json", "sess-B_turn-1.json"],
        )

        with mock.patch.object(store_turn, "EventDispatcher", FakeDispatcher):
            store_turn.handle_stop(self.stop_payload("sess-A", "turn-2", "answer A2"))
            store_turn.handle_stop(self.stop_payload("sess-B", "turn-1", "answer B1"))

        self.assertEqual(FakeDispatcher.calls, [("prompt A2", "answer A2"), ("prompt B1", "answer B1")])
        self.assertFalse(self.state_file("sess-A", "turn-2").exists())
        self.assertFalse(self.state_file("sess-B", "turn-1").exists())
        # The un-answered turn's prompt is preserved.
        self.assertTrue(self.state_file("sess-A", "turn-1").exists())

    def test_main_dispatches_on_hook_event_name(self):
        with mock.patch.object(store_turn, "handle_user_prompt_submit") as user_hook, \
                mock.patch.object(store_turn, "handle_stop") as stop_hook:
            store_turn.main(json.dumps({
                "hook_event_name": "UserPromptSubmit",
                "session_id": "s", "turn_id": "t", "prompt": "p",
            }))
            store_turn.main(json.dumps({
                "hook_event_name": "Stop",
                "session_id": "s", "turn_id": "t", "last_assistant_message": "a",
            }))
            store_turn.main(json.dumps({"hook_event_name": "SessionStart", "session_id": "s"}))

        user_hook.assert_called_once_with(
            {"hook_event_name": "UserPromptSubmit", "session_id": "s", "turn_id": "t", "prompt": "p"}
        )
        stop_hook.assert_called_once_with(
            {"hook_event_name": "Stop", "session_id": "s", "turn_id": "t", "last_assistant_message": "a"}
        )

    def test_main_swallows_malformed_input(self):
        for raw in ("this is not json", "", "42", '{"hook_event_name":}'):
            with self.subTest(raw=raw):
                store_turn.main(raw)  # must not raise

    def test_main_fails_open_on_dispatcher_error(self):
        self.state_dir.mkdir(parents=True)
        self.state_file("sess-1", "turn-1").write_text(
            json.dumps({"session_id": "sess-1", "turn_id": "turn-1", "prompt": "p"}),
            encoding="utf-8",
        )

        class OfflineDispatcher:
            def __init__(self, url=None):
                self.url = url

            def send_turn_data(self, user_input, agent_output):
                raise OSError("connection refused")

        with mock.patch.object(store_turn, "EventDispatcher", OfflineDispatcher):
            store_turn.main(json.dumps(self.stop_payload("sess-1", "turn-1")))  # must not raise

        # A failed POST must not interrupt the turn, but the state file is
        # still cleaned up so it never retries a dead turn.
        self.assertFalse(self.state_file("sess-1", "turn-1").exists())

    def test_corrupt_state_file_fails_open_and_is_cleaned(self):
        self.state_dir.mkdir(parents=True)
        self.state_file("sess-1", "turn-1").write_text("{not json", encoding="utf-8")

        with mock.patch.object(store_turn, "EventDispatcher", FakeDispatcher):
            store_turn.main(json.dumps(self.stop_payload("sess-1", "turn-1")))

        self.assertEqual(FakeDispatcher.calls, [])
        self.assertFalse(self.state_file("sess-1", "turn-1").exists())


class EventDispatcherTest(unittest.TestCase):
    """Tests the HTTP payload sent to LogAg's /record endpoint."""

    def test_default_url_points_at_logag_record(self):
        self.assertEqual(store_turn.EventDispatcher().url, "http://localhost:8000/record")

    def test_custom_url_respected(self):
        self.assertEqual(
            store_turn.EventDispatcher("http://127.0.0.1:9999/x").url,
            "http://127.0.0.1:9999/x",
        )

    def test_send_turn_data_posts_the_expected_request(self):
        captured = {}

        class FakeResponse:
            def __enter__(self):
                return self

            def __exit__(self, *exc):
                return False

            def read(self):
                return b"{}"

        def fake_urlopen(request, timeout=None):
            captured["request"] = request
            captured["timeout"] = timeout
            return FakeResponse()

        with mock.patch.object(store_turn, "urlopen", side_effect=fake_urlopen):
            store_turn.EventDispatcher().send_turn_data("Why is it slow?", "N+1 queries")

        request = captured["request"]
        self.assertEqual(request.full_url, "http://localhost:8000/record")
        self.assertEqual(request.method, "POST")
        self.assertEqual(
            json.loads(request.data),
            {"userInput": "Why is it slow?", "agentOutput": "N+1 queries"},
        )
        content_type = next(
            value for key, value in request.headers.items()
            if key.lower() == "content-type"
        )
        self.assertEqual(content_type, "application/json")
        self.assertEqual(captured["timeout"], 10)

    def test_send_turn_data_tolerates_empty_response_body(self):
        class FakeResponse:
            def __enter__(self):
                return self

            def __exit__(self, *exc):
                return False

            def read(self):
                return b""

        with mock.patch.object(store_turn, "urlopen", return_value=FakeResponse()):
            store_turn.EventDispatcher().send_turn_data("q", "a")  # must not raise


if __name__ == "__main__":
    unittest.main()