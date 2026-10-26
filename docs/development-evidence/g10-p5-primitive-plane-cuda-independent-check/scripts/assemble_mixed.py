"""Prove the delivered 222849/59631d83 fingerprint corresponds to old-files-LF + 4 new plane files CRLF."""
import hashlib
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


def blob(name):
    return subprocess.check_output(["git", "-C", str(ROOT), "cat-file", "blob", f"{COMMIT}:src/physics/collision/cuda/{name}"])


def work(name):
    return (SRC / name).read_bytes()


def assemble(parts, use_work):
    buf = b""
    for name in parts:
        buf += strip_local(work(name) if name in use_work else blob(name))
        if not buf.endswith(b"\n"):
            buf += b"\n"
    buf += blob("product.cu")
    return len(buf), hashlib.sha256(buf).hexdigest()


plane4 = names[21:]
n, s = assemble(names, set(plane4))
print("old21_LF__plane4_CRLF__product_LF", n, s)
print("delivery", 222849, "59631d8377e3a587cb98d396742f9698ea17105847a33586e8b7a89b8c84b917")
