"""Windows launch facts and identity checks; no launch, stop or game executor.

Caller retains its own Popen object while reading launcher facts. CIM contributes
only live-child parent linkage/path; exact creation times always come from Win32.
"""
import ctypes
from ctypes import wintypes
import ntpath
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scripts"))
from process_exit import ObservationRefused, _api, _exited


def require(condition, reason):
    if not condition:
        raise ObservationRefused(reason)


def same_path(left, right):
    return (isinstance(left, str) and isinstance(right, str)
            and ntpath.normcase(ntpath.normpath(left)) == ntpath.normcase(ntpath.normpath(right)))


def launcher_facts(process, requested_executable):
    """Read the caller-owned launch, including after exit while Popen retains it.

    Requested executable is request evidence, never a substitute for an observed
    image. An unavailable exited image remains null. Live image failure refuses.
    """
    require(ntpath.isabs(requested_executable), "invalid_requested_executable")
    # Borrow CPython's CreateProcess handle; do not reselect an exited PID or
    # close the handle owned by the retained Popen object (tested Python 3.15).
    _api()  # Refuse unsupported platforms before touching the Windows handle.
    handle = int(process._handle)
    times = [wintypes.FILETIME() for _ in range(4)]
    require(_api().GetProcessTimes(handle, *(ctypes.byref(t) for t in times)), "launcher_times_unavailable")
    creation = (times[0].dwHighDateTime << 32) | times[0].dwLowDateTime
    size = wintypes.DWORD(32768)
    image = ctypes.create_unicode_buffer(size.value)
    observed = (image.value if _api().QueryFullProcessImageNameW(handle, 0, image, ctypes.byref(size)) else None)
    exited = _exited(handle, 0)
    require(observed is not None or exited, "live_launcher_image_unavailable")
    require(observed is None or same_path(observed, requested_executable), "launcher_image_mismatch")
    code = None
    if exited:
        value = wintypes.DWORD()
        require(_api().GetExitCodeProcess(handle, ctypes.byref(value)), "launcher_exit_unavailable")
        code = value.value
    return {"pid": process.pid, "creation_time": creation,
            "requested_executable": requested_executable, "observed_executable": observed,
            "image_observed": observed is not None,
            "state": "exited" if exited else "running", "exit_code": code}


def checked_launcher_facts(process, requested_executable, record):
    """Record each held-handle sample before refusing an unsuccessful exit.

    The caller supplies the evidence writer. A write failure propagates before
    any sample is returned; neither failure permits binding or another launch.
    """
    facts = launcher_facts(process, requested_executable)
    record(facts.copy())
    require(facts["state"] != "exited" or facts["exit_code"] == 0, "launcher_exit_not_successful")
    return facts


def direct_identity(initial, current):
    require(initial["state"] == "running" and initial["exit_code"] is None, "direct_launcher_not_running")
    require(initial["pid"] == current.pid and initial["creation_time"] == current.creation_time, "direct_identity_changed")
    require(same_path(current.executable, initial["requested_executable"]), "effective_image_mismatch")
    return {"mode": "direct", "effective_pid": current.pid, "full_precision_identity_verified": True}


def handoff_identity(initial, parent_before, parent_now, child, metadata, expected_parent_executable,
                     *, expected_effective_executable=None):
    require(initial["state"] == "exited" and initial["exit_code"] == 0, "launcher_exit_not_successful")
    require(parent_before.pid == parent_now.pid and parent_before.creation_time == parent_now.creation_time,
            "parent_identity_changed")
    require(same_path(parent_before.executable, parent_now.executable)
            and same_path(parent_now.executable, expected_parent_executable), "parent_image_mismatch")
    require(child.pid not in (initial["pid"], parent_now.pid), "handoff_pid_not_distinct")
    require(parent_before.creation_time < initial["creation_time"] < child.creation_time
            <= initial["creation_time"] + 30 * 10000000, "handoff_creation_order_invalid")
    expected = initial["requested_executable"] if expected_effective_executable is None else expected_effective_executable
    require(isinstance(expected, str) and ntpath.isabs(expected), "invalid_effective_executable")
    require(same_path(child.executable, expected), "effective_image_mismatch")
    require(metadata["ProcessId"] == child.pid and metadata["ParentProcessId"] == parent_now.pid,
            "child_parent_link_mismatch")
    require(same_path(metadata["ExecutablePath"], child.executable), "child_metadata_image_mismatch")
    return {"mode": "handoff", "effective_pid": child.pid, "full_precision_identity_verified": True,
            "parent_identity_verified": True, "initial_image_observed": initial["image_observed"]}


def listener_identity(identity, rows, port):
    require(len(rows) == 1, "listener_count_mismatch")
    row = rows[0]
    require(row["LocalAddress"] == "127.0.0.1" and row["LocalPort"] == port
            and row["OwningProcess"] == identity.pid, "listener_identity_mismatch")
