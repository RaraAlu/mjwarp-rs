# -*- coding: utf-8 -*-
r"""A1 pre-flight part 2: tile-variant unique kernels with a stub TileSet + full factory matrix."""
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
    import numpy as np
    import warp as wp
    from warp._src.context import ModuleBuilder
    import mujoco_warp  # noqa
    import mujoco_warp._src.smooth as S
    import mujoco_warp._src.types as T

    # stub TileSet: only `size` is compile-time; adr/elemid are runtime arrays
    def stub_tile(size, nblocks=4, nelem=None):
        nelem = nelem or size * size
        return T.TileSet(
            adr=wp.array(np.arange(nblocks, dtype=np.int32), dtype=int),
            size=size,
            elemid=wp.array(np.arange(nblocks * nelem, dtype=np.int32), dtype=int),
        )

    rows = []

    def attempt(label, factory, args, extra_opts=None):
        row = {"label": label, "factory": getattr(factory, "__name__", "?")}
        try:
            t = time.time()
            kernel = factory(*args)
            mod = kernel.module
            opts = dict(mod.resolve_options(wp.config))
            opts.update(getattr(kernel, "options", {}) or {})
            if extra_opts:
                opts.update(extra_opts)
            src = ModuleBuilder(mod, opts).codegen("cuda")
            es = sorted(set(re.findall(r"__global__ void (\w+)\(", src)))
            row.update({"seconds": round(time.time() - t, 2), "cu_bytes": len(src.encode()),
                        "entries": es})
            if es:
                fn = OUT / f"{label}.cu"
                fn.write_text(src, encoding="utf-8", newline="\n")
                row["cu"] = fn.relative_to(ROOT).as_posix()
            p(label, "OK", es, row["cu_bytes"], "B", f"{row['seconds']}s")
        except Exception as e:  # noqa: BLE001
            row["error"] = repr(e)[:300]
            p(label, "ERROR", row["error"][:220])
        rows.append(row)

    # tile variants (tile.size = block size)
    for sz in (3, 6):
        attempt(f"tile_chol_{sz}", S._tile_cholesky_factorize_block, (stub_tile(sz),), {"output_arch": 89, "block_dim": 32})
    # all other unique factories
    for bs in (3, 6):
        attempt(f"small_chol_solve_{bs}", S._small_cholesky_solve_block, (bs,))
    for sz in (3, 6):
        attempt(f"tile_chol_solve_{sz}", S._tile_cholesky_solve_block, (stub_tile(sz),), {"output_arch": 89, "block_dim": 32})
    for sz in (3, 6):
        attempt(f"tile_chol_factor_solve_{sz}", S._tile_cholesky_factorize_solve_block, (stub_tile(sz),), {"output_arch": 89, "block_dim": 32})
    for bs in (3, 6):
        attempt(f"small_chol_factor_solve_{bs}", S._small_cholesky_factorize_solve_block, (bs,))
    for nv in (30, 60):
        attempt(f"factor_solve_lu_{nv}", S._factor_solve_lu_sparse_fused, (nv,))

    (OUT / "a1-codegen-tile.json").write_text(
        json.dumps({"rows": rows, "elapsed_s": round(time.time() - t0, 1)},
                   ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
    p("elapsed_s", round(time.time() - t0, 1))
    (OUT / "a1-codegen-tile.log").write_text("\n".join(log) + "\n", encoding="utf-8", newline="\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
