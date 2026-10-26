# -*- coding: utf-8 -*-
"""QA check 4: provenance truthfulness (device/GPU/driver/versions/script sha/cache/time).

Compares the new reference's provenance against the live machine (`nvidia-smi`,
installed packages) and the repository script bytes. Read-only; no GPU context.
"""
import hashlib
import importlib.metadata
import json
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
TASK = REPO / "docs/development-evidence/g10-p5-g2-gpu-reference-capture-independent-check"
NEW = REPO / "docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32-gpu.json"
SCRIPT = REPO / "docs/development-evidence/g10-p5-g2-gpu-reference-capture/capture_mjwarp_f32_gpu.py"
OLD_SCRIPT = REPO / "docs/development-evidence/g10-p5-gjk-epa/capture_mjwarp_f32.py"
META = REPO / "docs/development-evidence/g10-p5-g2-gpu-reference-capture/capture-meta.json"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(*args):
    out = subprocess.run(args, capture_output=True, text=True, encoding="utf-8", errors="replace")
    return out.stdout.strip(), out.returncode


ref = json.loads(NEW.read_text(encoding="utf-8"))
prov = ref["provenance"]
meta = json.loads(META.read_text(encoding="utf-8"))

# live nvidia-smi (read-only query, no compute context)
smi, rc = run("nvidia-smi", "--query-gpu=name,memory.total,driver_version,compute_cap",
              "--format=csv,noheader")
smi_fields = [part.strip() for part in smi.split(",")] if rc == 0 and smi else []
live_gpu = {
    "name": smi_fields[0] if len(smi_fields) > 0 else None,
    "memory_total": smi_fields[1] if len(smi_fields) > 1 else None,
    "driver": smi_fields[2] if len(smi_fields) > 2 else None,
    "compute_capability": smi_fields[3] if len(smi_fields) > 3 else None,
}

installed = {}
for package in ("warp-lang", "mujoco-warp", "mujoco", "numpy"):
    try:
        installed[package] = importlib.metadata.version(package)
    except importlib.metadata.PackageNotFoundError:
        installed[package] = None

script_sha = sha256(SCRIPT)
old_script_sha = sha256(OLD_SCRIPT)
new_ref_sha = sha256(NEW)

captured_at = prov.get("captured_at", "")
window_start = None
first_log = sorted((REPO / "docs/development-evidence/g10-p5-g2-gpu-reference-capture/logs").glob(
    "window-20261026-012956-*.txt"))
if first_log:
    m = re.search(r"window-(\d{8})-(\d{6})", first_log[0].name)
    if m:
        window_start = f"{m.group(1)[:4]}-{m.group(1)[4:6]}-{m.group(1)[6:]}T{m.group(2)[:2]}:{m.group(2)[2:4]}:{m.group(2)[4:]}Z"

result = {
    "new_reference": {"path": str(NEW.relative_to(REPO)).replace("\\", "/"),
                      "sha256": new_ref_sha, "bytes": NEW.stat().st_size,
                      "kind": ref.get("kind"), "task_id": ref.get("task_id")},
    "provenance": prov,
    "live_nvidia_smi": {"raw": smi, "parsed": live_gpu, "exit": rc},
    "gpu_match": {
        "name": prov.get("gpu", {}).get("name") == live_gpu["name"],
        "memory_total": prov.get("gpu", {}).get("memory_total") == live_gpu["memory_total"],
        "driver": prov.get("gpu", {}).get("driver") == live_gpu["driver"],
        "compute_capability": prov.get("gpu", {}).get("compute_capability") == live_gpu["compute_capability"],
    },
    "installed_versions": installed,
    "version_match": {
        "warp": prov["versions"].get("warp") == installed["warp-lang"],
        "mujoco_warp": prov["versions"].get("mujoco_warp") == installed["mujoco-warp"],
        "mujoco": prov["versions"].get("mujoco") == installed["mujoco"],
        "numpy": prov["versions"].get("numpy") == installed["numpy"],
        "python_prefix": installed.get("python") or ".".join(map(str, sys.version_info[:3]))
        == prov["versions"].get("python"),
        "python_runtime": ".".join(map(str, sys.version_info[:3])),
    },
    "script": {
        "path": str(SCRIPT.relative_to(REPO)).replace("\\", "/"),
        "sha256": script_sha,
        "provenance_sha256": prov.get("authoring_script_sha256"),
        "match": script_sha == prov.get("authoring_script_sha256"),
        "script_exists": SCRIPT.is_file(),
    },
    "old_script": {"sha256": old_script_sha,
                   "expected_from_delivery": "1edd7c8e5d2c7ec7ba7cb9eb218e4533e4f7fdb77c435de1107ad4086c624312",
                   "unchanged": old_script_sha == "1edd7c8e5d2c7ec7ba7cb9eb218e4533e4f7fdb77c435de1107ad4086c624312"},
    "upstream_revision": ref.get("upstream_revision"),
    "device": prov.get("device"),
    "authoring_flags": {key: prov.get(key) for key in
                        ("authoring_only", "reads_product_tree", "product_python_dependency")},
    "captured_at": captured_at,
    "captured_at_matches_meta": captured_at == meta.get("captured_at"),
    "window_start_from_log_names": window_start,
    "kernel_cache_root": prov.get("kernel_cache_root"),
    "kernel_cache_root_exists": (REPO / prov.get("kernel_cache_root", "")).is_dir()
    if prov.get("kernel_cache_root") else False,
}

(TASK / "qa-provenance.json").write_text(
    json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result, ensure_ascii=False, indent=2))
