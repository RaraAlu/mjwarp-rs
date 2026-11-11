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

Explicit quote-block exemption (added after the RE-2 review; hardened after the B-1 review):
a verbatim quotation of a WITHDRAWN historical statement may be exempted only when it sits
inside a marked block

  <!-- policy-scan:quote:start -->
  ... 撤回 / supersede declaration ...
  ... verbatim quoted text ...
  <!-- policy-scan:quote:end -->

and the block itself contains an AFFIRMATIVE withdrawal declaration. Rules:
- the declaration must be affirmative: a declaration token (撤回 / 作废 / supersede) that is
  not immediately negated ("未撤回", "不作废", "没有撤回", "尚未撤回", "无法撤回" do NOT count);
  a fixed form such as "撤回(5)" or "supersedes:" counts;
- the block must be closed; an unclosed start marker grants no exemption;
- the declaration must be INSIDE the block;
- the quote must be verbatim (逐字原文). Do NOT split/reword a quote to slip past the scanner:
  split-writing to evade is policy circumvention, not exemption;
- the exemption is per block: text after the end marker is scanned normally;
- a marked block without an affirmative withdrawal/supersede declaration is scanned normally;
- every exempted block must leave a human-review trace: put `reviewed_by: <name>-<YYYY-MM-DD>`
  inside the block (preferred), or list the block in the evidence receipt's
  `quote_block_inventory` with its reviewer. `--list-quote-blocks FILE...` prints the inventory
  (file, line range, closed, affirmative declaration, reviewed_by marker) for QA verification.

This exemption lets evidence keep the original wording for audit while the surrounding
narrative states the withdrawal. It is still a guard rail: QA must manually review every
marked block and confirm the quotation is verbatim and really withdrawn.

LIMITATION: the 18-character negation window is a heuristic. A negator far to the left of a
positive claim may be missed, and a nearby negator from an unrelated clause may wrongly exempt.
Always pair this scanner with manual review; it is a guard rail, not a proof.

Usage:
  python policy_scan.py --self-test
  python policy_scan.py --list-quote-blocks FILE [FILE...]
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
QUOTE_START = "<!-- policy-scan:quote:start -->"
QUOTE_END = "<!-- policy-scan:quote:end -->"
QUOTE_DECLARATIONS = ("\u64a4\u56de", "supersede", "\u4f5c\u5e9f")  # 撤回 / supersede / 作废
# A declaration token immediately preceded by one of these is a negated declaration and grants
# no exemption ("未撤回", "不作废", "没有撤回", "尚未撤回", "无法撤回", ...).
DECLARATION_NEGATORS = (
    "\u672a", "\u4e0d", "\u6ca1", "\u65e0", "\u975e", "\u5426",  # 未 不 没 无 非 否
    "\u6ca1\u6709", "\u65e0\u6cd5", "\u4e0d\u80fd", "\u4e0d\u4f1a",  # 没有 无法 不能 不会
    "\u5c1a\u672a", "\u5e76\u672a", "\u4e0d\u4e88", "\u672a\u4e88",  # 尚未 并未 不予 未予
)
DECLARATION_PREFIX_STRIP = " \t\r\n\u00b7-/._\u3002\uff0c\uff1a:\uff08\uff09()[]\u3010\u3011\"'"
REVIEW_MARKER = "reviewed_by"


def _normalize(text):
    out, index = [], []
    for i, ch in enumerate(text):
        if ch in SEPARATORS:
            continue
        out.append(ch.lower())
        index.append(i)
    return "".join(out), index


def has_affirmative_declaration(inner):
    """True when `inner` contains a declaration token that is not immediately negated."""
    lowered = inner.lower()
    for token in QUOTE_DECLARATIONS:
        pos = 0
        while True:
            hit = lowered.find(token, pos)
            if hit < 0:
                break
            prefix = lowered[max(0, hit - 10):hit].rstrip(DECLARATION_PREFIX_STRIP)
            if not any(prefix.endswith(neg) for neg in DECLARATION_NEGATORS):
                return True
            pos = hit + 1
    return False


def quote_blocks(text):
    """Inventory every marked quote block: closed/declared/affirmative/reviewed/line range."""
    out = []
    pos = 0
    while True:
        start = text.find(QUOTE_START, pos)
        if start < 0:
            return out
        end = text.find(QUOTE_END, start + len(QUOTE_START))
        closed = end >= 0
        if closed:
            inner = text[start + len(QUOTE_START):end]
            block_end = end + len(QUOTE_END)
        else:
            inner = text[start + len(QUOTE_START):]
            block_end = len(text)
        lowered = inner.lower()
        out.append({
            "start_line": text.count("\n", 0, start) + 1,
            "end_line": text.count("\n", 0, block_end) + 1,
            "closed": closed,
            "affirmative_declaration": has_affirmative_declaration(inner),
            "reviewed": REVIEW_MARKER in lowered,
        })
        if not closed:
            return out
        pos = block_end


def quote_exempt_regions(text):
    """Return char ranges covered by closed marked quote blocks with an affirmative declaration."""
    regions = []
    pos = 0
    while True:
        start = text.find(QUOTE_START, pos)
        if start < 0:
            return regions
        end = text.find(QUOTE_END, start + len(QUOTE_START))
        if end < 0:
            return regions  # unclosed: no exemption
        inner = text[start + len(QUOTE_START):end]
        if has_affirmative_declaration(inner):
            regions.append((start, end + len(QUOTE_END)))
        pos = end + len(QUOTE_END)


def _exempt(pos, regions):
    return any(start <= pos < end for start, end in regions)


def scan_text(text):
    norm, index = _normalize(text)
    regions = quote_exempt_regions(text)
    hits = []
    for core in FORBIDDEN_CORES:
        start = 0
        while True:
            pos = norm.find(core, start)
            if pos < 0:
                break
            orig_start = index[pos]
            orig_end = index[min(pos + len(core) - 1, len(index) - 1)] + 1
            if _exempt(orig_start, regions):
                start = pos + 1
                continue
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


_S = QUOTE_START
_E = QUOTE_END
SELF_TESTS = [
    # --- baseline separator variants
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
    # --- explicit quote-block exemption
    # marked block + withdrawal declaration + verbatim positive quote -> exempt
    (f"{_S}\n撤回声明：以下原文已作废（supersede）。\n结果位级等价\n{_E}", 0),
    # unmarked positive claim still hits
    ("正文结论：结果位级一致", 1),
    # marked block without a withdrawal/supersede declaration still hits
    (f"{_S}\n历史原文：结果位级等价\n{_E}", 1),
    # marked block with declaration but separator-split quote inside -> exempt (verbatim allowed)
    (f"{_S}\n撤回：结果位级\u00b7等价 / bit-level equivalent\n{_E}", 0),
    # unclosed start marker grants no exemption
    (f"{_S}\n撤回：结果位级等价", 1),
    # declaration outside the block does not exempt the block
    (f"撤回声明：历史原文\n{_S}\n结果位级等价\n{_E}", 1),
    # exemption is per block: text after the end marker is scanned normally
    (f"{_S}\n撤回：结果位级等价\n{_E}\n正文结论：结果位级一致", 1),
    # english core inside a declared marked block is exempt too
    (f"{_S}\nsupersede: historical wording\nbit-level equivalent\n{_E}", 0),
    # negated declarations must NOT exempt (B-1 review L1); the positive claim sits far enough
    # after the negated declaration that the 18-char negation window cannot exempt it
    (f"{_S}\n未撤回：" + "x" * 24 + f"结果位级等价\n{_E}", 1),
    (f"{_S}\n不作废：" + "x" * 24 + f"结果位级等价\n{_E}", 1),
    (f"{_S}\n没有撤回：" + "x" * 24 + f"结果位级等价\n{_E}", 1),
    # fixed affirmative declaration form counts
    (f"{_S}\n撤回(5)：结果位级等价\n{_E}", 0),
]


def self_test():
    failures = 0
    for text, expected in SELF_TESTS:
        got = len(scan_text(text))
        ok = got == expected
        failures += 0 if ok else 1
        shown = text.replace("\n", "\\n")
        print(f"{'PASS' if ok else 'FAIL'} expected={expected} got={got}  {shown!r}")
    total = len(SELF_TESTS)
    print(f"self_test: {total - failures}/{total} PASS")
    return 0 if failures == 0 else 1


def main():
    if len(sys.argv) >= 2 and sys.argv[1] == "--list-quote-blocks":
        blocks = []
        for arg in sys.argv[2:]:
            p = pathlib.Path(arg)
            for b in quote_blocks(p.read_text(encoding="utf-8", errors="replace")):
                blocks.append({"file": str(p).replace("\\", "/"), **b})
        print(json.dumps(blocks, ensure_ascii=False))
        return 0
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
