"""Independently reassemble the B1 device source from the b05190e5 worktree blobs and reproduce the fingerprint."""
import hashlib
import re
from pathlib import Path

WT = Path(r"D:\Rust\mjwarp-rs\target\qa-plane\wt-b1")
SRC = WT / "src/physics/collision/cuda"
source_rs = (WT / "src/runtime/collision_gpu/source.rs").read_text(encoding="utf-8")

# parse DEVICE_SOURCES entries in order
entries = []
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
            entries.append(m.group(1))
names = entries
print("device source entries:", len(names), names)

local_include = re.compile(r'^#include\s+"([^"]+\.cu)"')


def is_local(line):
    s = line.lstrip()
    if not s.startswith("#include"):
        return False
    rest = s[len("#include"):].lstrip()
    if not rest.startswith('"'):
        return False
    end = rest.find('"', 1)
    if end < 0:
        return False
    return rest[1:end].endswith(".cu")


def strip_local(text):
    out = []
    for line in re.findall(r"[^\n]*\n|[^\n]+$", text):
        if is_local(line):
            continue
        out.append(line)
    return "".join(out)


def assemble(parts):
    buf = ""
    for name in parts:
        text = (SRC / name).read_text(encoding="utf-8")
        buf += strip_local(text)
        if not buf.endswith("\n"):
            buf += "\n"
    buf += (SRC / "product.cu").read_text(encoding="utf-8")
    raw = buf.encode("utf-8")
    return len(raw), hashlib.sha256(raw).hexdigest()


n_all, sha_all = assemble(names)
print("new_25_segments", n_all, sha_all)
old = names[:21]
n_old, sha_old = assemble(old)
print("old_21_segments", n_old, sha_old)
print("delivery_claim_new", 222849, "59631d8377e3a587cb98d396742f9698ea17105847a33586e8b7a89b8c84b917")
print("delivery_claim_old", 198815, "52b225c2830863cd8ce58d1fd7275a50fa1beeac1e9fc9fe40ec025faaebca6c")
