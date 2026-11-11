# -*- coding: utf-8 -*-
r"""A1 pre-flight: can the unique-module factor/solve kernels be codegen'd offline?

Read-only w.r.t. repo; writes only under target/render-bvh/gl2-ptx-diff/refgen/a1/.
Run:
  $env:PYTHONPATH = "D:\Rust\mjwarp-rs\target\mjwarp-parallel\upstream"
  <venv python> a1_unique_codegen.py
"""
import hashlib
import inspect
import json
import pathlib
import re
import sys
import time

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
OUT = ROOT / "target/render-bvh/gl2-ptx-diff/refgen/a1"


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    log = []
    def p(*a):
        s = " ".join(str(x) for x in a)
        log.append(s)
        print(s)

    t0 = time.time()
    import warp as wp
    from warp._src.context import ModuleBuilder
    import mujoco_warp  # noqa
    import mujoco_warp._src.smooth as S
    import mujoco_warp._src.types as T

    p("warp", getattr(wp, "__version__", "?"), "| mjwarp", getattr(mujoco_warp, "__version__", "?"))
    p("TileSet fields:", [f.name for f in __import__("dataclasses").fields(T.TileSet)] if hasattr(T, "TileSet") else "n/a")

    candidates = []
    # (label, factory, args)
    for bs in (1, 3, 6, 10):
        candidates.append((f"small_chol_bs{bs}", S._small_cholesky_factorize_block, (bs,)))
    for nv, nl in ((30, 4), (60, 8), (24, 3)):
        candidates.append((f"solve_ld_{nv}_{nl}", S._solve_LD_sparse_fused, (nv, nl)))

    rows = []
    for label, factory, args in candidates:
        row = {"label": label, "factory": factory.__name__, "args": list(args)}
        try:
            t = time.time()
            kernel = factory(*args)
            row["kernel"] = getattr(kernel, "key", str(kernel))[:80]
            mod = kernel.module
            opts = dict(mod.resolve_options(wp.config))
            opts.update(getattr(kernel, "options", {}) or {})
            src = ModuleBuilder(mod, opts).codegen("cuda")
            es = sorted(set(re.findall(r"__global__ void (\w+)\(", src)))
            row.update({"seconds": round(time.time() - t, 2), "cu_bytes": len(src.encode()),
                        "entries": es, "module": mod.name})
            if es:
                fn = OUT / f"{label}.cu"
                fn.write_text(src, encoding="utf-8", newline="\n")
                row["cu"] = fn.relative_to(ROOT).as_posix()
            p(label, "OK", row["entries"], row["cu_bytes"], "B", f"{row['seconds']}s")
        except Exception as e:  # noqa: BLE001
            row["error"] = repr(e)[:300]
            p(label, "ERROR", row["error"][:200])
        rows.append(row)

    (OUT / "a1-codegen.json").write_text(
        json.dumps({"rows": rows, "elapsed_s": round(time.time() - t0, 1)},
                   ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
    p("elapsed_s", round(time.time() - t0, 1))
    (OUT / "a1-codegen.log").write_text("\n".join(log) + "\n", encoding="utf-8", newline="\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
