"""Per-file strip-size delta between git blob (LF) and working-tree bytes (checkout EOL) for the 25 device sources."""
import re
import subprocess
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
WT = ROOT / "target/qa-plane/wt-b1"
SRC = WT / "src/physics/collision/cuda"
COMMIT = "b05190e5"
source_rs = (WT / "src/runtime/collision_gpu/source.rs").read_text(encoding="utf-8")
names = []
in_list = False
for line in source_rs.splitlines():
    if "pub(crate) const DEVICE_SOURCES" in line:
        in_list = True
        continue
    if in_list:
        if line.startswith("];"):
            break
        m = re.match(r'\s*\("([^"]+\.cu)",\s*[A-Z_]+\)', line)
        if m:
            names.append(m.group(1))


def is_local(line):
    s = line.lstrip()
    if not s.startswith(b"#include"):
        return False
    rest = s[len(b"#include"):].lstrip()
    if not rest.startswith(b'"'):
        return False
    end = rest.find(b'"', 1)
    return end >= 0 and rest[1:end].endswith(b".cu")


def strip_local(data):
    return b"".join(line for line in re.findall(rb"[^\n]*\n|[^\n]+$", data) if not is_local(line))


total = 0
for name in names + ["product.cu"]:
    b = subprocess.check_output(["git", "-C", str(ROOT), "cat-file", "blob", f"{COMMIT}:src/physics/collision/cuda/{name}"])
    w = (SRC / name).read_bytes()
    d = len(strip_local(w)) - len(strip_local(b))
    total += d
    print(f"{name:36s} blob={len(b):6d} work={len(w):6d} strip_delta={d}")
print("total strip delta", total)
