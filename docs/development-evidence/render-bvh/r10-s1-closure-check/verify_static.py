"""QA static evidence check for the R10-S1 observation-closure review.

Recomputes split line counts, red-line diffs, module registration, the closure
receipt, manifest identities, encodings and forbidden terms. Read-only.
"""
import hashlib
import json
import subprocess
from pathlib import Path

R = Path(r"D:\Rust\mjwarp-rs")
S = R / "docs/development-evidence/render-bvh/r10-s1-closure-check"
EV = R / "docs/development-evidence/render-bvh/r10-s1-world-state"
problems = []
checks = 0


def check(cond, label):
    global checks
    checks += 1
    if not cond:
        problems.append(label)


def lf(rel):
    return (R / rel).read_bytes().count(b"\n")


def git(*args):
    return subprocess.run(["git", *args], cwd=R, capture_output=True, text=True,
                          encoding="utf-8", errors="replace").stdout


expected = {
    "src/render/context/world/mod.rs": 22,
    "src/render/context/world/errors.rs": 171,
    "src/render/context/world/state.rs": 199,
    "src/render/context/world/layout.rs": 175,
    "src/render/context.rs": 427,
    "src/render/mod.rs": 378,
    "tests/render_multi_world_layout.rs": 478,
    "src/render/context/camera_output.rs": 499,
    "src/model/render.rs": 500,
    "src/render/kernel/launch.rs": 500,
    "src/render/kernel/megakernel.rs": 498,
    "src/render/kernel/cast.rs": 494,
}
for rel, want in expected.items():
    check(lf(rel) == want, f"line count {rel}: {lf(rel)} != {want}")
check(not (R / "src/render/context/world.rs").exists(), "world.rs still exists")
red = git("diff", "--name-only", "248b33ff..HEAD", "--",
          "src/render/context/camera_output.rs", "src/model/render.rs", "src/render/kernel",
          "src/render/bvh", "fixtures", "Cargo.toml", "Cargo.lock")
check(red.strip() == "", f"red-line diff not empty {red.split()}")
touched = [line for line in git("diff", "--name-only", "248b33ff..HEAD",
                                "--", "src/", "tests/").splitlines() if line.strip()]
check(set(touched) == {
    "src/render/context/world.rs",
    "src/render/context/world/mod.rs",
    "src/render/context/world/errors.rs",
    "src/render/context/world/state.rs",
    "src/render/context/world/layout.rs",
    "tests/render_multi_world_layout.rs",
}, f"unexpected code paths {touched}")
md = (R / ".normify/modules/mjwarp/render.md").read_text(encoding="utf-8")
check(md.count("[[source]]") == 30, "module source count is not 30")
check("src/render/context/world/mod.rs" in md, "world/mod.rs not registered as module source")
tests = (R / "tests/render_multi_world_layout.rs").read_text(encoding="utf-8")
check(tests.count("#[test]") == 8, "test function count is not 8")
for name in ("empty_camera_selection_is_accepted_without_panic",
             "huge_world_count_returns_count_overflow_without_panic",
             "nworld_zero_has_priority_over_layout_errors"):
    check(name in tests, f"new test missing {name}")
closure = json.loads((EV / "obs-closure-receipt.json").read_text(encoding="utf-8"))
check(closure.get("task_id") == "2026-10-28-render-bvh-r10-s1-obs-closure", "closure receipt task_id")
check(set(closure.get("fixes", {})) >= {"F1", "F2", "F3"}, "closure fixes missing F1/F2/F3")
manifest = json.loads((S / "manifest.json").read_text(encoding="utf-8"))
bad = []
for entry in manifest.get("files", []):
    raw = (S / entry["path"]).read_bytes()
    if (hashlib.sha256(raw).hexdigest() != entry["sha256"]
            or len(raw) != entry["bytes"] or raw.count(b"\n") != entry["lf_lines"]):
        bad.append(entry["path"])
check(not bad, f"manifest mismatches {bad[:3]}")
check((S / "README.md").read_bytes().count(b"\n") <= 500, "README too long")
receipt = json.loads((S / "receipt.json").read_text(encoding="utf-8"))
check(receipt.get("task_id") == "2026-10-28-render-bvh-r10-s1-closure-check", "receipt task_id")
check(receipt.get("verdict") == "pass", "receipt verdict")
enc_bad = []
for path in sorted(S.rglob("*")):
    if path.is_file() and path.suffix in (".md", ".json", ".txt", ".log", ".csv", ".rs"):
        raw = path.read_bytes()
        if raw[:3] == b"\xef\xbb\xbf" or b"\r\n" in raw:
            enc_bad.append(str(path.relative_to(S)))
check(not enc_bad, f"encoding issues {enc_bad[:3]}")
text = (S / "README.md").read_text(encoding="utf-8") + json.dumps(receipt, ensure_ascii=False)
terms = [t for c in json.loads((R / ".normify/changes/2026-10-28-render-bvh-r10-s1-closure-check.json")
                               .read_text(encoding="utf-8"))["checks"]
         if c.get("kind") == "forbid_claim" for t in c["terms"]]
check(not [t for t in terms if t in text], "forbidden terms present")
print(f"checks={checks} problems={len(problems)} result=" + ("PASS" if not problems else "FAIL"))
for problem in problems:
    print("PROBLEM:", problem)
