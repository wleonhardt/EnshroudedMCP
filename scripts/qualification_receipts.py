"""Read-only cohort metadata correlation; no artifact reads or execution authority."""
import json
from pathlib import Path
import re


class ReceiptRefused(ValueError):
    """A bounded reason without source paths or receipt contents."""


def _require(condition, reason):
    if not condition:
        raise ReceiptRefused(reason)


def read_candidates(cohort: Path, *, source_commit: str, versions: dict[str, str]) -> dict:
    """Read only cohort/candidate-provenance.json against independent expectations.

    The caller must obtain expectations from its frozen plan/compiler profile,
    not from this receipt. This does not verify artifact bytes, signatures,
    installed files, native behavior or permission to install/launch.
    """
    _require(isinstance(source_commit, str) and re.fullmatch(r"[0-9a-f]{40}", source_commit),
             "invalid_source_expectation")
    _require(isinstance(versions, dict) and versions and
             all(isinstance(name, str) and isinstance(version, str) and name and version
                 for name, version in versions.items()), "invalid_version_expectation")
    try:
        with (cohort / "candidate-provenance.json").open("rb") as stream:
            raw = stream.read(65537)
        _require(len(raw) <= 65536, "receipt_too_large")
        data = json.loads(raw)
    except (OSError, ValueError) as error:
        if isinstance(error, ReceiptRefused):
            raise
        raise ReceiptRefused("receipt_unavailable_or_invalid") from None
    _require(isinstance(data, dict), "invalid_receipt")
    _require(data.get("source_commit") == source_commit, "source_mismatch")
    candidates = data.get("candidates")
    _require(isinstance(candidates, dict) and set(candidates) == set(versions), "artifact_set_mismatch")
    for name, version in versions.items():
        row = candidates[name]
        _require(isinstance(row, dict), "invalid_artifact_record")
        _require(row.get("package_version") == version, "artifact_version_mismatch")
        _require(type(row.get("bytes")) is int and row["bytes"] > 0, "invalid_artifact_size")
        _require(isinstance(row.get("sha256"), str) and re.fullmatch(r"[0-9a-f]{64}", row["sha256"]),
                 "invalid_artifact_hash")
        _require(isinstance(row.get("path"), str) and row["path"], "invalid_artifact_path")
    return candidates


def verify_shutdown_intent(record, *, operation: str, session: str) -> None:
    """Correlate an already-read owned intent; never reconstruct execution authority."""
    _require(isinstance(operation, str) and re.fullmatch(r"[A-Za-z0-9_-]{1,64}", operation),
             "invalid_operation_expectation")
    _require(isinstance(session, str) and 1 <= len(session) <= 128, "invalid_session_expectation")
    _require(record == {"kind": "server_shutdown_intent", "operation": operation, "session": session},
             "shutdown_intent_mismatch")
