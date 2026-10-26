# -*- coding: utf-8 -*-
"""QA check 3: two independent rounds (different process + different kernel cache).

Self-computed sha256; the round logs and cache directories are inspected as the
independent-process/cache evidence. Writes qa-two-rounds.json.
"""
import hashlib
import json
import re
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
EV = REPO / "docs/development-evidence/g10-p5-g2-gpu-reference-capture"
TASK = REPO / "docs/development-evidence/g10-p5-g2-gpu-reference-capture-independent-check"
NEW = REPO / "docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32-gpu.json"
CACHE_ROOT = REPO / "target/g10-p5-g2/warp-cache-gpu"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def cache_info(round_dir: Path) -> dict:
    if not round_dir.is_dir():
        return {"exists": False}
    files = [p for p in round_dir.rglob("*") if p.is_file()]
    kinds = {}
    for p in files:
        kinds[p.suffix or "(noext)"] = kinds.get(p.suffix or "(noext)", 0) + 1
    return {
        "exists": True,
        "path": str(round_dir),
        "files": len(files),
        "bytes": sum(p.stat().st_size for p in files),
        "kinds": kinds,
        "mtime_min": min((p.stat().st_mtime for p in files), default=None),
        "mtime_max": max((p.stat().st_mtime for p in files), default=None),
        "has_ptx": any(p.suffix == ".ptx" for p in files),
        "has_cpp": any(p.suffix == ".cpp" for p in files),
    }


def parse_round_log(path: Path) -> dict:
    text = read(path)
    out = {"cmd": "", "device": "", "kernel_cache_dir": "", "wrote": "",
           "wall_ms": None, "exit": None, "compiled_modules": 0, "case_lines": []}
    for line in text.splitlines():
        line = line.strip()
        if line.startswith("CMD:"):
            out["cmd"] = line
        elif line.startswith("device:"):
            out["device"] = line
        elif line.startswith("kernel_cache_dir:"):
            out["kernel_cache_dir"] = line
        elif line.startswith("wrote "):
            out["wrote"] = line
        elif "took" in line and "(compiled)" in line:
            out["compiled_modules"] += 1
        elif re.match(r"^(cylinder-box|ellipsoid-box|mesh-mesh|convex-mesh|degenerate-mesh):", line):
            out["case_lines"].append(line)
        elif line.startswith("EXIT="):
            out["exit"] = line
            m = re.search(r"WALL_MS=(\d+)", line)
            if m:
                out["wall_ms"] = int(m.group(1))
    return out


raw1 = EV / "rounds/round1-raw.json"
raw2 = EV / "rounds/round2-raw.json"
can1 = EV / "rounds/round1-canonical.json"
can2 = EV / "rounds/round2-canonical.json"
sha = {
    "round1_raw": sha256(raw1),
    "round2_raw": sha256(raw2),
    "round1_canonical": sha256(can1),
    "round2_canonical": sha256(can2),
    "final_new_reference": sha256(NEW),
}
meta = json.loads((EV / "capture-meta.json").read_text(encoding="utf-8"))
new_ref = json.loads(NEW.read_text(encoding="utf-8"))

result = {
    "sha256": sha,
    "raw_identical": sha["round1_raw"] == sha["round2_raw"],
    "canonical_identical": sha["round1_canonical"] == sha["round2_canonical"],
    "final_equals_round1_raw": sha["final_new_reference"] == sha["round1_raw"],
    "bytes_same": raw1.stat().st_size == raw2.stat().st_size == NEW.stat().st_size,
    "meta_claims": {
        "round1_raw": meta["round1"]["raw_sha256"],
        "round2_raw": meta["round2"]["raw_sha256"],
        "round1_canonical": meta["round1"]["canonical_sha256"],
        "round2_canonical": meta["round2"]["canonical_sha256"],
        "final": meta["final"]["sha256"],
        "meta_matches_recomputed": (
            meta["round1"]["raw_sha256"] == sha["round1_raw"]
            and meta["round2"]["raw_sha256"] == sha["round2_raw"]
            and meta["round1"]["canonical_sha256"] == sha["round1_canonical"]
            and meta["round2"]["canonical_sha256"] == sha["round2_canonical"]
            and meta["final"]["sha256"] == sha["final_new_reference"]
        ),
    },
    "round_logs": {
        "round1": parse_round_log(EV / "logs/window-20261026-012956-round1.txt"),
        "round2": parse_round_log(EV / "logs/window-20261026-012956-round2.txt"),
    },
    "caches": {
        "round1": cache_info(CACHE_ROOT / "round1"),
        "round2": cache_info(CACHE_ROOT / "round2"),
    },
    "two_captures_bitwise_identical": {name: case["two_captures_bitwise_identical"]
                                       for name, case in new_ref["cases"].items()},
    "new_reference_provenance": {
        "device": new_ref["provenance"].get("device"),
        "captured_at": new_ref["provenance"].get("captured_at"),
        "kernel_cache_root": new_ref["provenance"].get("kernel_cache_root"),
    },
    "meta_captured_at": meta.get("captured_at"),
}

# Independence indicators: different cache dirs / different wall times / separate CMD lines.
r1 = result["round_logs"]["round1"]
r2 = result["round_logs"]["round2"]
result["independence"] = {
    "different_cmd": r1["cmd"] != r2["cmd"],
    "round1_cache_dir": r1["kernel_cache_dir"],
    "round2_cache_dir": r2["kernel_cache_dir"],
    "different_cache_dir": r1["kernel_cache_dir"] != r2["kernel_cache_dir"],
    "round1_wall_ms": r1["wall_ms"],
    "round2_wall_ms": r2["wall_ms"],
    "different_wall": r1["wall_ms"] != r2["wall_ms"],
    "round1_compiled_modules": r1["compiled_modules"],
    "round2_compiled_modules": r2["compiled_modules"],
    "cache_dirs_distinct": (result["caches"]["round1"].get("path")
                            != result["caches"]["round2"].get("path")),
    "both_caches_nonempty": result["caches"]["round1"].get("files", 0) > 0
                            and result["caches"]["round2"].get("files", 0) > 0,
    "both_logs_exit_zero": (r1["exit"] or "").startswith("EXIT=0")
                           and (r2["exit"] or "").startswith("EXIT=0"),
    "separate_invocations": {
        "different_cmd": r1["cmd"] != r2["cmd"],
        "different_cache_dir": r1["kernel_cache_dir"] != r2["kernel_cache_dir"],
        "different_wall_ms": r1["wall_ms"] != r2["wall_ms"],
        "round_log_mtimes": {
            "round1": (EV / "logs/window-20261026-012956-round1.txt").stat().st_mtime,
            "round2": (EV / "logs/window-20261026-012956-round2.txt").stat().st_mtime,
        },
        "sequential_logs": (EV / "logs/window-20261026-012956-round1.txt").stat().st_mtime
                           < (EV / "logs/window-20261026-012956-round2.txt").stat().st_mtime,
        "note": "no PID was logged by the runner; independence evidence = two separate CMD "
                "invocations with different cache dirs, different wall times, sequential round "
                "logs, and two distinct non-empty kernel caches (each with its own .cu/.ptx set).",
    },
}

(TASK / "qa-two-rounds.json").write_text(
    json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result, ensure_ascii=False, indent=2))
