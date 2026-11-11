# -*- coding: utf-8 -*-
r"""RE-0 offline NVRTC compile with an EXPLICIT flag list (no driver, no GPU).

Mirrors warp/native/warp.cu option assembly (warp 1.15.0) so reference PTX can be
regenerated. Usage:
  python re0_compile.py --dll <nvrtc64_120_0.dll> --src <x.cu> --out <x.ptx> \
      --arch sm_89 --inc <include/warp-headers> [--fast-math] [--profile warp|min]
"""
import argparse
import ctypes
import hashlib
import json
import os
import pathlib
import sys


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--dll", required=True)
    ap.add_argument("--src", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--arch", default="sm_89")
    ap.add_argument("--inc", action="append", default=[])
    ap.add_argument("--fast-math", action="store_true")
    ap.add_argument("--profile", default="warp", choices=["warp", "min"])
    ap.add_argument("--extra", action="append", default=[])
    args = ap.parse_args()

    dll = pathlib.Path(args.dll).resolve()
    os.add_dll_directory(str(dll.parent))
    os.environ["PATH"] = str(dll.parent) + os.pathsep + os.environ.get("PATH", "")
    src = pathlib.Path(args.src).read_bytes()
    out = pathlib.Path(args.out)

    opts = []
    if args.profile == "warp":
        # warp 1.15.0: optimization_level=None -> default branch -> --Ofast-compile=0 (full opt)
        opts += [f"--gpu-architecture={args.arch}", "--std=c++17", "--Ofast-compile=0",
                 "--define-macro=NDEBUG", "--fmad=true"]
        if args.fast_math:
            opts += ["--use_fast_math"]
        opts += ["--device-as-default-execution-space", "--extra-device-vectorization",
                 "--restrict", "--diag-suppress=177,550"]
    elif args.profile == "warp-fastcompile":
        opts += [f"--gpu-architecture={args.arch}", "--std=c++17", "--Ofast-compile=max",
                 "--define-macro=NDEBUG", "--fmad=true"]
        if args.fast_math:
            opts += ["--use_fast_math"]
        opts += ["--device-as-default-execution-space", "--extra-device-vectorization",
                 "--restrict", "--diag-suppress=177,550"]
    else:
        opts += [f"--gpu-architecture={args.arch}", "--std=c++17", "--fmad=true"]
        if args.fast_math:
            opts += ["--use_fast_math"]
    opts += [f"--include-path={pathlib.Path(p).resolve()}" for p in args.inc]
    opts += list(args.extra)

    lib = ctypes.WinDLL(str(dll))
    lib.nvrtcVersion.argtypes = [ctypes.POINTER(ctypes.c_int)] * 2
    lib.nvrtcVersion.restype = ctypes.c_int
    major, minor = ctypes.c_int(), ctypes.c_int()
    lib.nvrtcVersion(ctypes.byref(major), ctypes.byref(minor))
    ver = f"{major.value}.{minor.value}"

    lib.nvrtcCreateProgram.argtypes = [ctypes.POINTER(ctypes.c_void_p), ctypes.c_char_p,
                                       ctypes.c_char_p, ctypes.c_int,
                                       ctypes.POINTER(ctypes.c_char_p),
                                       ctypes.POINTER(ctypes.c_char_p)]
    lib.nvrtcCreateProgram.restype = ctypes.c_int
    lib.nvrtcCompileProgram.argtypes = [ctypes.c_void_p, ctypes.c_int,
                                        ctypes.POINTER(ctypes.c_char_p)]
    lib.nvrtcCompileProgram.restype = ctypes.c_int
    for fn in ("nvrtcGetProgramLogSize", "nvrtcGetPTXSize"):
        getattr(lib, fn).argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_size_t)]
        getattr(lib, fn).restype = ctypes.c_int
    lib.nvrtcGetProgramLog.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    lib.nvrtcGetProgramLog.restype = ctypes.c_int
    lib.nvrtcGetPTX.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    lib.nvrtcGetPTX.restype = ctypes.c_int
    lib.nvrtcDestroyProgram.argtypes = [ctypes.POINTER(ctypes.c_void_p)]
    lib.nvrtcDestroyProgram.restype = ctypes.c_int

    prog = ctypes.c_void_p()
    name = pathlib.Path(args.src).name.encode()
    rc = lib.nvrtcCreateProgram(ctypes.byref(prog), src, name, 0, None, None)
    if rc != 0:
        print("nvrtcCreateProgram failed", rc)
        return 1
    argv = (ctypes.c_char_p * len(opts))(*[o.encode() for o in opts])
    rc = lib.nvrtcCompileProgram(prog, len(opts), argv)
    logsz = ctypes.c_size_t()
    lib.nvrtcGetProgramLogSize(prog, ctypes.byref(logsz))
    log = ctypes.create_string_buffer(logsz.value)
    lib.nvrtcGetProgramLog(prog, log)
    text = log.value.decode("utf-8", "replace")
    if rc != 0:
        print("COMPILE FAILED", rc)
        print(text[-4000:])
        return 1
    ptxsz = ctypes.c_size_t()
    lib.nvrtcGetPTXSize(prog, ctypes.byref(ptxsz))
    buf = ctypes.create_string_buffer(ptxsz.value)
    lib.nvrtcGetPTX(prog, buf)
    data = buf.raw[:ptxsz.value]
    lib.nvrtcDestroyProgram(ctypes.byref(prog))
    out.write_bytes(data)
    meta = {"src": str(args.src), "out": str(out), "nvrtc": ver, "options": opts,
            "ptx_sha256": hashlib.sha256(data).hexdigest(), "ptx_bytes": len(data),
            "warnings_tail": text[-2000:]}
    out.with_suffix(out.suffix + ".meta.json").write_text(
        json.dumps(meta, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print("wrote", out, len(data), hashlib.sha256(data).hexdigest()[:16], "nvrtc", ver)
    return 0


if __name__ == "__main__":
    sys.exit(main())
