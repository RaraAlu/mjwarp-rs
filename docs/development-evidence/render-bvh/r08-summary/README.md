# R08-RGB 逐相机掩码片 汇总（整片收口）

上游计划：`docs/development-evidence/render-bvh-precheck/plan.json`（本片对应 slice **R08-S3**，2026-10-30 标 delivered）
任务链：离线实现 `2026-10-29-render-r08-rgb-hetero` → 窗口 `2026-10-29-render-r08-rgb-hetero-validate` → 复核 `2026-10-30-render-r08-rgb-hetero-validate-check`
窗口时间：**2026-10-30 12:05:31–12:06:27（+08:00）**，已签常规窗口（`util<40`，未使用任何例外、无 `-OpenUtilLimit`、判据零改动）

## 1. 本片做了什么

| 步 | 内容 | 证据/提交 |
| --- | --- | --- |
| 离线实现 | host 逐相机三通道掩码 + 内核 `render_rgb/depth/seg[nrender]` 语义修正（含 latent 负偏移修复，按 S7b3 先例保持文件棘轮） | `2026-10-29-render-r08-rgb-hetero`（S1 结构反转 `7de14d30`） |
| 窗口材料 | 实体表 9 条、`freeze-guards-r08-rgb-hetero.ps1`（ASCII、`-Attempt`）、窗口计划一页、基线快照 | `e796d45e`（`r08-rgb-hetero-window-material/`） |
| 验证窗 | 新 suite `render-r08-rgb-hetero` attempt=1：冻结 77 条目 → `-GuardsOnly` → 9 实体 → 窗后复查 | `c5592d65`（`r08-rgb-hetero-validate/`）+ guards 产物 `2099430e`（原字节，blob sha == 工作树） |
| 复核 | QA 独立复核：pass / 阻断 0 / 需修 1 / 观察 8；唯一需修项（manifest 两行按 committed blob 重算）已修 | 复核卡 `337abc14`；QA 收口 `9b1c4d81`；需修 `89afd36e` |

## 2. 门禁与 guards

- 门禁（交付时挡板）：`fmt` / `clippy` / `clippy-cuda` / `unit` 全过；`--no-run` 日志含 exit 码与 `Finished`（离线实现片）。
- 板面：收口时 **583 验收 / 0 运行 / 0 错误 / 0 警告**（HEAD `9b1c4d81`）。
- guards：`scripts/validate/render-bvh/guards-r08-rgb-hetero.json` sha256 **`e81d69a4daa0268777eff698ecf7621dee333076915f805c2e447cc9d0c34f3a`**、**28,970 B**、条目 **77** = 9 exe + 68 只读输入、aggregate **`3ad8b3522314222629f1cf8e5a9703a9c53ad75433373d0c06f402803a517f1c`**；stash 9/9；`-GuardsOnly` = `GUARDS_OK=77/77`。

## 3. 窗口六项（`r08-rgb-hetero-validate/receipt.json`）

1. **9 实体 exits**：全 0（new `render_r08_cameras` 11,644 ms；8 回归 1,312–9,378 ms）
2. **sha_guards**：77/77（9 exe + 68 输入），mismatches 0
3. **窗后 exe 复查**：9/9、mismatches 0、residual 0
4. **零编译**：`zero_rust_compile=True`（deps `1839→1839`、max_write 不变）
5. **资源**：monitor 60/60（util 2–8、free_min 13,452、ge40=0、filtered/build 0）；窗内 samples 177、util_max 7、free_min 13,179、used_peak 2,883、delta 276 ≤ 3072、`util_streak_ge_80=0`；run-window floor 10,240 与 pre-window floor 10,374 均未破
6. **monitor 历史**：第五轮达标；前四轮失败（外部 viz CUDA context 0/60、外部重负载 37/60 与 7/60、桌面 3D 引擎尖峰 37/60）原样保留在 `monitor-history/`

## 4. 逐相机判据摘要

- 异构 RGB：cam0_on `[0,-1]` sha `f62f4d73…`、cam1_on `[-1,0]` sha `1325755c…`、all_off `[-1,-1]` sha `b09e2eec…`
- 异构 depth/seg：`depth_adr=[0,-1]`、`seg_adr=[-1,0]` 逐相机对参考成立、未启用相机拒绝成立
- 矩阵 4 例（all / mask_first / names_all / indices_reversed）：`rgb beyond_1_lsb=0`、`depth mismatch=0/max_ulp=0`、`seg mismatch=0`（120,000 或 76,800 像素/对）
- cam1 三通道全关 early-return、隔离/重排位级一致、动态 vs 预计算 depth/seg 位级一致、两跑 sha 相等（`f9fe1b9e…`）
- 8 回归：各自 `test result: ok`（0 failed），既有位级/ULP 判据零漂移

## 5. 已知缺口处置

- `2026-10-28-render-per-camera-rgb-gap`（逐相机 RGB 通道开关）：**已被本片窗口覆盖**，卡按「已覆盖」关闭（abandon），R08 范围内无剩余项。
- `2026-10-30-render-window-guard-aggregate-suite-hardcode`（QA O1）：`guard_aggregate.suite` 标签硬编码 R05 串，属共享窗口工具缺陷；aggregate_sha256 与条目不受影响；单独立片，proposed 不 start。

## 6. 未覆盖项（明确不宣称）

- per-world 几何/灯位姿渲染（方案 L v1，尚未实现；灯位姿 v1 不覆盖）
- SDF `gtype==8` 剔除背 latent fix（等 R08-S3 收口后独立片）
- AA + per-world（L v1 显式 `Unsupported`）
- mesh polygon 容量 16→20（等用户批 A1/A2/C）
- 性能声明、T4/Linux 双平台
- 本片不宣称与上游行为相同、不宣称产品级验收、不宣称跨平台结论

## 7. 文件

- 窗口证据：`docs/development-evidence/render-bvh/r08-rgb-hetero-validate/`（README/receipt/facts/entity-results/sha-guards/logs 9 份/monitor/monitor-history/materials/manifest）
- 窗口材料：`docs/development-evidence/render-bvh/r08-rgb-hetero-window-material/`
- 计划：`docs/development-evidence/render-bvh-precheck/plan.json`（R08-S3 slice + delivery row + N13）
