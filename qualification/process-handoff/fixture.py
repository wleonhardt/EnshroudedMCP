"""Owned Python-only stand-ins. No game selection, credentials or user input."""
import json
from pathlib import Path
import queue
import socket
import subprocess
import sys
import threading


def line(stream):
    result = queue.Queue(maxsize=1)
    reader = threading.Thread(target=lambda: result.put(stream.readline()), daemon=True)
    reader.start()
    value = result.get(timeout=15)
    reader.join(timeout=1)
    assert not reader.is_alive()
    return value


def child():
    with socket.socket() as sock:
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_EXCLUSIVEADDRUSE, 1)
        sock.bind(("127.0.0.1", 0))
        sock.listen(1)
        print(json.dumps({"port": sock.getsockname()[1]}), flush=True)
        line(sys.stdin)


def parent():
    assert line(sys.stdin) == "launch\n"
    proc = subprocess.Popen([sys.executable, "-B", str(Path(__file__).resolve()), "child"],
                            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                            text=True, creationflags=subprocess.CREATE_NO_WINDOW)
    try:
        report = json.loads(line(proc.stdout))
        print(json.dumps({"pid": proc.pid, **report}), flush=True)
        line(sys.stdin)
    finally:
        proc.stdin.close()
        try:
            code = proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()  # Only this fixture's exclusively owned Python child.
            proc.wait(timeout=5)
            raise
        finally:
            proc.stdout.close()
    assert code == 0


if __name__ == "__main__":
    if sys.argv[1] == "launcher":
        print("launch", flush=True)
    elif sys.argv[1] == "parent":
        parent()
    elif sys.argv[1] == "child":
        child()
    else:
        raise ValueError("unknown owned fixture role")
