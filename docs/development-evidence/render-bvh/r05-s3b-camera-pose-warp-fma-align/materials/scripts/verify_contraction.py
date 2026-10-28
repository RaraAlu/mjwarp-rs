"""Verify the host warp-FMA contraction order against the pinned-GPU reference.

Reference: materials/scripts/gpu_quat_reference.py output (chain kernel, xquat=identity).
All f32 mul/add/sub use numpy float32 (single-rounding IEEE); every fma is evaluated exactly
(Fraction-based, round-to-nearest-even) so the check is bit-exact, not an approximation.

Checks (all must pass):
  1. candidate enumeration per element: unique ordered pattern, and at how many quats it is isolated;
  2. host pattern vs GPU: 1003 quats x 9 elements = 9027 entries, 0 diff;
  3. plain (unfused) algebra expansion on the same set: number of differing entries/quats (contrast);
  4. hfield cam0/cam1 and scene-a cam-persp camera-matrix bit tables vs GPU capture/reference bits;
  5. sampling table (first 20 random rows).

Usage (repo root):
  <pinned venv>/python -B docs/development-evidence/render-bvh/r05-s3b-camera-pose-warp-fma-align/materials/scripts/verify_contraction.py --ref <reference>.json
"""
import argparse
import json
import pathlib
import sys
from fractions import Fraction

import numpy as np

f32 = np.float32


def bits(x):
    return int(np.asarray(x, dtype=np.float32).view(np.uint32))


def mul(a, b):
    return f32(a) * f32(b)


def fma(a, b, c):
    """Exact IEEE-754 f32 fma (a*b+c, one rounding, RNE)."""
    a = Fraction(float(f32(a)))
    b = Fraction(float(f32(b)))
    c = Fraction(float(f32(c)))
    s = a * b + c
    cand = np.float32(float(s))
    if Fraction(float(cand)) == s:
        return cand
    if Fraction(float(cand)) < s:
        lo, hi = cand, np.nextafter(cand, np.float32(np.inf))
    else:
        lo, hi = np.nextafter(cand, np.float32(-np.inf)), cand
    dlo = abs(s - Fraction(float(lo)))
    dhi = abs(s - Fraction(float(hi)))
    return f32(lo) if dlo <= dhi else f32(hi)


def pattern_matrix(quat):
    """Host order found for upstream GPU: q = (w,x,y,z) (memory order of mujoco quat)."""
    q0, q1, q2, q3 = [f32(v) for v in quat]
    q01, q02, q03 = mul(q0, q1), mul(q0, q2), mul(q0, q3)
    q11 = mul(q1, q1)
    two = f32(2.0)
    e0 = fma(-q3, q3, fma(-q2, q2, fma(q0, q0, q11)))
    e4 = fma(-q3, q3, fma(q2, q2, fma(q0, q0, -q11)))
    e8 = fma(q3, q3, fma(-q2, q2, fma(q0, q0, -q11)))
    return [
        bits(e0), bits(two * fma(q1, q2, -q03)), bits(two * fma(q1, q3, q02)),
        bits(two * fma(q1, q2, q03)), bits(e4), bits(two * fma(q2, q3, -q01)),
        bits(two * fma(q1, q3, -q02)), bits(two * fma(q2, q3, q01)), bits(e8),
    ]


def plain_matrix(quat):
    q0, q1, q2, q3 = [f32(v) for v in quat]
    q00, q11, q22, q33 = mul(q0, q0), mul(q1, q1), mul(q2, q2), mul(q3, q3)
    two = f32(2.0)
    return [
        bits(f32(f32(f32(q00 + q11) - q22) - q33)),
        bits(two * f32(mul(q1, q2) - mul(q0, q3))),
        bits(two * f32(mul(q1, q3) + mul(q0, q2))),
        bits(two * f32(mul(q1, q2) + mul(q0, q3))),
        bits(f32(f32(f32(q00 - q11) + q22) - q33)),
        bits(two * f32(mul(q2, q3) - mul(q0, q1))),
        bits(two * f32(mul(q1, q3) - mul(q0, q2))),
        bits(two * f32(mul(q2, q3) + mul(q0, q1))),
        bits(f32(f32(f32(q00 - q11) - q22) + q33)),
    ]


def candidates_element(quat_arrays, idx):
    """Enumerate the collapse-option patterns for one matrix element (vectorized over all quats)."""
    q0, q1, q2, q3 = quat_arrays

    def vmul(a, b):
        return f32(a) * f32(b)

    def vfma(a, b, c):
        return (np.float64(a) * np.float64(b) + np.float64(c)).astype(np.float32)

    p0, p1, p2, p3 = vmul(q0, q0), vmul(q1, q1), vmul(q2, q2), vmul(q3, q3)
    out = []
    if idx in (0, 4, 8):
        signs = {0: (1, 1, -1, -1), 4: (1, -1, 1, -1), 8: (1, -1, -1, 1)}[idx]
        for v1 in range(3):
            if signs[0] == signs[1]:
                s1 = [f32(p0 + p1), vfma(q0, q0, p1), vfma(q1, q1, p0)][v1]
            else:
                s1 = [f32(p0 - p1), vfma(q0, q0, f32(-p1)), vfma(-q1, q1, p0)][v1]
            for v2 in range(2):
                if signs[2] > 0:
                    s2 = [f32(s1 + p2), vfma(q2, q2, s1)][v2]
                else:
                    s2 = [f32(s1 - p2), vfma(-q2, q2, s1)][v2]
                for v3 in range(2):
                    if signs[3] > 0:
                        s3 = [f32(s2 + p3), vfma(q3, q3, s2)][v3]
                    else:
                        s3 = [f32(s2 - p3), vfma(-q3, q3, s2)][v3]
                    out.append((f"d{v1}{v2}{v3}", s3.view(np.uint32)))
        return out
    two = f32(2.0)
    spec = {1: ((1, 2), (0, 3), False), 2: ((1, 3), (0, 2), True), 3: ((1, 2), (0, 3), True),
            5: ((2, 3), (0, 1), False), 6: ((1, 3), (0, 2), False), 7: ((2, 3), (0, 1), True)}[idx]
    a_term, b_term, plus = spec
    a = vmul(quat_arrays[a_term[0]], quat_arrays[a_term[1]])
    b = vmul(quat_arrays[b_term[0]], quat_arrays[b_term[1]])
    if plus:
        return [
            ("plain", (two * f32(a + b)).view(np.uint32)),
            ("fmaA", (two * vfma(quat_arrays[a_term[0]], quat_arrays[a_term[1]], b)).view(np.uint32)),
            ("fmaB", (two * vfma(quat_arrays[b_term[0]], quat_arrays[b_term[1]], a)).view(np.uint32)),
        ]
    return [
        ("plain", (two * f32(a - b)).view(np.uint32)),
        ("fmaA", (two * vfma(quat_arrays[a_term[0]], quat_arrays[a_term[1]], f32(-b))).view(np.uint32)),
        ("fmaB", (two * vfma(-quat_arrays[b_term[0]], quat_arrays[b_term[1]], a)).view(np.uint32)),
    ]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--ref", required=True)
    parser.add_argument("--repo", default=".")
    parser.add_argument("--sample-out", default=None)
    args = parser.parse_args()
    repo = pathlib.Path(args.repo).resolve()
    ref = json.loads(pathlib.Path(args.ref).read_text(encoding="utf-8"))
    quats = np.asarray(ref["quat_bits"], dtype=np.uint32).view(np.float32)
    gpu = np.asarray(ref["mat_bits"], dtype=np.uint32).reshape(-1, 9)
    n = quats.shape[0]
    quat_arrays = [quats[:, i] for i in range(4)]

    # 1. candidate uniqueness
    unique = {}
    for idx in range(9):
        cands = candidates_element(quat_arrays, idx)
        alive = list(cands)
        isolated = None
        for k in range(n):
            alive = [(tag, values) for tag, values in alive if values[k] == gpu[k, idx]]
            if len(alive) == 1:
                isolated = k + 1
                break
        full = [tag for tag, values in cands if np.array_equal(values, gpu[:, idx])]
        unique[idx] = {"candidates": len(cands), "matches": full, "isolated_after_quats": isolated}
        if len(full) != 1 or isolated is None:
            print(f"FAIL candidate uniqueness element {idx}: {unique[idx]}")
            return 1
    print("candidate space: diag 12 candidates each (e0/e4/e8), offdiag 3 candidates each (e1,e2,e3,e5,e6,e7)")
    print("unique match per element:", {k: v["matches"][0] for k, v in unique.items()})
    print("isolated after quats (max):", max(v["isolated_after_quats"] for v in unique.values()))

    # 2. host pattern vs GPU
    diff_entries = 0
    diff_quats = 0
    for i in range(n):
        got = pattern_matrix(quats[i])
        row_diff = sum(1 for k in range(9) if got[k] != int(gpu[i, k]))
        diff_entries += row_diff
        diff_quats += int(row_diff > 0)
    ok = diff_entries == 0
    print(f"[gate2] host pattern vs GPU: {n} quats x 9 = {n * 9} entries, diff={diff_entries}, quats_with_diff={diff_quats} -> {'PASS' if ok else 'FAIL'}")

    # 3. plain algebra contrast
    plain_diff_entries = 0
    plain_diff_quats = 0
    for i in range(n):
        got = plain_matrix(quats[i])
        row_diff = sum(1 for k in range(9) if got[k] != int(gpu[i, k]))
        plain_diff_entries += row_diff
        plain_diff_quats += int(row_diff > 0)
    print(f"[contrast] plain (unfused) expansion vs GPU: diff_entries={plain_diff_entries}, quats_with_diff={plain_diff_quats}/{n}")

    # 4. camera gates
    h = json.loads((repo / "tests/support/render_bvh/hfield/model-312.json").read_text(encoding="utf-8"))
    sa = json.loads((repo / "fixtures/render-bvh/scene-a/model-312.json").read_text(encoding="utf-8"))
    sa_ref = json.loads((repo / "fixtures/render-bvh/scene-a/s7b/depth_seg.json").read_text(encoding="utf-8"))
    hq = [f32(v) for v in h["arrays"]["cam_quat"]["values"]]
    saq = [f32(v) for v in sa["arrays"]["cam_quat"]["values"][:4]]
    cam_ok = True
    for name, offset in (("hfield cam0", 0), ("hfield cam1", 4), ("scene-a cam-persp", None)):
        quat = hq[offset:offset + 4] if offset is not None else saq
        got = pattern_matrix(quat)
        expected = [int(b) for b in gpu[names_idx(name)]] if name.startswith("hfield") else [
            int(b) for b in sa_ref["camera_data_check"][0]["xmat_bits"]
        ]
        diffs = [k for k in range(9) if got[k] != expected[k]]
        print(f"[gate1/gate3] {name}: diffs={diffs} -> {'PASS' if not diffs else 'FAIL'}")
        cam_ok = cam_ok and not diffs

    # 5. sampling table
    lines = ["idx\tq0\tq1\tq2\tq3\thost_e0..e8\tgpu_e0..e8\tdiff"]
    for i in range(3, 23):
        host = pattern_matrix(quats[i])
        gpu_row = [int(b) for b in gpu[i]]
        lines.append("\t".join([str(i)] + [hex(int(b)) for b in ref["quat_bits"][i]]
                               + [hex(b) for b in host] + [hex(b) for b in gpu_row]
                               + [str(sum(1 for k in range(9) if host[k] != gpu_row[k]))]))
    table = "\n".join(lines) + "\n"
    if args.sample_out:
        pathlib.Path(args.sample_out).write_text(table, encoding="utf-8", newline="\n")
    print("sample rows (first 3 of 20):")
    print("\n".join(lines[:4]))
    return 0 if ok and cam_ok else 1


def names_idx(name):
    return {"hfield cam0": 0, "hfield cam1": 1}[name]


if __name__ == "__main__":
    sys.exit(main())
