#!/usr/bin/env python3
"""R05-S3b hfield target-entity authoring 独立复核重算脚本（check 卡口径）。"""

import hashlib
import json
import pathlib
import re
import struct
import subprocess
import sys
import zipfile

try:
    sys.stdout.reconfigure(encoding="utf-8")
except Exception:
    pass

R = pathlib.Path(
    subprocess.run(["git", "rev-parse", "--show-toplevel"],
                   cwd=pathlib.Path(__file__).resolve().parent,
                   capture_output=True, text=True, check=True).stdout.strip()
)
EV = "docs/development-evidence/render-bvh/r05-s3b-hfield-target-authoring"
CHK = "docs/development-evidence/render-bvh/r05-s3b-hfield-target-authoring-check"
REVIEWED = ["2c56cc1a", "d77b728e", "a6bcca80"]
BASE_HEAD = "a6bcca80"
HEAD = subprocess.run(["git", "rev-parse", "HEAD"], cwd=R, capture_output=True, text=True, check=True).stdout.strip()
GLOBAL_WORDS = [
    (71, 49, 48, 24050, 23454, 29616), (23436, 25972, 29289, 29702, 36890, 36807),
    (21452, 24179, 21488, 36890, 36807), (19978, 28216, 23436, 20840, 31561, 20215),
    (21407, 29983, 25918, 34892), (31934, 24230, 36798, 26631),
    (23436, 25972, 28210, 26524, 36890, 36807), (23436, 25972, 82, 48, 49, 36890, 36807),
    (20135, 21697, 28210, 26524, 39564, 25910, 36890, 36807),
]
CARD_WORDS = [
    (31383, 21475, 36890, 36807), (31934, 24230, 36798, 26631),
    (19978, 28216, 31561, 20215), (20135, 21697, 28210, 26524, 39564, 25910, 36890, 36807),
]
BANNED = sorted({"".join(chr(c) for c in seq) for seq in GLOBAL_WORDS + CARD_WORDS})

problems = []
findings = []


def blob(rev, rel):
    return subprocess.run(["git", "show", f"{rev}:{rel}"], cwd=R, capture_output=True, check=True).stdout


def head(rel):
    return blob("HEAD", rel)


def sha(b):
    return hashlib.sha256(b).hexdigest()


def lf(b):
    return b.count(b"\n")


def text_lines(b):
    return len(b.decode("utf-8").split("\n"))


def check(cond, msg):
    if not cond:
        problems.append(msg)


rep = {"head": HEAD, "reviewed_commits": REVIEWED}

# 0. HEAD 漂移
drift = [p for p in subprocess.run(["git", "diff", "--name-only", f"{BASE_HEAD}..HEAD"],
                                   cwd=R, capture_output=True, text=True, check=True).stdout.split("\n") if p.strip()]
reviewed_prefixes = ("tests/render_hfield_reference.rs", "tests/support/render_bvh/hfield",
                     "tests/support/render_bvh/hfield_scene.rs", "tests/support/render_bvh/shadow.rs",
                     EV + "/")
touched_reviewed = [p for p in drift if p.startswith(reviewed_prefixes)]
rep["head_drift"] = {
    "commits_since_base": subprocess.run(["git", "log", "--oneline", f"{BASE_HEAD}..HEAD"], cwd=R,
                                         capture_output=True).stdout.decode("utf-8", "replace").strip().split("\n"),
    "touched_reviewed": touched_reviewed}
check(not touched_reviewed, f"HEAD drift touched reviewed paths: {touched_reviewed}")

# 1. 行数 / 字节 / sha
FILES = {
    "tests/render_hfield_reference.rs": {"bytes": 14568, "sha": "57dc06252db831be2297748afc45be00b9326942a641dc33ec04c2f677278620", "text_lines": 393},
    "tests/support/render_bvh/hfield.rs": {"bytes": 12879, "sha": "961817f1a495b29c3fd048e594ee3598b5aa884c748f950704a8a18ef3a3da7e", "text_lines": 355},
    "tests/support/render_bvh/hfield_scene.rs": {"bytes": 17389, "sha": "a935d039f953b49a94a742b20151b0d62d03c17cf0924ce229638394daf0ff44", "text_lines": 500},
    "tests/support/render_bvh/hfield/model-312.json": {"bytes": 52527, "sha": "bc53747492121d76580d0f19b2062ff40676443292d8907c7964f115798ac321", "text_lines": 145},
    "tests/support/render_bvh/hfield/model-mixed-312.json": {"bytes": 53207, "sha": "c9a886ae8714b5a977a6bcb570960396b4cad2aa9e4c947dc26eef7e1ecac801", "text_lines": 145},
    "tests/support/render_bvh/shadow.rs": {"bytes": 16593, "sha": "c47aa4b05b6976a4ff1015be1d4dffd787a1affb4f40b8dafb02aff335a2d2d9", "text_lines": 423},
}
rep["files"] = {}
for rel, want in FILES.items():
    b = head(rel)
    info = {"bytes": len(b), "lf_lines": lf(b), "text_lines": text_lines(b), "sha256": sha(b),
            "bytes_ok": len(b) == want["bytes"], "sha_ok": sha(b) == want["sha"],
            "text_lines_ok": text_lines(b) == want["text_lines"],
            "lf_plus_one_equals_text": lf(b) + 1 == text_lines(b),
            "le_500_lf": lf(b) <= 500}
    rep["files"][rel] = info
    check(info["bytes_ok"] and info["sha_ok"] and info["text_lines_ok"], f"file claim mismatch: {rel}")
    check(info["le_500_lf"], f"LF line count > 500: {rel} {lf(b)}")
rep["line_count_convention"] = {
    "authoritative": "committed blob 的 LF（0x0A）计数（= git numstat / wc -l / 仓库证据惯例）",
    "lf_counts": {rel: info["lf_lines"] for rel, info in rep["files"].items()},
    "text_counts_in_authoring_docs": {rel: info["text_lines"] for rel, info in rep["files"].items()},
}
check(rep["files"]["tests/support/render_bvh/hfield_scene.rs"]["lf_lines"] == 499,
      "hfield_scene.rs LF count != 499")

# 2. 断言强度（与 R09-S3b 同级）
hscene = head("tests/support/render_bvh/hfield_scene.rs").decode()
sscene = head("tests/support/render_bvh/shadow_scene.rs").decode()
test = head("tests/render_hfield_reference.rs").decode()


def extract_fn(text, name):
    start = text.index(f"pub fn {name}(")
    depth = 0
    i = text.index("{", start)
    j = i
    while j < len(text):
        if text[j] == "{":
            depth += 1
        elif text[j] == "}":
            depth -= 1
            if depth == 0:
                return text[start:j + 1]
        j += 1
    raise ValueError(name)


body_h = extract_fn(hscene, "assert_group_criteria")
body_s = extract_fn(sscene, "assert_group_criteria")
normalize = lambda s: re.sub(r"\s+", " ", s.replace("HfieldReference", "ShadowReference")).strip()
rep["criteria_equivalence"] = {
    "identical_to_r09_s3b_after_type_normalization": normalize(body_h) == normalize(body_s),
    "assert_count": len(re.findall(r"assert", body_h)),
    "checks": {
        "rgb_beyond_one_lsb_zero": "beyond_one_lsb_pixels" in body_h,
        "depth_mismatched_zero": "diff.mismatched, 0" in body_h,
        "depth_max_ulp": "max_ulp <= MAX_DEPTH_ULP" in body_h,
        "depth_beyond_ulp": "diff.beyond_ulp, 0" in body_h,
        "seg_exact": "seg 必须逐像素全等" in body_h,
    },
}
check(rep["criteria_equivalence"]["identical_to_r09_s3b_after_type_normalization"],
      "assert_group_criteria differs from R09-S3b version")
for k, v in rep["criteria_equivalence"]["checks"].items():
    check(v, f"criteria check missing: {k}")
output_src = head("src/render/kernel/output.rs").decode()
mulp = re.search(r"MAX_DEPTH_ULP[^=]*=\s*(\d+)", output_src)
rep["criteria_equivalence"]["max_depth_ulp_source"] = mulp.group(1) if mulp else None
check(rep["criteria_equivalence"]["max_depth_ulp_source"] == "4", "MAX_DEPTH_ULP != 4")

rep["six_groups_and_determinism"] = {
    "ignored_tests": re.findall(r'#\[ignore = "([^"]+)"\]', test),
    "two_ignored": len(re.findall(r"^#\[ignore", test, re.M)) == 2,
    "run_group_calls_compare": "hfield_scene::compare_group(" in test,
    "run_group_calls_assert": "hfield_scene::assert_group_criteria(" in test,
    "two_run_sha_asserts": test.count("hfield_scene::hash_rgb") >= 2 and "hash_depth(" in test and "hash_seg(" in test,
    "prints_compare_scale_and_stats": all(s in test + hscene for s in (
        "compared={}", "mismatched={}", "max_byte_diff={}", "beyond_1_lsb={}", "max_ulp={}", "beyond_ulp={}")),
    "no_print_only_ignored_test": all(
        "run_group(" in block for block in re.findall(r"#\[ignore[^\n]*\]\nfn \w+\(\) \{(.*?)\n\}", test, re.S)),
    "host_asserts_cameras_2": "cameras(), 2" in test,
    "host_asserts_total_pixels": "120_000" in test,
}
for k, v in rep["six_groups_and_determinism"].items():
    if k == "ignored_tests":
        continue
    check(bool(v), f"test strength check failed: {k}")

# 3. O5
rep["o5"] = {
    "half_from_build_hfield_mesh": "let half = mesh.half();" in hscene,
    "injection_via_set_hfield": "resolved.set_hfield(&fixture.inputs())" in hscene,
    "no_host_recompute": "half_extents" not in hscene and "fn host_half" not in hscene,
    "hfield_leaf_uses_half_var": "(half, [0.0, 0.0, 0.0])" in hscene,
    "hardcoded_half_in_scene_leaf": bool(re.search(r"\(\s*\[1\.5,\s*1\.0,\s*0\.125\]\s*,\s*\[0\.0",
                                                  hscene)),
    "test_asserts_half_value": "mesh.half()," in test,
}
for k, v in rep["o5"].items():
    if k == "hardcoded_half_in_scene_leaf":
        check(not v, "hardcoded half in scene leaf")
    else:
        check(v, f"O5 check failed: {k}")

# 4. host 自检独立复算
reverse = head("docs/development-evidence/render-bvh/r05-s3a-hfield-capture/materials/reverse-calc.txt")
rep["host_self_check"] = {
    "reverse_calc_sha256": sha(reverse),
    "reverse_calc_sha_ok": sha(reverse) == "3f404550bbc6acd5d3aaf8720dfa2490699e62c1fd865e2c619a816cbadc7ad5",
}
check(rep["host_self_check"]["reverse_calc_sha_ok"], "reverse-calc.txt sha mismatch")
m = re.search(r"\[1\] mjm\.hfield_data raw flat order.*?\n(\[[^\n]+\])", reverse.decode())
bits_txt = [int(x, 16) for x in re.findall(r"0x([0-9a-fA-F]+)", m.group(1))]
const = re.search(r"HFIELD_DATA_BITS: \[u32; 16\] = \[(.*?)\];", hscene, re.S).group(1)
bits_const = [int(x.replace("_", ""), 16) for x in re.findall(r"0x([0-9a-fA-F_]+)", const)]
rep["host_self_check"]["bits_const_equal"] = bits_txt == bits_const
check(bits_txt == bits_const, "HFIELD_DATA_BITS != reverse-calc [1]")
size_m = re.search(r"HFIELD_SIZE: \[f32; 4\] = \[([^\]]+)\]", hscene)
rep["host_self_check"]["hfield_size_const"] = size_m.group(1).strip()
scene_xml = head("fixtures/render-bvh/scene-hfield/scene.xml").decode()
rep["host_self_check"]["scene_xml_hfield_size"] = re.search(r'<hfield[^>]*size="([^"]+)"', scene_xml).group(1)
check(rep["host_self_check"]["scene_xml_hfield_size"] == "1.5 1.0 0.25 0.1", "scene.xml hfield size mismatch")

F32 = lambda v: struct.unpack("<f", struct.pack("<f", v))[0]


def upstream_tri(nr, nc, sx, sy, sz, data):
    width = 0.5 * max(nc - 1, 1)
    height = 0.5 * max(nr - 1, 1)
    points, pmap, indices = [], {}, []

    def gpi(r, c):
        if (r, c) in pmap:
            return pmap[(r, c)]
        x = F32(sx * F32(float(c) / width - 1.0))
        y = F32(sy * F32(float(r) / height - 1.0))
        z = F32(data[r * nc + c] * sz)
        idx = len(points)
        points.append((x, y, z))
        pmap[(r, c)] = idx
        return idx

    visited = [[False] * (nc - 1) for _ in range(nr - 1)]
    THR = F32(1e-5)
    for r in range(nr - 1):
        for c in range(nc - 1):
            if visited[r][c]:
                continue
            z00, z01 = F32(data[r * nc + c]), F32(data[r * nc + c + 1])
            z10, z11 = F32(data[(r + 1) * nc + c]), F32(data[(r + 1) * nc + c + 1])
            if abs(F32(F32(z00 + z11) - F32(z01 + z10))) >= THR:
                i00, i01 = gpi(r, c), gpi(r, c + 1)
                i10, i11 = gpi(r + 1, c), gpi(r + 1, c + 1)
                indices += [i00, i01, i11, i00, i11, i10]
                visited[r][c] = True
                continue
            slope_x, slope_y = F32(z01 - z00), F32(z10 - z00)
            w = h = 1

            def fits(rr, cc):
                if rr >= nr - 1 or cc >= nc - 1:
                    return False
                cz00, cz01 = F32(data[rr * nc + cc]), F32(data[rr * nc + cc + 1])
                cz10, cz11 = F32(data[(rr + 1) * nc + cc]), F32(data[(rr + 1) * nc + cc + 1])
                if abs(F32(F32(cz00 + cz11) - F32(cz01 + cz10))) >= THR:
                    return False
                zp = F32(z00 + F32(rr - r) * slope_y)
                zp = F32(zp + F32(cc - c) * slope_x)
                if abs(F32(cz00 - zp)) >= THR:
                    return False
                if abs(F32(F32(cz01 - cz00) - slope_x)) >= THR or abs(F32(F32(cz10 - cz00) - slope_y)) >= THR:
                    return False
                return True

            while c + w < nc - 1 and not visited[r][c + w] and fits(r, c + w):
                w += 1
            while r + h < nr - 1 and all(not visited[r + h][c + k] and fits(r + h, c + k) for k in range(w)):
                h += 1
            for rr in range(r, r + h):
                for cc in range(c, c + w):
                    visited[rr][cc] = True
            itl, itr = gpi(r, c), gpi(r, c + w)
            ibl, ibr = gpi(r + h, c), gpi(r + h, c + w)
            indices += [itl, itr, ibr, itl, ibr, ibl]
    xs = [p[0] for p in points]
    ys = [p[1] for p in points]
    zs = [p[2] for p in points]
    half = [F32(0.5 * F32(max(v) - min(v))) for v in (xs, ys, zs)]
    return points, indices, half


data = [struct.unpack("<f", struct.pack("<I", b))[0] for b in bits_txt]
pts, idxs, half = upstream_tri(4, 4, F32(1.5), F32(1.0), F32(0.25), data)
rep["host_self_check"]["upstream_replica"] = {
    "vertices": len(pts), "indices": len(idxs), "triangles": len(idxs) // 3,
    "half_bits": [f"0x{struct.unpack('<I', struct.pack('<f', h))[0]:08x}" for h in half],
}
check((len(pts), len(idxs), len(idxs) // 3) == (13, 36, 12), "upstream replica counts != 13/36/12")
check([struct.unpack("<I", struct.pack("<f", h))[0] for h in half] == [0x3FC00000, 0x3F800000, 0x3E000000],
      "upstream replica half bits mismatch")

probe = R / "target/qa-s3b-auth-chk/probe.log"
if probe.exists():
    t = probe.read_text(encoding="utf-8", errors="replace")
    vm = re.search(r"^V (\d+)", t, re.M)
    im = re.search(r"^I (\d+)", t, re.M)
    tm = re.search(r"^T (\d+)", t, re.M)
    hm = re.search(r"^HALF ([0-9a-f]{8}) ([0-9a-f]{8}) ([0-9a-f]{8})", t, re.M)
    rep["host_self_check"]["qa_probe"] = {
        "V": int(vm.group(1)) if vm else None, "I": int(im.group(1)) if im else None,
        "T": int(tm.group(1)) if tm else None, "half_bits": list(hm.groups()) if hm else None}
    check((rep["host_self_check"]["qa_probe"]["V"], rep["host_self_check"]["qa_probe"]["I"],
           rep["host_self_check"]["qa_probe"]["T"]) == (13, 36, 12), "QA probe counts != 13/36/12")
    check(rep["host_self_check"]["qa_probe"]["half_bits"] == ["3fc00000", "3f800000", "3e000000"],
          "QA probe half bits mismatch")

# 5. dump provenance
dumper = R / "target/render-bvh/s0a/compile_scene_312.exe"
rep["model_dump"] = {
    "dumper_exists": dumper.exists(),
    "dumper_sha_bytes": (sha(dumper.read_bytes()), len(dumper.read_bytes())) if dumper.exists() else None,
    "commands_in_readme": "compile_scene_312.exe fixtures/render-bvh/scene-hfield/scene.xml"
    in head(f"{EV}/README.md").decode(),
    "inputs_signed_sha_in_readme": ["b3b9a6b4" in head(f"{EV}/README.md").decode(),
                                    "bf9bb966" in head(f"{EV}/README.md").decode()],
}
check(rep["model_dump"]["dumper_exists"] and rep["model_dump"]["dumper_sha_bytes"][0]
      == "cd91e099659ebda3f51a84d5392f7d726c0c61378bfa2487215ddd00b2de2970"
      and rep["model_dump"]["dumper_sha_bytes"][1] == 161280, "dumper provenance mismatch")
for group, rel in (("hfield-only", "tests/support/render_bvh/hfield/model-312.json"),
                   ("hfield-primitive", "tests/support/render_bvh/hfield/model-mixed-312.json")):
    model = json.loads(head(rel))
    arrays = model["arrays"]
    rep["model_dump"][group] = {
        "version": model.get("mujoco_version"),
        "geoms": len(arrays["geom_type"]["values"]),
        "geom_type": arrays["geom_type"]["values"],
        "cams": len(arrays["cam_mode"]["values"]),
        "lights": len(arrays["light_type"]["values"]),
        "tex_bytes": len(arrays["tex_data"]["values"]),
    }
    check(model.get("mujoco_version") == "3.12.0", f"{group}: version != 3.12.0")
    check(rep["model_dump"][group]["cams"] == 2 and rep["model_dump"][group]["lights"] == 2
          and rep["model_dump"][group]["tex_bytes"] == 64 * 64 * 3, f"{group}: counts mismatch")
check(rep["model_dump"]["hfield-only"]["geoms"] == 1 and rep["model_dump"]["hfield-primitive"]["geoms"] == 2,
      "geom counts mismatch")
check(rep["model_dump"]["hfield-primitive"]["geom_type"] == [1, 6], "mixed geom_type != [1,6]")

# 6. shadow.rs 改动面
diff = subprocess.run(["git", "diff", "2c56cc1a~1..2c56cc1a", "--", "tests/support/render_bvh/shadow.rs"],
                      cwd=R, capture_output=True, check=True).stdout.decode()
removed = [l[1:] for l in diff.split("\n") if l.startswith("-") and not l.startswith("---")]
added = [l[1:] for l in diff.split("\n") if l.startswith("+") and not l.startswith("+++")]
rep["shadow_rs"] = {
    "numstat_added_removed": (len(added), len(removed)),
    "removed_all_fn_signatures": all(l.startswith("fn ") for l in removed),
    "added_visibility_or_continuation": all(l.startswith("pub(super) fn ") or l.startswith("    ") or l.startswith(")")
                                            for l in added),
    "assert_lines_changed": [l for l in removed + added if "assert" in l],
    "readme_claims_plus8_minus8": "+8/−8" in head(f"{EV}/README.md").decode()
    or "+8/-8" in head(f"{EV}/README.md").decode(),
}
check(rep["shadow_rs"]["removed_all_fn_signatures"], "shadow.rs removed non-signature lines")
check(rep["shadow_rs"]["added_visibility_or_continuation"], "shadow.rs added unexpected lines")
check(not rep["shadow_rs"]["assert_lines_changed"], "shadow.rs changed assertion lines")

# 7. 静态门
gates = {}
for name, marker in (("fmt", "FMT_EXIT"), ("clippy", "CLIPPY_EXIT"), ("no-run", "NORUN_EXIT"),
                     ("targeted", "TARGETED_EXIT")):
    p = R / "target/qa-s3b-auth-chk/clean" / f"{name}.log"
    if not p.exists():
        gates[name] = "missing"
        continue
    t = p.read_text(encoding="utf-8-sig", errors="replace")
    em = re.search(rf"{marker}=(\d+)", t)
    gates[name] = {"exit": em.group(1) if em else None}
    check(bool(em) and em.group(1) == "0", f"gate {name} exit != 0")
    if name == "targeted":
        rm = re.search(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored", t)
        gates[name].update({"passed": int(rm.group(1)), "failed": int(rm.group(2)),
                            "ignored": int(rm.group(3))})
        check(gates[name]["passed"] == 3 and gates[name]["failed"] == 0 and gates[name]["ignored"] == 2,
              f"targeted gate {gates[name]}")
        gates[name]["host_print_lines"] = [l for l in t.split("\n") if l.startswith("r05-s3b[host]")]
rep["gates"] = gates

# 8. 证据清单 / 编码 / 禁词 / receipt
manifest_rows = []
for line in head(f"{EV}/manifest.txt").decode().split("\n"):
    line = line.strip()
    if line and not line.startswith("#"):
        h, n, p = line.split(" ", 2)
        manifest_rows.append((h, int(n), p))
ev_files = sorted(p for p in (R / EV).rglob("*") if p.is_file())
ev_rel = {f"{EV}/" + p.relative_to(R / EV).as_posix() for p in ev_files if p.name != "manifest.txt"}
man_paths = {p for _, _, p in manifest_rows if p.startswith(EV + "/")}
man_problems = []
if ev_rel != man_paths:
    man_problems.append(f"set mismatch {sorted(ev_rel ^ man_paths)}")
for h, n, p in manifest_rows:
    b = head(p)
    if (sha(b), len(b)) != (h, n):
        man_problems.append(f"{p}: {h}/{n} vs {sha(b)}/{len(b)}")
bom, crlf, hits = [], [], []
for p in ev_files:
    b = p.read_bytes()
    rel = p.relative_to(R / EV).as_posix()
    if b.startswith(b"\xef\xbb\xbf"):
        bom.append(rel)
    if b"\r" in b:
        crlf.append(rel)
    t = p.read_text(encoding="utf-8", errors="replace")
    hits += [(rel, w) for w in BANNED if w in t]
receipt = json.loads(head(f"{EV}/receipt.json"))
rep["evidence"] = {
    "files": len(ev_files), "manifest_rows": len(manifest_rows), "manifest_problems": man_problems,
    "manifest_self_reference": any(p == "manifest.txt" for _, _, p in manifest_rows),
    "bom_files": bom, "crlf_files": crlf, "forbidden_word_hits": hits,
    "receipt_task_id": receipt.get("task_id"),
    "receipt_numeric_conclusion_false": receipt.get("numeric_conclusion") is False,
    "receipt_gpu_window_false": receipt.get("gpu_window") is False,
    "receipt_gpu_entities_executed_false": receipt.get("gpu_entities_executed") is False,
    "receipt_lines_fields": [f.get("lines") for f in receipt.get("files", [])],
}
check(not man_problems, f"manifest problems: {man_problems}")
check(not rep["evidence"]["manifest_self_reference"], "manifest self reference")
check(not bom, f"BOM files: {bom}")
check(not hits, f"forbidden words: {hits}")
check(receipt.get("task_id") == "2026-10-28-render-bvh-r05-s3b-hfield-target-authoring", "receipt task_id")
check(receipt.get("numeric_conclusion") is False and receipt.get("gpu_window") is False, "receipt flags")

entity = json.loads(head(f"{EV}/window/entity-table.json"))
guards = json.loads(head(f"{EV}/window/guards-skeleton.json"))
entities = entity.get("entities", [])
regression = [e["entity"] for e in entities if e.get("source") == "regression"]
new_entities = [e["entity"] for e in entities if e.get("source") == "new"]
frozen = [e["entity"] for e in guards.get("frozen_exes", [])]
rep["window_materials"] = {
    "entities": len(entities), "new": new_entities, "regression": regression,
    "guards_frozen_exes": len(frozen),
    "guards_not_frozen": guards.get("freeze_status") == "not_frozen",
    "guards_hashes_empty": all(not e.get("sha256") and e.get("bytes") is None
                               for e in guards.get("frozen_exes", [])),
    "entity_list_matches_guards": [e["entity"] for e in entities] == frozen,
}
check(len(entities) == 9 and len(new_entities) == 1 and len(regression) == 8,
      f"entity table: {rep['window_materials']}")
check(rep["window_materials"]["guards_not_frozen"] and rep["window_materials"]["guards_hashes_empty"],
      "guards skeleton not empty/unfrozen")
check(rep["window_materials"]["entity_list_matches_guards"], "entity vs guards mismatch")
r09_guards = R / "target/s3b-validation-window/guards.json"
if r09_guards.exists():
    g = json.loads(r09_guards.read_text(encoding="utf-8-sig"))
    r09_reg = [e["entity"] for e in g.get("frozen_exes", []) if e.get("source") == "regression"]
    rep["window_materials"]["r09_regression_list"] = r09_reg
    rep["window_materials"]["matches_r09_regression"] = sorted(r09_reg) == sorted(regression)
    check(sorted(r09_reg) == sorted(regression), f"regression list differs from R09 guards: {r09_reg}")
else:
    findings.append("R09-S3b window guards.json 不在盘（target/ 瞬态），regression 清单仅按证据 README 交叉")

# 9. 零渲染数值结论扫描
patterns = [r"beyond_1_lsb", r"beyond_one_lsb", r"max_ulp", r"mismatched", r"mismatch\s*=",
            r"≤\s*1\s*LSB", r"≤\s*4\s*ULP"]
allowed_markers = ("判据", "口径", "不新增", "不放宽", "超判据", "尚未", "留 S3b", "零渲染数值结论",
                   "criteria", "threshold", "not executed", "not_run",
                   "打印", "per camera", "max_ulp<=4", "beyond_ulp==0", "mismatched==0",
                   "beyond_one_lsb_pixels ==", "beyond_1_lsb", "max_byte_diff", "max_ulp<=4",
                   "RGB <= 1 LSB", "seg exact", "两相机", "打印点",
                   "depth_mismatch", "bitwise-first", "mismatched == 0", "max_ulp <= 4",
                   "beyond_ulp == 0", "pixels/depth_mismatch")
hits = []
for rel in sorted(p for p in (R / EV).rglob("*") if p.is_file()):
    t = rel.read_text(encoding="utf-8", errors="replace")
    for pat in patterns:
        for m in re.finditer(pat, t):
            ln = t[:m.start()].count("\n") + 1
            line = t.split("\n")[ln - 1].strip()
            if any(k in line for k in allowed_markers):
                continue
            hits.append((rel.relative_to(R / EV).as_posix(), ln, pat, line[:160]))
rep["numeric_conclusion_suspicious"] = hits
check(not hits, f"suspicious numeric-conclusion lines: {hits[:5]}")

# 10. npz 参考统计独立复算（结构，非比较）
def npz_members(path):
    out = {}
    with zipfile.ZipFile(path) as zf:
        for name in zf.namelist():
            if not name.endswith(".npy"):
                continue
            with zf.open(name) as fh:
                data = fh.read()
            major = data[6]
            if major == 1:
                hlen = struct.unpack("<H", data[8:10])[0]
                header = data[10:10 + hlen].decode("latin1")
                payload = data[10 + hlen:]
            else:
                hlen = struct.unpack("<I", data[8:12])[0]
                header = data[12:12 + hlen].decode("latin1")
                payload = data[12 + hlen:]
            dtype = re.search(r"'descr':\s*'([^']+)'", header).group(1)
            shape = tuple(int(x) for x in re.search(r"'shape':\s*\(([^)]*)\)", header).group(1).split(",") if x.strip())
            out[name] = (dtype, shape, payload)
    return out


depth_total_zero = 0
shapes = {}
per_group_zero = {}
for group in ("hfield-only", "hfield-primitive"):
    group_zero = 0
    members = npz_members(R / f"fixtures/render-bvh/scene-hfield/capture-hfield/{group}/depth.npz")
    for name, (dtype, shape, payload) in members.items():
        assert dtype == "<f4", dtype
        values = struct.unpack(f"<{len(payload) // 4}f", payload)
        z = sum(1 for v in values if v == 0.0)
        group_zero += z
        shapes[f"{group}:{name}"] = {"shape": shape, "zeros": z}
    per_group_zero[group] = group_zero
    depth_total_zero += group_zero
rep["reference_statistics"] = {"depth_zero_total": depth_total_zero,
                               "per_group": per_group_zero,
                               "matches_host_print_66826_per_group": all(v == 66826 for v in per_group_zero.values()),
                               "depth_shapes": shapes}
check(all(v == 66826 for v in per_group_zero.values()),
      f"depth zero-pixel per-group {per_group_zero} != 66826 (host print)")

rep["findings"] = findings
rep["problems"] = problems
out = R / CHK / "recompute.json"
out.write_text(json.dumps(rep, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
print(json.dumps(rep, ensure_ascii=False, indent=2))
print("RESULT problems=" + str(len(problems)) + " findings=" + str(len(findings)))
for item in problems:
    print("PROBLEM " + str(item))
for item in findings:
    print("FINDING " + str(item))
