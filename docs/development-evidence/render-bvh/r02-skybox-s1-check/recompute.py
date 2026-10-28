"""QA independent recompute for R02-S1 skybox offline slice check.

Read-only: reads pinned upstream sources, committed evidence blobs, on-disk PTX
dumps and the repo tree. No GPU, no writes outside target/qa-r02.
"""
import hashlib
import json
import pathlib
import re
import subprocess
import sys

REPO = pathlib.Path(r"D:\Rust\mjwarp-rs")
EV = "docs/development-evidence/render-bvh/r02-skybox-s1"
PIN = REPO / "target/render-bvh-precheck/venv312/Lib/site-packages"
GEN_CU = REPO / ("target/render-bvh/s3a-shadow-cache/1.15.0/"
                 "wp__build_megakernel__locals___render_megakernel_92d94e46_dde26ad/"
                 "wp__build_megakernel__locals___render_megakernel_92d94e46_dde26ad.cu")
PTX = REPO / "target/r02-ptx"
OUT = {}


def blob(path):
    r = subprocess.run(["git", "show", f"HEAD:{path}"], cwd=REPO, capture_output=True)
    if r.returncode != 0:
        raise SystemExit(f"git show failed: {path}")
    return r.stdout


def btext(path):
    return blob(path).decode("utf-8", errors="replace")


def sha(b):
    return hashlib.sha256(b).hexdigest()


def file_lines(p):
    return pathlib.Path(p).read_bytes().count(b"\n")


def main():
    # ---------------------------------------------------------------- 1 upstream
    rp = PIN / "mujoco_warp/_src/render.py"
    rb = rp.read_bytes()
    rl = rb.decode("utf-8", errors="replace").splitlines()
    OUT["upstream_render_py"] = {"bytes": len(rb), "lines": rb.count(b"\n"), "sha256": sha(rb)}
    fn = rl[493:556]
    fn_lines = [l.strip() for l in fn]
    fn_text = "\n".join(fn_lines)
    # face -> (sc, tc) parsed from the upstream source itself (no hardcoded line offsets)
    faces = {}
    pending = None
    for l in fn_lines:
        m = re.match(r"face = (\d+)", l)
        if m:
            pending = int(m.group(1))
            faces[pending] = {}
            continue
        if pending is not None:
            m = re.match(r"(sc|tc) = (-?[a-z]+)", l)
            if m:
                faces[pending][m.group(1)] = m.group(2)
    expected = {0: ("-rz", "-ry"), 1: ("rz", "-ry"), 2: ("rx", "rz"),
                3: ("rx", "-rz"), 4: ("rx", "-ry"), 5: ("-rx", "-ry")}
    faces_ok = all(faces.get(k, {}).get("sc") == v[0] and faces.get(k, {}).get("tc") == v[1]
                   for k, v in expected.items())
    checks = {
        "rotation_x_z_neg_y": ("rx = ray_dir_world[0]" in fn_lines
                                and "ry = ray_dir_world[2]" in fn_lines
                                and "rz = -ray_dir_world[1]" in fn_lines),
        "six_faces_sc_tc_parsed_from_source": faces_ok,
        "face_sc_tc_table": {str(k): faces.get(k) for k in sorted(faces)},
        "safe_div_s_and_t": ("math.safe_div(sc, ma)" in fn_text and "math.safe_div(tc, ma)" in fn_text),
        "t_min_and_clamp": ("t_min = 0.5 * face_width_inv" in fn_text and "wp.clamp(t, t_min, 1.0 - t_min)" in fn_text),
        "v_formula_1_6": "v = (float(face) + t) * wp.static(1.0 / 6.0)" in fn_text,
        "texture_sample_vec4": "wp.texture_sample(skybox_tex, wp.vec2(s, v), dtype=wp.vec4)" in fn_text,
        "face_stack_order_comment": "+X, -X, +Y, -Y, +Z, -Z" in fn_text,
        "max_axis_chain": ("if arx >= ary and arx >= arz:" in fn_text and "elif ary >= arz:" in fn_text),
        "ma_assignments": all(x in fn_text for x in ("ma = arx", "ma = ary", "ma = arz")),
    }
    OUT["upstream_sample_skybox_lines_494_556"] = {"first_line": fn_lines[0], "last_line": fn_lines[-1],
                                                   "checks": checks}
    call = rl[1258:1276]
    OUT["upstream_call_site_1259_1268"] = {
        "lines": [l.rstrip() for l in call],
        "has_static_skybox_and_render_rgb": "wp.static(rc_static[\"render_skybox\"]) and render_rgb[camid]" in call[0],
        "has_skybox_id_mod_shape": "skybox_tex_id[worldid % skybox_tex_id.shape[0]]" in "\n".join(call),
        "face_width_inv_expr": "1.0 / float(skybox_face_width[worldid % skybox_face_width.shape[0]])" in "\n".join(call),
        "store_pixel_call": "store_pixel(worldid, rgb_adr[camid] + rayid_local, skybox_color" in "\n".join(call),
        "elif_render_rgb": "elif render_rgb[camid]:" in "\n".join(call),
    }
    sp = rl[1016:1031]
    OUT["upstream_store_pixel_1017_1031"] = {
        "lines": [l.rstrip() for l in sp],
        "aa_adds_color_times_255": "aa_accum_out[worldid, adr] += color * 255.0" in "\n".join(sp),
        "non_aa_pack_with_255": "pack_rgba_to_uint32(color[0] * 255.0, color[1] * 255.0, color[2] * 255.0, 255.0)" in "\n".join(sp),
    }
    # render_util.py 514-535
    ru = (PIN / "mujoco_warp/_src/render_util.py").read_text(encoding="utf-8", errors="replace").splitlines()
    out_ru = ru[513:535]
    OUT["upstream_render_util_514_535"] = {
        "skybox_ids_nonzero": any("tex_type == mujoco.mjtTexture.mjTEXTURE_SKYBOX" in l for l in out_ru),
        "has_skybox_condition": any("render_skybox and skybox_tex_ids.size > 0" in l for l in out_ru),
        "face_width_is_tex_width": any("skybox_face_width_np = np.array([mjm.tex_width[skybox_tex_ids[0]]]" in l for l in out_ru),
        "render_skybox_set_false_otherwise": any("render_skybox = False" in l for l in out_ru),
        "registry_gate": any("if use_textures or has_skybox:" in l for l in out_ru),
        "empty_tex_1x1x4": any("wp.Texture2D(width=1, height=1, num_channels=4, dtype=wp.float32)" in l for l in out_ru),
        "loop_uses_or_skybox_id": any("if use_textures or i == skybox_id:" in l for l in out_ru),
    }
    mp = (PIN / "mujoco_warp/_src/math.py").read_text(encoding="utf-8", errors="replace").splitlines()
    OUT["upstream_safe_div_math_318_319"] = {"lines": [l.rstrip() for l in mp[316:319]],
                                            "mj_minval_1e-15": "MJ_MINVAL" in "\n".join(mp[316:319])}

    # ------------------------------------------------- 2 generated CUDA semantics
    g = GEN_CU.read_text(encoding="utf-8", errors="replace")
    gl = g.splitlines()
    one_sixth = [l.strip() for l in gl if "0.16666666666666666" in l]
    OUT["generated_cu"] = {
        "file": str(GEN_CU),
        "sample_skybox_present": "sample_skybox_0(" in g,
        "one_sixth_decl": one_sixth[:2],
        "one_sixth_declared_as_float32": all("wp::float32" in l for l in one_sixth) and bool(one_sixth),
        "f32_bits_of_one_sixth": hex(int.from_bytes(__import__("struct").pack("<f", 1.0 / 6.0), "little")),
        "clamp_via_min_max": "wp::clamp(var_92, var_94, var_96)" in g,
        "texture_sample_vec4_with_lod": bool(re.search(r"texture_sample<wp::vec_t<4, wp::float32>>\(var_skybox_tex, var_102, var_104\)", g)),
        "call_site_static_and_render_rgb": "wp.static(rc_static[\"render_skybox\"]) and render_rgb[camid]" in g,
        "skybox_call_present": "sample_skybox_0(var_220, var_218, var_58)" in g,
        "store_pixel_call_after": "store_pixel_0(var_0, var_223, var_219, var_rgb_out, var_aa_accum_out)" in g,
    }
    # warp clamp definition
    bh = (PIN / "warp/native/builtin.h")
    if bh.exists():
        bt = bh.read_text(encoding="utf-8", errors="replace")
        OUT["generated_cu"]["warp_clamp_def"] = [l.strip() for l in bt.splitlines() if "clamp(T x, T a, T b)" in l][:2]

    # ------------------------------------------------------ 3 implementation side
    impl = btext("src/render/kernel/cuda/render_shade.cu")
    OUT["impl_shade_cu"] = {
        "has_sample_skybox": "V3 sample_skybox(" in impl,
        "uses_safe_div_render": "safe_div_render(sc, ma)" in impl and "safe_div_render(tc, ma)" in impl,
        "t_min_half_texel": "float t_min = 0.5f * face_width_inv;" in impl,
        "clamp_min_max": "fminf(fmaxf(t, t_min), 1.0f - t_min)" in impl,
        "v_formula_f32": "((float)face + t) * (1.0f / 6.0f)" in impl,
        "texture_sample_vec4_lod0": "wp::texture_sample<wp::vec4>(tex, wp::vec2(s, v), 0.0f)" in impl,
        "six_face_branches": all(tok in impl for tok in
                                 ("face = 0;", "face = 1;", "face = 2;", "face = 3;", "face = 4;", "face = 5;")),
    }
    # parse the CUDA sample_skybox face table the same way and compare to upstream
    shade_fn_start = impl.index("__device__ __forceinline__ V3 sample_skybox(")
    shade_fn_end = impl.index("// \u4e0a\u6e38 ertex_normal", shade_fn_start) if False else impl.index("V3 vertex_normal(", shade_fn_start)
    shade_fn = impl[shade_fn_start:shade_fn_end]
    impl_faces = {}
    pending = None
    for l in shade_fn.splitlines():
        ls = l.strip()
        m = re.match(r"face = (\d+);", ls)
        if m:
            pending = int(m.group(1))
            impl_faces[pending] = {}
            continue
        if pending is not None:
            m = re.match(r"(sc|tc) = (-?[a-z]+);", ls)
            if m:
                impl_faces[pending][m.group(1)] = m.group(2)
    OUT["impl_shade_cu"]["face_table_matches_upstream"] = all(
        impl_faces.get(k, {}).get("sc") == v[0] and impl_faces.get(k, {}).get("tc") == v[1]
        for k, v in expected.items())
    OUT["impl_shade_cu"]["impl_face_table"] = {str(k): impl_faces.get(k) for k in sorted(impl_faces)}
    OUT["impl_shade_cu"]["rotation_lines"] = [ls for ls in (l.strip() for l in shade_fn.splitlines())
                                                if ls.startswith(("float rx =", "float ry =", "float rz ="))]
    OUT["impl_shade_cu"]["max_axis_chain"] = ("if (arx >= ary && arx >= arz) {" in shade_fn
                                              and "} else if (ary >= arz) {" in shade_fn)
    mk = btext("src/render/kernel/cuda/render_megakernel.cu")
    OUT["impl_megakernel_cu"] = {
        "define_skybox_0x200": "#define MK_FLAG_SKYBOX 0x200u" in mk,
        "branch_gated_by_bit9": "(flag_bits & MK_FLAG_SKYBOX) != 0u" in mk,
        "branch_inside_geom_minus_one": mk.find("(flag_bits & MK_FLAG_SKYBOX)") > mk.find("if (hit.geom_id == -1)"),
        "old_aa_branch_kept_as_else_if": "} else if (aa) {" in mk,
        "non_aa_pack_255": "pack_rgba_to_uint32(skybox_color.x * 255.0f" in mk,
        "aa_store_shared_helper": "aa_store(aa_accum_out, rgb_offset, skybox_color, ray_base == 0)" in mk,
        "slots_read_only_in_branch": mk.count("skybox_tex_id.data[") == 1 and mk.count("skybox_face_width.data[") == 1,
    }
    # host side
    ti = btext("src/render/context/texture_inputs.rs")
    OUT["impl_texture_inputs"] = {
        "lines": ti.count("\n"),
        "branches_use_textures": "if use_textures {" in ti,
        "branch_skybox_only": "let Some(skybox) = skybox else {" in ti,
        "dummy_1x1x4_zero": "TextureData::from_raw_at(texture, 1, 1, 4, 0, &DUMMY_TEXEL)" in ti,
        "dummy_all_zero": "const DUMMY_TEXEL: [u8; 4] = [0; 4];" in ti,
        "index_equals_texture_id": "texture == skybox_id" in ti,
        "face_width_check": "if face_width != skybox.face_width" in ti,
        "empty_when_neither": "return Ok(TextureRegistry::default());" in ti,
    }
    lc = btext("src/render/context/lifecycle.rs")
    OUT["impl_lifecycle_gate"] = {
        "line": next((i + 1 for i, l in enumerate(lc.splitlines()) if "let wants_textures" in l), None),
        "expr": "let wants_textures = resolved.use_textures || resolved.skybox.is_some();" in lc,
    }
    up = btext("src/render/kernel/megakernel_uploads.rs")
    OUT["impl_uploads_gate"] = {
        "line": next((i + 1 for i, l in enumerate(up.splitlines()) if "Some(resources) if resolved.use_textures" in l), None),
        "expr": "Some(resources) if resolved.use_textures || resolved.render_skybox => {" in up,
        "reject_when_skybox_but_no_array": "if resolved.render_skybox && texture_array.is_none()" in up,
        "slots_none_zero": "None => ([0i32], [0i32])," in up,
        "slots_some_values": "[skybox.texture_id]" in up,
    }
    ar = btext("src/render/kernel/megakernel_arrays.rs")
    OUT["impl_flag_bit"] = {
        "rust_bit9": "pub(crate) const FLAG_SKYBOX_BIT: u32 = 1 << 9;" in ar,
        "flag_set_only_when_render_skybox": "if resolved.render_skybox {" in ar and "flag_bits |= FLAG_SKYBOX_BIT;" in ar,
        "high_word_packing_unchanged": "(u64::from(resolved.background_color_packed) << 32) | u64::from(flag_bits)" in ar,
    }
    ctx = btext("src/render/context.rs")
    OUT["impl_context"] = {
        "render_skybox_is_skybox_is_some": "render_skybox: skybox.is_some()," in ctx,
        "pub_mod_texture_inputs": "pub mod texture_inputs;" in ctx,
        "pub_use_texture_registry": "pub use texture_inputs::texture_registry;" in ctx,
    }

    # ------------------------------------------------------------- 4 invariants
    slots_def = btext("src/runtime/launcher/render/args.rs")
    OUT["megakernel_slots"] = {
        "definition_unchanged_in_slice": subprocess.run(
            ["git", "diff", "--quiet", "8aa33e00^..HEAD", "--", "src/runtime/launcher/render/args.rs"],
            cwd=REPO).returncode == 0,
        "declared_len": re.search(r"pub static MEGAKERNEL_SLOTS: \[RenderArgKind; (\d+)\]", slots_def).group(1),
    }
    OUT["device_sources"] = re.search(r"DEVICE_SOURCES: \[\(&str, &str\); (\d+)\]",
                                      btext("src/runtime/collision_gpu/source.rs")).group(1)
    OUT["collision_args_bytes"] = "size_of::<CollisionArgs>(), 504" in btext("src/runtime/collision_gpu/abi_tests.rs")

    # ------------------------------------------------------------ 5 line counts
    limits = {
        "src/render/kernel/cuda/render_shade.cu": 470,
        "src/render/kernel/cuda/render_megakernel.cu": 420,
        "src/render/kernel/megakernel.rs": 412,
        "src/render/kernel/megakernel_arrays.rs": 340,
        "src/render/kernel/megakernel_uploads.rs": 480,
        "src/render/context.rs": 470,
        "src/render/context/lifecycle.rs": 499,
        "src/render/context/texture_inputs.rs": 160,
        "tests/render_skybox.rs": 260,
        "src/render/texture.rs": 486,
        "src/render/kernel/cast.rs": 481,
        "src/render/kernel/cuda/render.cu": 499,
        "src/render/kernel/launch.rs": 500,
        "src/model/render.rs": 500,
    }
    lc_out = {}
    for f, lim in limits.items():
        n = file_lines(REPO / f)
        lc_out[f] = {"lines": n, "limit": lim, "ok": n <= lim}
    lc_out["all_ok"] = all(v["ok"] for v in lc_out.values() if isinstance(v, dict))
    OUT["line_counts_lf"] = lc_out

    # ------------------------------------------------------------- 6 kernel ptx
    ptx = {}
    for n in ("baseline", "current"):
        p = PTX / f"{n}.ptx"
        if p.exists():
            t = p.read_text(encoding="utf-8", errors="replace")
            lines = t.splitlines()
            ptx[n] = {
                "bytes": p.stat().st_size,
                "sha256": sha(p.read_bytes()),
                "lines": len(lines),
                "tex_level_2d": t.count("tex.level.2d"),
                "and_b32_512": sum(1 for l in lines if "and.b32" in l and "512" in l),
                "and_b32_256": sum(1 for l in lines if "and.b32" in l and "256" in l),
                "and_b32_total": sum(1 for l in lines if "and.b32" in l),
                "max_f32": t.count("max.f32"),
                "min_f32": t.count("min.f32"),
                "sqrt": t.count("sqrt"),
            }
    OUT["ptx_artifacts"] = ptx
    gate_log = REPO / "target/qa-r02/log/kernel-gate.log"
    outlook = {}
    if gate_log.exists():
        gt = gate_log.read_text(encoding="utf-8", errors="replace")
        for k in ("source_sha256", "ptx_sha256", "ptx_bytes"):
            m = re.search(k + r"=([0-9a-f]+|\d+)", gt)
            outlook[k] = m.group(1) if m else None
    OUT["kernel_gate_observed"] = outlook

    # --------------------------------------------------------------- 7 evidence
    man = btext(f"{EV}/manifest.txt")
    rows = [l.split() for l in man.splitlines() if l and not l.startswith("#")]
    bad_c, bad_d, selfref = [], [], False
    for digest, size, path in rows:
        size = int(size)
        if path.endswith("r02-skybox-s1/manifest.txt"):
            selfref = True
        b = blob(path)
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
    # slice diff scope
    diff = subprocess.run(["git", "diff", "--name-status", "8aa33e00^..HEAD"], cwd=REPO,
                          capture_output=True).stdout.decode().splitlines()
    OUT["slice_diff"] = [l for l in diff if l]
    # forbidden words (codepoint-built so the literal terms never appear in this file)
    terms = [[0x47, 0x31, 0x30, 0x5DF2, 0x5B9E, 0x73B0], [0x5B8C, 0x6574, 0x7269, 0x7406, 0x901A, 0x8FC7],
             [0x53CC, 0x5E73, 0x53F0, 0x901A, 0x8FC7], [0x4E0A, 0x6E38, 0x5B8C, 0x5168, 0x7B49, 0x4EF7],
             [0x539F, 0x751F, 0x653E, 0x884C], [0x7CBE, 0x5EA6, 0x8FBE, 0x6807],
             [0x7A97, 0x53E3, 0x901A, 0x8FC7], [0x4EA7, 0x54C1, 0x6E32, 0x67D3, 0x9A8C, 0x6536, 0x901A, 0x8FC7],
             [0x4E0A, 0x6E38, 0x7B49, 0x4EF7]]
    forb = ["".join(chr(c) for c in s) for s in terms]
    hits = []
    for p in tree:
        t = btext(p)
        h = [f"TERM{i}" for i, w in enumerate(forb) if w in t]
        if h:
            hits.append({"path": p, "terms": h})
    OUT["forbidden_hits"] = hits
    OUT["module_sources"] = {
        "count_in_module_doc": btext(".normify/modules/mjwarp/render.md").count("[[source]]"),
        "texture_inputs_registered": "src/render/context/texture_inputs.rs" in btext(".normify/modules/mjwarp/render.md"),
    }
    # tests: host reference mirror vs upstream formula
    ts = btext("tests/render_skybox.rs")
    OUT["tests_host_only"] = {
        "lines": ts.count("\n"),
        "has_skybox_uv_reference": "fn skybox_uv(" in ts,
        "uses_safe_div_1e-15": "1.0e-15" in ts,
        "pins_one_sixth_bits": "0x3E2A_AAAB" in ts,
        "pins_t_min_bits": "t_min * (1.0f32 / 6.0f32)" in ts,
        "asserts_slots_81_bounds_1_scalars_7": "assert_eq!((bounds, scalars), (1, 7));" in ts,
        "asserts_define_table": "MK_FLAG_SKYBOX 0x200u" in ts,
    }
    # --------------------------------------------------------------- problems
    problems = []
    if OUT["upstream_render_py"]["sha256"] != "16bfe7edfa73033ef88fb4bbc46364867935f0c527ca53a23ff65c5c40fb3536" \
            or OUT["upstream_render_py"]["bytes"] != 51362 or OUT["upstream_render_py"]["lines"] != 1604:
        problems.append("upstream render.py identity")
    if not all(v for v in checks.values() if isinstance(v, bool)):
        problems.append("upstream sample_skybox checks")
    if not all(v for k, v in OUT["impl_shade_cu"].items() if isinstance(v, bool)):
        problems.append("impl shade.cu")
    if not all(v for k, v in OUT["impl_megakernel_cu"].items() if isinstance(v, bool)):
        problems.append("impl megakernel.cu")
    if not all(v for k, v in OUT["impl_texture_inputs"].items() if isinstance(v, bool)):
        problems.append("impl texture_inputs")
    if OUT["impl_context"]["render_skybox_is_skybox_is_some"] is not True:
        problems.append("render_skybox/skybox consistency")
    if not OUT["impl_lifecycle_gate"]["expr"] or not OUT["impl_uploads_gate"]["expr"]:
        problems.append("two gates")
    if not OUT["megakernel_slots"]["definition_unchanged_in_slice"] or OUT["megakernel_slots"]["declared_len"] != "81":
        problems.append("megakernel slots")
    if not OUT["line_counts_lf"]["all_ok"]:
        problems.append("line counts")
    p = ptx.get("current", {})
    if p.get("sha256") != "0b9474928ab8a7add27e5da7ac0869ee955a75405cd976cd04839f2312e88f3e" or p.get("bytes") != 268275:
        problems.append("current ptx identity")
    b = ptx.get("baseline", {})
    if b.get("sha256") != "97467d835e27107a4a0803678e194f4868fb552707543a544e270cbde8a9bd13" or b.get("bytes") != 264015:
        problems.append("baseline ptx identity")
    if (p.get("tex_level_2d"), b.get("tex_level_2d")) != (2, 1):
        problems.append("tex.level.2d delta")
    if (p.get("and_b32_512"), b.get("and_b32_512")) != (1, 0):
        problems.append("and.b32 512 delta")
    if (p.get("max_f32"), b.get("max_f32")) != (40, 39):
        problems.append("max.f32 delta")
    if (p.get("min_f32"), b.get("min_f32")) != (10, 9):
        problems.append("min.f32 delta")
    if p.get("sqrt") != b.get("sqrt"):
        problems.append("sqrt delta")
    if OUT["kernel_gate_observed"].get("source_sha256") != "e68e3412c1e1809e71100d9e842d5b0f5b886ffdec88d8401bb01c76e812904c":
        problems.append("kernel gate source sha")
    if OUT["kernel_gate_observed"].get("ptx_sha256") != p.get("sha256"):
        problems.append("kernel gate ptx sha vs dump")
    if OUT["evidence_manifest"]["rows"] != 5 or OUT["evidence_manifest"]["bad_committed"] \
            or OUT["evidence_manifest"]["bad_disk"] or OUT["evidence_manifest"]["self_reference"] \
            or OUT["evidence_manifest"]["unlisted"]:
        problems.append("evidence manifest")
    if OUT["forbidden_hits"]:
        problems.append("forbidden words")
    if OUT["module_sources"]["count_in_module_doc"] != 40 or not OUT["module_sources"]["texture_inputs_registered"]:
        problems.append("module sources")
    out["problems"] = problems

    pathlib.Path(r"D:\Rust\mjwarp-rs\target\qa-r02\recompute.json").write_text(
        json.dumps(OUT, ensure_ascii=False, indent=1), encoding="utf-8", newline="\n")
    print("upstream render.py:", OUT["upstream_render_py"])
    print("sample_skybox checks:", all(checks.values()), checks)
    print("call site:", OUT["upstream_call_site_1259_1268"]["has_static_skybox_and_render_rgb"],
          OUT["upstream_call_site_1259_1268"]["elif_render_rgb"])
    print("generated cu:", {k: v for k, v in OUT["generated_cu"].items() if k != "file"})
    print("impl checks:", OUT["impl_shade_cu"], OUT["impl_megakernel_cu"])
    print("host:", OUT["impl_texture_inputs"], OUT["impl_context"])
    print("gates:", OUT["impl_lifecycle_gate"], OUT["impl_uploads_gate"])
    print("slots:", OUT["megakernel_slots"], "device sources:", OUT["device_sources"], "collision504:", OUT["collision_args_bytes"])
    print("lines ok:", OUT["line_counts_lf"]["all_ok"])
    print("ptx:", {k: {kk: vv for kk, vv in v.items() if kk != "sha256"} for k, v in ptx.items()})
    print("gate observed:", OUT["kernel_gate_observed"])
    print("manifest:", OUT["evidence_manifest"])
    print("module sources:", OUT["module_sources"])
    print("tests host-only:", OUT["tests_host_only"])
    print("PROBLEMS", problems)
    return 0


out = OUT
if __name__ == "__main__":
    sys.exit(main())
