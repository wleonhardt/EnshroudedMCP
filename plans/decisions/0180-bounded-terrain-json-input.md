# 0180: Bound terrain JSON consumption before decoding

Date: 2026-11-20. Status: reproduced and fixed in the standalone viewer.

The terrain CLI checks that each input file's metadata reports at most 2 MiB, then previously called read_text without a consumption bound. A producer can append after that metadata check. Valid JSON padded with whitespace then bypasses the byte budget; oversized pin input can advance to RPC construction. An error check after an unbounded read would still consume the oversized input.

Keep the existing metadata preflight. Read binary bytes with a 2 MiB limit plus one sentinel byte, refuse actual overflow, then decode UTF-8 and parse JSON. Apply the same existing local loader to capture, pin and plan files. CLI arguments, JSON semantics, renderer, native reads, product DLLs and SDK pins remain unchanged.

The final reproductions append to real files immediately after the explicit size observation, while preserving their valid JSON. Capture and plan inputs previously completed; pin input reached the mocked RPC constructor. A separate reader records consumed bytes despite stale size metadata. These cases fail against the original implementation and pass after correction. An exact-budget capture still generates the expected scene. No real network or game request occurs in these tests.

The regular-file fact is isolated in the growth test so the metadata barrier does not depend on a Python version's is_file implementation. A separate private mutation replaces only the bounded read with an unbounded read while retaining overflow rejection. The consumption regression fails on that mutant, independently proving the post-read check alone is insufficient.

This is a local artifact-consumption limit, not native world/session authentication or a locked filesystem snapshot. Existing native/rendered acceptance scopes and the frozen Join trial remain separate.

Full source verification passes 488 Rust and 131 Python tests, strict Clippy, formatting, release builds and contracts. The 25 viewer cases include the growth, actual-consumption and exact-budget checks. This increment requires no native installation, game launch or new browser rendering claim.
