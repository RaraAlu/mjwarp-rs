"""QA independent recompute for R02-S1b negative-controls check (tests-only slice).

Read-only: git objects, repo tree, my own cargo log. No GPU.
"""
import hashlib
import json
import pathlib
import re
import subprocess
import sys

REPO = pathlib.Path(r"D:\Rust\mjwarp-rs")
EV = "docs/development-evidence/render-bvh/r02-skybox-s1b-negative-controls"
OUT = {}

FILES = ["tests/render_e2e.rs", "tests/render_kernel_flags.rs", "tests/render_kernel_rgb.rs"]


def blob(rev_path):
    r = subprocess.run(["git", "show", rev_path], cwd=REPO, capture_output=True)
    if r.returncode != 0:
        raise SystemExit(f"git show failed: {rev_path}")
    return r.stdout


def btext(p):
    return blob(f"HEAD:{p}").decode("utf-8", errors="replace")


def sha(b):
    return hashlib.sha256(b).hexdigest()


def lines_at(rev, path):
    return blob(f"{rev}:{path}").count(b"\n")


def main():
    # 1. line counts before/after (LF) + red line
    counts = {}
    for f in FILES:
        before = lines_at("f2ee32d0^", f)
        after = lines_at("f2ee32d0", f)
        counts[f] = {"before": before, "after": after, "net": after - before,
                     "limit": 500, "ok": after <= 500}
    OUT["line_counts"] = counts
    OUT["e2e_net_non_positive"] = counts["tests/render_e2e.rs"]["net"] <= 0

    # 2. slice scope: only tests + card (+evidence/runs from later commits)
    scope = subprocess.run(["git", "diff", "--name-status", "f2ee32d0^..f2ee32d0"],
                           cwd=REPO, capture_output=True).stdout.decode().splitlines()
    OUT["code_commit_diff"] = scope
    OUT["src_untouched_in_code_commit"] = all(not l.split("\t")[-1].startswith("src/") for l in scope)

    # 3. stale rejection scan: every render_skybox occurrence in tests/ + src/*.rs
    grep = subprocess.run(["git", "grep", "-n", "render_skybox", "--", "tests", "src"],
                          cwd=REPO, capture_output=True).stdout.decode().splitlines()
    stale = []
    for line in grep:
        path, lineno, text = line.split(":", 2)
        text = text.strip()
        # evidence-side scan window: look +-8 lines for a rejection expectation
        all_lines = btext(path).splitlines()
        i = int(lineno) - 1
        window = "\n".join(all_lines[max(0, i - 8): i + 9])
        if "render_skybox" in text and ("必须被接受" in text or "必须可创建" in text):
            continue
        suspicious = False
        for marker in ("必须在写前拒绝", "仍写前拒绝", "Unsupported(\"S7b3 不含天空盒\")", "skybox rejected"):
            if marker in window:
                suspicious = True
        if suspicious:
            stale.append({"path": path, "line": int(lineno), "text": text,
                          "window": window[-300:]})
    OUT["stale_rejection_hits"] = stale
    # 3b. repo-wide scan coverage vs the supplied scan file
    scan = btext(f"{EV}/materials/render-skybox-scan.txt").splitlines()
    scan_hits = [l for l in scan if re.match(r"^(tests|src)[\\/]", l)]
    mine = {(l.split(":", 2)[0].replace("/", "\\"), l.split(":", 2)[1])
            for l in grep}
    theirs = {(l.split(":", 2)[0].replace("/", "\\"), l.split(":", 2)[1]) for l in scan_hits}
    OUT["scan_coverage"] = {
        "my_grep_hits": len(grep),
        "evidence_scan_rows": len(scan_hits),
        "missing_from_evidence": sorted([f"{p}:{n}" for p, n in (mine - theirs)]),
        "extra_in_evidence": sorted([f"{p}:{n}" for p, n in (theirs - mine)]),
    }
    # scan scope word check: does the .cu file contribute render_skybox hits?
    cu = subprocess.run(["git", "grep", "-c", "render_skybox", "--", "src/render/kernel/cuda"],
                        cwd=REPO, capture_output=True).stdout.decode().strip()
    OUT["scan_coverage"]["cuda_hits"] = cu
    # 3c. stale statement that does not contain the literal "render_skybox"
    mk = btext("src/render/kernel/megakernel.rs").splitlines()
    OUT["stale_rejection_statements"] = [
        {"path": "src/render/kernel/megakernel.rs", "line": i + 1, "text": l.strip()}
        for i, l in enumerate(mk)
        if "仍写前拒绝" in l
    ]

    # 4. assertion inventory in the three files (added/kept negative controls)
    inv = {}
    for f in FILES:
        t = btext(f)
        inv[f] = {
            "assert_macros": len(re.findall(r"\bassert(?:_eq|_ne)?!", t)),
            "has_dynamic_ray_rejection": "use_precomputed_rays=false" in t,
            "has_skybox_accept": "render_skybox=true" in t,
            "has_splat_rejection": "splat 必须在写前拒绝" in t,
            "has_aa_ray_count_check": "ray_count" in t,
            "catch_unwind_continue": "Err(_) => continue" in t,
        }
    OUT["assertion_inventory"] = inv
    out_removed = {}
    for f in FILES:
        old = btext(f).replace("", "")
        out_removed[f] = {}
    # which negative-control cases were removed: compare rejection case counts old vs new
    for f in FILES:
        old = blob(f"f2ee32d0^:{f}").decode("utf-8", errors="replace")
        new = btext(f)
        out_removed[f] = {
            "old_render_skybox_rejection_case": "render_skybox" in old and "render_skybox" in new,
            "old_rejected_counter": "rejected" in old,
            "new_rejected_counter": "rejected" in new,
            "removed_assert_eq_rejected_2": "assert_eq!(rejected, 2" in old and "assert_eq!(rejected, 2" not in new,
            "e2e_read_rgb": "outputs().read(ImageOutput::Rgb).is_ok()" in new,
        }
    OUT["removed_assertions"] = out_removed

    # 5. read() semantics (does read verify content?)
    lc = btext("src/render/context/lifecycle.rs")
    OUT["output_read_semantics"] = {
        "read_signature": "pub fn read(&self, output: ImageOutput) -> Result<OutputValues, OwnerError>" in lc,
        "read_copies_device_to_host": "buffer.read_range_into(0, &mut values)?;" in lc,
        "content_checked": "assert" in lc.split("pub fn read")[1][:600] if "pub fn read" in lc else None,
    }

    # 6. unit counts from my own run
    log = REPO / "target/qa-s1b/log/test.log"
    if log.exists():
        t = log.read_text(encoding="utf-8", errors="replace")
        tot = re.findall(r"test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored", t)
        OUT["unit_observed"] = {"result_lines": len(tot),
                                "passed": sum(int(x[1]) for x in tot),
                                "failed": sum(int(x[2]) for x in tot),
                                "ignored": sum(int(x[3]) for x in tot)}
    # 7. evidence manifest
    man = btext(f"{EV}/manifest.txt")
    rows = [l.split() for l in man.splitlines() if l and not l.startswith("#")]
    bad_c, bad_d, selfref = [], [], False
    for digest, size, path in rows:
        size = int(size)
        if path.endswith("r02-skybox-s1b-negative-controls/manifest.txt"):
            selfref = True
        b = blob(f"HEAD:{path}")
        if sha(b) != digest or len(b) != size:
            bad_c.append(path)
        p = REPO / path
        if not p.exists():
            bad_d.append({"path": path, "why": "missing"})
        else:
            wb = p.read_bytes()
            if sha(wb) != digest or len(wb) != size:
                bad_d.append({"path": path, "why": "mismatch"})
    tree = subprocess.run(["git", "ls-tree", "-r", "--name-only", "HEAD", f"{EV}/"],
                          cwd=REPO, capture_output=True).stdout.decode().split()
    listed = {r[2] for r in rows}
    OUT["evidence_manifest"] = {"rows": len(rows), "bad_committed": bad_c, "bad_disk": bad_d,
                                "self_reference": selfref,
                                "unlisted": sorted(p for p in tree if p not in listed and not p.endswith("manifest.txt"))}
    # 8. forbidden literals in evidence (codepoint-built, negative-statement scan)
    terms = [[0x47, 0x31, 0x30, 0x5DF2, 0x5B9E, 0x73B0], [0x5B8C, 0x6574, 0x7269, 0x7406, 0x901A, 0x8FC7],
             [0x53CC, 0x5E73, 0x53F0, 0x901A, 0x8FC7], [0x4E0A, 0x6E38, 0x5B8C, 0x5168, 0x7B49, 0x4EF7],
             [0x539F, 0x751F, 0x653E, 0x884C], [0x7CBE, 0x5EA6, 0x8FBE, 0x6807],
             [0x7A97, 0x53E3, 0x901A, 0x8FC7], [0x4EA7, 0x54C1, 0x6E32, 0x67D3, 0x9A8C, 0x6536, 0x901A, 0x8FC7],
             [0x4E0A, 0x6E38, 0x7B49, 0x4EF7]]
    words = ["".join(chr(c) for c in s) for s in terms]
    hits = []
    for t in tree:
        txt = btext(t)
        found = [f"T{i}" for i, w in enumerate(words) if w in txt]
        if found:
            hits.append({"path": t.split("/")[-1], "terms": found,
                         "lines": [l.strip() for l in txt.splitlines() if any(w in l for w in words)][:3]})
    OUT["forbidden_literal_hits"] = hits

    # 9. cross-evidence: did the updated tests actually run in the S3 attempt2 window?
    s3 = "docs/development-evidence/render-bvh/r02-skybox-s3-validate/attempt2/logs/"
    cross = {}
    for n in ("render_e2e.log", "render_kernel_flags.log", "render_kernel_rgb.log"):
        try:
            t = btext(s3 + n)
        except SystemExit:
            cross[n] = None
            continue
        cross[n] = {
            "test_result_ok": "test result: ok." in t,
            "new_println_seen": ("shadows+skybox accepted" in t or "skybox supported" in t),
            "failed": "failed" in t and "0 failed" not in t,
        }
    OUT["cross_check_s3_attempt2"] = cross

    # problems (must be empty)
    problems = []
    if not OUT["e2e_net_non_positive"]:
        problems.append("e2e net > 0")
    if not all(v["ok"] for v in counts.values()):
        problems.append("line count over 500")
    if not OUT["src_untouched_in_code_commit"]:
        problems.append("src touched")
    if stale:
        problems.append("stale rejection hits")
    if OUT["evidence_manifest"]["rows"] != 5 or OUT["evidence_manifest"]["bad_committed"] \
            or OUT["evidence_manifest"]["bad_disk"] or OUT["evidence_manifest"]["self_reference"] \
            or OUT["evidence_manifest"]["unlisted"]:
        problems.append("evidence manifest")
    u = OUT.get("unit_observed", {})
    if not (u.get("passed") == 1104 and u.get("failed") == 0 and u.get("ignored") == 390
            and u.get("result_lines") == 136):
        problems.append("unit totals")
    OUT["problems"] = problems

    pathlib.Path(r"D:\Rust\mjwarp-rs\target\qa-s1b\recompute.json").write_text(
        json.dumps(OUT, ensure_ascii=False, indent=1), encoding="utf-8", newline="\n")
    print("line counts:", counts)
    print("code commit diff:", scope)
    print("src untouched:", OUT["src_untouched_in_code_commit"])
    print("stale rejection hits:", stale)
    print("scan coverage:", OUT["scan_coverage"])
    print("stale statements (no literal):", OUT["stale_rejection_statements"])
    print("assertion inventory:", json.dumps(inv, ensure_ascii=False))
    print("removed assertions:", json.dumps(out_removed, ensure_ascii=False))
    print("output read semantics:", OUT["output_read_semantics"])
    print("unit observed:", OUT.get("unit_observed"))
    print("manifest:", OUT["evidence_manifest"])
    print("forbidden literal hits:", OUT["forbidden_literal_hits"])
    print("cross check s3 attempt2:", cross)
    print("PROBLEMS", problems)
    return 0


if __name__ == "__main__":
    sys.exit(main())
