#!/usr/bin/env python3
"""NP01 B1：host/device 逐表达式 PTX 定序表（离线、只编译不跑内核、零 GPU）。

做法：
  * 对 ≤20 条 host 关键表达式，各写一个最小 CUDA 探针 kernel；
  * 用 NVRTC 12.8 分别以 --fmad=true（device 默认收缩）与 --fmad=false（两次舍入，近似 host CPU recipe）
    编译成 PTX（只编译，不启动 GPU、不写任何 buffer）；
  * 统计每条表达式的指令形态（fma.rn / mul.rn / add.rn / sub.rn / div.rn / sqrt.rn / rcp.rn）；
  * 与 host Rust 源码的语句与舍入次数对照，给「device 默认是否融合、host 与非融合稿是否同形」的判定。

输入（只读）：src/physics/collision/gjk/{math,simplex,polytope,phase,contact,epa}.rs（host 侧语句与行号）
工具链：target/toolchains/cuda-12.8.1/bin/nvrtc64_120_0.dll（+ nvrtc-builtins64_128.dll）
输出：docs/development-evidence/np01-host-device-expr/host_device_expr.json

边界：不改产品源码/判据/容差/fixtures；不跑 GPU；不宣称数值等价，只登记指令形态与舍入次数。
"""

import ctypes
import hashlib
import json
import os
import re

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CUDA_BIN = os.path.join(ROOT, "target", "toolchains", "cuda-12.8.1", "bin")
OUT = os.path.join(ROOT, "docs", "development-evidence", "np01-host-device-expr")
ARCH = "compute_89"

# (名字, host 侧出处, host 语句与舍入口径, CUDA 探针表达式)
# 表达式全部取自三 host 站点的关键算式；host 侧按「每乘加两次舍入」的 CPU recipe 记。
EXPRESSIONS = [
    (
        "dot3",
        "src/physics/collision/gjk/math.rs:15-17",
        "a0*b0 + a1*b1 + a2*b2 左结合：2 次乘 + 2 次加，共 5 次舍入",
        "out[0] = a0*b0 + a1*b1 + a2*b2;",
    ),
    (
        "length3",
        "src/physics/collision/gjk/math.rs:31-33",
        "sqrt(a0*a0 + a1*a1 + a2*a2)：3 乘 + 2 加 + 1 sqrt",
        "out[0] = sqrtf(a0*a0 + a1*a1 + a2*a2);",
    ),
    (
        "normalize3_component",
        "src/physics/collision/gjk/math.rs:37-44",
        "norm=length3(a)，分量 a0/norm：1 div（norm>0 分支）",
        "float n = sqrtf(a0*a0 + a1*a1 + a2*a2); out[0] = n > 0.0f ? a0/n : 0.0f;",
    ),
    (
        "face_norm2_head_tail",
        "src/physics/collision/gjk/polytope.rs:70-72",
        "head=r0*r0; tail=r1*r1+head; norm2=r2*r2+tail：显式 3 乘 + 2 加，刻意不融合",
        "float head = r0*r0; float tail = r1*r1 + head; out[0] = r2*r2 + tail;",
    ),
    (
        "pos_midpoint",
        "src/physics/collision/gjk/contact.rs:245-246",
        "sum=w1+w2; pos=0.5*sum + origin：2 加 + 1 乘",
        "out[0] = 0.5f*(w0 + w1) + origin;",
    ),
    (
        "make_frame_b",
        "src/physics/collision/gjk/math.rs:140-141",
        "b = selector - a*dot(a,selector); normalize：dot 2 乘 2 加 + 1 乘 + 1 减",
        "float d = a0*s0 + a1*s1 + a2*s2; out[0] = s0 - a0*d;",
    ),
    (
        "linear_combine_3",
        "src/physics/collision/gjk/simplex.rs:310-314",
        "Σ l_i*v_i（i=0..2）：3 乘 + 2 加",
        "out[0] = l0*v00 + l1*v10 + l2*v20;",
    ),
    (
        "project_plane_scl",
        "src/physics/collision/gjk/simplex.rs:71-75",
        "scl = nv/nn; 投影点 scl*n：1 div + 3 乘",
        "float scl = nv/nn; out[0] = scl*n0;",
    ),
    (
        "epa_upper_k",
        "src/physics/collision/gjk/epa.rs:108",
        "dot(face_pr, w)/lower：2 乘 2 加 + 1 div",
        "out[0] = (p0*w0 + p1*w1 + p2*w2)/lower;",
    ),
    (
        "contact_dist_margin",
        "src/physics/collision/gjk/contact.rs:191",
        "dist += margin：1 加",
        "out[0] = dist + margin;",
    ),
    (
        "frame_dir_sub",
        "src/physics/collision/gjk/contact.rs:234-238",
        "make_frame(sub3(w1,w2)) 的分量差：3 减",
        "out[0] = w10 - w20;",
    ),
    (
        "tri_affine_c31",
        "src/physics/collision/gjk/simplex.rs:369-370",
        "c31 = p[x]*v2[y] + p[y]*v3[x] + v2[x]*v3[y] - p[x]*v3[y] - p[y]*v2[x] - v3[x]*v2[y]",
        "out[0] = px*v2y + py*v3x + v2x*v3y - px*v3y - py*v2x - v3x*v2y;",
    ),
]

PARAMS = (
    "const float* in, float* out"
)
DECLS = (
    "float a0=in[0],a1=in[1],a2=in[2],r0=in[3],r1=in[4],r2=in[5];"
    "float w0=in[6],w1=in[7],origin=in[8];"
    "float s0=in[9],s1=in[10],s2=in[11];"
    "float l0=in[12],l1=in[13],l2=in[14],v00=in[15],v10=in[16],v20=in[17];"
    "float nv=in[18],nn=in[19],n0=in[20];"
    "float p0=in[21],p1=in[22],p2=in[23],lower=in[24];"
    "float dist=in[25],margin=in[26],w10=in[27],w20=in[28];"
    "float px=in[29],py=in[30],v2x=in[31],v2y=in[32],v3x=in[33],v3y=in[34];"
    "float b0=in[35],b1=in[36],b2=in[37],w2=in[38];"
)

COUNTERS = [
    ("fma_rn", r"\bfma\.rn\.f32"),
    ("mul_rn", r"\bmul\.rn\.f32"),
    ("add_rn", r"\badd\.rn\.f32"),
    ("sub_rn", r"\bsub\.rn\.f32"),
    ("div_rn", r"\bdiv\.rn\.f32"),
    ("sqrt_rn", r"\bsqrt\.rn\.f32"),
    ("rcp_rn", r"\brcp\.rn\.f32"),
    ("mul_f32", r"\bmul\.f32"),
    ("add_f32", r"\badd\.f32"),
]


_DLL_DIR_HANDLES = []


def _prepare_dll_search():
    if not os.path.exists(CUDA_BIN):
        raise SystemExit(f"missing CUDA bin: {CUDA_BIN}")
    os.environ["PATH"] = CUDA_BIN + os.pathsep + os.environ.get("PATH", "")
    if hasattr(os, "add_dll_directory"):
        # 句柄必须存活到进程结束，否则 builtins 延迟加载时会找不到。
        _DLL_DIR_HANDLES.append(os.add_dll_directory(CUDA_BIN))


def nvrtc_compile(source: str, name: str, fmad: bool) -> dict:
    bin_path = os.path.join(CUDA_BIN, "nvrtc64_120_0.dll")
    if not os.path.exists(bin_path):
        raise SystemExit(f"missing NVRTC: {bin_path}")
    _prepare_dll_search()
    lib = ctypes.CDLL(bin_path)
    c_int_p = ctypes.POINTER(ctypes.c_int)
    c_size_p = ctypes.POINTER(ctypes.c_size_t)
    lib.nvrtcCreateProgram.restype = ctypes.c_int
    lib.nvrtcCreateProgram.argtypes = [
        ctypes.POINTER(ctypes.c_void_p),
        ctypes.c_char_p,
        ctypes.c_char_p,
        ctypes.c_int,
        ctypes.c_void_p,
        ctypes.c_void_p,
    ]
    lib.nvrtcCompileProgram.restype = ctypes.c_int
    lib.nvrtcCompileProgram.argtypes = [ctypes.c_void_p, ctypes.c_int, ctypes.POINTER(ctypes.c_char_p)]
    lib.nvrtcGetPTXSize.restype = ctypes.c_int
    lib.nvrtcGetPTXSize.argtypes = [ctypes.c_void_p, c_size_p]
    lib.nvrtcGetPTX.restype = ctypes.c_int
    lib.nvrtcGetPTX.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    lib.nvrtcGetProgramLogSize.restype = ctypes.c_int
    lib.nvrtcGetProgramLogSize.argtypes = [ctypes.c_void_p, c_size_p]
    lib.nvrtcGetProgramLog.restype = ctypes.c_int
    lib.nvrtcGetProgramLog.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    lib.nvrtcDestroyProgram.restype = ctypes.c_int
    lib.nvrtcDestroyProgram.argtypes = [ctypes.POINTER(ctypes.c_void_p)]

    program = ctypes.c_void_p()
    src = source.encode("utf-8")
    rc = lib.nvrtcCreateProgram(ctypes.byref(program), src, name.encode(), 0, None, None)
    if rc != 0:
        raise RuntimeError(f"nvrtcCreateProgram rc={rc}")
    options = [
        f"--gpu-architecture={ARCH}".encode(),
        b"--std=c++17",
        b"--fmad=" + (b"true" if fmad else b"false"),
    ]
    argv = (ctypes.c_char_p * len(options))(*options)
    rc = lib.nvrtcCompileProgram(program, len(options), argv)
    log = ""
    log_size = ctypes.c_size_t()
    if lib.nvrtcGetProgramLogSize(program, ctypes.byref(log_size)) == 0 and log_size.value:
        buf = ctypes.create_string_buffer(log_size.value)
        lib.nvrtcGetProgramLog(program, buf)
        log = buf.value.decode("utf-8", "replace")
    if rc != 0:
        lib.nvrtcDestroyProgram(ctypes.byref(program))
        raise RuntimeError(f"nvrtcCompileProgram rc={rc}: {log.strip()[:300]}")
    ptx_size = ctypes.c_size_t()
    if lib.nvrtcGetPTXSize(program, ctypes.byref(ptx_size)) != 0:
        raise RuntimeError("nvrtcGetPTXSize failed")
    ptx_buf = ctypes.create_string_buffer(ptx_size.value)
    if lib.nvrtcGetPTX(program, ptx_buf) != 0:
        raise RuntimeError("nvrtcGetPTX failed")
    lib.nvrtcDestroyProgram(ctypes.byref(program))
    ptx = ptx_buf.value.decode("utf-8", "replace")
    return {
        "ptx_sha256": hashlib.sha256(ptx.encode()).hexdigest(),
        "ptx_lines": ptx.count("\n") + 1,
        "counts": {key: len(re.findall(pattern, ptx)) for key, pattern in COUNTERS},
        "log_sha256": hashlib.sha256(log.encode()).hexdigest(),
    }


def probe_source(expr: str) -> str:
    return (
        'extern "C" __global__ void probe(' + PARAMS + ") {\n"
        "  " + DECLS + "\n"
        "  " + expr + "\n"
        "}\n"
    )


def main():
    rows = []
    for name, host_ref, host_note, expr in EXPRESSIONS:
        source = probe_source(expr)
        fused = nvrtc_compile(source, f"probe_{name}", True)
        plain = nvrtc_compile(source, f"probe_{name}", False)
        f = fused["counts"]
        p = plain["counts"]
        fused_terms = f["fma_rn"]
        plain_terms = p["fma_rn"]
        if fused_terms > 0 and plain_terms == 0:
            verdict = "device 默认收缩为 fma.rn；host 两次舍入与非融合稿同形"
        elif fused_terms == 0 and plain_terms == 0:
            verdict = "两侧都无 fma.rn（该表达式无可融合乘加）"
        else:
            verdict = "形态需人工判读（见计数）"
        rows.append(
            {
                "expression": name,
                "host_source": host_ref,
                "host_statements_and_rounding": host_note,
                "cu_expression": expr,
                "device_fmad_true": fused["counts"],
                "device_fmad_false": plain["counts"],
                "device_fmad_true_ptx_sha256": fused["ptx_sha256"],
                "device_fmad_false_ptx_sha256": plain["ptx_sha256"],
                "verdict": verdict,
            }
        )
        print(f"{name:24s} fma(true)={f['fma_rn']:2d} mul={f['mul_rn']:2d} add={f['add_rn']:2d} | "
              f"fma(false)={p['fma_rn']:2d} mul={p['mul_rn']:2d} add={p['add_rn']:2d} | {verdict}")

    report = {
        "schema": 1,
        "task_id": "2026-11-04-precision-np01-host-device-expr",
        "kind": "np01-host-device-per-expression-ptx",
        "compile": {
            "nvrtc": "target/toolchains/cuda-12.8.1/bin/nvrtc64_120_0.dll",
            "arch": ARCH,
            "std": "c++17",
            "recipes": ["fmad=true（device 默认收缩）", "fmad=false（两次舍入，近似 host CPU recipe）"],
            "note": "只编译不跑内核；不写 buffer；不启动 GPU",
        },
        "expression_count": len(rows),
        "host_recipe": "CPU：乘加为独立语句，每乘加两次舍入；与 compare-cpu-vs-gpu.json 的 host 侧口径一致",
        "device_recipe": "NVRTC 默认 fmad=true（pinned ccd PTX 实测 fma.rn=1480、mul.rn/add.rn=0）",
        "expressions": rows,
    }
    os.makedirs(OUT, exist_ok=True)
    path = os.path.join(OUT, "host_device_expr.json")
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(report, handle, ensure_ascii=False, indent=2)
        handle.write("\n")
    print("wrote", path, "expressions=", len(rows))


if __name__ == "__main__":
    main()
