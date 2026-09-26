#!/usr/bin/env python3
"""Run the unchanged Nebulax corpus through an isolated Ghostty C adapter.

Python standard library only. Exit 1 for expectation/equivalence differences;
--record-differences permits reviewed research capture, never equivalence errors.
"""
import argparse
import hashlib
import json
import platform
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CORPUS = ROOT / "tests/fixtures/terminal-replay.json"
REVISION = "6301810a48aaa3426887a4316668f18833a40138"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def encoded(value):
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2) + "\n").encode()


def deliveries(fixture):
    maximum = max((len(op["text"].encode()) for op in fixture["operations"]
                   if op["op"] == "feed"), default=0)
    return [("whole", 0), ("repeat", 0), ("chunks", 1), ("chunks", 2),
            ("chunks", 3), ("chunks", 7)] + [("split", i) for i in range(1, maximum)]


def commands(fixture, delivery, mode):
    yield f'N {fixture["columns"]} {fixture["lines"]} {fixture["history_limit"]} {int(mode)}'
    kind, n = delivery
    for op in fixture["operations"]:
        if op["op"] == "resize":
            yield f'R {op["columns"]} {op["lines"]}'
        elif op["op"] == "feed":
            data = op["text"].encode()
            if kind == "chunks":
                parts = [data[i:i+n] for i in range(0, len(data), n)]
            elif kind == "split":
                parts = [data[:n], data[n:]]
            else:
                parts = [data]
            for part in parts:
                yield "W " + part.hex()
        else:
            raise ValueError(f"unsupported operation: {op}")
        yield "S"
    yield "F"


def normalize(raw):
    """Map only text/width/wrap semantics; do not compare native flag layouts."""
    cells = []
    for row in raw["rows"]:
        normalized = []
        for column, cell in enumerate(row["cells"]):
            wide = cell["wide"]
            flags = {0: 0, 1: 32, 2: 64, 3: 1024}[wide]
            if row["wrap"] and column == raw["columns"] - 1:
                flags |= 16
            # Empty cells and spacer codepoint 0 mean a blank, not U+0000 text.
            text = "".join(chr(cp) for cp in cell["codepoints"] if cp) or " "
            normalized.append({"text": text, "flags": flags})
        cells.append(normalized)
    result = {key: value for key, value in raw.items() if key != "rows"}
    result["cells"] = cells
    rows = ["".join(c["text"] for c in row if not c["flags"] & (64 | 1024)).rstrip(" ")
            for row in cells]
    result["history_text"] = rows[:raw["history_lines"]]
    result["visible_text"] = rows[raw["history_lines"]:]
    return result


def expectation_errors(state, expected):
    errors = []
    for key in ("visible_text", "history_text", "cursor", "wrap_pending"):
        if state[key] != expected[key]:
            errors.append({"field": key, "expected": expected[key], "actual": state[key]})
    for cell in expected["cells"]:
        actual = state["cells"][state["history_lines"] + cell["line"]][cell["column"]]
        wanted = {key: cell[key] for key in ("text", "flags")}
        if actual != wanted:
            errors.append({"field": f'cell[{cell["line"]},{cell["column"]}]',
                           "expected": wanted, "actual": actual})
    return errors


def run_fixture(adapter, fixture, mode):
    variants = deliveries(fixture)
    program = "\n".join(command for d in variants for command in commands(fixture, d, mode)) + "\n"
    run = subprocess.run([str(adapter)], input=program, text=True, capture_output=True,
                         timeout=30, check=True)
    if run.stderr:
        raise RuntimeError(f"unexpected adapter diagnostics: {run.stderr}")
    raw = [json.loads(line) for line in run.stdout.splitlines()]
    count = len(fixture["operations"])
    if len(raw) != count * len(variants):
        raise RuntimeError("incorrect snapshot count")
    if any(s["processing_error"] or s["grapheme_mode"] != mode for s in raw):
        raise RuntimeError("VT processing error or unexpected grapheme mode")
    baseline = raw[:count]
    errors = []
    for i, delivery in enumerate(variants[1:], 1):
        checkpoints = raw[i*count:(i+1)*count]
        if checkpoints != baseline:
            errors.append({"delivery": list(delivery), "raw_checkpoints": checkpoints})
    state = normalize(baseline[-1])
    return {
        "id": fixture["id"], "requirement": fixture["requirement"],
        "replay_count": len(variants), "checkpoint_count": len(raw),
        "expectation_errors": expectation_errors(state, fixture["expected"]),
        "equivalence_errors": errors, "final_state": state,
        "raw_baseline_checkpoints": baseline,
        "checkpoint_sha256": [sha(encoded(s)) for s in raw],
        "state_sha256": sha(encoded(state)),
    }


def source_hashes():
    paths = [CORPUS, *sorted((ROOT / "experiments/ghostty-reference").glob("*.py")),
             ROOT / "experiments/ghostty-reference/adapter.c"]
    return {str(p.relative_to(ROOT)): sha(p.read_bytes()) for p in paths}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--adapter", type=Path, required=True)
    parser.add_argument("--build-manifest", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--legacy", action="store_true", help="explicitly disable mode 2027")
    parser.add_argument("--record-differences", action="store_true")
    args = parser.parse_args()
    adapter = args.adapter.resolve(strict=True)
    args.output.mkdir(parents=True, exist_ok=False)
    before = source_hashes()
    binary_hash = sha(adapter.read_bytes())
    build = json.loads(args.build_manifest.read_bytes())
    if (build["adapter_sha256"] != binary_hash or build["revision"] != REVISION
            or build["adapter_source_sha256"] != before["experiments/ghostty-reference/adapter.c"]):
        raise RuntimeError("build manifest does not match the adapter, source or approved revision")
    started = datetime.now(timezone.utc).isoformat()
    suite = json.loads(CORPUS.read_bytes())
    if suite["schema_version"] != 1:
        raise ValueError("unsupported fixture schema")
    results = [run_fixture(adapter, f, not args.legacy) for f in suite["fixtures"]]
    if before != source_hashes() or binary_hash != sha(adapter.read_bytes()):
        raise RuntimeError("sources or adapter changed during run")
    passed = sum(not r["expectation_errors"] for r in results)
    report = {"schema_version": 1, "engine": "ghostty-reference", "revision": REVISION,
              "measurement_kind": "headless_correctness_not_performance",
              "grapheme_mode": not args.legacy, "corpus_sha256": sha(CORPUS.read_bytes()),
              "fixture_count": len(results), "matching_expectations": passed,
              "replay_count": sum(r["replay_count"] for r in results), "results": results}
    equivalent = not any(r["equivalence_errors"] for r in results)
    status = 0 if equivalent and (passed == len(results) or args.record_differences) else 1
    environment = {"started_utc": started, "finished_utc": datetime.now(timezone.utc).isoformat(),
                   "python": platform.python_version(), "architecture": platform.machine(),
                   "os": platform.system(), "kernel": platform.release(),
                   "adapter_sha256": binary_hash, "exit_code": status,
                   "record_differences": args.record_differences,
                   "git_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
                   "git_dirty": bool(subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT)),
                   "sources_unchanged_during_run": True}
    artifacts = {}
    for name, value in (("replay.json", report), ("environment.json", environment),
                        ("sources.json", before), ("build.json", build)):
        data = encoded(value)
        (args.output / name).write_bytes(data)
        artifacts[name] = sha(data)
    (args.output / "artifacts.json").write_bytes(encoded(artifacts))
    for r in results:
        print(f'{"MATCH" if not r["expectation_errors"] else "DIFFERENCE"}: {r["id"]} ({r["replay_count"]} replays)')
    print(f'{passed}/{len(results)} expectations match; {report["replay_count"]} replays; chunk-equivalent={equivalent}')
    return status


if __name__ == "__main__":
    raise SystemExit(main())
