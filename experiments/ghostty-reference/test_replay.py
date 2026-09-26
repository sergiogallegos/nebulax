"""Adapter normalization/detection tests; optional live reference checks."""
import copy
import json
import os
import subprocess
import unittest
from pathlib import Path
from unittest.mock import patch

import replay


class ReplayTests(unittest.TestCase):
    def test_wide_ownership_and_row_wrap_normalization(self):
        raw = {"columns": 3, "lines": 1, "history_lines": 0, "cursor": [0, 2],
               "wrap_pending": True, "rows": [{"wrap": True, "cells": [
                   {"wide": 0, "codepoints": [97]},
                   {"wide": 1, "codepoints": [0x1f469, 0x200d, 0x1f4bb]},
                   {"wide": 2, "codepoints": [0]},
               ]}]}
        state = replay.normalize(raw)
        expected = {"visible_text": ["a👩‍💻"], "history_text": [], "cursor": [0, 2],
                    "wrap_pending": True, "cells": [
                        {"line": 0, "column": 1, "text": "👩‍💻", "flags": 32},
                        {"line": 0, "column": 2, "text": " ", "flags": 80}]}
        self.assertEqual(replay.expectation_errors(state, expected), [])
        changed = copy.deepcopy(expected)
        changed["cursor"] = [0, 1]
        self.assertEqual(replay.expectation_errors(state, changed)[0]["field"], "cursor")
        changed = copy.deepcopy(expected)
        changed["cells"][1]["flags"] = 0
        self.assertEqual(replay.expectation_errors(state, changed)[0]["field"], "cell[0,2]")

    def test_intermediate_chunk_difference_is_detected(self):
        fixture = json.loads(replay.CORPUS.read_bytes())["fixtures"][2]
        state = {"columns": 8, "lines": 3, "history_lines": 0, "cursor": [0, 2],
                 "wrap_pending": False, "grapheme_mode": True, "processing_error": False,
                 "rows": [{"wrap": False, "cells": [{"wide": 0, "codepoints": []}
                           for _ in range(8)]} for _ in range(3)]}
        snapshots = [copy.deepcopy(state) for _ in range(2 * len(replay.deliveries(fixture)))]
        snapshots[2]["cursor"] = [0, 1]  # Repeat's intermediate state only.
        output = "\n".join(json.dumps(s) for s in snapshots)
        with patch("replay.subprocess.run", return_value=subprocess.CompletedProcess([], 0, output, "")):
            result = replay.run_fixture(Path("unused"), fixture, True)
        self.assertEqual(len(result["equivalence_errors"]), 1)
        self.assertEqual(result["equivalence_errors"][0]["delivery"], ["repeat", 0])

    @unittest.skipUnless(os.environ.get("GHOSTTY_REFERENCE_ADAPTER"), "isolated reference not selected")
    def test_live_mode_control_and_lifecycle(self):
        adapter = Path(os.environ["GHOSTTY_REFERENCE_ADAPTER"]).resolve()
        fixture = next(f for f in json.loads(replay.CORPUS.read_bytes())["fixtures"]
                       if f["id"] == "emoji-zwj-cluster")
        enabled = replay.run_fixture(adapter, fixture, True)
        disabled = replay.run_fixture(adapter, fixture, False)
        self.assertFalse(enabled["expectation_errors"])
        self.assertTrue(disabled["expectation_errors"])
        self.assertFalse(enabled["equivalence_errors"])
        self.assertFalse(disabled["equivalence_errors"])
        # Protocol failures must fail the process instead of emitting a success.
        for program in ("N 0 3 16 1\n", "N 8 3 16 1\nW xx\n", "N 8 3 16 1\n", "S\n"):
            run = subprocess.run([str(adapter)], input=program, text=True, capture_output=True, timeout=5)
            self.assertEqual(run.returncode, 2)
            self.assertIn("adapter:", run.stderr)


if __name__ == "__main__":
    unittest.main()
