"""R06-S3a 离线 NVRTC 预检：把 `MEGAKERNEL_SOURCE` 的拼接源码用与产品相同的 NVRTC
recipe 编译成 PTX（只编译，不创建 CUDA 上下文、不发射内核、不占 GPU），并对 PTX 跑
`check_render_ptx` 的等价规则（W15 两条 `_svd_config<double>` 常量精确放行、sin/cos
归约三元组、禁止 f16/bf16/tf32、入口存在）。

用法（仓库根目录；pinned 3.12 venv 的 python，纯 ctypes 载入本机 nvrtc DLL）：
  target/mjwarp-parallel/venv/Scripts/python.exe \
      docs/development-evidence/render-bvh/r06-s3a-flex-device/tools/nvrtc_precheck.py \
      --repo . --cuda target/toolchains/cuda-12.8.1 --out target/r06-s3a-nvrtc-precheck

recipe 依据：`src/runtime/launcher/recipe.rs::CompileRecipe::WARP_1_15_RENDER`
（fmad=true、fast_math=false、std=c++17、compute_70、无 defines）+ 渲染头树
`include/warp-headers`（`src/render/kernel/launch.rs::W15_HEADERS_DIR`）。
"""

import argparse
import ctypes
import hashlib
import os
import pathlib
import sys

SOURCE_ORDER = [
    "src/render/bvh/cuda/scene.cu",
    "src/render/kernel/cuda/render_math.cu",
    "src/render/kernel/cuda/render_flex.cu",
    "src/render/kernel/cuda/render_common.cu",
    "src/render/kernel/cuda/render_shadow.cu",
    "src/render/kernel/cuda/render_shade.cu",
    "src/render/kernel/cuda/render_megakernel.cu",
]
ENTRY = "render_megakernel"
SVD_CONSTANTS = (
    ".global .align 8 .f64 _ZN2wp11_svd_configIdE11SVD_EPSILONE = 0d3D719799812DEA11;",
    ".global .align 8 .f64 _ZN2wp11_svd_configIdE17QR_GIVENS_EPSILONE = 0d3D719799812DEA11;",
)
SIN_COS_CONSTANT = "0d3BF921FB54442D19"


def nvrtc(api, name, argtypes, restype=ctypes.c_int):
    fn = getattr(api, name)
    fn.argtypes = argtypes
    fn.restype = restype
    return fn


def compile_ptx(cuda_dir, source, entry, include_dirs):
    dll = cuda_dir / "bin" / "nvrtc64_120_0.dll"
    if not dll.is_file():
        raise SystemExit(f"缺少 NVRTC DLL：{dll}")
    # NVRTC 需要同目录的 `nvrtc-builtins64_128.dll`；显式加入 DLL 搜索路径并预加载。
    os.environ["PATH"] = str(cuda_dir / "bin") + os.pathsep + os.environ.get("PATH", "")
    os.add_dll_directory(str(cuda_dir / "bin"))
    ctypes.WinDLL(str(cuda_dir / "bin" / "nvrtc-builtins64_128.dll"))
    api = ctypes.WinDLL(str(dll))
    nvrtcCreateProgram = nvrtc(api, "nvrtcCreateProgram", [ctypes.c_void_p, ctypes.c_char_p,
                             ctypes.c_char_p, ctypes.c_int, ctypes.c_void_p, ctypes.c_void_p])
    nvrtcCompileProgram = nvrtc(api, "nvrtcCompileProgram", [ctypes.c_void_p, ctypes.c_int,
                            ctypes.POINTER(ctypes.c_char_p)])
    nvrtcGetPTXSize = nvrtc(api, "nvrtcGetPTXSize", [ctypes.c_void_p, ctypes.POINTER(ctypes.c_size_t)])
    nvrtcGetPTX = nvrtc(api, "nvrtcGetPTX", [ctypes.c_void_p, ctypes.c_char_p])
    nvrtcGetProgramLogSize = nvrtc(api, "nvrtcGetProgramLogSize",
                                   [ctypes.c_void_p, ctypes.POINTER(ctypes.c_size_t)])
    nvrtcGetProgramLog = nvrtc(api, "nvrtcGetProgramLog", [ctypes.c_void_p, ctypes.c_char_p])
    nvrtcGetErrorString = nvrtc(api, "nvrtcGetErrorString", [ctypes.c_int], ctypes.c_char_p)
    nvrtcVersion = nvrtc(api, "nvrtcVersion", [ctypes.POINTER(ctypes.c_int)] * 2)

    major, minor = ctypes.c_int(), ctypes.c_int()
    nvrtcVersion(ctypes.byref(major), ctypes.byref(minor))

    program = ctypes.c_void_p()
    status = nvrtcCreateProgram(ctypes.byref(program), source.encode("utf-8"),
                                f"{entry}.cu".encode(), 0, None, None)
    if status != 0:
        raise SystemExit(f"nvrtcCreateProgram 失败：{nvrtcGetErrorString(status).decode()}")

    options = [
        "--std=c++17",
        "--gpu-architecture=compute_70",
        "--fmad=true",
        # `src/runtime/launcher/artifact/nvrtc.rs` L92：产品编译无条件追加该选项
        # （W14/W15 头树里有未标注执行空间的 inline 函数）。
        "--device-as-default-execution-space",
        # 产品配方 `fast_math=false`：NVRTC 默认即关闭，`--use_fast_math` 不带参数表示开启，
        # 因此这里不传该选项（与 `CompileOptions { use_fast_math: Some(false) }` 等价）。
    ] + [f"-I{path}" for path in include_dirs]
    c_options = (ctypes.c_char_p * len(options))(*[opt.encode() for opt in options])
    status = nvrtcCompileProgram(program, len(options), c_options)

    log_size = ctypes.c_size_t()
    nvrtcGetProgramLogSize(program, ctypes.byref(log_size))
    log = ctypes.create_string_buffer(log_size.value)
    nvrtcGetProgramLog(program, log)
    log_text = log.value.decode("utf-8", errors="replace")
    if status != 0:
        raise SystemExit(f"NVRTC 编译失败（{nvrtcGetErrorString(status).decode()}）：\n{log_text}")

    ptx_size = ctypes.c_size_t()
    nvrtcGetPTXSize(program, ctypes.byref(ptx_size))
    ptx = ctypes.create_string_buffer(ptx_size.value)
    nvrtcGetPTX(program, ptx)
    return major.value, minor.value, log_text, ptx.value.decode("utf-8")


def check_ptx(ptx, entry):
    """`src/render/kernel/mod.rs::check_render_ptx` 的等价规则（证据侧独立复算）。"""
    lines = [line.split("//")[0].strip() for line in ptx.splitlines()]
    lines = [line for line in lines if line]
    allowed_seen = [0, 0]
    reduction_count = 0
    declarations = None
    index = 0
    while index < len(lines):
        code = lines[index]
        if code.startswith(".reg .f64 ") or code.startswith(".reg .f64\t"):
            declarations = declarations if declarations is not None else index + 1
            index += 1
            continue
        words = code.replace(",", " ").replace(";", " ").split()
        if len(words) == 3 and words[0] == "cvt.rn.f64.s64" and index + 2 < len(lines):
            second = lines[index + 1].replace(",", " ").replace(";", " ").split()
            third = lines[index + 2].replace(",", " ").replace(";", " ").split()
            if (len(second) == 4 and second[0] in ("mul.rn.f64", "mul.f64")
                    and second[2] == words[1] and second[3] == SIN_COS_CONSTANT
                    and len(third) == 3 and third[0] == "cvt.rn.f32.f64" and third[2] == second[1]):
                reduction_count += 1
                index += 3
                continue
        if ".f64 " in code or code.endswith(".f64"):
            if code.startswith(".global"):
                if code in SVD_CONSTANTS:
                    allowed_seen[SVD_CONSTANTS.index(code)] += 1
                    index += 1
                    continue
                raise SystemExit(f"PTX 门禁：未放行的 f64 全局声明：{code}")
            raise SystemExit(f"PTX 门禁：未解释的 f64 行：{code}")
        for word in code.split():
            token = word.split(".")[1].strip(";") if "." in word else ""
            if token in ("f16", "f16x2", "bf16", "bf16x2", "tf32"):
                raise SystemExit(f"PTX 门禁：禁止的精度 token：{code}")
        index += 1
    if declarations is not None and reduction_count == 0:
        raise SystemExit("PTX 门禁：.reg .f64 没有 sin/cos 归约三元组解释")
    if allowed_seen != [1, 1]:
        raise SystemExit(f"PTX 门禁：W15 两条 f64 常量出现次数异常：{allowed_seen}")
    if f".visible .entry {entry}" not in ptx:
        raise SystemExit(f"PTX 门禁：缺少入口 {entry}")
    return {
        "f64_globals": allowed_seen,
        "sin_cos_reductions": reduction_count,
        "f16_bf16_tf32": 0,
        "entry": entry,
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", default=".")
    parser.add_argument("--cuda", required=True)
    parser.add_argument("--out", required=True)
    args = parser.parse_args()
    repo = pathlib.Path(args.repo).resolve()
    cuda_dir = (repo / args.cuda).resolve()
    out = (repo / args.out).resolve()
    out.parent.mkdir(parents=True, exist_ok=True)

    pieces = []
    for relative in SOURCE_ORDER:
        path = repo / relative
        pieces.append(f"// ---- {relative} ----\n" + path.read_text(encoding="utf-8"))
    source = "".join(pieces)
    source_sha = hashlib.sha256(source.encode("utf-8")).hexdigest()

    major, minor, log_text, ptx = compile_ptx(
        cuda_dir, source, ENTRY, [str(repo / "include" / "warp-headers")])
    checks = check_ptx(ptx, ENTRY)
    ptx_sha = hashlib.sha256(ptx.encode("utf-8")).hexdigest()
    (out.with_suffix(".ptx")).write_text(ptx, encoding="utf-8")
    report = [
        "R06-S3a offline NVRTC precheck (compile only; no CUDA context / no kernel launch)",
        f"nvrtc={major}.{minor}",
        f"options=--std=c++17 --gpu-architecture=compute_70 --fmad=true --use_fast_math=false "
        f"-Iinclude/warp-headers",
        f"entry={ENTRY}",
        f"source_files={len(SOURCE_ORDER)}",
        f"source_concat_sha256={source_sha}",
        f"ptx_bytes={len(ptx.encode('utf-8'))}",
        f"ptx_sha256={ptx_sha}",
        f"gate_checks={checks}",
        f"compile_log_bytes={len(log_text.encode('utf-8'))}",
    ]
    (out.with_suffix(".txt")).write_text("\n".join(report) + "\n", encoding="utf-8")
    print("\n".join(report))
    if log_text.strip():
        (out.with_suffix(".nvrtc-log.txt")).write_text(log_text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    sys.exit(main())
