# per-world S1 验证窗口材料（离线；未开窗、未冻结、未跑 GPU）

任务：`2026-11-03-render-perworld-s1-window-material`（Arch 2026-11-03 注册；材料片）
被服务任务：`2026-10-30-render-perworld-impl-s1`（逐 world 渲染实现 S1，方案 L；本材料只准备其验证窗）
上游缺口：`2026-10-28-render-per-world-geom-light-gap`（已转 L v1 + L v2/K 背卡）

## 0. 一句话

为 perworld S1 的验证窗准备**用户双签材料**：实体表（9 实体）、窗口计划（suite/逐字命令/floor/失败条款）、
ASCII 冻结器（9 exe + 83 只读输入 = 92 条）、开窗前基线 SOP 与双签栏。**这是材料，不是执行**：
本片不开窗、不跑 GPU、不冻结 guards、不写 `fixtures/`/`src/`/`tests/`，也不产生任何数值结论。

## 1. 交付物

| 交付 | 路径 | 说明 |
| --- | --- | --- |
| 实体表 | `materials/entity-table.json` | 9 实体：`render_per_world`（新）+ 8 回归；判据与理由；`render_sdf_reference` 明确排除 |
| 窗口计划 | `materials/window-plan.txt` | ASCII 一页：suite/guards/逐字命令/floor/失败条款/双签栏/NOT CLAIMED |
| 冻结器 | `scripts/validate/render-bvh/freeze-guards-perworld-s1.ps1` | ASCII-only、PS 5.1 可解析、LF；硬断言 9 exe + 83 输入 = 92 条 + stash 9/9 |
| 证据 | `README.md` / `receipt.json` / `manifest.txt` | 材料身份、脚本 sha/bytes/行数、签署状态（待签）与自洽清单 |

## 2. 实体与判据

- **新增** `render_per_world`：逐 world 循环（`update_poses` → `refit_bvh` → 单 world 发射；输出用 adr 值位移
  `world*P + adr[cam]` 定位），逐 world 对 `fixtures/render-bvh/scene-a/perworld/nw3-hinge02040/` 比对：
  rgb `mismatched=0 / max_byte_diff=0 / beyond_1_lsb=0`；depth 位级 `mismatched=0` 且 `max_ulp<=4` 且 `beyond_ulp=0`；
  seg 全等；world 隔离；两跑 rgb/depth/seg sha256 相等。
- **8 回归**：`render_e2e`、`render_kernel_rgb`、`render_kernel_depth_seg`、`render_kernel_flags`、
  `render_r10_multi_world`、`render_skybox_reference`、`render_context_lifecycle`、`render_mesh_bvh`，
  沿用各自既有位级基线、零漂移。
- **刻意不含** `render_sdf_reference`：SDF 的冻结身份是 criterion v2.2 + `guards-sdf-validate.json` 独立一套，
  混进本窗会混用两个验收面。
- **AA/spp>1 不覆盖**：`kernel/per_world.rs` 对 `nsamples>1` 显式返回 `LauncherError::Unsupported`（CPU 单测钉住），
  参考是 spp=1；AA+perworld 留 L v2/K。

## 3. 冻结与窗口（执行顺序；本片不执行）

1. 开窗前：树干净、build 进程 0、`cargo test --no-run` EXIT=0。
2. 5 分钟三点复采基线：`baseline_free = min(free)`，`signed floor = baseline_free - 3072 MiB`。
3. 常规 monitor `util<40` 60/60（`-Required 60 -MaxMinutes 30`，默认 `MaxUtil=40`，**无例外预批**）。
4. monitor 通过后才冻结：`freeze-guards-perworld-s1.ps1 -Attempt 1`（期望 `total=92 exes=9 inputs=83`）。
5. `-GuardsOnly` 全参数复核：`GUARDS_OK=92/92`。
6. body：`run-window.ps1 -Attempt 1 -GuardsFile ... -OutDir ...`，期望 9/9 exit 0、sha_guards 92/92、
   post-window exe recheck 9/9、zero Rust compile、floor 未破、residual 0。
7. 证据归档（facts/logs/monitor/entity-results/guards 副本 + manifest）。

**floor 口径（同 attempt5 先例）**：`signed floor` 生效；runner 自身 floor =
`min(OpenFreeMiB=10240, baseline_free-3072)` 只是**下界**——若某样本落在 `[10240, signed)`，runner 不会自停，
**操作员必须按 signed floor 手动 STOP**并原样保留事实。基线不达标 → 顺延，不放宽。

**窗口构建规则**：冻结后只允许 `--features cuda-probe` 口径构建/运行；窗口内禁止默认 features 构建
（会重链 exe、破坏已签 guards）。

## 4. 未声明 / 边界

- 不做任何性能声明；L v1 只覆盖 per-world geom 位姿（灯数组仍模型级单份，per-world 灯留 L v2/K）。
- 不覆盖 AA+perworld；不对上游 nworld>1 做数值声明。
- guards/manifest 哈希只作完整性证据，**不单独作为任何通过依据**。
- 本片零 GPU：不冻结、不开窗；窗口执行与冻结归实现片，用户双签（计划+基线）后进行。

## 5. 证据清单

| 文件 | 内容 |
| --- | --- |
| `materials/entity-table.json` | 9 实体 + 判据 + 期望守卫数（9/83/92）+ 排除项 |
| `materials/window-plan.txt` | ASCII 一页窗口计划（逐字命令/floor/失败条款/双签栏） |
| `receipt.json` | 材料 id、脚本身份（sha/bytes/LF/ASCII）、签署状态（pending）、not_claimed |
| `manifest.txt` | 本目录除自身外全部文件 sha256/bytes（repo-relative） |
