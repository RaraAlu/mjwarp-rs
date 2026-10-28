"""QA independent recompute for camera world-pose FMA contraction alignment.

Nothing here is imported from the implementation side. Exact IEEE-754 binary32
arithmetic is implemented from scratch on top of Fraction (round-to-nearest-even),
so every `fma` below is a true single-rounding fused multiply-add and every
`mul`/`add`/`sub` a true single-rounding binary32 op.

Inputs come from the committed git blobs (never the worktree):
  * materials/gpu-reference-digest.json  (pinned-GPU chain-kernel reference)
"""
import hashlib
import json
import pathlib
import subprocess
import sys
from fractions import Fraction

REPO = pathlib.Path(r"D:\Rust\mjwarp-rs")
EV = "docs/development-evidence/render-bvh/r05-s3b-camera-pose-warp-fma-align"


def blob(path):
    p = subprocess.run(["git", "show", f"HEAD:{path}"], cwd=REPO, capture_output=True)
    if p.returncode != 0:
        raise SystemExit(f"git show failed for {path}: {p.stderr[:200]!r}")
    return p.stdout


def blob_json(path):
    return json.loads(blob(path))


# ---------------------------------------------------------------- exact f32 core
def f32_round(fr):
    """Correctly rounded (RNE) binary32 -> 32-bit pattern of an exact Fraction."""
    if fr == 0:
        return 0x00000000
    sign = 0
    if fr < 0:
        sign = 0x80000000
        fr = -fr
    # exponent e with 2**e <= fr < 2**(e+1)
    e = fr.numerator.bit_length() - fr.denominator.bit_length()
    while Fraction(2) ** e > fr:
        e -= 1
    while Fraction(2) ** (e + 1) <= fr:
        e += 1

    def rne(x):
        n, d = x.numerator, x.denominator
        q, r = divmod(n, d)
        if 2 * r > d or (2 * r == d and q & 1):
            q += 1
        return q

    if e < -126:  # subnormal (or zero)
        m = rne(fr / Fraction(2) ** -149)
        if m == 0:
            return sign
        if m >= (1 << 23):  # rounds up to the smallest normal
            return sign | (1 << 23)  # e = -126 -> biased 1, mantissa 0
        return sign | m
    m = rne(fr / Fraction(2) ** (e - 23))
    if m == (1 << 24):
        m >>= 1
        e += 1
    if e > 127:
        return sign | 0x7F800000
    return sign | ((e + 127) << 23) | (m - (1 << 23))


def from_bits(bits):
    return Fraction(*_decode(bits))


def f32(bits):
    return from_bits(bits)


def _decode(bits):
    """binary32 pattern -> exact Fraction (mantissa/exponent form, no float detour)."""
    s = -1 if bits >> 31 else 1
    ex = (bits >> 23) & 0xFF
    mant = bits & 0x7FFFFF
    if ex == 0:
        return (s * mant, 1 << 149)          # subnormal: mant * 2**-149
    if ex == 0xFF:
        raise ValueError("inf/nan in input")
    m = mant | (1 << 23)
    k = ex - 150                             # value = m * 2**k
    if k >= 0:
        return (s * m * (1 << k), 1)
    return (s * m, 1 << (-k))


def mul(a, b):
    return from_bits(f32_round(a * b))


def add(a, b):
    return from_bits(f32_round(a + b))


def fma(a, b, c):
    return from_bits(f32_round(a * b + c))


def C(bits):
    return f32(bits)


def B(x):
    return x


# ------------------------------------------------------------------ formula sets
Z = None


def impl_formula(q):
    """Transcription of src/render/kernel/cast.rs::warp_quat_to_mat (read by hand)."""
    q0, q1, q2, q3 = q
    n = lambda x: -x  # exact negation of an f32 value
    q01, q02, q03 = mul(q0, q1), mul(q0, q2), mul(q0, q3)
    q11 = mul(q1, q1)
    e0 = fma(n(q3), q3, fma(n(q2), q2, fma(q0, q0, q11)))
    e4 = fma(n(q3), q3, fma(q2, q2, fma(q0, q0, n(q11))))
    e8 = fma(q3, q3, fma(n(q2), q2, fma(q0, q0, n(q11))))
    two = Fraction(2)
    return [
        e0,
        mul(two, fma(q1, q2, n(q03))),
        mul(two, fma(q1, q3, q02)),
        mul(two, fma(q1, q2, q03)),
        e4,
        mul(two, fma(q2, q3, n(q01))),
        mul(two, fma(q1, q3, n(q02))),
        mul(two, fma(q2, q3, q01)),
        e8,
    ]


def plain_formula(q):
    """Prior (unfused) algebraic expansion, PTX source order, no FMA."""
    q0, q1, q2, q3 = q
    q00, q11, q22, q33 = mul(q0, q0), mul(q1, q1), mul(q2, q2), mul(q3, q3)
    two = Fraction(2)
    return [
        add(add(add(q00, q11), -q22), -q33),
        mul(two, add(mul(q1, q2), -mul(q0, q3))),
        mul(two, add(mul(q1, q3), mul(q0, q2))),
        mul(two, add(mul(q1, q2), mul(q0, q3))),
        add(add(add(q00, -q11), q22), -q33),
        mul(two, add(mul(q2, q3), -mul(q0, q1))),
        mul(two, add(mul(q1, q3), -mul(q0, q2))),
        mul(two, add(mul(q2, q3), mul(q0, q1))),
        add(add(add(q00, -q11), -q22), q33),
    ]


# ------------------------------------------------------- independent enumeration
# Source order is fixed by the PTX excerpt (materials/upstream-camera-kernel-ptx-excerpt.txt,
# slice B lines 14734-14782): f106=q0*q0 f110=q1*q1 f113=q2*q2 f115=q3*q3
# f116=A+B f117=f116-C f118=f117-D ; e1=2*(H-G) e2=2*(I+F) e3=2*(H+G)
# e5=2*(J-E) e6=2*(I-F) e7=2*(E+J), with E=q0*q1 F=q0*q2 G=q0*q3 H=q1*q2 I=q1*q3 J=q2*q3.
SIGNS = {0: (1, 1, -1, -1), 4: (1, -1, 1, -1), 8: (1, -1, -1, 1)}
TERMS = [(0, 0), (1, 1), (2, 2), (3, 3)]
OFFDIAG = {
    1: ((1, 2), (0, 3), -1),  # h - g
    2: ((1, 3), (0, 2), +1),  # i + f
    3: ((1, 2), (0, 3), +1),  # h + g
    5: ((2, 3), (0, 1), -1),  # j - e
    6: ((1, 3), (0, 2), -1),  # i - f
    7: ((2, 3), (0, 1), +1),  # e + j
}


def diag_candidates(idx, q):
    """All fusion choices for one diagonal element, source order preserved."""
    signs = SIGNS[idx]
    q0, q1, q2, q3 = q
    p = [mul(q0, q0), mul(q1, q1), mul(q2, q2), mul(q3, q3)]
    qq = [q0, q1, q2, q3]
    out = []

    # first binary op: p[0]*s0 + p[1]*s1  (both same sign for e0; mixed for e4/e8)
    first = []
    a, b = qq[0], qq[1]
    va, vb = signs[0], signs[1]
    if va == vb:
        first.append(("v0", add(p[0], p[1])))
        first.append(("v1", fma(a, b * 0 + a, p[1])))       # fuse left product
        first.append(("v2", fma(b, b, p[0])))               # fuse right product
    else:
        first.append(("v0", add(p[0], -p[1])))
        first.append(("v1", fma(a, a, -p[1])))
        first.append(("v2", fma(-b, b, p[0])))

    def step(acc, vi, sign):
        s = qq[vi]
        pp = p[vi]
        if sign > 0:
            return [("0", add(acc, pp)), ("1", fma(s, s, acc))]
        return [("0", add(acc, -pp)), ("1", fma(-s, s, acc))]

    for t1, s1 in first:
        for t2, s2 in step(s1, 2, signs[2]):
            for t3, s3 in step(s2, 3, signs[3]):
                out.append((f"{t1}{t2}{t3}", s3))
    return out


def offdiag_candidates(idx, q):
    (a0, a1), (b0, b1), sgn = OFFDIAG[idx]
    q = list(q)
    pa, pb = mul(q[a0], q[a1]), mul(q[b0], q[b1])
    two = Fraction(2)
    if sgn > 0:
        return [
            ("plain", mul(two, add(pa, pb))),
            ("fuseA", mul(two, fma(q[a0], q[a1], pb))),
            ("fuseB", mul(two, fma(q[b0], q[b1], pa))),
        ]
    return [
        ("plain", mul(two, add(pa, -pb))),
        ("fuseA", mul(two, fma(q[a0], q[a1], -pb))),
        ("fuseB", mul(two, fma(-q[b0], q[b1], pa))),
    ]


def candidates(idx, q):
    return diag_candidates(idx, q) if idx in SIGNS else offdiag_candidates(idx, q)


def main():
    ref_raw = blob(f"{EV}/materials/gpu-reference-digest.json")
    ref = json.loads(ref_raw)
    quats = [[C(b) for b in row] for row in ref["quat_bits"]]
    gpu = ref["mat_bits"]
    n = len(quats)
    out = {"quats": n, "reference_sha256": hashlib.sha256(ref_raw).hexdigest()}

    # 0. provenance: digest == on-disk raw chain reference (bytes)
    rawp = REPO / "target/render-bvh/r05-s3b-window/ptx-order/chain-gpu-reference.json"
    regen = REPO / "target/render-bvh/r05-s3b-window/fix-local-logs/gpu-reference-regen.json"
    if rawp.exists():
        raw = json.loads(rawp.read_text(encoding="utf-8"))
        out["digest_semantically_equal_raw_reference"] = all(
            raw.get(k) == ref.get(k) for k in ("quat_bits", "mat_bits", "mul_quat_bits"))
        out["digest_bytes_equal_raw_reference"] = hashlib.sha256(rawp.read_bytes()).hexdigest() == out["reference_sha256"]
    if regen.exists():
        out["digest_bytes_equal_regen"] = hashlib.sha256(regen.read_bytes()).hexdigest() == out["reference_sha256"]

    # 1. mul_quat(identity, cam_quat) == cam_quat, machine count
    mq = ref["mul_quat_bits"]
    rows = sum(1 for i in range(n) if list(mq[i]) == list(ref["quat_bits"][i]))
    out["mul_quat_equals_quat_rows"] = rows

    # 2. per-element enumeration -> uniqueness + isolation distance
    per = {}
    for idx in range(9):
        # build candidate columns
        cols = {}
        for i, q in enumerate(quats):
            for tag, v in candidates(idx, q):
                cols.setdefault(tag, [None] * n)[i] = f32_round(v)
        full = [tag for tag in cols if cols[tag] == gpu_col(gpu, idx)]
        alive = list(cols)
        iso = None
        for k in range(n):
            alive = [t for t in alive if cols[t][k] == gpu[k][idx]]
            if len(alive) == 1:
                iso = k + 1
                break
        per[idx] = {
            "candidates": len(cols),
            "full_set_matches": full,
            "isolated_after_quats": iso,
        }
    out["elements"] = per
    out["candidate_space_global"] = 12 ** 3 * 3 ** 6

    # 3. implementation formula vs GPU: all 1003 x 9
    bad = 0
    badq = 0
    first_bad = []
    for i, q in enumerate(quats):
        got = [f32_round(v) for v in impl_formula(q)]
        d = [k for k in range(9) if got[k] != int(gpu[i][k])]
        bad += len(d)
        badq += int(bool(d))
        if d and len(first_bad) < 5:
            first_bad.append({"row": i, "elems": d,
                              "got": [hex(x) for x in got], "gpu": [hex(int(x)) for x in gpu[i]]})
    out["impl_vs_gpu"] = {"entries": n * 9, "diff_entries": bad, "diff_rows": badq,
                          "first_mismatches": first_bad}

    # 4. plain (unfused) formula contrast
    pbad = 0
    pbadq = 0
    for i, q in enumerate(quats):
        got = [f32_round(v) for v in plain_formula(q)]
        d = sum(1 for k in range(9) if got[k] != int(gpu[i][k]))
        pbad += d
        pbadq += int(d > 0)
    out["plain_vs_gpu"] = {"diff_entries": pbad, "diff_rows": pbadq}

    # 5. camera rows cross-checked against fixtures / verified references
    hf = json.loads((REPO / "tests/support/render_bvh/hfield/model-312.json").read_text(encoding="utf-8"))
    hq = [float(v) for v in hf["arrays"]["cam_quat"]["values"]]
    sa = json.loads((REPO / "fixtures/render-bvh/scene-a/model-312.json").read_text(encoding="utf-8"))
    saq = [float(v) for v in sa["arrays"]["cam_quat"]["values"][:4]]
    sa_ref = json.loads((REPO / "fixtures/render-bvh/scene-a/s7b/depth_seg.json").read_text(encoding="utf-8"))
    import struct
    hb = [[struct.unpack("<I", struct.pack("<f", hq[o + j]))[0] for j in range(4)] for o in (0, 4)]
    sab = [struct.unpack("<I", struct.pack("<f", v))[0] for v in saq]
    out["fixture_quat_rows"] = {"hfield_cam0": [hex(x) for x in hb[0]],
                                "hfield_cam1": [hex(x) for x in hb[1]],
                                "scenea_cam0": [hex(x) for x in sab]}
    out["digest_row_matches_fixture_quat"] = {
        "hfield_cam0": list(ref["quat_bits"][0]) == hb[0],
        "hfield_cam1": list(ref["quat_bits"][1]) == hb[1],
        "scenea_cam0": list(ref["quat_bits"][2]) == sab,
    }
    cam = {}
    for name, qb, expect in (
        ("hfield_cam0", hb[0], None),
        ("hfield_cam1", hb[1], None),
    ):
        got = [f32_round(v) for v in impl_formula([C(b) for b in qb])]
        cam[name] = {"impl_bits": [hex(x) for x in got],
                     "gpu_chain_bits": [hex(int(x)) for x in (ref["mat_bits"][0 if name.endswith("cam0") else 1])],
                     "diffs": [k for k in range(9) if got[k] != int(ref["mat_bits"][0 if name.endswith("cam0") else 1][k])]}
    got = [f32_round(v) for v in impl_formula([C(b) for b in sab])]
    exp = [int(b) for b in sa_ref["camera_data_check"][0]["xmat_bits"]]
    cam["scenea_campersp"] = {"impl_bits": [hex(x) for x in got],
                              "verified_reference_bits": [hex(x) for x in exp],
                              "diffs": [k for k in range(9) if got[k] != exp[k]],
                              "digest_row2_diffs": [k for k in range(9) if int(ref["mat_bits"][2][k]) != exp[k]]}
    out["camera_gates"] = cam

    # 6. sampling: 20 random rows, own bits
    samp = []
    for i in range(3, 23):
        got = [f32_round(v) for v in impl_formula(quats[i])]
        samp.append({"idx": i, "diff": sum(1 for k in range(9) if got[k] != int(gpu[i][k])),
                     "q": [hex(int(x)) for x in ref["quat_bits"][i]],
                     "got": [hex(x) for x in got], "gpu": [hex(int(x)) for x in gpu[i]]})
    out["sampling"] = samp
    out["sampling_zero_diff"] = all(s["diff"] == 0 for s in samp)

    # 7. PTX basis: pinned sm89.ptx, slice A (mul_quat) vs slice B (quat_to_mat)
    ptx_path = REPO / "target/render-bvh/r05-s3a-hfield-cache/1.15.0/wp_mujoco_warp._src.smooth_ca013d6/wp_mujoco_warp._src.smooth_ca013d6.sm89.ptx"
    ptx = {}
    if ptx_path.exists():
        raw = ptx_path.read_bytes()
        ptx["bytes"] = len(raw)
        ptx["sha256"] = hashlib.sha256(raw).hexdigest()
        lines = raw.decode("utf-8", errors="replace").splitlines()
        ptx["lines"] = len(lines)
        ptx["entry_line_hits"] = [i + 1 for i, l in enumerate(lines) if "_cam_local_to_global" in l][:6]
        slice_a = lines[14718:14733]   # 14719..14733
        slice_b = lines[14733:14782]   # 14734..14782
        ptx["slice_a_fma_rn_count"] = sum(1 for l in slice_a if "fma.rn" in l)
        ptx["slice_b_fma_rn_count"] = sum(1 for l in slice_b if "fma.rn" in l)
        ptx["slice_b_mul_add_sub_count"] = sum(
            1 for l in slice_b if any(o in l for o in ("mul.f32", "add.f32", "sub.f32")))
        ptx["slice_a_first"] = slice_a[0].strip() if slice_a else None
        ptx["slice_a_last"] = slice_a[-1].strip() if slice_a else None
        ptx["slice_b_first"] = slice_b[0].strip() if slice_b else None
        ptx["slice_b_last"] = slice_b[-1].strip() if slice_b else None
        ptx["line_14783_blank"] = lines[14782].strip() == ""
        # the committed excerpt must be verbatim from this file (instruction lines only)
        ev = blob(f"{EV}/materials/upstream-camera-kernel-ptx-excerpt.txt").decode("utf-8")
        ev_lines = [l for l in ev.splitlines() if l.strip() and not l.strip().startswith("#")]
        ev_set = {l.strip() for l in ev_lines}
        fset = {x.strip() for x in lines}
        ptx["excerpt_instruction_lines"] = len(ev_lines)
        ptx["excerpt_lines_not_in_ptx"] = [l for l in ev_lines if l.strip() not in fset]
        ptx["excerpt_slice_a_verbatim"] = all(l.strip() in fset and l.strip() in ev_set for l in slice_a)
        ptx["excerpt_slice_b_verbatim"] = all(l.strip() in fset and l.strip() in ev_set for l in slice_b)
    out["ptx_basis"] = ptx

    # 8. hard gate 1 provenance: digest rows 0/1 vs the pinned-GPU capture d.cam_xmat bits
    cw = REPO / "docs/development-evidence/render-bvh/r05-s3b-hfield-cam1-depth-rca/materials/upstream-camworld.json"
    cap = {}
    if cw.exists():
        c = json.loads(cw.read_text(encoding="utf-8"))
        warp = c["warp"]["xmat_bits"]
        cap["capture_cam0"] = [hex(x) for x in warp[:9]]
        cap["capture_cam1"] = [hex(x) for x in warp[9:18]]
        cap["digest_row0_equals_capture_cam0"] = [int(x) for x in ref["mat_bits"][0]] == warp[:9]
        cap["digest_row1_equals_capture_cam1"] = [int(x) for x in ref["mat_bits"][1]] == warp[9:18]
        cap["impl_equals_capture_cam0"] = [f32_round(v) for v in impl_formula(quats[0])] == warp[:9]
        cap["impl_equals_capture_cam1"] = [f32_round(v) for v in impl_formula(quats[1])] == warp[9:18]
        mjd = c["mjd"]["xmat_bits"]
        cap["mujoco_host_cpu_cam1"] = [hex(x) for x in mjd[9:18]]
        cap["mujoco_host_cpu_differs_from_capture_cam1"] = mjd[9:18] != warp[9:18]
    out["hard_gate1_capture_provenance"] = cap

    # 9. evidence manifest: sha256+bytes vs committed blob, no self reference
    base = f"{EV}/"
    man = blob(base + "manifest.txt").decode("utf-8")
    rows = [l for l in man.splitlines() if l and not l.startswith("#")]
    bad_rows = []
    for l in rows:
        sha, size, *rest = l.split()
        rel = rest[-1]
        b = subprocess.run(["git", "show", f"HEAD:{base}{rel}"], cwd=REPO, capture_output=True).stdout
        if hashlib.sha256(b).hexdigest() != sha or len(b) != int(size):
            bad_rows.append(rel)
    listing = subprocess.run(["git", "ls-tree", "-r", "--name-only", "HEAD", base.rstrip("/")],
                             cwd=REPO, capture_output=True).stdout.decode().split()
    rel_files = [f[len(base):] for f in listing]
    listed = {l.split()[-1] for l in rows}
    eol = {}
    for rel in rel_files:
        b = subprocess.run(["git", "show", f"HEAD:{base}{rel}"], cwd=REPO, capture_output=True).stdout
        try:
            b.decode("utf-8")
            u8 = True
        except UnicodeDecodeError:
            u8 = False
        if b.count(b"\x00") or b.count(b"\r") or not u8 or b[:2] in (b"\xff\xfe", b"\xfe\xff"):
            eol[rel] = {"nul": b.count(b"\x00"), "cr": b.count(b"\r"), "utf8": u8}
    out["evidence_manifest_impl"] = {
        "rows": len(rows), "files_in_dir": len(rel_files),
        "mismatched_rows": bad_rows,
        "unlisted_files": [r for r in rel_files if r not in listed and r != "manifest.txt"],
        "self_reference": "manifest.txt" in " ".join(l.split()[-1] for l in rows),
        "non_lf_or_non_utf8_files": eol,
    }

    # 10. static caller map of the changed public path
    callers = {}
    gr = subprocess.run(["git", "grep", "-l", "CameraPose::from_fields", "--", "*.rs"],
                        cwd=REPO, capture_output=True).stdout.decode().split()
    for f in gr:
        hits = subprocess.run(["git", "grep", "-n", "CameraPose::from_fields", "--", f],
                              cwd=REPO, capture_output=True).stdout.decode().strip().splitlines()
        callers[f] = hits
    out["from_fields_static_callers"] = callers

    # 11. self-check: list of failed expectations (must be empty)
    problems = []
    if out["impl_vs_gpu"]["diff_entries"] != 0:
        problems.append("impl vs gpu diff")
    if (out["plain_vs_gpu"]["diff_entries"], out["plain_vs_gpu"]["diff_rows"]) != (2614, 947):
        problems.append("plain contrast does not reproduce 2614/947")
    if out["mul_quat_equals_quat_rows"] != n:
        problems.append("mul_quat identity rows")
    for idx, v in per.items():
        if len(v["full_set_matches"]) != 1 or v["isolated_after_quats"] is None:
            problems.append(f"contraction uniqueness e{idx}")
    for k, v in cam.items():
        if v["diffs"]:
            problems.append(f"camera gate {k}")
    if cam["scenea_campersp"]["digest_row2_diffs"]:
        problems.append("digest row2 vs verified reference")
    if not out["sampling_zero_diff"]:
        problems.append("sampling rows")
    if not all(v for k, v in out["hard_gate1_capture_provenance"].items() if isinstance(v, bool) and k.startswith("impl_")):
        problems.append("capture provenance (impl vs d.cam_xmat)")
    pb = out["ptx_basis"]
    if pb.get("slice_b_fma_rn_count") != 0 or not pb.get("slice_a_fma_rn_count") or not pb.get("excerpt_slice_a_verbatim") or not pb.get("excerpt_slice_b_verbatim"):
        problems.append("ptx basis")
    mm = out["evidence_manifest_impl"]
    if mm["mismatched_rows"] or mm["unlisted_files"] or mm["self_reference"]:
        problems.append("evidence manifest")
    out["problems"] = problems

    pathlib.Path(r"D:\Rust\mjwarp-rs\target\qa-cam-fma\recompute.json").write_text(
        json.dumps(out, ensure_ascii=False, indent=1), encoding="utf-8", newline="\n")
    print("quats", n)
    print("digest sha", out["reference_sha256"])
    print("digest bytes==regen", out.get("digest_bytes_equal_regen"), "semantic==raw", out.get("digest_semantically_equal_raw_reference"), "bytes==raw", out.get("digest_bytes_equal_raw_reference"))
    print("mul_quat==quat rows", rows)
    for k, v in per.items():
        print(f"e{k}: cand={v['candidates']} matches={v['full_set_matches']} iso={v['isolated_after_quats']}")
    print("candidate space", out["candidate_space_global"])
    print("impl vs gpu", out["impl_vs_gpu"]["diff_entries"], "entries /", out["impl_vs_gpu"]["diff_rows"], "rows")
    if first_bad:
        print("FIRST MISMATCH", json.dumps(first_bad[0])[:400])
    print("plain vs gpu", out["plain_vs_gpu"])
    print("row quat match", out["digest_row_matches_fixture_quat"])
    for k, v in cam.items():
        print(k, "diffs", v["diffs"], (v.get("digest_row2_diffs")))
    print("sampling zero diff", out["sampling_zero_diff"])
    print("ptx", {k: out["ptx_basis"].get(k) for k in ("bytes","lines","slice_a_fma_rn_count","slice_b_fma_rn_count","slice_b_mul_add_sub_count","entry_line_hits")})
    print("excerpt not in ptx", out["ptx_basis"].get("excerpt_lines_not_in_ptx"))
    print("capture provenance", {k: v for k, v in out["hard_gate1_capture_provenance"].items() if isinstance(v,bool)})
    print("manifest", out["evidence_manifest_impl"])
    print("callers", list(out["from_fields_static_callers"]))
    print("PROBLEMS", out["problems"])
    return 0


def gpu_col(gpu, idx):
    return [int(row[idx]) for row in gpu]


if __name__ == "__main__":
    sys.exit(main())
