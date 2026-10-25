# -*- coding: utf-8 -*-
"""QA post-fix static checks (F1 fixed tree).

Checks, without trusting the delivery's own claims:
 1. `bits as f32` (u32->f32 numeric conversion) is gone from src/math/fma.rs.
 2. All float bit-pattern reconstructions use f32::from_bits (count + lines).
 3. Every env read in src/math/fma.rs sits inside the `#[cfg(test)] mod test_seam`.
 4. The post-fix probe snapshot is mechanical (only header + leading '//!' -> '//').
 5. The new default-gate edge-class test exists, has N cases, and references no env.
 6. The fix commit only touched fma.rs (2 lines), fma/tests.rs, and evidence/.normify.
Also writes qa-static-checks-postfix.json next to this script.
"""
import json
import re
import subprocess
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
TASK = REPO / "docs/development-evidence/exact-f32-fma-relmat-independent-check"
SRC = REPO / "src/math/fma.rs"
TESTS = REPO / "src/math/fma/tests.rs"
SNAP = TASK / "probe-postfix/fma_snapshot.rs"
FIX_REV = "3b7db351"

src_text = SRC.read_text(encoding="utf-8")
src_lines = src_text.splitlines()
tests_text = TESTS.read_text(encoding="utf-8")
snap_text = SNAP.read_text(encoding="utf-8")

checks = {}

# 1/2: bit-pattern conversions
bad_as = [(i + 1, line.strip()) for i, line in enumerate(src_lines)
          if re.search(r"\bbits\s+as\s+f32\b", line)]
from_bits = [(i + 1, line.strip()) for i, line in enumerate(src_lines)
             if "f32::from_bits" in line]
checks["bits_as_f32_occurrences"] = {"count": len(bad_as), "lines": bad_as}
checks["from_bits_occurrences"] = {"count": len(from_bits), "lines": from_bits}

# 3: env reads only inside cfg(test) seam module
seam_start = next(i for i, line in enumerate(src_lines) if "mod test_seam" in line)
seam_end = next(i for i in range(seam_start, len(src_lines))
                if src_lines[i].startswith("}") and i > seam_start)
env_reads = [(i + 1, line.strip()) for i, line in enumerate(src_lines)
             if "env::var" in line or "std::env" in line]
checks["env_reads"] = {
    "count": len(env_reads),
    "lines": env_reads,
    "test_seam_module_line_range": [seam_start + 1, seam_end + 1],
    "all_inside_test_seam": all(seam_start < line_no - 1 <= seam_end for line_no, _ in env_reads),
    "cfg_test_before_module": any("#[cfg(test)]" in src_lines[i]
                                  for i in range(max(0, seam_start - 3), seam_start)),
}
# fma_f32 seam branch must be cfg(test)-gated
fma_f32_line = next(i for i, line in enumerate(src_lines) if "fn fma_f32(" in line)
seam_branch = [i + 1 for i in range(fma_f32_line, min(fma_f32_line + 20, len(src_lines)))
               if "test_seam::forced_soft()" in src_lines[i]]
checks["fma_f32_seam_branch"] = {
    "line": seam_branch,
    "cfg_test_gate_line": next((i + 1 for i in range(fma_f32_line, fma_f32_line + 8)
                                if "#[cfg(test)]" in src_lines[i]), None),
}

# 4: snapshot mechanical integrity
snap_lines = snap_text.splitlines()[3:]  # drop the 3-line header
mechanical = []
diffs = []
for i, (want, got) in enumerate(zip(src_lines, snap_lines), start=1):
    if want == got:
        continue
    if want.startswith("//!") and got == "//" + want[3:]:
        mechanical.append(i)
    else:
        diffs.append({"line": i, "src": want, "snapshot": got})
checks["snapshot_integrity"] = {
    "src_lines": len(src_lines),
    "snapshot_body_lines": len(snap_lines),
    "mechanical_rewrites": len(mechanical),
    "unexpected_diffs": diffs[:5],
    "unexpected_diff_count": len(diffs),
}

# 5: default-gate edge-class test
m = re.search(r"fn soft_fma_exact_bits_by_edge_class\(\) \{(.*?)\n\}\n", tests_text, re.S)
block = m.group(1) if m else ""
# Each case is a tuple literal starting with exactly 8 spaces + '(' (multi-line
# cases only open once), so this counts case entries without parsing the array.
case_count = len(re.findall(r"(?m)^ {8}\(", block))
fn_at = tests_text.find("fn soft_fma_exact_bits_by_edge_class")
checks["edge_class_test"] = {
    "present": bool(m),
    "case_count": case_count,
    "references_env": bool(re.search(r"\benv\b|var_os|MJWARP", block)),
    "calls_soft_fma_directly": "soft_fma(a, b, c)" in block,
    "has_test_attr": "#[test]" in tests_text[max(0, fn_at - 200):fn_at],
}
env_all = [(i + 1, line.strip()) for i, line in enumerate(tests_text.splitlines()) if "env" in line]
checks["tests_env_reads"] = {"count": len(env_all), "lines": env_all}

# 6: fix commit boundary
try:
    names = subprocess.run(["git", "-C", str(REPO), "diff", "--name-only", f"{FIX_REV}~1..{FIX_REV}"],
                           capture_output=True, text=True, check=True).stdout.split()
except Exception as exc:  # pragma: no cover - evidence only
    names = [f"git_error: {exc}"]
checks["fix_commit"] = {
    "rev": FIX_REV,
    "files": names,
    "product_src_files": [n for n in names if n.startswith("src/")],
    "touches_gates_or_scripts": [n for n in names if "gates" in n or "scripts/" in n or n.startswith("tests/")],
}

(TASK / "qa-static-checks-postfix.json").write_text(
    json.dumps(checks, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({k: (v if not isinstance(v, dict) else {kk: vv for kk, vv in v.items() if kk not in ("lines",)})
                  for k, v in checks.items()}, ensure_ascii=False, indent=2)[:3000])
