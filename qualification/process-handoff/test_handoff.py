"""Actual owned Python process/loopback checks and synthetic refusal cases."""
from contextlib import contextmanager
from dataclasses import replace
import json
import os
from pathlib import Path
import subprocess
import sys
import unittest

from fixture import line
from handoff import direct_identity, handoff_identity, launcher_facts, listener_identity
from process_exit import Identity, ObservationRefused, capture_identity

FIXTURE = Path(__file__).with_name("fixture.py")


def powershell(code):
    return json.loads(subprocess.check_output(["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", code], text=True))


@contextmanager
def owned(role):
    proc = subprocess.Popen([sys.executable, "-B", str(FIXTURE), role], stdin=subprocess.PIPE,
                            stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True,
                            creationflags=subprocess.CREATE_NO_WINDOW)
    try:
        yield proc
    finally:
        proc.stdin.close()
        try:
            code = proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()  # Only this fixture's exclusively owned Python process.
            proc.wait(timeout=5)
            raise
        finally:
            proc.stdout.close()
        assert code == 0


@unittest.skipUnless(os.name == "nt", "Windows process/loopback observation")
class ActualProcesses(unittest.TestCase):
    def bindings(self, port):
        rows = powershell(f"@(Get-NetTCPConnection -LocalPort {port} -State Listen -ErrorAction Stop | Select-Object LocalAddress,LocalPort,OwningProcess) | ConvertTo-Json -Compress")
        return [rows] if isinstance(rows, dict) else rows

    def test_direct_live_identity_and_owned_listener(self):
        with owned("child") as proc:
            port = json.loads(line(proc.stdout))["port"]
            facts = launcher_facts(proc, sys.executable)
            self.assertTrue(facts["image_observed"])
            current = capture_identity(proc.pid)
            self.assertEqual(direct_identity(facts, current)["mode"], "direct")
            listener_identity(current, self.bindings(port), port)
            with self.assertRaises(ObservationRefused):
                listener_identity(replace(current, pid=current.pid + 1), self.bindings(port), port)

    def test_exited_launcher_keeps_exit_creation_and_unknown_image_distinct(self):
        for exit_code in (0, 7, 259):
            with self.subTest(exit_code=exit_code):
                proc = subprocess.Popen([sys.executable, "-B", "-c", "import sys;sys.exit(int(sys.argv[1]))", str(exit_code)],
                                        stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                                        creationflags=subprocess.CREATE_NO_WINDOW)
                self.assertEqual(proc.wait(timeout=5), exit_code)
                facts = launcher_facts(proc, sys.executable)
                self.assertEqual(facts["state"], "exited")
                self.assertEqual(facts["exit_code"], exit_code)
                self.assertGreater(facts["creation_time"], 0)
                self.assertEqual(facts["image_observed"], facts["observed_executable"] is not None)
                with self.assertRaises(ObservationRefused):
                    direct_identity(facts, Identity(proc.pid, facts["creation_time"], sys.executable))

    def test_single_launcher_causes_owned_parent_handoff(self):
        with owned("parent") as parent:
            before = capture_identity(parent.pid)
            initial = subprocess.Popen([sys.executable, "-B", str(FIXTURE), "launcher"],
                                       stdin=subprocess.DEVNULL, stdout=parent.stdin, stderr=subprocess.DEVNULL,
                                       creationflags=subprocess.CREATE_NO_WINDOW)
            self.assertEqual(initial.wait(timeout=5), 0)
            facts = launcher_facts(initial, sys.executable)
            report = json.loads(line(parent.stdout))
            child = capture_identity(report["pid"])
            now = capture_identity(parent.pid)
            metadata = powershell(f"Get-CimInstance Win32_Process -Filter \"ProcessId={child.pid}\" | Select-Object ProcessId,ParentProcessId,ExecutablePath | ConvertTo-Json -Compress")
            # A lossy/unrelated CIM timestamp must never replace exact Win32 time.
            metadata["CreationFileTime"] = 0
            result = handoff_identity(facts, before, now, child, metadata, sys.executable)
            self.assertTrue(result["parent_identity_verified"])
            listener_identity(child, self.bindings(report["port"]), report["port"])
            with self.assertRaises(ObservationRefused):
                handoff_identity(facts, before, replace(now, creation_time=now.creation_time + 1), child, metadata, sys.executable)


class Refusals(unittest.TestCase):
    def setUp(self):
        self.initial = {"pid": 10, "creation_time": 200, "requested_executable": "C:/fixture/child.exe",
                        "observed_executable": None, "image_observed": False, "state": "exited", "exit_code": 0}
        self.parent = Identity(11, 100, "C:/fixture/parent.exe")
        self.child = Identity(12, 300, "C:/fixture/child.exe")
        self.metadata = {"ProcessId": 12, "ParentProcessId": 11, "ExecutablePath": self.child.executable}

    def check(self, initial=None, parent=None, child=None, metadata=None):
        return handoff_identity(initial or self.initial, self.parent, parent or self.parent,
                                child or self.child, metadata or self.metadata, self.parent.executable)

    def test_success_requires_exact_parent_creation_and_linkage(self):
        self.assertEqual(self.check()["mode"], "handoff")
        for parent, metadata in [
            (replace(self.parent, creation_time=101), self.metadata),
            (replace(self.parent, executable="C:/fixture/other.exe"), self.metadata),
            (self.parent, {**self.metadata, "ParentProcessId": 99}),
            (self.parent, {**self.metadata, "ExecutablePath": None}),
        ]:
            with self.subTest(parent=parent, metadata=metadata), self.assertRaises(ObservationRefused):
                self.check(parent=parent, metadata=metadata)

    def test_stale_late_wrong_image_and_unsuccessful_launch_refuse(self):
        cases = [
            ({**self.initial, "state": "running", "exit_code": None}, self.child),
            ({**self.initial, "exit_code": 7}, self.child),
            (self.initial, replace(self.child, creation_time=200)),
            (self.initial, replace(self.child, creation_time=300000201)),
            (self.initial, replace(self.child, executable="C:/fixture/other.exe")),
        ]
        for initial, child in cases:
            with self.subTest(initial=initial, child=child), self.assertRaises(ObservationRefused):
                self.check(initial=initial, child=child)

    def test_nonloopback_duplicate_and_wrong_port_refuse(self):
        good = {"LocalAddress": "127.0.0.1", "LocalPort": 1234, "OwningProcess": self.child.pid}
        listener_identity(self.child, [good], 1234)
        for rows in [[{**good, "LocalAddress": "0.0.0.0"}], [good, good], [{**good, "LocalPort": 1235}]]:
            with self.subTest(rows=rows), self.assertRaises(ObservationRefused):
                listener_identity(self.child, rows, 1234)


if __name__ == "__main__":
    unittest.main()
