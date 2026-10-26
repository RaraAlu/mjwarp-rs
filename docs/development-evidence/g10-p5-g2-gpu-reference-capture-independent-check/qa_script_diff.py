# -*- coding: utf-8 -*-
"""QA check 5: derived capture script diff is limited to the declared whitelist.

Self-written textual/block comparison (does not run the delivery's verify_inputs.py).
Writes qa-script-diff.json.
"""
import difflib
import hashlib
import json
import re
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
TASK = REPO / "docs/development-evidence/g10-p5-g2-gpu-reference-capture-independent-check"
OLD = REPO / "docs/development-evidence/g10-p5-gjk-epa/capture_mjwarp_f32.py"
NEW = REPO / "docs/development-evidence/g10-p5-g2-gpu-reference-capture/capture_mjwarp_f32_gpu.py"
PROVIDED_DIFF = REPO / "docs/development-evidence/g10-p5-g2-gpu-reference-capture/script-diff.txt"


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def block(text: str, start: str, end_pattern: str) -> str:
    start_at = text.find(start)
    if start_at < 0:
        return ""
    tail = text[start_at + len(start):]
    m = re.search(end_pattern, tail)
    if not m:
        return text[start_at:]
    return text[start_at:start_at + len(start) + m.start()]


old = read(OLD)
new = read(NEW)

# Core blocks that must be byte-identical.
blocks = {
    "CASES": (block(old, "CASES = [", r"\n\]\n"), block(new, "CASES = [", r"\n\]\n")),
    "FLOAT_FIELDS": (block(old, "FLOAT_FIELDS = ", r"\nINT_FIELDS"), block(new, "FLOAT_FIELDS = ", r"\nINT_FIELDS")),
    "INT_FIELDS": (block(old, "INT_FIELDS = ", r"\nCASES"), block(new, "INT_FIELDS = ", r"\nCASES")),
    "capture()": (block(old, "def capture(", r"\n\ndef "), block(new, "def capture(", r"\n\ndef ")),
    "load_qpos()": (block(old, "def load_qpos(", r"\n\ndef "), block(new, "def load_qpos(", r"\n\ndef ")),
    "bits32()": (block(old, "def bits32(", r"\n\ndef "), block(new, "def bits32(", r"\n\ndef ")),
}
block_result = {name: {"identical": a == b, "old_len": len(a), "new_len": len(b)}
                for name, (a, b) in blocks.items()}

# Line diff and whitelist classification.
old_lines = old.splitlines()
new_lines = new.splitlines()
diff = list(difflib.unified_diff(old_lines, new_lines, fromfile="old", tofile="new", lineterm=""))
allowed_patterns = [
    r"set_device", r"upstream-mjwarp-f32-gpu", r"capture_mjwarp_f32_gpu",
    r"g2-upstream-mjwarp-f32-gpu-reference", r"gpu-reference-capture",
    r"task_id", r"purpose", r"provenance", r"gpu_info", r"nvidia-smi",
    r"CAPTURED_AT", r"captured_at", r"kernel_cache", r"CACHE_ROOT", r"CACHE =",
    r"OUT =", r"sys.argv", r"sys.stdout", r"reconfigure", r"import os",
    r"import subprocess", r"import sys", r"MJWARP_CAPTURE_AT", r"authoring_script",
    r"authoring_only", r"reads_product_tree", r"product_python_dependency",
    r"upstream_revision", r"cuda", r"print\(", r"ROOT =", r"EVIDENCE =",
    r"^[-+]\s*$", r"^[-+]\s*\"\"\"", r"性质", r"旧 CPU", r"派生", r"用法",
    r"环境变量", r"一次 GPU", r"sha256\(__file__\)", r"report =", r"kind",
]
path_or_device_patterns = [
    r"EVIDENCE\.mkdir", r"OUT\.parent\.mkdir", r"\"device\"", r"EVIDENCE /",
    r"OUT\.write_text", r"str\(wp\.get_device\(\)\)",
]


def docstring_region(lines):
    start = next((i for i, line in enumerate(lines) if line.startswith('"""')), None)
    if start is None:
        return set()
    end = start + 1
    while end < len(lines) and not lines[end].rstrip().endswith('"""'):
        end += 1
    return set(lines[start:end + 1])


def function_region(lines, header):
    for index, line in enumerate(lines):
        if line.startswith(header):
            end = index + 1
            while end < len(lines) and not lines[end].startswith("def "):
                end += 1
            return set(lines[index:end])
    return set()


doc_allowed = docstring_region(old_lines) | docstring_region(new_lines)
gpu_allowed = function_region(new_lines, "def gpu_info(")
changed = [line for line in diff if line.startswith(("+", "-")) and not line.startswith(("+++", "---"))]
categories = {"docstring": 0, "provenance_helper": 0, "path_or_device": 0,
              "whitelisted": 0, "unmatched": []}
for line in changed:
    content = line[1:]
    if content.strip() == "" or content in doc_allowed:
        categories["docstring"] += 1
    elif content in gpu_allowed:
        categories["provenance_helper"] += 1
    elif any(re.search(pattern, content) for pattern in path_or_device_patterns):
        categories["path_or_device"] += 1
    elif any(re.search(pattern, content) for pattern in allowed_patterns):
        categories["whitelisted"] += 1
    else:
        categories["unmatched"].append(line)
unmatched = categories["unmatched"]

provided = PROVIDED_DIFF.read_text(encoding="utf-8") if PROVIDED_DIFF.is_file() else ""
provided_hunks = [ln for ln in provided.splitlines() if ln.startswith("@@")]

result = {
    "old": {"path": str(OLD.relative_to(REPO)).replace("\\", "/"), "sha256": sha256(OLD),
            "lines": len(old_lines)},
    "new": {"path": str(NEW.relative_to(REPO)).replace("\\", "/"), "sha256": sha256(NEW),
            "lines": len(new_lines)},
    "core_blocks_identical": all(v["identical"] for v in block_result.values()),
    "core_blocks": block_result,
    "diff_hunks": len([ln for ln in diff if ln.startswith("@@")]),
    "changed_lines": len(changed),
    "change_categories": categories,
    "whitelisted_changes": len(changed) - len(unmatched),
    "unmatched_changes": unmatched,
    "capture_twice_logic_present": (
        "first = capture(xml_path, qpos, nworld)" in new
        and "second = capture(xml_path, qpos, nworld)" in new
        and "raw_identical = first == second" in new),
    "field_count_17": len(re.findall(r'"[a-z_]+"', blocks["FLOAT_FIELDS"][1]))
    + len(re.findall(r'"[a-z_]+"', blocks["INT_FIELDS"][1])),
    "provided_diff": {
        "exists": PROVIDED_DIFF.is_file(),
        "sha256": sha256(PROVIDED_DIFF) if PROVIDED_DIFF.is_file() else None,
        "bytes": PROVIDED_DIFF.stat().st_size if PROVIDED_DIFF.is_file() else 0,
        "hunk_headers": provided_hunks,
        "hunks_match_my_diff": len(provided_hunks) == len([ln for ln in diff if ln.startswith("@@")]),
    },
    "blocking": (not result_ok) if (result_ok := all(v["identical"] for v in block_result.values())
                                    and not unmatched) else True,
}
(TASK / "qa-script-diff.json").write_text(
    json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result, ensure_ascii=False, indent=2))
