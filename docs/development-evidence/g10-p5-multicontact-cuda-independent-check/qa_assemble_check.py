"""QA 独立复算：16 段设备源拼装 + NVRTC PTX（不创建 CUDA 上下文、不跑 GPU）。

不复用产品的 `assemble_device_source()`：自己按 `source.rs` 的 `DEVICE_SOURCES` 顺序
读取 16 个 `.cu`、删除本地 `#include "*.cu"` 行、末尾补 `product.cu`，再独立计算
len/sha256；然后用 ctypes 直接调 NVRTC（选项与产品一致：
`--fmad=true --std=c++17 --gpu-architecture=sm_89`）得到 PTX 长度与入口名。

用法：
  python qa_assemble_check.py            # 打印 JSON 结果
  python qa_assemble_check.py --json out # 写入文件
"""

import argparse
import hashlib
import json
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
CUDA_DIR = ROOT / "src/physics/collision/cuda"
SOURCE_RS = ROOT / "src/runtime/collision_gpu/source.rs"
NVRTC_DIR = ROOT / "target/toolchains/cuda-12.8.1/bin"

INCLUDE_RE = re.compile(r'^\s*#include\s*"([^"]*\.cu)"')


def read_text(rev, relpath):
    """读文件内容：rev=None 读工作树，否则 `git show <rev>:<relpath>`（钉住被复核提交）。"""
    if rev is None:
        return (ROOT / relpath).read_text(encoding="utf-8")
    proc = subprocess.run(["git", "-C", str(ROOT), "show", f"{rev}:{relpath}"],
                          capture_output=True, check=True)
    return proc.stdout.decode("utf-8")


def device_source_names(rev=None):
    """从 source.rs 里解析 DEVICE_SOURCES 的 (name, CONST) 对。"""
    text = read_text(rev, "src/runtime/collision_gpu/source.rs")
    block = re.search(r"DEVICE_SOURCES:\s*\[\(&str, &str\);\s*(\d+)\]\s*=\s*\[(.*?)\];",
                      text, re.S)
    if not block:
        raise SystemExit("未能解析 DEVICE_SOURCES")
    count = int(block.group(1))
    names = re.findall(r'\("([^"]+)",\s*[A-Z_]+\)', block.group(2))
    if len(names) != count:
        raise SystemExit(f"DEVICE_SOURCES 解析数量不符：{len(names)} != {count}")
    return names


def strip_local_includes(text: str) -> str:
    return "".join(
        line for line in text.splitlines(keepends=True)
        if not INCLUDE_RE.match(line)
    )


def assemble(rev=None):
    names = device_source_names(rev)
    out = []
    per_piece = []
    for name in names:
        raw = read_text(rev, f"src/physics/collision/cuda/{name}")
        stripped = strip_local_includes(raw)
        per_piece.append({
            "name": name,
            "raw_bytes": len(raw.encode("utf-8")),
            "stripped_bytes": len(stripped.encode("utf-8")),
            "sha256_stripped": hashlib.sha256(stripped.encode("utf-8")).hexdigest(),
        })
        out.append(stripped)
    source = "".join(out)
    product = read_text(rev, "src/physics/collision/cuda/product.cu")
    source += product
    return names, per_piece, source, product


def nvrtc_ptx(source: str):
    import ctypes
    import os

    # nvrtc 运行期还会按 PATH 找 `nvrtc-builtins64_128.dll`；与产品窗口同一 PATH 前置口径。
    os.environ["PATH"] = str(NVRTC_DIR) + os.pathsep + os.environ.get("PATH", "")
    lib_path = NVRTC_DIR / "nvrtc64_120_0.dll"
    if not lib_path.exists():
        raise SystemExit(f"NVRTC 不存在：{lib_path}")
    nvrtc = ctypes.CDLL(str(lib_path))
    nvrtc.nvrtcCreateProgram.restype = ctypes.c_int
    prog = ctypes.c_void_p()
    src_buf = ctypes.create_string_buffer(source.encode("utf-8"))
    name_buf = ctypes.create_string_buffer(b"qa_primitive_check.cu")
    rc = nvrtc.nvrtcCreateProgram(
        ctypes.byref(prog), src_buf, name_buf, 0, None, None)
    if rc != 0:
        raise SystemExit(f"nvrtcCreateProgram 失败 rc={rc}")
    opts = [b"--fmad=true", b"--std=c++17", b"--gpu-architecture=sm_89"]
    arr = (ctypes.c_char_p * len(opts))(*opts)
    rc = nvrtc.nvrtcCompileProgram(prog, len(opts), arr)
    if rc != 0:
        size = ctypes.c_size_t()
        nvrtc.nvrtcGetProgramLogSize(prog, ctypes.byref(size))
        log = ctypes.create_string_buffer(size.value)
        nvrtc.nvrtcGetProgramLog(prog, log)
        raise SystemExit(f"nvrtcCompileProgram 失败 rc={rc}\n{log.value.decode('utf-8', 'replace')}")
    size = ctypes.c_size_t()
    rc = nvrtc.nvrtcGetPTXSize(prog, ctypes.byref(size))
    if rc != 0:
        raise SystemExit(f"nvrtcGetPTXSize 失败 rc={rc}")
    ptx = ctypes.create_string_buffer(size.value)
    rc = nvrtc.nvrtcGetPTX(prog, ptx)
    if rc != 0:
        raise SystemExit(f"nvrtcGetPTX 失败 rc={rc}")
    nvrtc.nvrtcDestroyProgram(ctypes.byref(prog))
    return ptx.value


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--json", default=None)
    parser.add_argument("--skip-ptx", action="store_true")
    parser.add_argument("--rev", default=None,
                        help="钉住被复核提交（如 6e2bd529）：用 git show 读源码，隔离他人未提交改动")
    args = parser.parse_args()

    names, per_piece, source, product = assemble(args.rev)
    data = {
        "source_rev": args.rev or "worktree",
        "device_source_names": names,
        "device_source_count": len(names),
        "assembled_len_bytes": len(source.encode("utf-8")),
        "assembled_sha256": hashlib.sha256(source.encode("utf-8")).hexdigest(),
        "product_source_bytes": len(product.encode("utf-8")),
        "pieces": per_piece,
    }
    if not args.skip_ptx:
        ptx = nvrtc_ptx(source)
        entries = re.findall(rb"\.visible \.entry (\w+)", ptx)
        entries += re.findall(rb"^\s*\.entry (\w+)", ptx, re.M)
        data["ptx_bytes"] = len(ptx)
        data["ptx_sha256"] = hashlib.sha256(ptx).hexdigest()
        data["ptx_entries"] = sorted({e.decode() for e in entries})
        data["ptx_entry_count"] = len(data["ptx_entries"])
        data["ptx_saved_to"] = None
        ptx_path = ROOT / "target/qa-primitive/assembled.ptx"
        ptx_path.parent.mkdir(parents=True, exist_ok=True)
        ptx_path.write_bytes(ptx)
        data["ptx_saved_to"] = str(ptx_path)
    text = json.dumps(data, indent=2, ensure_ascii=False)
    print(text)
    if args.json:
        pathlib.Path(args.json).write_text(text + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
