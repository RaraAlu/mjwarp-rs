# -*- coding: utf-8 -*-
"""QA check 1: the old CPU reference and its capture script are byte-identical.

Independent of the delivery's own sha logs: uses git objects + worktree bytes.
"""
import hashlib
import json
import subprocess
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
TASK = REPO / "docs/development-evidence/g10-p5-g2-gpu-reference-capture-independent-check"
OLD_REF = "docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32.json"
OLD_SCRIPT = "docs/development-evidence/g10-p5-gjk-epa/capture_mjwarp_f32.py"
NEW_REF = "docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32-gpu.json"


def git(*args):
    out = subprocess.run(["git", "-C", str(REPO), *args], capture_output=True,
                         text=True, encoding="utf-8", errors="replace")
    return out.stdout.strip(), out.returncode


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


head, _ = git("rev-parse", "HEAD")
parent, _ = git("rev-parse", "HEAD^")
head = head.strip()
parent = parent.strip()
result = {
    "head": head,
    "parent": parent,
    "old_reference": OLD_REF,
    "old_script": OLD_SCRIPT,
    "new_reference": NEW_REF,
}

# 1) the capture commit must not touch the old files
diff_old, _ = git("diff", "--name-status", f"{head}^..{head}", "--", OLD_REF, OLD_SCRIPT)
result["commit_diff_old_files"] = diff_old if diff_old else "(none)"

# 2) blob identity across parent/head for the old files
blob = {}
for path in (OLD_REF, OLD_SCRIPT):
    a, _ = git("rev-parse", f"{parent}:{path}")
    b, _ = git("rev-parse", f"{head}:{path}")
    work = (REPO / path).read_bytes()
    blob[path] = {
        "parent_blob": a,
        "head_blob": b,
        "blob_same": a == b,
        "worktree_sha256": sha256(work),
        "head_content_sha256": sha256(subprocess.run(
            ["git", "-C", str(REPO), "show", f"{head}:{path}"],
            capture_output=True).stdout),
        "worktree_equals_head": sha256(work) == sha256(subprocess.run(
            ["git", "-C", str(REPO), "show", f"{head}:{path}"], capture_output=True).stdout),
    }
result["old_blobs"] = blob

# 3) new reference must be an ADD in this commit (absent in parent, present in head)
diff_new, _ = git("diff", "--name-status", f"{head}^..{head}", "--", NEW_REF)
_, rc_parent = git("cat-file", "-e", f"{parent}:{NEW_REF}")
_, rc_head = git("cat-file", "-e", f"{head}:{NEW_REF}")
new_bytes = (REPO / NEW_REF).read_bytes()
result["new_reference"] = {
    "commit_status": diff_new if diff_new else "(not in commit)",
    "absent_in_parent": rc_parent != 0,
    "present_in_head": rc_head == 0,
    "worktree_exists": (REPO / NEW_REF).is_file(),
    "worktree_sha256": sha256(new_bytes),
    "worktree_bytes": len(new_bytes),
}

# 4) commit file classification: no product/script/fixture/criteria changes
names, _ = git("diff", "--name-status", f"{head}^..{head}")
allowed_prefixes = (
    "docs/development-evidence/g10-p5-g2-gpu-reference-capture/",
    "docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32-gpu.json",
    ".normify/",
)
unexpected = []
for line in [ln for ln in names.splitlines() if ln.strip()]:
    path = line.split("\t", 1)[1]
    if not any(path.startswith(prefix) for prefix in allowed_prefixes):
        unexpected.append(line)
result["commit_files"] = [ln for ln in names.splitlines() if ln.strip()]
result["unexpected_commit_files"] = unexpected
result["commit_touches_src"] = any(
    ln.split("\t", 1)[1].startswith(("src/", "tests/", "fixtures/", "scripts/"))
    for ln in names.splitlines() if ln.strip())
result["commit_touches_case_compare"] = "case_compare.rs" in names

# 5) current worktree status for the old evidence dir and fixtures
status, _ = git("status", "--porcelain", "--", "docs/development-evidence/g10-p5-gjk-epa", "fixtures/g10-p5")
result["worktree_status_old_area"] = status if status else "(clean)"

# 6) new reference is NOT yet the criterion: case_compare.rs still reads the old CPU file
cc, rc = git("show", f"{head}:src/physics/collision/gjk/case_compare.rs")
result["case_compare_still_old_reference"] = ("upstream-mjwarp-f32.json" in cc)
result["case_compare_references_gpu_file"] = ("upstream-mjwarp-f32-gpu.json" in cc)
src_diff, _ = git("diff", "--name-status", "209b1039..HEAD", "--", "src")
result["src_diff_since_209b1039"] = src_diff if src_diff else "(none)"

(TASK / "qa-old-untouched.json").write_text(
    json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result, ensure_ascii=False, indent=2))
