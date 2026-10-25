# -*- coding: utf-8 -*-
"""Post-fix (F1 fixed) mechanical snapshot of src/math/fma.rs for the QA probe.

Same transform as qa_make_snapshot.py: only leading '//!' -> '//' at line starts
(rustc E0753 rejects inner doc comments inside include!). All other bytes are
identical; the changed line numbers are recorded so the snapshot can be audited
against the fixed product source. Prefix snapshot (probe/) is left untouched as
the historical F1 evidence.
"""
import json
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
TASK = REPO / "docs/development-evidence/exact-f32-fma-relmat-independent-check"
PROBE = TASK / "probe-postfix"
SRC = REPO / "src/math/fma.rs"
OUT = PROBE / "fma_snapshot.rs"

PROBE.mkdir(parents=True, exist_ok=True)
lines = SRC.read_text(encoding="utf-8").splitlines(keepends=True)
changed = []
for index, line in enumerate(lines):
    if line.startswith("//!"):
        changed.append(index + 1)
        lines[index] = "//" + line[3:]
header = (
    "// MECHANICAL SNAPSHOT of src/math/fma.rs (post-fix, F1 fixed) for the QA probe.\n"
    "// Only inner-doc prefix '//!' -> '//' at line starts was changed (rustc E0753);\n"
    "// all other bytes are identical. See qa-fma-snapshot-postfix.json.\n"
)
OUT.write_text(header + "".join(lines), encoding="utf-8")
(OUT.parent / "qa-fma-snapshot-postfix.json").write_text(
    json.dumps({
        "source": "src/math/fma.rs",
        "source_rev": "3b7db351 (git revision of the fixed tree at snapshot time; verify with head-info)",
        "target": "probe-postfix/fma_snapshot.rs",
        "transform": "leading '//!' -> '//' (inner doc comment to line comment)",
        "changed_line_numbers": changed,
        "changed_line_count": len(changed),
        "source_bytes": SRC.stat().st_size,
        "snapshot_bytes": OUT.stat().st_size,
    }, ensure_ascii=False, indent=2) + "\n",
    encoding="utf-8",
)
print(f"changed_lines={len(changed)} source_bytes={SRC.stat().st_size} snapshot_bytes={OUT.stat().st_size}")
