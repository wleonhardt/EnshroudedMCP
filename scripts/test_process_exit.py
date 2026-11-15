"""Independent exit evidence using disposable children only; no game processes."""
import json
import os
from pathlib import Path
import subprocess
import sys
import unittest
from contextlib import contextmanager

from process_exit import Identity, ObservationRefused, capture_identity, observe

SCRIPT = Path(__file__).with_name("process_exit.py")

@unittest.skipUnless(os.name == "nt", "Windows process observation")
class ProcessExitTests(unittest.TestCase):
    @contextmanager
    def child(self):
        process = subprocess.Popen(
            [sys.executable, "-c", "import sys; line=sys.stdin.readline(); sys.exit(int(line) if line else 0)"],
            stdin=subprocess.PIPE, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, text=True,
            creationflags=subprocess.CREATE_NO_WINDOW,
        )
        try:
            yield process
        finally:
            if process.stdin is not None:
                process.stdin.close()
                process.stdin = None
            process.wait(timeout=5)

    def started(self, child):
        return capture_identity(child.pid).creation_time

    def observer(self, child, ticks, executable=None, wait=10000):
        return subprocess.Popen(
            [sys.executable, "-B", str(SCRIPT), "--pid", str(child.pid),
             "--expected-creation", str(ticks), "--expected-executable", executable or sys.executable,
             "--timeout-ms", str(wait)],
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, creationflags=subprocess.CREATE_NO_WINDOW,
        )

    def test_successful_and_nonzero_exit_remain_separate_from_save_proof(self):
        for exit_code in (0, 7, 259):
            with self.subTest(exit_code=exit_code), self.child() as child:
                observer = self.observer(child, self.started(child))
                ready = json.loads(observer.stdout.readline())
                self.assertEqual(ready, {"state":"observing", "identity_verified":True})
                child.stdin.write(str(exit_code)+"\n")
                child.stdin.flush()
                output, error = observer.communicate(timeout=15)
                self.assertEqual(observer.returncode, 0, error)
                result = json.loads(output)
                self.assertEqual(result["state"], "exited")
                self.assertTrue(result["process_exit_verified"])
                self.assertEqual(result["exit_code"], exit_code)
                self.assertEqual(result["successful_exit"], exit_code == 0)
                self.assertFalse(result["final_save_verified"])
                self.assertFalse(result["causal_completion_verified"])
                self.assertNotIn(sys.executable.casefold(), output.casefold())
                self.assertNotIn("process_id", result)

    def test_timeout_does_not_stop_target_or_invent_an_exit_code(self):
        with self.child() as child:
            observer = self.observer(child, self.started(child), wait=50)
            output, error = observer.communicate(timeout=15)
            self.assertEqual(observer.returncode, 2, error)
            ready, result = map(json.loads, output.splitlines())
            self.assertEqual(ready["state"], "observing")
            self.assertEqual(result["state"], "running")
            self.assertIsNone(result["exit_code"])
            self.assertFalse(result["process_exit_verified"])
            self.assertIsNone(child.poll())

    def test_invalid_wait_or_identity_refuses_before_readiness(self):
        with self.child() as child:
            identity = capture_identity(child.pid)
            cases = [(identity, wait) for wait in (-1, 30001, True, 1.5)] + [
                (Identity(0, identity.creation_time, identity.executable), 0),
                (Identity(child.pid, 0, identity.executable), 0),
                (Identity(child.pid, identity.creation_time, "relative.exe"), 0),
            ]
            ready = []
            for expected, wait in cases:
                with self.assertRaisesRegex(ObservationRefused, "invalid_target"):
                    observe(expected, wait, ready=lambda: ready.append(True))
                self.assertIsNone(child.poll())
            self.assertEqual(ready, [])

    def test_wrong_creation_time_or_executable_refuses_without_touching_target(self):
        with self.child() as child:
            ticks = self.started(child)
            for expected_ticks, executable in ((ticks+1, sys.executable), (ticks, str(Path(sys.executable).with_name("unrelated.exe")))):
                observer = self.observer(child, expected_ticks, executable)
                output, error = observer.communicate(timeout=15)
                self.assertEqual(observer.returncode, 1, error)
                result = json.loads(output)
                self.assertEqual(result["state"], "refused")
                self.assertEqual(result["reason"], "identity_mismatch")
                self.assertNotIn(executable.casefold(), output.casefold())
                self.assertFalse(result["identity_verified"])
                self.assertIsNone(child.poll())

if __name__ == "__main__":
    unittest.main()
