import hashlib
import json
import pathlib
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
UPSTREAM = ROOT / "target" / "mjwarp-parallel" / "upstream"
CACHE = ROOT / "target" / "mjwarp-parallel" / "gravity-cache-1.15.0"
OUT = ROOT / "target" / "mjwarp-parallel" / "gravity-artifact-1.15.0"

if CACHE.exists() or OUT.exists():
  raise RuntimeError("authoring outputs already exist")
CACHE.mkdir(parents=True)
OUT.mkdir(parents=True)
sys.path.insert(0, str(UPSTREAM))

import warp as wp

if wp.__version__ != "1.15.0":
  raise RuntimeError(f"unexpected Warp version: {wp.__version__}")
revision = subprocess.check_output(
  ["git", "-C", str(UPSTREAM), "rev-parse", "HEAD"], text=True
).strip()
if revision != "71da24d956378a87a703b6e1442b13aec0c4ac29":
  raise RuntimeError(f"unexpected upstream revision: {revision}")

wp.config.kernel_cache_dir = str(CACHE)
wp.init()
import mujoco_warp._src.passive as passive

module = wp.get_module(passive.__name__)
wp.load_module(module=module, device="cuda:0")

files = [path for path in CACHE.rglob("*") if path.is_file()]
cu = [path for path in files if path.suffix == ".cu"]
ptx = [path for path in files if path.suffix == ".ptx" and "sm89" in path.name]
if len(cu) != 1 or len(ptx) != 1:
  raise RuntimeError(f"unexpected cache outputs: cu={cu}, ptx={ptx}")
cu_text = cu[0].read_text(encoding="utf-8")
entries = []
for line in cu_text.splitlines():
  marker = 'extern "C" __global__ void '
  if marker in line and "_gravity_force_" in line and line.rstrip().endswith("("):
    entries.append(line.split(marker, 1)[1][:-1])
if len(entries) != 1:
  raise RuntimeError(f"unexpected gravity entries: {entries}")

cu_out = OUT / "module.cu"
ptx_out = OUT / "module.ptx"
shutil.copyfile(cu[0], cu_out)
shutil.copyfile(ptx[0], ptx_out)

def digest(path):
  return hashlib.sha256(path.read_bytes()).hexdigest()

receipt = {
  "schema": 1,
  "evidence_kind": "frozen-generated",
  "upstream_revision": revision,
  "warp_version": wp.__version__,
  "module": passive.__name__,
  "passive_source_sha256": digest(UPSTREAM / "mujoco_warp" / "_src" / "passive.py"),
  "support_source_sha256": digest(UPSTREAM / "mujoco_warp" / "_src" / "support.py"),
  "codegen_source_sha256": digest(pathlib.Path(wp.__file__).parent / "_src" / "codegen.py"),
  "target_arch": 89,
  "mode": "release",
  "fast_math": False,
  "fuse_fp": True,
  "entry": entries[0],
  "launch_rank": 3,
  "array_arguments": 11,
  "cu_file": cu_out.name,
  "ptx_file": ptx_out.name,
  "cu_sha256": digest(cu_out),
  "ptx_sha256": digest(ptx_out),
}
(OUT / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
print(json.dumps(receipt, indent=2))
