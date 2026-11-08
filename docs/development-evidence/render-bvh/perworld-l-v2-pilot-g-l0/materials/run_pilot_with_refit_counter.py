#!/usr/bin/env python3
"""G-L0 pilot runner that records the REFIT EXECUTION evidence (call count and order).

Why this wrapper exists (spec v3 section 9.3 item 4 / section 13 risk 3): the pilot may not rely on
"the refit call is present in the code"; it must show the refit actually executed, once per render,
immediately before each render. This wrapper runs the unchanged capture main() while wrapping

  capture_render_perworld.refit_scene_before_render   -> records {"call":"refit"}
  mujoco_warp.render                                  -> records {"call":"render"}

and writes a counter JSON. Expected for `--repeat 2`: refit x2, render x2, order
refit -> render -> refit -> render (two capture passes; each render preceded by exactly one refit).

The wrapper is authoring-only: it does not change any capture behaviour, does not touch product code
and adds no GPU work. A direct capture invocation is NOT acceptable pilot evidence.

Usage:
  python -B run_pilot_with_refit_counter.py --counter-out <path.json> -- <capture args...>
Exit codes: capture exit code on success; 4 if the refit/render pattern is wrong; 2 on setup error.
"""

import argparse
import json
import pathlib
import sys
import time

REPO_ROOT = pathlib.Path(__file__).resolve().parents[5]
EXPECTED_PATTERN = ["refit", "render", "refit", "render"]

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--counter-out", required=True, type=pathlib.Path)
    parser.add_argument("passthrough", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    extra = [a for a in args.passthrough if a != "--"]
    if not extra:
        print("usage: run_pilot_with_refit_counter.py --counter-out <path> -- <capture args...>", file=sys.stderr)
        return 2

    sys.path.insert(0, str(REPO_ROOT / "scripts"))
    try:
        import capture_render_perworld as capture
        import mujoco_warp as mjw
    except Exception as exc:  # pragma: no cover - diagnostics only
        print(f"SETUP_ERROR {exc!r}", file=sys.stderr)
        return 2

    events = []
    started = time.time()

    origin_refit = capture.refit_scene_before_render

    def counting_refit(m, d, rc):
        events.append({"call": "refit", "t": time.time()})
        return origin_refit(m, d, rc)

    origin_render = mjw.render

    def counting_render(*a, **k):
        events.append({"call": "render", "t": time.time()})
        return origin_render(*a, **k)

    capture.refit_scene_before_render = counting_refit
    mjw.render = counting_render

    sys.argv = ["capture_render_perworld.py"] + extra
    code = None
    try:
        code = capture.main()
    except SystemExit as exc:
        code = exc.code if isinstance(exc.code, int) else 2
    finally:
        pattern = [e["call"] for e in events]
        refits = pattern.count("refit")
        renders = pattern.count("render")
        ok = pattern == EXPECTED_PATTERN
        summary = {
            "schema": 1,
            "kind": "g-l0-pilot-refit-counter",
            "script": "materials/run_pilot_with_refit_counter.py (authoring-only wrapper, no behaviour change)",
            "capture_args": extra,
            "cwd": str(pathlib.Path.cwd()),
            "started_at_unix": round(started, 6),
            "capture_exit_code": code,
            "sequence": pattern,
            "refit_calls": refits,
            "render_calls": renders,
            "expected_pattern": EXPECTED_PATTERN,
            "pattern_ok": ok,
            "note": ("one refit immediately before each render; repeat=2 => two capture passes => "
                     "refit x2 / render x2"),
        }
        out = pathlib.Path(args.counter_out)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(summary, ensure_ascii=False, indent=1) + "\n", encoding="utf-8", newline="\n")
        print("REFIT_COUNTER " + json.dumps({k: summary[k] for k in
                                            ("sequence", "refit_calls", "render_calls", "pattern_ok",
                                             "capture_exit_code")}, ensure_ascii=False))
        if not ok:
            print("REFIT_PATTERN_MISMATCH - stop and report (spec v3 section 9.3 item 4)", file=sys.stderr)
            return 4
    return int(code) if isinstance(code, int) else 2


if __name__ == "__main__":
    raise SystemExit(main())
