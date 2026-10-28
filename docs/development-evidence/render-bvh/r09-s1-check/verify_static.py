"""QA static evidence check for the R09-S1 independent review.

Recomputes line counts, red-line diffs, wiring line numbers and module source
registration, manifest identities, encodings and forbidden terms. Read-only.
"""
import hashlib
import json
import subprocess
from pathlib import Path

R = Path(r"D:\Rust\mjwarp-rs")
S = R / "docs/development-evidence/render-bvh/r09-s1-check"
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
    "src/render/context/lighting.rs": 184,
    "src/render/context/params.rs": 189,
    "src/render/mod.rs": 380,
    "tests/render_lighting_params.rs": 319,
    "src/render/context/camera_output.rs": 499,
    "src/model/render.rs": 500,
    "src/render/kernel/launch.rs": 500,
    "src/render/kernel/megakernel.rs": 491,
    "src/render/kernel/cast.rs": 494,
}
for rel, want in expected.items():
    check(lf(rel) == want, f"line count {rel}: {lf(rel)} != {want}")
red = git("diff", "--name-only", "ef557f6e^..0a72c653", "--",
          "src/render/kernel", "src/render/kernel/cuda", "fixtures", "Cargo.toml",
          "Cargo.lock", "src/render/context/camera_output.rs", "src/model/render.rs")
check(red.strip() == "", f"red-line diff not empty {red.split()}")
num = git("show", "--numstat", "--format=", "ef557f6e").splitlines()
check(any(row.startswith("6\t0\tsrc/render/context/params.rs") for row in num),
      "params.rs is not +6")
check(any(row.startswith("2\t0\tsrc/render/mod.rs") for row in num), "render/mod.rs is not +2")
check(not any(row.endswith("src/render/context.rs") for row in num), "context.rs was modified")
md = (R / ".normify/modules/mjwarp/render.md").read_text(encoding="utf-8")
check("src/render/context/lighting.rs" not in md, "lighting.rs is registered (observation O1 stale)")
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
check(receipt.get("task_id") == "2026-10-28-render-bvh-r09-s1-check", "receipt task_id")
check(receipt.get("verdict") == "pass", "receipt verdict")
enc_bad = []
for path in sorted(S.rglob("*")):
    if path.is_file() and path.suffix in (".md", ".json", ".txt", ".log", ".csv"):
        raw = path.read_bytes()
        if raw[:3] == b"\xef\xbb\xbf" or b"\r\n" in raw:
            enc_bad.append(str(path.relative_to(S)))
check(not enc_bad, f"encoding issues {enc_bad[:3]}")
text = (S / "README.md").read_text(encoding="utf-8") + json.dumps(receipt, ensure_ascii=False)
terms = [t for c in json.loads((R / ".normify/changes/2026-10-28-render-bvh-r09-s1-check.json")
                               .read_text(encoding="utf-8"))["checks"]
         if c.get("kind") == "forbid_claim" for t in c["terms"]]
check(not [t for t in terms if t in text], "forbidden terms present")
print(f"checks={checks} problems={len(problems)} result=" + ("PASS" if not problems else "FAIL"))
for problem in problems:
    print("PROBLEM:", problem)
