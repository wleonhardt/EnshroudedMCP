"""Observation ordering with owned workspace files and synthetic callbacks only."""
import json
import os
from pathlib import Path
import tempfile
import unittest

from qualification_observations import ObservationFailed, observe

ROOT = Path(__file__).resolve().parents[1]


class ObservationTests(unittest.TestCase):
    def setUp(self):
        base = (ROOT / "target" / "observation-fixtures").resolve()
        self.assertTrue(base.is_relative_to(ROOT.resolve()))
        base.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=base)
        self.directory = Path(self.temp.name).resolve()
        self.assertTrue(self.directory.is_relative_to(base))
        self.addCleanup(self.temp.cleanup)
        self.path = self.directory / "observations.jsonl"
        self.started = False
        self.calls = 0

    def record(self, event):
        with self.path.open("ab" if self.started else "xb") as stream:
            stream.write((json.dumps(event) + "\n").encode())
            stream.flush()
            os.fsync(stream.fileno())
        self.started = True

    def rows(self):
        return [json.loads(line) for line in self.path.read_text().splitlines()]

    def action(self, reply):
        def call():
            self.calls += 1
            self.assertEqual(self.rows()[-1]["kind"], "observation_intent")
            return reply
        return call

    def test_first_native_refusal_is_durable_before_caller_asserts(self):
        reply = {"isError": True, "content": [{"type":"text", "text":"synthetic native refusal"}]}
        with self.assertRaises(AssertionError):
            result = observe("survey", self.action(reply), self.record)
            self.assertIs(result, reply)
            self.assertIs(result["isError"], False)
        rows = self.rows()
        self.assertEqual([r["kind"] for r in rows], ["observation_intent", "observation_reply"])
        self.assertEqual(rows[-1]["reply"], reply)
        self.assertEqual(self.calls, 1)

    def test_later_refusal_retains_prior_success_and_order(self):
        good = {"isError":False,"content":[]}
        bad = {"isError":True,"content":[{"text":"later native refusal"}]}
        self.assertIs(observe("first", self.action(good), self.record), good)
        self.assertIs(observe("second", self.action(bad), self.record), bad)
        rows = self.rows()
        self.assertEqual([r["label"] for r in rows], ["first","first","second","second"])
        self.assertEqual(rows[1]["reply"], good)
        self.assertEqual(rows[3]["reply"], bad)
        self.assertEqual(self.calls, 2)

    def test_transport_and_invalid_reply_record_bounded_reasons_without_retry(self):
        for error, reason in [(TimeoutError("synthetic-private-diagnostic"), "transport_error"),
                              (ValueError("synthetic-private-diagnostic"), "invalid_reply")]:
            def call():
                self.calls += 1
                raise error
            with self.subTest(reason=reason), self.assertRaises(ObservationFailed) as caught:
                observe(reason, call, self.record)
            self.assertEqual(str(caught.exception), reason)
            row = self.rows()[-1]
            self.assertEqual(row, {"kind":"observation_error","label":reason,"reason":reason})
            self.assertNotIn("synthetic-private-diagnostic", self.path.read_text())
        self.assertEqual(self.calls, 2)

    def test_existing_journal_refuses_before_action_without_overwriting(self):
        self.path.write_bytes(b"owned prior evidence")
        with self.assertRaisesRegex(ObservationFailed, "observation_record_unconfirmed"):
            observe("survey", self.action({}), self.record)
        self.assertEqual(self.calls, 0)
        self.assertEqual(self.path.read_bytes(), b"owned prior evidence")

    def test_partial_reply_record_failure_is_unknown_and_does_not_retry(self):
        def record(event):
            if event["kind"] == "observation_intent":
                self.record(event)
            else:
                with self.path.open("ab") as stream:
                    stream.write(b"partial reply")
                    stream.flush()
                    os.fsync(stream.fileno())
                raise OSError("synthetic-private-write-error")
        with self.assertRaisesRegex(ObservationFailed, "observation_record_unconfirmed"):
            observe("survey", self.action({"isError":True}), record)
        self.assertEqual(self.calls, 1)
        self.assertTrue(self.path.read_bytes().endswith(b"partial reply"))

    def test_error_record_failure_remains_unconfirmed(self):
        def call():
            self.calls += 1
            raise TimeoutError("synthetic-private-transport-error")
        def record(event):
            if event["kind"] == "observation_intent": self.record(event)
            else: raise OSError("synthetic-private-write-error")
        with self.assertRaises(ObservationFailed) as caught:
            observe("survey", call, record)
        self.assertEqual(str(caught.exception), "observation_record_unconfirmed")
        self.assertEqual(self.calls, 1)
        self.assertEqual(len(self.rows()), 1)

    def test_returned_json_shapes_are_recorded_without_normalization(self):
        for reply in [None, [], {}, {"unexpected":"shape"}]:
            self.assertIs(observe("shape", self.action(reply), self.record), reply)
            self.assertEqual(self.rows()[-1]["reply"], reply)
        self.assertEqual(self.calls, 4)

    def test_unexpected_programming_failure_is_not_reclassified(self):
        error = AssertionError("synthetic programming failure")
        def call():
            self.calls += 1
            raise error
        with self.assertRaises(AssertionError) as caught:
            observe("survey", call, self.record)
        self.assertIs(caught.exception, error)
        self.assertEqual(self.calls, 1)
        self.assertEqual([r["kind"] for r in self.rows()], ["observation_intent"])


if __name__ == "__main__":
    unittest.main()
