# -*- coding: utf-8 -*-
r"""RE-0 G2: offline warp codegen of a physics module, compared against the quarantined
capture-time module source (entry names + hashes). Read-only w.r.t. repo; writes only
under target/render-bvh/gl2-ptx-diff/refgen/.

Run (18:00 window):
  $env:PYTHONPATH = "D:\Rust\mjwarp-rs\target\mjwarp-parallel\upstream"
  D:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Scripts\python.exe <this file>
"""
import hashlib
import json
import pathlib
import re
import sys
import time

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
OUT = ROOT / "target/render-bvh/gl2-ptx-diff/refgen"
REF_CACHE = ROOT / ("fixtures/gravity-upstream-abi/cache-quarantine/"
                    "warp-1.15.0-double-version-2026-10-18/"
                    "wp_mujoco_warp._src.smooth_ca013d6/"
                    "wp_mujoco_warp._src.smooth_ca013d6.cu")
ENTRY_RE = re.compile(r"\b(_[A-Za-z0-9_]+?_[0-9a-f]{8}_cuda_kernel_(?:forward|backward))\b")

out_lines = []
def p(*a):
    s = " ".join(str(x) for x in a)
    out_lines.append(s)
    print(s)


def entries(text: str):
    return sorted(set(ENTRY_RE.findall(text)))


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    t0 = time.time()
    import warp as wp
    from warp._src.context import ModuleBuilder
    import mujoco_warp  # noqa: F401  (registers modules)
    import mujoco_warp._src.smooth as S
    import mujoco_warp._src.types as T

    p("warp:", wp.__version__ if hasattr(wp, "__version__") else "?",
      "| mujoco_warp:", getattr(mujoco_warp, "__version__", "?"))
    p("wp.config.enable_backward:", getattr(wp.config, "enable_backward", "?"),
      "| lineinfo:", getattr(wp.config, "lineinfo", "?"),
      "| mode:", getattr(wp.config, "mode", "?"))

    kernel = S._kinematics_branch
    module = kernel.module
    p("module:", module.name, "| test kernel:", kernel.key)

    # target entries from the frozen capture-time module source
    ref_entries = entries(REF_CACHE.read_text(encoding="utf-8", errors="replace"))
    p("target(.cu) entries:", len(ref_entries))
    target_fwd = [e for e in ref_entries if e.endswith("_forward")]
    p("target forward:", len(target_fwd), "backward:", len(ref_entries) - len(target_fwd))

    bd_default = T.BlockDim()
    p("BlockDim defaults:", json.dumps({k: getattr(bd_default, k) for k in (
        "cholesky_factorize", "cholesky_factorize_solve", "cholesky_solve",
        "small_cholesky", "solve_LD_sparse_fused")}, ensure_ascii=False))

    variants = []
    for eb in (True, False):
        for bd in (None, 32, 64, 128, 256):
            try:
                opts = dict(module.resolve_options(wp.config, block_dim=bd))
                opts["enable_backward"] = eb
                t = time.time()
                src = ModuleBuilder(module, opts).codegen("cuda")
                dt = time.time() - t
                es = entries(src)
                hit = [e for e in target_fwd if e in es]
                variants.append({
                    "enable_backward": eb, "block_dim": bd, "codegen_s": round(dt, 2),
                    "cu_bytes": len(src.encode("utf-8")),
                    "entries": len(es), "target_forward_hits": f"{len(hit)}/{len(target_fwd)}",
                    "options": {k: str(v) for k, v in opts.items()},
                })
                p(f"VAR eb={eb} bd={bd} codegen={dt:.2f}s bytes={variants[-1]['cu_bytes']} "
                  f"entries={len(es)} target_hits={len(hit)}/{len(target_fwd)}")
                name = f"smooth_eb{int(eb)}_bd{bd}.cu"
                (OUT / name).write_text(src, encoding="utf-8", newline="\n")
                if len(hit) == len(target_fwd) and len(es) == len(ref_entries):
                    p("MATCH shape:", name)
            except Exception as e:  # noqa: BLE001
                variants.append({"enable_backward": eb, "block_dim": bd, "error": str(e)[:300]})
                p(f"VAR eb={eb} bd={bd} ERROR {str(e)[:200]}")

    (OUT / "g2-codegen-variants.json").write_text(
        json.dumps({"module": module.name, "target_entries": len(ref_entries),
                    "variants": variants, "elapsed_s": round(time.time() - t0, 1)},
                   ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
    p("elapsed_s:", round(time.time() - t0, 1))
    (OUT / "g2-codegen.log").write_text("\n".join(out_lines) + "\n", encoding="utf-8", newline="\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
