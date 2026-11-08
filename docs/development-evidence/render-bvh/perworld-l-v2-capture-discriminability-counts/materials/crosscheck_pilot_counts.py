"""CPU cross-check: new spec §2.4 discriminability counts vs the G-L0 pilot readout.

Authoring-only, zero GPU, no capture. The pilot attempt-3 npz live under `target/`
(not committed); this script recomputes the counts with the production helper
`scripts/perworld_capture_meta.py::discriminability_counts` and compares them with
`target/render-bvh/perworld-l-v2-gl0/window/readout.json` (fixed readout script
`readout_c1_c2.py`, sha 4a054aba…/16,711 B).

Usage: <pinned python> -B <this script>   (run from the repository root or anywhere)
"""

import hashlib
import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[5]
sys.path.insert(0, str(ROOT / "scripts"))

import numpy as np  # noqa: E402

import perworld_capture_meta as meta  # noqa: E402

PILOT = ROOT / "target/render-bvh/perworld-l-v2-gl0/pilot-out/nw3-light-hinge02040"
READOUT = ROOT / "target/render-bvh/perworld-l-v2-gl0/window/readout.json"
EVIDENCE = pathlib.Path(__file__).resolve().parents[1]
EXPECTED = {
    "c1_rgb_changed_px": {"0->1": {"magnitude": 38285, "bitwise": 43604},
                          "1->2": {"magnitude": 40644, "bitwise": 41815}},
    "c2_depth_changed_px": {"0->1": 35828, "1->2": 33883},
    "c2_seg_changed_px": {"0->1": 16041, "1->2": 16754},
    "campersp_depth_seg_changed_px": {"0->1": {"depth": 0, "seg": 0}, "1->2": {"depth": 0, "seg": 0}},
    "camhinge_rgb_changed_px": {"0->1": {"magnitude": 50363, "bitwise": 54380},
                                "1->2": {"magnitude": 52329, "bitwise": 54250}},
}


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main() -> int:
    arrays = {}
    for kind in ("rgb", "depth", "seg"):
        with np.load(PILOT / f"{kind}.npz") as npz:
            for name in npz.files:
                arrays[name] = npz[name]
    counts = meta.discriminability_counts(arrays)
    readout = json.loads(READOUT.read_text(encoding="utf-8"))
    readout_flat = {
        "c1_rgb_changed_px": {pair: {"magnitude": value["rgb_changed_px"],
                                     "bitwise": value["rgb_bitwise_changed_px"]}
                              for pair, value in readout["c1_cam_persp"].items()},
        "c2_depth_changed_px": {pair: value["depth_changed_px"] for pair, value in readout["c2_cam_hinge"].items()},
        "c2_seg_changed_px": {pair: value["seg_changed_px"] for pair, value in readout["c2_cam_hinge"].items()},
        "campersp_depth_seg_changed_px": {pair: {"depth": value["depth_changed_px"], "seg": value["seg_changed_px"]}
                                          for pair, value in readout["diagnostics_no_threshold"]["cam_persp_depth_seg"].items()},
        "camhinge_rgb_changed_px": {pair: {"magnitude": value["rgb_changed_px"],
                                           "bitwise": value["rgb_bitwise_changed_px"]}
                                    for pair, value in readout["diagnostics_no_threshold"]["cam_hinge_rgb"].items()},
    }
    failures = []
    if counts != EXPECTED:
        failures.append(f"counts != expected readout constants: {counts}")
    if counts != readout_flat:
        failures.append(f"counts != readout.json: {counts} vs {readout_flat}")
    if set(counts) != set(EXPECTED):
        failures.append(f"field set mismatch: {sorted(counts)}")
    artifact = {
        "schema": 1,
        "kind": "capture-2-4-discriminability-crosscheck",
        "helper": "scripts/perworld_capture_meta.py::discriminability_counts",
        "pilot_dir": PILOT.relative_to(ROOT).as_posix(),
        "pilot_arrays_sha256": {name: sha256(PILOT / f"{name}.npz") for name in ("rgb", "depth", "seg", "state")},
        "readout": {"path": READOUT.relative_to(ROOT).as_posix(), "sha256": sha256(READOUT),
                    "verdict": readout.get("verdict", {}).get("verdict")},
        "counts": counts,
        "expected_readout_constants": EXPECTED,
        "matches_readout_json": counts == readout_flat,
        "unexpected": len(failures),
        "result": "PASS" if not failures else "FAIL",
    }
    (EVIDENCE / "artifacts").mkdir(parents=True, exist_ok=True)
    (EVIDENCE / "artifacts/crosscheck-pilot-counts.json").write_text(
        json.dumps(artifact, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
    lines = [
        "===== CPU cross-check: capture §2.4 discriminability counts vs G-L0 pilot readout (exit="
        f"{0 if not failures else 1}) =====",
        "helper: scripts/perworld_capture_meta.py::discriminability_counts",
        f"pilot: {PILOT.relative_to(ROOT).as_posix()} (npz under target/, not committed)",
        f"readout: {READOUT.relative_to(ROOT).as_posix()} sha256={artifact['readout']['sha256']}",
        json.dumps(counts, ensure_ascii=False, indent=2),
        f"matches_readout_json: {artifact['matches_readout_json']}",
    ]
    for failure in failures:
        lines.append(f"FAIL {failure}")
    lines.append("SUMMARY " + ("PASS" if not failures else "FAIL"))
    (EVIDENCE / "logs").mkdir(parents=True, exist_ok=True)
    (EVIDENCE / "logs/crosscheck-pilot-counts.log").write_text("\n".join(lines) + "\n",
                                                               encoding="utf-8", newline="\n")
    print("\n".join(lines))
    return 0 if not failures else 1


if __name__ == "__main__":
    sys.exit(main())
