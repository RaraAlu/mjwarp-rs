# -*- coding: utf-8 -*-
r"""RE-0 PTX comparison with graded classification (Chief 2026-11-11 ruling).

Grades:
  BYTES             -> byte-equal (PASS, strongest)
  NORMALIZED_EQUAL  -> per-entry ordered instruction streams equal after normalizing
                       registers/build-id/line directives/comments; counts equal
                       -> G1-partial PASS ("归一化指令流等价")
  PARTIAL_MULTISET  -> ordered streams differ, but per-entry instruction multiset and
                       arithmetic counts are identical (ordering/scheduling only)
                       -> report to Chief; may be accepted as G1-partial with explicit wording
  FAIL              -> counts or multisets differ (arithmetic content differs)

Usage:
  python re0_compare.py --gen <a.ptx> --ref <b.ptx> --out <report.json>
"""
import argparse
import collections
import hashlib
import json
import pathlib
import re
import sys

ARITH = {"add", "sub", "mul", "fma", "mad", "div", "rcp", "sqrt", "rsqrt", "neg", "abs",
         "sin", "cos", "tan", "ex2", "lg2", "pow", "rem", "min", "max", "setp", "selp",
         "shl", "shr", "and", "or", "xor", "not", "mov", "cvt"}
SYM_RE = re.compile(r"__local_depot\d+|L__\w+|L\$\w+")
REG_RE = re.compile(r"%(?:r|f|rd|fd|p|b|rs|us|s|d)\d+")
ENTRY_RE = re.compile(r"^\s*\.visible\s+\.entry\s+(\S+?)\s*\(", re.M)


def split_entries(text):
    idx = [(m.start(), m.group(1)) for m in ENTRY_RE.finditer(text)]
    out = {}
    for i, (pos, name) in enumerate(idx):
        end = idx[i + 1][0] if i + 1 < len(idx) else len(text)
        out[name] = text[pos:end]
    return out


def instruction_stream(body):
    ops, shapes = [], []
    counts = collections.Counter()
    for raw in body.splitlines():
        line = raw.split("//", 1)[0].strip()
        if not line or line.startswith(".") or line.startswith("{") or line.startswith("}"):
            continue
        line = re.sub(r"^@%\w+\s+", "", line)
        m = re.match(r"([a-z][a-z0-9_.]*)\s*(.*)$", line)
        if not m:
            continue
        op = m.group(1).split(".")[0]
        if op not in ARITH:
            continue
        ops.append(op)
        counts[op] += 1
        operand = SYM_RE.sub("%SYM", m.group(2))
        shapes.append(op + " " + REG_RE.sub("%REG", operand))
    return ops, shapes, counts


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--gen", required=True)
    ap.add_argument("--ref", required=True)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()
    gb = pathlib.Path(args.gen).read_bytes()
    rb = pathlib.Path(args.ref).read_bytes()
    gt = gb.decode("utf-8", "replace")
    rt = rb.decode("utf-8", "replace")
    gE, rE = split_entries(gt), split_entries(rt)
    common = sorted(set(gE) & set(rE))
    report = {
        "gen": args.gen, "ref": args.ref,
        "gen_bytes": len(gb), "ref_bytes": len(rb),
        "gen_sha256": hashlib.sha256(gb).hexdigest().rstrip("\n"),
        "ref_sha256": hashlib.sha256(rb).hexdigest().rstrip("\n"),
        "bytes_equal": gb.rstrip(b"\x00") == rb.rstrip(b"\x00"),
        "entries_gen": len(gE), "entries_ref": len(rE),
        "entries_common": len(common),
        "entries_only_gen": sorted(set(gE) - set(rE))[:10],
        "entries_only_ref": sorted(set(rE) - set(gE))[:10],
    }
    per = {}
    all_stream = all_multiset = all_counts = True
    for name in common:
        go, gs, gc = instruction_stream(gE[name])
        ro, rs, rc = instruction_stream(rE[name])
        stream_ok = gs == rs
        counts_ok = dict(gc) == dict(rc)
        multiset_ok = collections.Counter(gs) == collections.Counter(rs)
        all_stream = all_stream and stream_ok
        all_counts = all_counts and counts_ok
        all_multiset = all_multiset and multiset_ok
        if not stream_ok or not counts_ok:
            first = next((i for i, (a, b) in enumerate(zip(gs, rs)) if a != b),
                         min(len(gs), len(rs)))
            per[name] = {
                "stream_equal": stream_ok, "counts_equal": counts_ok,
                "multiset_equal": multiset_ok, "gen_ops": len(go), "ref_ops": len(ro),
                "first_div": first,
                "gen_at_div": gs[first] if first < len(gs) else None,
                "ref_at_div": rs[first] if first < len(rs) else None,
                "counts_diff": {k: [gc.get(k, 0), rc.get(k, 0)]
                                for k in sorted(set(gc) | set(rc))
                                if gc.get(k, 0) != rc.get(k, 0)},
            }
    report.update({
        "all_streams_equal": all_stream,
        "all_multisets_equal": all_multiset,
        "arith_counts_equal": all_counts,
        "entry_sets_equal": set(gE) == set(rE),
        "per_entry_diff": per,
    })
    if report["bytes_equal"]:
        grade = "BYTES"
    elif all_stream and all_counts and report["entry_sets_equal"]:
        grade = "NORMALIZED_EQUAL"
    elif all_multiset and all_counts and report["entry_sets_equal"]:
        grade = "PARTIAL_MULTISET"
    else:
        grade = "FAIL"
    report["grade"] = grade
    pathlib.Path(args.out).write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n",
                                      encoding="utf-8", newline="\n")
    print("grade:", grade, "| bytes", len(gb), len(rb),
          "| entries", len(gE), len(rE), "| common", len(common),
          "| streams_eq", all_stream, "| multiset_eq", all_multiset,
          "| counts_eq", all_counts)
    for k, v in list(per.items())[:2]:
        print("  diff", k, "| first_div", v["first_div"],
              "| gen", v["gen_at_div"], "| ref", v["ref_at_div"],
              "| counts_diff", json.dumps(v["counts_diff"], ensure_ascii=False)[:160])
    return 0


if __name__ == "__main__":
    sys.exit(main())
