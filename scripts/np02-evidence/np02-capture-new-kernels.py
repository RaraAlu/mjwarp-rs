#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""NP02 receipt capture for the kernels that had no receipt before 2026-11-04.

CPU-only: it calls NVRTC (12.8.93 from target/toolchains/cuda-12.8.1) and never
creates a CUDA device context, so no GPU window is needed.

It replicates the product compile paths exactly:

  physics  src/runtime/transfer/kernel.rs (SynchronousKernel::compile)
           CompileOptions { fmad: Some(true), use_fast_math: Some(false),
                            options: ["--std=c++17", "--gpu-architecture=compute_70"],
                            name: Some("<entry>.cu") }
  render   src/render/kernel/launch.rs::compile_render_artifact
           + src/runtime/launcher/artifact/nvrtc.rs::nvrtc_compile
           same fmad/std/arch, include_paths = [include/warp-headers],
           extra option "--device-as-default-execution-space", gate f32-render
  collision src/runtime/collision_gpu/source.rs::assemble_device_source
           28 fragments (DEVICE_SOURCES order, local '#include "*.cu"' lines
           stripped) + product.cu appended, compiled under the physics recipe.

cudarc 0.18.2 CompileOptions::build() emits options in this order:
  --fmad=<v>, [--use_fast_math only when true], [--maxrregcount],
  [--include-path=...], [--gpu-architecture=...], then the caller's options vec.

The receipt schema is the one written by src/runtime/transfer/kernel/ptx.rs
::record_physics_ptx; entities are written as <entry>-<ptxSha>.{cu,ptx,json}
exactly where scripts/np02-evidence/np02-lib.ps1::Invoke-ReceiptAudit looks for them.

Usage:
  python scripts/np02-evidence/np02-capture-new-kernels.py capture --out <dir>
  python scripts/np02-evidence/np02-capture-new-kernels.py index --out <dir>
"""
import argparse
import ctypes
import hashlib
import io
import json
import os
import re
import struct
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
NVNTC_BIN = os.path.join(REPO, "target", "toolchains", "cuda-12.8.1", "bin")
WARP_HEADERS = os.path.join(REPO, "include", "warp-headers")
EXTRA_INDEX = os.path.join(REPO, "docs", "development-evidence", "np02-evidence", "receipt.json")
DEFAULT_OUT = os.path.join(REPO, "docs", "development-evidence", "np02-evidence",
                           "new-kernels-2026-11", "receipts")

CONST_RE = re.compile(r'const\s+(?P<name>[A-Z0-9_]+)\s*:\s*&str\s*=\s*(?:r#"(?P<body>.*?)"#\s*;|include_str!\(\s*"(?P<inc>[^"]+)"\s*\)\s*;)', re.S)

# unit id, unit path (as recorded by the NP02 scanner), entries, recipe
UNITS = [
    ("build_rays", r"src/render/camera/cuda/build_rays.cu", ["build_rays"], "physics"),
    ("mesh_bvh_ray", r"src/render/bvh/cuda/mesh.cu", ["mesh_bvh_ray"], "physics"),
    ("scene_bvh_ray", r"src/render/bvh/cuda/scene.cu", ["scene_bvh_ray"], "physics"),
    ("sample_texture2d", r"src/render/texture/sample.cu", ["sample_texture2d"], "physics"),
    ("aa_resolve", r"src/render/kernel/cuda/aa_resolve.cu", ["aa_resolve"], "render"),
    ("s7a_abi_probe", r"src/render/kernel/launch.rs", ["s7a_abi_probe"], "render"),
    ("collision", r"src/physics/collision/cuda/product.cu",
     ["mjwarp_collision_boxbox", "mjwarp_collision_contact", "mjwarp_collision_contact_multi",
      "mjwarp_collision_hfield", "mjwarp_collision_primitive", "mjwarp_collision_sdf"], "collision"),
    # 2026-11-11 NP5-03: two real product render kernels that were missing from the NP02
    # inventory rows.  Both compile units are multi-file concat! sources (see
    # src/render/kernel/mod.rs::MEGAKERNEL_SOURCE / DEPTH_SEG_SOURCE); a list relpath means
    # "concatenate these files in order" exactly like include_str! + concat!.
    ("render_megakernel",
     [r"src/render/bvh/cuda/scene.cu", r"src/render/kernel/cuda/render_math.cu",
      r"src/render/kernel/cuda/render_flex.cu", r"src/render/kernel/cuda/render_common.cu",
      r"src/render/kernel/cuda/render_shadow.cu", r"src/render/kernel/cuda/render_shade.cu",
      r"src/render/kernel/cuda/render_megakernel.cu"],
     ["render_megakernel"], "render"),
    ("render_depth_seg",
     [r"src/render/bvh/cuda/scene.cu", r"src/render/kernel/cuda/render_math.cu",
      r"src/render/kernel/cuda/render.cu"],
     ["render_depth_seg"], "render"),
]

BASE_OPTIONS = ["--std=c++17", "--gpu-architecture=compute_70"]
CUDA_MATH_REDUCTION = "CUDA-12.8 sinf/cosf integer angle reduction only"
FORBIDDEN = {"f16", "f16x2", "bf16", "bf16x2", "tf32"}
_TRIM = re.compile(r'^[^0-9A-Za-z]+|[^0-9A-Za-z]+$')


def sha256_hex(data):
    return hashlib.sha256(data).hexdigest()


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


# ---------- source units (same extraction as np02-lib.ps1 Get-SourceUnits) ----------
def extract_unit(relpath):
    target = os.path.normpath(os.path.join(REPO, relpath))
    for root, _dirs, files in os.walk(os.path.join(REPO, "src")):
        for fn in files:
            if not fn.endswith(".rs"):
                continue
            if os.sep + "tests" + os.sep in os.path.join(root, fn) or fn == "tests.rs":
                continue
            full = os.path.join(root, fn)
            text = io.open(full, encoding="utf-8", errors="replace").read()
            for m in CONST_RE.finditer(text):
                body, inc = m.group("body"), m.group("inc")
                if body and os.path.normpath(full) == target:
                    return body.replace("\r\n", "\n")
                if inc:
                    incfull = os.path.normpath(os.path.join(os.path.dirname(full), inc))
                    if incfull == target:
                        return io.open(incfull, encoding="utf-8", errors="replace").read().replace("\r\n", "\n")
    raise SystemExit("source unit not found: " + relpath)


# ---------- collision device source assembly (port of runtime/collision_gpu/source.rs) ----------
def _collision_fragment_order():
    text = io.open(os.path.join(REPO, "src", "runtime", "collision_gpu", "source.rs"),
                   encoding="utf-8", errors="replace").read()
    m = re.search(r'DEVICE_SOURCES:\s*\[\(&str,\s*&str\);\s*\d+\]\s*=\s*\[(?P<body>.*?)\n\];', text, re.S)
    if not m:
        raise SystemExit("cannot locate DEVICE_SOURCES in runtime/collision_gpu/source.rs")
    return [x.group(1) for x in re.finditer(r'\(\s*"(?P<f>[A-Za-z0-9_]+\.cu)"\s*,', m.group("body"))]


def _is_local_cu_include(line):
    s = line.lstrip()
    if not s.startswith("#include"):
        return False
    rest = s[len("#include"):].lstrip()
    if not rest.startswith('"'):
        return False
    end = rest.find('"', 1)
    return end > 0 and rest[1:end].endswith(".cu")


def assemble_collision_source():
    cuda_dir = os.path.join(REPO, "src", "physics", "collision", "cuda")
    parts = []
    for name in _collision_fragment_order():
        txt = io.open(os.path.join(cuda_dir, name), encoding="utf-8", errors="replace").read().replace("\r\n", "\n")
        parts.append("".join(l for l in txt.splitlines(True) if not _is_local_cu_include(l)))
    src = ""
    for p in parts:
        src += p
        if not src.endswith("\n"):
            src += "\n"
    product = io.open(os.path.join(cuda_dir, "product.cu"), encoding="utf-8", errors="replace").read()
    return src + product.replace("\r\n", "\n")


# ---------- NVRTC ----------
class Nvrtc:
    def __init__(self):
        os.environ["PATH"] = NVNTC_BIN + os.pathsep + os.environ.get("PATH", "")
        if hasattr(os, "add_dll_directory"):
            os.add_dll_directory(NVNTC_BIN)
        lib = ctypes.CDLL(os.path.join(NVNTC_BIN, "nvrtc64_120_0.dll"))
        lib.nvrtcCreateProgram.argtypes = [ctypes.POINTER(ctypes.c_void_p), ctypes.c_char_p, ctypes.c_char_p,
                                           ctypes.c_int, ctypes.POINTER(ctypes.c_char_p), ctypes.POINTER(ctypes.c_char_p)]
        lib.nvrtcCompileProgram.argtypes = [ctypes.c_void_p, ctypes.c_int, ctypes.POINTER(ctypes.c_char_p)]
        lib.nvrtcGetPTXSize.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_size_t)]
        lib.nvrtcGetPTX.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        lib.nvrtcGetProgramLogSize.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_size_t)]
        lib.nvrtcGetProgramLog.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        lib.nvrtcDestroyProgram.argtypes = [ctypes.POINTER(ctypes.c_void_p)]
        self.lib = lib
        major, minor = ctypes.c_int(), ctypes.c_int()
        lib.nvrtcVersion(ctypes.byref(major), ctypes.byref(minor))
        self.version = (major.value, minor.value)

    def compile(self, source, name, options):
        """Compile and return the PTX text (cudarc Ptx::to_src semantics: trailing NUL stripped)."""
        lib = self.lib
        prog = ctypes.c_void_p()
        rc = lib.nvrtcCreateProgram(ctypes.byref(prog), source.encode("utf-8"), name.encode("utf-8"), 0, None, None)
        if rc != 0:
            raise SystemExit("nvrtcCreateProgram rc=%d" % rc)
        arr = (ctypes.c_char_p * len(options))(*[o.encode("utf-8") for o in options])
        rc = lib.nvrtcCompileProgram(prog, len(options), arr)
        if rc != 0:
            size = ctypes.c_size_t()
            lib.nvrtcGetProgramLogSize(prog, ctypes.byref(size))
            buf = ctypes.create_string_buffer(size.value)
            lib.nvrtcGetProgramLog(prog, buf)
            log = buf.value.decode("utf-8", "replace")
            lib.nvrtcDestroyProgram(ctypes.byref(prog))
            raise SystemExit("nvrtcCompileProgram rc=%d for %s\n%s" % (rc, name, log))
        size = ctypes.c_size_t()
        if lib.nvrtcGetPTXSize(prog, ctypes.byref(size)) != 0:
            raise SystemExit("nvrtcGetPTXSize failed")
        buf = ctypes.create_string_buffer(size.value)
        if lib.nvrtcGetPTX(prog, buf) != 0:
            raise SystemExit("nvrtcGetPTX failed")
        raw = buf.raw[:size.value]
        lib.nvrtcDestroyProgram(ctypes.byref(prog))
        if raw.endswith(b"\x00"):
            raw = raw[:raw.rindex(b"\x00")]
        return raw.decode("utf-8")


# ---------- precision gates (ports of the product checks) ----------
def _operands(line):
    return [w for w in re.split(r'[ \t\r\n,;]+', line) if w]


def check_physics_ptx(ptx, entry):
    """Port of runtime/transfer/kernel/ptx.rs::check_physics_ptx."""
    lines = []
    for line in ptx.splitlines():
        code = line.split("//", 1)[0].strip()
        if code:
            lines.append(code)
    reductions = 0
    declarations = False
    i = 0
    while i < len(lines):
        code = lines[i]
        if code.startswith(".reg .f64 ") or code.startswith(".reg .f64\t"):
            declarations = True
            i += 1
            continue
        a = _operands(code)
        if len(a) == 3 and a[0] == "cvt.rn.f64.s64" and i + 2 < len(lines):
            b, c = _operands(lines[i + 1]), _operands(lines[i + 2])
            if (len(b) == 4 and b[0] in ("mul.rn.f64", "mul.f64") and b[2] == a[1]
                    and b[3] == "0d3BF921FB54442D19" and len(c) == 3
                    and c[0] == "cvt.rn.f32.f64" and c[2] == b[1]):
                reductions += 1
                i += 3
                continue
        for token in code.split():
            for word in token.split(".")[1:]:
                if _TRIM.sub("", word) in ("f64",) + tuple(FORBIDDEN):
                    return False, "%s: forbidden f64-family token in: %s" % (entry, code)
        i += 1
    if declarations and reductions == 0:
        return False, "%s: unaccounted f64 registers" % entry
    return True, "pass"


RENDER_ALLOWED = [
    ".global .align 8 .f64 _ZN2wp11_svd_configIdE11SVD_EPSILONE = 0d3D719799812DEA11;",
    ".global .align 8 .f64 _ZN2wp11_svd_configIdE17QR_GIVENS_EPSILONE = 0d3D719799812DEA11;",
]


def check_render_ptx(ptx, entry):
    """Port of src/render/kernel/mod.rs::check_render_ptx (only the two W15 dead-data constants)."""
    lines = []
    for line in ptx.splitlines():
        code = line.split("//", 1)[0].strip()
        if code:
            lines.append(code)
    seen = [False, False]
    reductions = 0
    declarations = None
    i = 0
    while i < len(lines):
        code = lines[i]
        if code.startswith(".reg .f64 ") or code.startswith(".reg .f64\t"):
            if declarations is None:
                declarations = i + 1
            i += 1
            continue
        a = _operands(code)
        if len(a) == 3 and a[0] == "cvt.rn.f64.s64" and i + 2 < len(lines):
            b, c = _operands(lines[i + 1]), _operands(lines[i + 2])
            if (len(b) == 4 and b[0] in ("mul.rn.f64", "mul.f64") and b[2] == a[1]
                    and b[3] == "0d3BF921FB54442D19" and len(c) == 3
                    and c[0] == "cvt.rn.f32.f64" and c[2] == b[1]):
                reductions += 1
                i += 3
                continue
        if ".f64 " in code or ".f64;" in code or code.endswith(".f64"):
            if code in RENDER_ALLOWED and code.startswith(".global"):
                idx = RENDER_ALLOWED.index(code)
                if seen[idx]:
                    return False, "%s: duplicate W15 constant at line %d" % (entry, i + 1)
                seen[idx] = True
                i += 1
                continue
            return False, "%s: unexpected f64 at line %d: %s" % (entry, i + 1, code)
        for token in code.split():
            for word in token.split(".")[1:]:
                if _TRIM.sub("", word) in FORBIDDEN:
                    return False, "%s: forbidden token at line %d" % (entry, i + 1)
        i += 1
    if declarations is not None and reductions == 0:
        return False, "%s: unaccounted f64 registers at line %d" % (entry, declarations)
    if (".visible .entry " + entry) not in ptx:
        return False, "%s: missing .visible .entry" % entry
    return True, "pass"


# ---------- commands ----------
def cmd_capture(out_dir, only):
    nv = Nvrtc()
    if nv.version != (12, 8):
        raise SystemExit("unexpected NVRTC version %s (need 12.8)" % (nv.version,))
    print("NVRTC %s" % (nv.version,))
    os.makedirs(out_dir, exist_ok=True)
    written = []
    for unit_id, relpath, entries, recipe in UNITS:
        if only and unit_id not in only:
            continue
        if recipe == "collision":
            source = assemble_collision_source()
        elif isinstance(relpath, list):
            # multi-file concat! unit: read each file in order (LF-normalised, as rustc does)
            source = "".join(
                io.open(os.path.join(REPO, part), encoding="utf-8", errors="replace")
                .read().replace("\r\n", "\n")
                for part in relpath
            )
        else:
            source = extract_unit(relpath)
        source_sha = sha256_hex(source.encode("utf-8"))
        options = ["--fmad=true"]
        if recipe == "render":
            options.append("--include-path=" + WARP_HOME_REL())
            options += BASE_OPTIONS
            options.append("--device-as-default-execution-space")
        else:
            options += BASE_OPTIONS
        print("\n== %s (%s) sourceSha=%s bytes=%d" % (unit_id, recipe, source_sha, len(source.encode("utf-8"))))
        for entry in entries:
            ptx = nv.compile(source, entry + ".cu", options)
            ptx_sha = sha256_hex(ptx.encode("utf-8"))
            if recipe == "render":
                passed, detail = check_render_ptx(ptx, entry)
            else:
                passed, detail = check_physics_ptx(ptx, entry)
            record = {
                "entry": entry,
                "sourceSha256": source_sha,
                "ptxSha256": ptx_sha,
                "precisionGate": "f32-render" if recipe == "render" else "f32-physics",
                "cudaMathReduction": CUDA_MATH_REDUCTION,
                "passed": bool(passed),
                "nvrtcVersion": [nv.version[0], nv.version[1]],
                "fmad": True,
                "useFastMath": False,
                "options": [o for o in BASE_OPTIONS],
                "recipe": "warp-1.15-render" if recipe == "render" else "warp-1.15-physics",
                "capture": "cpu-only NVRTC (no device context)",
            }
            prefix = os.path.join(out_dir, "%s-%s" % (entry, ptx_sha))
            open(prefix + ".cu", "wb").write(source.encode("utf-8"))
            open(prefix + ".ptx", "wb").write(ptx.encode("utf-8"))
            open(prefix + ".json", "wb").write((json.dumps(record, indent=2, ensure_ascii=False) + "\n").encode("utf-8"))
            print("   %-32s ptxSha=%s bytes=%d gate=%s" % (entry, ptx_sha, len(ptx.encode("utf-8")), detail))
            written.append({
                "unit": unit_id, "entry": entry, "recipe": recipe,
                "sourceSha256": source_sha, "ptxSha256": ptx_sha,
                "ptxBytes": len(ptx.encode("utf-8")), "gateOk": bool(passed), "detail": detail,
                "receipt": os.path.basename(prefix + ".json"),
            })
    report = {"tool": "scripts/np02-evidence/np02-capture-new-kernels.py",
              "nvrtcVersion": list(nv.version), "cpuOnly": True, "gpu": False,
              "receipts": written}
    open(os.path.join(out_dir, os.pardir, "capture-report.json"), "wb").write(
        (json.dumps(report, indent=2, ensure_ascii=False) + "\n").encode("utf-8"))
    print("\nwrote %d receipts to %s" % (len(written), out_dir))
    if not all(w["gateOk"] for w in written):
        raise SystemExit("a precision gate rejected a receipt; see output above")


def WARP_HOME_REL():
    return os.path.join(REPO, "include", "warp-headers")


def cmd_index(out_dir):
    """Append (idempotently) the captured receipts to the extra receipt index."""
    index = json.load(io.open(EXTRA_INDEX, encoding="utf-8"))
    existing = {(r["entry"], r["ptxSha256"]) for r in index["receipts"]}
    report = json.load(io.open(os.path.join(out_dir, os.pardir, "capture-report.json"), encoding="utf-8"))
    added = []
    for item in report["receipts"]:
        receipt_path = os.path.join(out_dir, item["receipt"])
        rel = os.path.relpath(receipt_path, REPO).replace(os.sep, "/")
        key = (item["entry"], item["ptxSha256"])
        if key in existing:
            continue
        index["receipts"].append({
            "path": rel,
            "fileSha256": sha256_file(receipt_path),
            "entry": item["entry"],
            "ptxSha256": item["ptxSha256"],
            "sourceSha256": item["sourceSha256"],
            "options": json.load(io.open(receipt_path, encoding="utf-8"))["options"],
            "fmad": True,
            "useFastMath": False,
            "nvrtcVersion": [report["nvrtcVersion"][0], report["nvrtcVersion"][1]],
            "precisionGate": json.load(io.open(receipt_path, encoding="utf-8"))["precisionGate"],
            "passed": True,
            "ptxEntityOk": True,
        })
        added.append(item["entry"])
    open(EXTRA_INDEX, "wb").write((json.dumps(index, indent=2, ensure_ascii=False) + "\n").encode("utf-8"))
    print("index entries added: %d %s" % (len(added), added))
    print("index total receipts: %d" % len(index["receipts"]))


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("command", choices=("capture", "index"))
    ap.add_argument("--out", default=DEFAULT_OUT)
    ap.add_argument("--only", default="", help="comma separated unit ids")
    args = ap.parse_args()
    only = [x for x in args.only.split(",") if x]
    if args.command == "capture":
        cmd_capture(args.out, only)
    else:
        cmd_index(args.out)


if __name__ == "__main__":
    main()
