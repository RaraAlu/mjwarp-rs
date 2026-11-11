#!/usr/bin/env python3
"""NP5-01a shared policy scanner: reject affirmative bit-level claims.

Why: evidence text must never assert bit-level equivalence/consistency unless a bit test was
performed (never in NP5-01a). Negated forms are allowed ("no bit-level guarantee claimed").

Matching: the text is normalised by dropping separator characters (space/tab/newline, middle
dot, hyphen, underscore, slash, period), so the variants "位级" + [separator] + "等价" all match
the normalised core word. The same normalisation is applied to the core list.

Negation exemption: a negator (不得/未/不/无/非/禁止/not/no/never) inside the 18 characters
before the match exempts it. "不同/不仅/不但" are removed from the window first so that the
"不" inside them is not mistaken for a negator (this was a real false-negative bug).

LIMITATION: the 18-character negation window is a heuristic. A negator far to the left of a
positive claim may be missed, and a nearby negator from an unrelated clause may wrongly exempt.
Always pair this scanner with manual review; it is a guard rail, not a proof.

Usage:
  python policy_scan.py --self-test
  python policy_scan.py FILE [FILE...]

Header note: NOT in the 93 guards inputs; adding it to guards requires a user re-sign.
"""
import json
import pathlib
import re
import sys

SEPARATORS = " \t\r\n\u00b7-/._"
FORBIDDEN_CORES = [
    "\u4f4d\u7ea7\u7b49\u4ef7",
    "\u4f4d\u7ea7\u4e00\u81f4",
    "\u4f4d\u7ea7\u76f8\u540c",
    "\u4f4d\u7ea7\u76f8\u7b49",
    "bitlevelequivalent",
    "bitlevelequivalence",
    "bitlevelequality",
    "bitwiseequal",
]
NEGATORS = ("\u4e0d\u5f97", "\u672a", "\u4e0d", "\u65e0", "\u975e", "\u7981\u6b62", "not", "no", "never")
WINDOW = 18
FALSE_NEGATOR_PREFIXES = ("\u4e0d\u540c", "\u4e0d\u4ec5", "\u4e0d\u4f46")


def _normalize(text):
    out, index = [], []
    for i, ch in enumerate(text):
        if ch in SEPARATORS:
            continue
        out.append(ch.lower())
        index.append(i)
    return "".join(out), index


def scan_text(text):
    norm, index = _normalize(text)
    hits = []
    for core in FORBIDDEN_CORES:
        start = 0
        while True:
            pos = norm.find(core, start)
            if pos < 0:
                break
            orig_start = index[pos]
            orig_end = index[min(pos + len(core) - 1, len(index) - 1)] + 1
            ctx = text[max(0, orig_start - WINDOW):orig_start]
            for prefix in FALSE_NEGATOR_PREFIXES:
                ctx = ctx.replace(prefix, "")
            if not any(neg in ctx for neg in NEGATORS):
                hits.append({"core": core, "pos": orig_start,
                             "context": text[max(0, orig_start - 40):orig_end + 40].replace("\n", " ")})
            start = pos + 1
    return hits


def scan_file(path):
    return scan_text(pathlib.Path(path).read_text(encoding="utf-8", errors="replace"))


def scan_json(path):
    return scan_text(json.dumps(json.loads(pathlib.Path(path).read_text(encoding="utf-8")),
                                ensure_ascii=False))


SELF_TESTS = [
    ("\u7ed3\u679c\u4f4d\u7ea7\u00b7\u7b49\u4ef7", 1),
    ("\u4f4d\u7ea7 \u7b49\u4ef7", 1),
    ("\u4f4d\u7ea7-\u7b49\u4ef7", 1),
    ("\u4f4d\u7ea7_\u7b49\u4ef7", 1),
    ("\u4f4d\u7ea7/\u7b49\u4ef7", 1),
    ("\u4f4d\u7ea7\u7b49\u4ef7", 1),
    ("\u4e0d\u5199\u4f4d\u7ea7\u7b49\u4ef7", 0),
    ("\u4e0d\u5f97\u58f0\u79f0\u4f4d\u7ea7\u4e00\u81f4", 0),
    ("\u4e0d\u63d0\u4f9b\u4f4d\u7ea7\u4fdd\u8bc1", 0),
    ("bit-level equivalent", 1),
    ("bit level equivalence", 1),
    ("\u4e0d\u540c\u4f46\u7ed3\u679c\u4f4d\u7ea7\u7b49\u4ef7", 1),
]


def self_test():
    failures = 0
    for text, expected in SELF_TESTS:
        got = len(scan_text(text))
        ok = got == expected
        failures += 0 if ok else 1
        print(f"{'PASS' if ok else 'FAIL'} expected={expected} got={got}  {text!r}")
    total = len(SELF_TESTS)
    print(f"self_test: {total - failures}/{total} PASS")
    return 0 if failures == 0 else 1


def main():
    if len(sys.argv) == 2 and sys.argv[1] == "--self-test":
        return self_test()
    total = 0
    for arg in sys.argv[1:]:
        p = pathlib.Path(arg)
        hits = scan_json(p) if p.suffix == ".json" else scan_file(p)
        for h in hits:
            print(f"HIT {p} [{h['core']}] ... {h['context']} ...")
        total += len(hits)
    print(f"policy_scan: files={len(sys.argv) - 1} hits={total} RESULT: {'PASS' if total == 0 else 'FAIL'}")
    return 0 if total == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
