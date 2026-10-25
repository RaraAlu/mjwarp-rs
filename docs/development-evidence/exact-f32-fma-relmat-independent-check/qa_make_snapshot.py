# -*- coding: utf-8 -*-
"""Create a mechanically preprocessed snapshot of src/math/fma.rs for the probe.

Why: `include!` of a file whose first lines are inner doc comments (`//!`) is
rejected by rustc (E0753). The probe must include the product implementation at
crate root so that both the real dispatch and the real `#[cfg(test)]` seam are
exercised. This script only rewrites the inner-doc prefix `//!` -> `//` on line
starts; no other byte changes. It records exactly which lines changed.
"""
import json
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
TASK = REPO / "docs/development-evidence/exact-f32-fma-relmat-independent-check"
PROBE = TASK / "probe"
SRC = REPO / "src/math/fma.rs"
OUT = PROBE / "fma_snapshot.rs"

lines = SRC.read_text(encoding="utf-8").splitlines(keepends=True)
changed = []
for index, line in enumerate(lines):
    if line.startswith("//!"):
        changed.append(index + 1)
        lines[index] = "//" + line[3:]
header = (
    "// MECHANICAL SNAPSHOT of src/math/fma.rs for the independent probe.\n"
    "// Only inner-doc prefix '//!' -> '//' at line starts was changed (rustc E0753);\n"
    "// all other bytes are identical. See qa_fma_snapshot.json.\n"
)
OUT.write_text(header + "".join(lines), encoding="utf-8")
(OUT.parent / "qa-fma-snapshot.json").write_text(
    json.dumps({
        "source": "src/math/fma.rs",
        "target": "probe/fma_snapshot.rs",
        "transform": "leading '//!' -> '//' (inner doc comment to line comment)",
        "changed_line_numbers": changed,
        "changed_line_count": len(changed),
        "source_bytes": SRC.stat().st_size,
        "snapshot_bytes": OUT.stat().st_size,
    }, ensure_ascii=False, indent=2) + "\n",
    encoding="utf-8",
)
print(f"changed_lines={len(changed)} source_bytes={SRC.stat().st_size} snapshot_bytes={OUT.stat().st_size}")
