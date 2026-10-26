"""Independently reassemble the 27-segment device source at e221e6c2 (B3) and
compare to the delivered nvrtc smoke fingerprint."""
import hashlib
import re
import subprocess
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
COMMIT = "097c54b9"
SRB = ROOT / "src/runtime/collision_gpu/source.rs"


def entries():
    text = SRB.read_text(encoding="utf-8")
    names = []
    in_list = False
    for line in text.splitlines():
        if "pub(crate) const DEVICE_SOURCES" in line:
            in_list = True
            continue
        if in_list:
            if line.startswith("];"):
                break
            m = re.match(r'\s*\("([^"]+\.cu)",\s*[A-Z_]+\)', line)
            if m:
                names.append(m.group(1))
                continue
            m2 = re.match(r'\s*"([^"]+\.cu)",\s*$', line)
            if m2:
                names.append(m2.group(1))
    return names


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
    return subprocess.check_output(
        ["git", "-C", str(ROOT), "cat-file", "blob", f"{COMMIT}:src/physics/collision/cuda/{name}"]
    )


def work(name):
    return (ROOT / "src/physics/collision/cuda" / name).read_bytes()


def assemble(get, parts):
    buf = b""
    for name in parts:
        buf += strip_local(get(name))
        if not buf.endswith(b"\n"):
            buf += b"\n"
    buf += strip_local(get("product.cu"))
    return len(buf), hashlib.sha256(buf).hexdigest()


names = entries()
print("segments", len(names))
for label, getter in (("git_blob", blob), ("worktree", work)):
    n, s = assemble(getter, names)
    print(label, "27", n, s)
print("delivery_smoke", 237922, "92ee236e519dc04b26b2d24400fa57f382ed9cea1b5b28596c238ab6cfa4cec8")
