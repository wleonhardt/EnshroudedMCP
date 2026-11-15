"""Read-only Windows process exit evidence; never a shutdown executor or save proof."""
import argparse
from contextlib import contextmanager
import ctypes
from ctypes import wintypes
from dataclasses import dataclass
from functools import cache
import json
import ntpath
import os


class ObservationRefused(RuntimeError):
    """A bounded reason without raw process metadata or system error text."""


@dataclass(frozen=True)
class Identity:
    pid: int
    creation_time: int  # Raw Windows FILETIME; no wall-clock conversion.
    executable: str


@cache
def _api():
    if os.name != "nt":
        raise ObservationRefused("unsupported_platform")
    api = ctypes.WinDLL("kernel32", use_last_error=True)
    filetime = ctypes.POINTER(wintypes.FILETIME)
    dword = ctypes.POINTER(wintypes.DWORD)
    for name, arguments, result in (
        ("OpenProcess", [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD], wintypes.HANDLE),
        ("CloseHandle", [wintypes.HANDLE], wintypes.BOOL),
        ("GetProcessTimes", [wintypes.HANDLE, filetime, filetime, filetime, filetime], wintypes.BOOL),
        ("QueryFullProcessImageNameW", [wintypes.HANDLE, wintypes.DWORD, wintypes.LPWSTR, dword], wintypes.BOOL),
        ("WaitForSingleObject", [wintypes.HANDLE, wintypes.DWORD], wintypes.DWORD),
        ("GetExitCodeProcess", [wintypes.HANDLE, dword], wintypes.BOOL),
    ):
        function = getattr(api, name)
        function.argtypes, function.restype = arguments, result
    return api


@contextmanager
def _handle(pid):
    if type(pid) is not int or not 1 <= pid <= 0xFFFFFFFF:
        raise ObservationRefused("invalid_target")
    api = _api()
    # PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE; no write/terminate rights.
    handle = api.OpenProcess(0x1000 | 0x00100000, False, pid)
    if not handle:
        raise ObservationRefused("observation_unavailable")
    try:
        yield handle
    finally:
        api.CloseHandle(handle)


def _identity(handle, pid):
    times = [wintypes.FILETIME() for _ in range(4)]
    if not _api().GetProcessTimes(handle, *(ctypes.byref(value) for value in times)):
        raise ObservationRefused("observation_unavailable")
    size = wintypes.DWORD(32768)
    path = ctypes.create_unicode_buffer(size.value)
    if not _api().QueryFullProcessImageNameW(handle, 0, path, ctypes.byref(size)):
        raise ObservationRefused("observation_unavailable")
    creation = (times[0].dwHighDateTime << 32) | times[0].dwLowDateTime
    return Identity(pid, creation, path.value)


def _exited(handle, timeout_ms):
    result = _api().WaitForSingleObject(handle, timeout_ms)
    if result not in (0, 258):  # WAIT_OBJECT_0, WAIT_TIMEOUT
        raise ObservationRefused("observation_unavailable")
    return result == 0


def capture_identity(pid):
    """Capture a caller-selected live PID; the caller independently chooses its executable."""
    with _handle(pid) as handle:
        if _exited(handle, 0):
            raise ObservationRefused("exited_before_observation")
        return _identity(handle, pid)


def observe(expected, timeout_ms=10000, ready=None):
    """Pin and verify identity before readiness; hold that handle through one bounded wait."""
    if (type(timeout_ms) is not int or not 0 <= timeout_ms <= 30000 or
            type(expected.creation_time) is not int or expected.creation_time <= 0 or
            not isinstance(expected.executable, str) or not ntpath.isabs(expected.executable) or
            not ntpath.splitdrive(expected.executable)[0]):
        raise ObservationRefused("invalid_target")
    with _handle(expected.pid) as handle:
        if _exited(handle, 0):
            raise ObservationRefused("exited_before_observation")
        actual = _identity(handle, expected.pid)
        if (actual.creation_time != expected.creation_time or
                ntpath.normcase(ntpath.normpath(actual.executable)) != ntpath.normcase(ntpath.normpath(expected.executable))):
            raise ObservationRefused("identity_mismatch")
        if _exited(handle, 0):
            raise ObservationRefused("exited_before_observation")
        if ready is not None:
            ready()
        exited = _exited(handle, timeout_ms)
        code = wintypes.DWORD()
        if exited and not _api().GetExitCodeProcess(handle, ctypes.byref(code)):
            raise ObservationRefused("observation_unavailable")
        return {"state": "exited" if exited else "running", "identity_verified": True,
                "process_exit_verified": exited, "exit_code": code.value if exited else None,
                "successful_exit": code.value == 0 if exited else None,
                "final_save_verified": False, "causal_completion_verified": False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pid", required=True, type=int)
    parser.add_argument("--expected-creation", required=True, type=int)
    parser.add_argument("--expected-executable", required=True)
    parser.add_argument("--timeout-ms", type=int, default=10000)
    args = parser.parse_args()
    def emit(value):
        print(json.dumps(value), flush=True)
    try:
        result = observe(Identity(args.pid, args.expected_creation, args.expected_executable), args.timeout_ms,
                         ready=lambda: emit({"state": "observing", "identity_verified": True}))
        emit(result)
        return 0 if result["process_exit_verified"] else 2
    except ObservationRefused as error:
        emit({"state": "refused", "identity_verified": False, "reason": str(error)})
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
