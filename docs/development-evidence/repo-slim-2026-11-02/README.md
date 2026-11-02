# Repo slim 2026-11-02 (disk cleanup record)

Record of the disk cleanup performed by Chief on **2026-11-02** (A→B→C), with the numbers as
reported by the operator and the recovery steps required afterwards. Zero GPU; no criterion/
reference/fixture/product change.

## Before / after

| metric | before | after |
| --- | --- | --- |
| `target/` size | 428 GiB | 0 (removed entirely) |
| D: free space | 226.3 GB | **641.9 GB** (+415.6 GB) |
| working tree | — | 371 MB |
| `.git` | — | 126 MB (workspace + .git ≈ 0.5 GB) |

`git status` was clean after the cleanup; HEAD at that moment was `0337e2f0`.

## What was removed

1. **7 stale git worktrees** (removed before the directory deletion):
   - `target/qa-frame-diag/tree`
   - `target/qa-frame-diag/tree-noconfig`
   - `target/qa-g2-fix`
   - `target/qa-plane/wt-b1`
   - `target/qa-s2a-head`
   - `target/worktrees/faithful-5e000cb9`
   - `target/wt-rne`

   Discarded contents: temporary diagnostic/verification checkouts and their build outputs
   (uncommitted scratch); none of them was evidence. **The discarded worktree items are NOT
   evidence and do not affect the attempt5 conclusion.**
2. **498 historical scratch directories** under `target/`.
3. `cargo clean` (190.4 GiB of build cache).
4. `u038` scratch (230 MB).
5. `render-bvh` rendered-副本 directories.

## Collateral deletion (operator error, must be recovered before GPU/native work)

`cargo clean` also removed the prepared toolchains that were intentionally kept under `target/`:

- `target/toolchains/cuda-12.8.1/`
- `target/toolchains/cuda-12.9.1-nvrtc/`
- `target/toolchains/libmathdx-0.3.1/`
- `target/toolchains/mujoco-3.12.0/`
- `target/mathdx11/`

  Directory names follow the restore run on 2026-11-02 (these are exactly the directory names
  the two prepare scripts create; the earlier wording `libmathdx/` was wrong and the
  `cuda-12.9.1-nvrtc` directory was missing from this list).

**Consequence: until they are restored, the default cuda build and native tests are NOT
available** (`clippy-cuda` / `unit` with the default feature set require the CUDA toolchain;
`run-window.ps1`'s default `-CudaBin target/toolchains/cuda-12.8.1/bin` also does not exist).

## Recovery steps

1. `scripts/prepare-cuda-probe.ps1` — CUDA 12.8.1 + libmathdx, with the script's own sha256
   verification; keep its output as evidence (download size / elapsed time recorded by the caller).
2. `scripts/prepare-native-probe.ps1` — MuJoCo 3.12.0, with the script's own sha256 verification.
3. mathdx11: follow the reproduction steps in
   `docs/development-evidence/g03-mathdx-alignment/README.md`.

After recovery, verify `target/toolchains/cuda-12.8.1/bin/nvcc` exists and list the three
toolchain directories.

## The attempt5 freeze after the cleanup

- QA verdict anchor (unchanged by this cleanup): guards sha
  `9e000bfc1f8438a4bd6fd5f8c8ccf214c2ab43aee7c1ed1fce78ed51e7646c32`, aggregate
  `034c7551226e321fab83b0ec4ef92a2dd82e9077ee3f31e7ad88f9a5d3b305b9`, QA recomputation time
  **2026-11-02T16:13:11+08:00** (`docs/development-evidence/render-bvh/sdf-validate/receipt.json`
  `/qa_verdict`).
- The live guards copy and the frozen exes were inside `target/`; after the cleanup **this freeze
  is a historical record only — it must not be re-compared or claimed as an executable freeze.**
- Any new `target/` built after recovery is a fresh build cache; it is unrelated to the attempt5
  freeze and its new exe identities must NOT be used to re-identify the attempt5 freeze.

## Boundaries

No criterion numeric values, lists, references, fixtures, product src or kernel identity were
changed by the cleanup or by this record. This file records the operator's cleanup numbers and the
recovery requirements; it makes no release/acceptance/parity statement.
## Recovery result (2026-11-02 20:2x, executed by Arch)

- `scripts/prepare-cuda-probe.ps1`: **EXIT=0, elapsed 889.2 s (~14.8 min)**, download list
  cuda_nvrtc, cuda_cudart, cuda_cccl, cuda_nvcc, libnvjitlink, libmathdx 0.3.1, cuda_nvrtc 12.9.86;
  the script's own sha256 checks passed (exit 0). Restored toolchains:
  `target/toolchains/cuda-12.8.1` (3,683.8 MB), `target/toolchains/cuda-12.9.1-nvrtc`
  (2,631.4 MB), `target/toolchains/libmathdx-0.3.1` (102.9 MB) - the restored names match the
  collateral-deletion list above. `target/toolchains/cuda-12.8.1/bin/nvcc.exe`
  exists ("NVIDIA (R) Cuda compiler driver, Copyright (c) 2005-2025"). Log: `logs/prepare-cuda.txt`.
- `scripts/prepare-native-probe.ps1`: **EXIT=0, elapsed 13.1 s**; MuJoCo 3.12.0 restored
  (`target/toolchains/mujoco-3.12.0/package/include/mujoco/mujoco.h` present; zip sha256 verified by
  the script). Log: `logs/prepare-native.txt`.
- `target/mathdx11`: **NOT restored** (absent after recovery); follow
  `docs/development-evidence/g03-mathdx-alignment/README.md` before any mathdx/g03 work.
- Note: the prepare scripts are UTF-8 without BOM and contain CJK; Windows PowerShell 5.1 cannot
  parse them (parser error). They were executed through a temporary UTF-8-with-BOM copy in the same
  directory (so `$PSScriptRoot` stays correct) which was deleted immediately after the run; the
  repository scripts were not modified.
- The new `target/` is a fresh build cache. It is unrelated to the attempt5 freeze and its new exe
  identities must not be used to re-identify that freeze.
