# SDF 渲染参考采集窗口材料（待用户双签；未执行）

材料任务：`2026-10-30-render-sdf-capture-authoring`（U2）
被执行任务：`2026-10-30-render-sdf-capture`（U3，GPU，需用户双签）
上游事实：渲染器对 SDF 求交的是其底层 mesh（`render.py:738-761`、`render_test.py:1251`），采集不需要 SDF 距离场代码。

## 0. 一句话

跑两次 pinned 采集（`sdf-mesh` / `sdf-occluder` 两组），把 SDF 渲染参考落到
`fixtures/render-bvh/scene-sdf/capture-sdf/<group>/`（rgb/depth/seg npz + capture.json）。
**这是采集，不是验收**：本窗不产生任何渲染数值判定；数值结论只在 U4 验证窗（U1b host 解锁后）。

## 1. 脚本身份

`scripts/capture_render_sdf.py` sha256 `8064596876aeec3ee2fe1f63e83d7de87f898a8dbe9a1ef64f64026de8a77ba9`、**21,349 B / 441 行 / 非 ASCII 0**，派生自 R05-S3a `capture_render_hfield.py`（环境 pin / 确定性 npz / 两跑字节相等 / legacy sha 清单全保留）；派生器与被删改片段见 `sdf-capture-authoring/README.md` §1。**硬检查**：out-dir 非空即拒（非零退出）、真实采集要求 `--repeat >= 2`、两跑先在 out-dir 之外的临时目录比对、全相等才创建/写入 out-dir（失败不留产物、重跑不被残留挡住）。

## 2. 场景

- `scene.xml`（group `sdf-mesh`）sha256 `376eb81c…`、3,388 B：SDF cube（volume SDF、无 plugin、`contype=conaffinity=0`）+ plane + sphere + 2 相机（320×240 透视 / 240×180 俯视）+ 2 个 castshadow 灯。
- `scene-mixed.xml`（group `sdf-occluder`）sha256 `7869cfae…`、3,062 B：同上 + 视线内静态 box occluder。
- 静态校验（CPU-only pinned MuJoCo）：`sdf-capture-authoring/logs/scene-check.json`。

## 3. 命令（逐字，见 `materials/window-plan.txt`）

```text
# 开窗前自检（不建 CUDA device、不写盘）
target\mjwarp-parallel\venv\Scripts\python.exe -B scripts/capture_render_sdf.py --dry-run --group sdf-mesh --out-dir fixtures/render-bvh/scene-sdf/capture-sdf/sdf-mesh
target\mjwarp-parallel\venv\Scripts\python.exe -B scripts/capture_render_sdf.py --dry-run --group sdf-occluder --out-dir fixtures/render-bvh/scene-sdf/capture-sdf/sdf-occluder

# 窗口内唯二 GPU 命令（各跑一次）
target\mjwarp-parallel\venv\Scripts\python.exe -B scripts/capture_render_sdf.py --group sdf-mesh --use-fast-math false --use-shadows true --repeat 2 --out-dir fixtures/render-bvh/scene-sdf/capture-sdf/sdf-mesh
target\mjwarp-parallel\venv\Scripts\python.exe -B scripts/capture_render_sdf.py --group sdf-occluder --use-fast-math false --use-shadows true --repeat 2 --out-dir fixtures/render-bvh/scene-sdf/capture-sdf/sdf-occluder
```

## 4. 门槛与基线（常规，无例外）

- monitor：`util < 40`、`-Required 60 -MaxMinutes 30`、**不预批例外、不加 `-OpenUtilLimit`**（本窗没有 `run-window`/guards 步骤，例外参数也无处可加）。
- 基线：开窗前 5 分钟三点 free/util + compute-apps raw/filtered + build 进程 0 + 树干净；`floor = baseline_min_free − 3072`；窗内任一采样 free 破 floor 或 used 增量 > 3072 MiB → 停手。
- 失败即停手（不重试、不调参、不放宽）：任一非零退出 / 两跑不相等（数组或 npz 字节）/ legacy 69 条变化 / 写盘越界 / free 破 floor / 外部 GPU 负载或 util ≥ 40。

## 5. 预期产物

`fixtures/render-bvh/scene-sdf/capture-sdf/<group>/`：`rgb.npz`、`depth.npz`、`seg.npz`、`capture.json`（字段清单见 `materials/expected-products.json`）。两组各 ≈1.4 MB。

## 6. 双签（已签）与 shell 约束

- **用户签字**：user (via Chief) **2026-10-30T23:10:52+08:00**，decision = **approve**；`materials/window-plan.txt` 的 `USER DOUBLE-SIGN BLOCK` 已填（approved_by / 时间戳 / scope 四项 / Chief collated at）。
- **被批准的 plan 版本 sha256**：`eb484aa74507f1f9568d72ce0c199491bdafd591c257b342a41675db23bd2fb2`（签字前的计划文本版本）；签字后本文件（含签字块与 shell 约束段）的 sha256 以 `manifest.txt` 与 `sdf-capture` 证据的 receipt 为准。
- **scope（4 项用户决定）**：① SDF 两窗合并（本 U3 采集 + 之后 U4 验证）、只做 volume SDF、1 场景 2 相机含 shadow 开/关对比；② per-world 采集窗批准（另窗，稍后）；③ mesh A1（16→20）实现批准，U8 是否纳入判据在其窗前后再确认；④ U8 判据确认不属于本计划。
- **shell 硬约束**：本计划所有命令、聚合与自检**必须用 Windows PowerShell 5.1**（`powershell.exe -NoProfile -ExecutionPolicy Bypass -File …`），**不得用 pwsh 7**——`aggregate_sha256` 依赖 `Sort-Object` 的文化敏感排序，5.1（NLS）与 7（ICU）对同一文件集算出的摘要不同；窗口事实里记录实际 shell 版本。（U3 采集窗本身不产生 aggregate；U4 会用到。）
- 本窗不冻结测试可执行文件、不需要新 guards；U4 验证窗才需要新冻结（U1 改了 `.cu`，旧 exe 哈希全部失效，绝不复用 R05/R02/R08 guards）。

## 7. authoring-only `util<60` 例外（用户已签 2026-10-31T00:00+08:00）

- **五要素**（详见 `materials/window-plan.txt` 的 `EXCEPTION BLOCK`）：`mode=authoring_only_exception_util_lt_60`、`approved_by=user via Chief 2026-10-31T00:00+08:00`、reason = 桌面 3D 噪声（dwm / Harness GUI / msedgewebview2；常规 monitor 354 样本 / longest 21 / 84 个 ≥40 / exit 20，free_min 13,388 与 floor 10,445 未破、filtered=0、build=0）、`original_normal_threshold_attempts_retained`（失败 CSV + 快照 + failure facts 原样保留）、`not_a_normal_threshold_pass=true`。
- **例外 monitor**：`powershell -NoProfile -ExecutionPolicy Bypass -File scripts\validate\render-bvh\monitor-until-clean.ps1 -OutCsv target\render-bvh\sdf-capture-window\monitor-pre-window-attempt2.csv -Required 60 -MaxMinutes 30 -MaxUtil 60`（新 CSV，不覆盖失败那份；**PowerShell 5.1**）。
- **例外生效证据口径**：从例外 CSV 独立重算「util≥40 但仍 clean 的样本与连续段」、`ge60=0`、`util_max<60`、free/floor、filtered/build；若 60 样本全 <40 则记 `exception_effectively_not_load_bearing=true`，不得声称 60 门槛生效。
- 本例外同时覆盖 per-world 采集窗（同一份用户签字，scope 见例外块）。
