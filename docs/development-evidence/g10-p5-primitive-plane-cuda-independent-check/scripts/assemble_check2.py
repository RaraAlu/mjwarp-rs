"""Compare LF-blob assembly vs raw worktree-byte assembly for the B1 device source."""
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


def is_local(line_bytes):
    s = line_bytes.lstrip()
    if not s.startswith(b"#include"):
        return False
    rest = s[len(b"#include"):].lstrip()
    if not rest.startswith(b'"'):
        return False
    end = rest.find(b'"', 1)
    if end < 0:
        return False
    return rest[1:end].endswith(b".cu")


def strip_local_bytes(data):
    # emulate Rust split_inclusive('\n') on bytes
    out = []
    for line in re.findall(rb"[^\n]*\n|[^\n]+$", data):
        if is_local(line):
            continue
        out.append(line)
    return b"".join(out)


def assemble_from_bytes(get_bytes, parts):
    buf = b""
    for name in parts:
        buf += strip_local_bytes(get_bytes(name))
        if not buf.endswith(b"\n"):
            buf += b"\n"
    buf += get_bytes("product.cu")
    return len(buf), hashlib.sha256(buf).hexdigest()


def blob_bytes(path):
    return subprocess.check_output(["git", "-C", str(ROOT), "cat-file", "blob", f"{COMMIT}:src/physics/collision/cuda/{path}"])


def work_bytes(path):
    return (SRC / path).read_bytes()


for label, getter in (("git_blob", blob_bytes), ("worktree_raw", work_bytes)):
    n_new, s_new = assemble_from_bytes(getter, names)
    n_old, s_old = assemble_from_bytes(getter, names[:21])
    print(label, "new", n_new, s_new, "old", n_old, s_old)

for name in names[-4:] + ["product.cu"]:
    b = blob_bytes(name)
    w = work_bytes(name)
    print(name, "blob", len(b), "work", len(w), "crlf_in_work", w.count(b"\r\n"), "lines", w.count(b"\n"))
