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

    def state_files(self, session_id, turn_id):
        """Pending state files for a session/turn, oldest first."""
        return sorted(
            (
                path
                for path in self.state_dir.glob(f"{session_id}_{turn_id}_*.json")
                if path.is_file()
            ),
            key=lambda path: path.stat().st_mtime_ns,
        )

    def state_file(self, session_id, turn_id):
        """Newest pending state file, or a non-existent placeholder path."""
        files = self.state_files(session_id, turn_id)
        if files:
            return files[-1]
        return self.state_dir / f"{session_id}_{turn_id}_empty.json"

    def stop_payload(self, session_id, turn_id, last_assistant_message="answer"):
        return {
            "hook_event_name": "Stop",
            "session_id": session_id,
            "turn_id": turn_id,
            "last_assistant_message": last_assistant_message,
        }

    def seed_state(self, session_id, turn_id, prompt="p"):
        """Write a pending state file as a UserPromptSubmit would have."""
        self.state_dir.mkdir(parents=True)
        path = self.state_dir / f"{session_id}_{turn_id}_seed.json"
        path.write_text(
            json.dumps({"session_id": session_id, "turn_id": turn_id, "prompt": prompt}),
            encoding="utf-8",
        )
        return path

    def test_user_prompt_submit_writes_state_file(self):
        store_turn.handle_user_prompt_submit({
            "hook_event_name": "UserPromptSubmit",
            "session_id": "sess-1",
            "turn_id": "turn-1",
            "prompt": "Help me debug the auth panic",
        })

        path = self.state_files("sess-1", "turn-1")
        self.assertEqual(len(path), 1)
        self.assertTrue(path[0].is_file())
        # Every submission gets a UUID-tagged name so concurrent agents
        # sharing this session/turn never collide on the same file.
        self.assertRegex(path[0].name, r"^sess-1_turn-1_[0-9a-f]{32}\.json$")
        self.assertEqual(
            json.loads(path[0].read_text(encoding="utf-8")),
            {"session_id": "sess-1", "turn_id": "turn-1", "prompt": "Help me debug the auth panic"},
        )
        # State files can hold sensitive prompts: the file is owner-only
        # and the state dir is not group/world accessible.
        self.assertEqual(os.stat(path[0]).st_mode & 0o777, 0o600)
        self.assertEqual(os.stat(self.state_dir).st_mode & 0o700, 0o700)

    def test_user_prompt_submit_creates_distinct_file_per_submission(self):
        self.state_dir.mkdir(parents=True)
        self.state_file("sess-1", "turn-1").write_text(
            json.dumps({"session_id": "sess-1", "turn_id": "turn-1", "prompt": "old"}),
            encoding="utf-8",
        )
        store_turn.handle_user_prompt_submit(
            {"session_id": "sess-1", "turn_id": "turn-1", "prompt": "new"}
        )

        files = self.state_files("sess-1", "turn-1")
        self.assertEqual(len(files), 2)  # nothing was overwritten
        self.assertEqual(json.loads(files[-1].read_text(encoding="utf-8"))["prompt"], "new")

    def test_user_prompt_submit_skips_when_prompt_missing(self):
        store_turn.handle_user_prompt_submit({"session_id": "s", "turn_id": "t"})
        self.assertFalse(self.state_dir.exists())

    def test_user_prompt_submit_skips_when_turn_id_missing(self):
        store_turn.handle_user_prompt_submit({"session_id": "s", "prompt": "p"})
        self.assertFalse(self.state_dir.exists())

    def test_user_prompt_submit_skips_when_session_id_missing(self):
        store_turn.handle_user_prompt_submit({"turn_id": "t", "prompt": "p"})
        self.assertFalse(self.state_dir.exists())

    def test_user_prompt_submit_skips_when_session_id_empty(self):
        store_turn.handle_user_prompt_submit(
            {"session_id": "", "turn_id": "t", "prompt": "p"}
        )
        self.assertFalse(self.state_dir.exists())

    def test_user_prompt_submit_skips_when_turn_id_empty(self):
        store_turn.handle_user_prompt_submit(
            {"session_id": "s", "turn_id": "", "prompt": "p"}
        )
        self.assertFalse(self.state_dir.exists())

    def test_user_prompt_submit_skips_when_prompt_empty(self):
        store_turn.handle_user_prompt_submit(
            {"session_id": "s", "turn_id": "t", "prompt": ""}
        )
        self.assertFalse(self.state_dir.exists())

    def test_stop_pairs_stored_prompt_with_answer_and_cleans_up(self):
        self.seed_state("sess-1", "turn-1", "Why is it slow?")

        with mock.patch.object(store_turn, "EventDispatcher", FakeDispatcher):
            store_turn.handle_stop(self.stop_payload("sess-1", "turn-1", "Because of N+1 queries"))

        self.assertEqual(FakeDispatcher.calls, [("Why is it slow?", "Because of N+1 queries")])
        self.assertFalse(self.state_file("sess-1", "turn-1").exists())

    def test_stop_without_state_is_a_noop(self):
        with mock.patch.object(store_turn, "EventDispatcher", FakeDispatcher):
            store_turn.handle_stop(self.stop_payload("sess-9", "turn-9"))

        self.assertEqual(FakeDispatcher.calls, [])
        self.assertFalse(self.state_dir.exists())

    def test_stop_skips_when_answer_missing(self):
        self.seed_state("sess-1", "turn-1")

        with mock.patch.object(store_turn, "EventDispatcher", FakeDispatcher):
            store_turn.handle_stop({"session_id": "sess-1", "turn_id": "turn-1"})

        self.assertEqual(FakeDispatcher.calls, [])
        # Early return leaves the state file in place for a later Stop.
        self.assertTrue(self.state_file("sess-1", "turn-1").exists())

    def test_stop_skips_when_turn_id_missing(self):
        self.seed_state("sess-1", "turn-1")

        with mock.patch.object(store_turn, "EventDispatcher", FakeDispatcher):
            store_turn.handle_stop({"session_id": "sess-1", "last_assistant_message": "a"})

        self.assertEqual(FakeDispatcher.calls, [])
        self.assertTrue(self.state_file("sess-1", "turn-1").exists())

    def test_stop_skips_when_session_id_missing(self):
        self.seed_state("sess-1", "turn-1")

        with mock.patch.object(store_turn, "EventDispatcher", FakeDispatcher):
            store_turn.handle_stop({"turn_id": "turn-1", "last_assistant_message": "a"})

        self.assertEqual(FakeDispatcher.calls, [])
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

        self.assertEqual(len(self.state_files("sess-A", "turn-1")), 1)
        self.assertEqual(len(self.state_files("sess-B", "turn-1")), 1)
        self.assertEqual(len(self.state_files("sess-A", "turn-2")), 1)

        with mock.patch.object(store_turn, "EventDispatcher", FakeDispatcher):
            store_turn.handle_stop(self.stop_payload("sess-A", "turn-2", "answer A2"))
            store_turn.handle_stop(self.stop_payload("sess-B", "turn-1", "answer B1"))

        self.assertEqual(FakeDispatcher.calls, [("prompt A2", "answer A2"), ("prompt B1", "answer B1")])
        self.assertFalse(self.state_file("sess-A", "turn-2").exists())
        self.assertFalse(self.state_file("sess-B", "turn-1").exists())
        # The un-answered turn's prompt is preserved.
        self.assertTrue(self.state_file("sess-A", "turn-1").exists())

    def test_multiple_agents_on_same_session_turn_do_not_overwrite(self):
        # Two agents mid-turn on the same session/turn: each gets its own
        # UUID-tagged file, so neither prompt clobbers the other.
        store_turn.handle_user_prompt_submit(
            {"session_id": "sess-1", "turn_id": "turn-1", "prompt": "agenta prompt"}
        )
        store_turn.handle_user_prompt_submit(
            {"session_id": "sess-1", "turn_id": "turn-1", "prompt": "agentb prompt"}
        )

        files = self.state_files("sess-1", "turn-1")
        self.assertEqual(len(files), 2)
        self.assertNotEqual(files[0].name, files[1].name)

        # Each Stop consumes one pending prompt (newest first); neither
        # agent's prompt is lost.
        with mock.patch.object(store_turn, "EventDispatcher", FakeDispatcher):
            store_turn.handle_stop(self.stop_payload("sess-1", "turn-1", "answer B"))
            store_turn.handle_stop(self.stop_payload("sess-1", "turn-1", "answer A"))

        self.assertEqual(
            FakeDispatcher.calls,
            [("agentb prompt", "answer B"), ("agenta prompt", "answer A")],
        )
        self.assertEqual(self.state_files("sess-1", "turn-1"), [])

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

    def test_main_swallows_non_json_stdin(self):
        store_turn.main("this is not json")  # must not raise

    def test_main_swallows_empty_stdin(self):
        store_turn.main("")  # must not raise

    def test_main_swallows_scalar_json_stdin(self):
        store_turn.main("42")  # must not raise

    def test_main_swallows_malformed_json_stdin(self):
        store_turn.main('{"hook_event_name":}')  # must not raise

    def test_main_fails_open_on_dispatcher_error(self):
        self.seed_state("sess-1", "turn-1")

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
        (self.state_dir / "sess-1_turn-1_corrupt.json").write_text("{not json", encoding="utf-8")

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