"""Synthetic qualification receipts; no artifact, game or secret reads."""
import copy
import json
from pathlib import Path
import tempfile
import unittest

from qualification_receipts import ReceiptRefused, read_candidates, verify_shutdown_intent

ROOT = Path(__file__).resolve().parents[1]
SOURCE = "a" * 40
VERSIONS = {"mcp.dll": "0.51.0", "mcp_client.dll": "0.22.0"}


def receipt(source=SOURCE):
    return {"source_commit": source, "candidates": {
        name: {"path": "synthetic-unopened/" + name, "sha256": str(i) * 64,
               "bytes": i, "package_version": version}
        for i, (name, version) in enumerate(VERSIONS.items(), 1)}}


class ReceiptTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.cohort = Path(self.temp.name)

    def write(self, value, directory=None):
        directory = directory or self.cohort
        (directory / "candidate-provenance.json").write_text(json.dumps(value), encoding="utf-8")

    def read(self, directory=None):
        return read_candidates(directory or self.cohort, source_commit=SOURCE, versions=VERSIONS)

    def test_canonical_cohort_receipt_preserves_records_without_opening_artifacts(self):
        data = receipt()
        self.write(data)
        (self.cohort / "other.json").write_text("not the candidate receipt")
        self.assertEqual(self.read(), data["candidates"])
        self.assertFalse((self.cohort / "synthetic-unopened").exists())

    def test_distinct_old_cohort_is_refused_even_when_versions_match(self):
        old = self.cohort / "old"
        old.mkdir()
        data = receipt("b" * 40)
        data["candidates"]["mcp.dll"]["sha256"] = "c" * 64
        self.write(data, old)
        self.write(receipt())
        self.assertEqual(self.read(), receipt()["candidates"])
        with self.assertRaisesRegex(ReceiptRefused, "source_mismatch"):
            self.read(old)

    def test_missing_extra_and_wrong_version_artifacts_refuse(self):
        cases = []
        data = receipt(); del data["candidates"]["mcp.dll"]; cases.append(data)
        data = receipt(); data["candidates"]["unexpected.dll"] = data["candidates"]["mcp.dll"]; cases.append(data)
        data = receipt(); data["candidates"]["mcp.dll"]["package_version"] = "0.50.6"; cases.append(data)
        for data in cases:
            with self.subTest(data=data):
                self.write(data)
                with self.assertRaises(ReceiptRefused): self.read()

    def test_malformed_identity_size_hash_and_records_refuse(self):
        for key, value in [("bytes", True), ("bytes", 0), ("bytes", -1),
                           ("sha256", "f" * 63), ("sha256", "z" * 64),
                           ("path", ""), ("path", 7), ("package_version", None)]:
            data = receipt(); data["candidates"]["mcp.dll"][key] = value
            self.write(data)
            with self.subTest(key=key, value=value), self.assertRaises(ReceiptRefused): self.read()
        for data in [None, [], {}, {"source_commit": SOURCE, "candidates": []}]:
            self.write(data)
            with self.assertRaises(ReceiptRefused): self.read()
        self.write(receipt())
        for source in [None, "", "a" * 39]:
            with self.assertRaises(ReceiptRefused):
                read_candidates(self.cohort, source_commit=source, versions=VERSIONS)

    def test_missing_oversized_and_invalid_json_have_bounded_reasons(self):
        path = self.cohort / "candidate-provenance.json"
        for raw in [None, "synthetic private diagnostic", "x" * 65537]:
            if raw is not None: path.write_text(raw)
            with self.assertRaises(ReceiptRefused) as error: self.read()
            self.assertNotIn(str(self.cohort), str(error.exception))
            self.assertNotIn("synthetic private diagnostic", str(error.exception))

    def test_shutdown_intent_matches_shared_producer_fixture_and_exact_fields(self):
        record = json.loads((ROOT / "qualification/server-shutdown/intent.json").read_text())
        original = copy.deepcopy(record)
        verify_shutdown_intent(record, operation="con", session="active")
        self.assertEqual(record, original)
        for changed in [dict(record, kind="server_shutdown"), dict(record, extra=True),
                        {"kind":record["kind"], "operation":"con"}, None, []]:
            with self.assertRaises(ReceiptRefused):
                verify_shutdown_intent(changed, operation="con", session="active")

    def test_shutdown_wrong_operation_session_and_invalid_expectations_refuse(self):
        record = {"kind":"server_shutdown_intent", "operation":"new", "session":"current"}
        verify_shutdown_intent(record, operation="new", session="current")
        for operation, session in [("old","current"), ("new","old"), ("","current"),
                                   ("a/b","current"), ("界","current"), ("new",""),
                                   ("new", None), ("new", "x" * 129)]:
            with self.subTest(operation=operation, session=session), self.assertRaises(ReceiptRefused):
                verify_shutdown_intent(record, operation=operation, session=session)


if __name__ == "__main__":
    unittest.main()
