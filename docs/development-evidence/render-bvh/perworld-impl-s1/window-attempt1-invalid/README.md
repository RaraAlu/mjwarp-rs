# per-world S1 窗口 attempt1 证据（INVALID_ROUND，record-only，原样留档）

被服务任务：`2026-10-30-render-perworld-impl-s1`（逐 world 渲染实现 S1，方案 L）
窗口卡时点：本 attempt 由 2026-11-03T22:24:54+08:00 的用户双签窗口执行（窗口计划 v1 + baseline 三点 + signed floor **11,673 MiB**）。
结论：**INVALID_ROUND（runner exit 10）**。未重试、未改阈值/容差/参考；本目录为原样事实。

## 1. 环境（全绿，非失败原因）

- 开窗前三点 free **14,368 / 14,368 / 14,368** MiB ≥ signed floor 11,673；util 10/9/10%；filtered compute 0、build 0
- 常规 monitor：82 样本、longest_clean **60/60**、free_min 14,368、exit 0
- freeze attempt1：94 条（11 exe + 83 输入），aggregate `62ebcd7adf7e13e582d247c54fd19e816efce59d9d0a47d3e269df29372d6cad`；GuardsOnly `GUARDS_OK=94/94`
- body：`zero_rust_compile=True`、`floor_breached=False`、`increment_exceeded=False`、`post_window_exe_recheck=11/11`、`residual_processes=0`、`guard_aborted=False`；body 47.9 s、161 采样、used_peak 1,546 MiB、util_max 8%

## 2. 实体结果（3 过 / 8 败）

| 实体 | exit | 结果 |
| --- | --- | --- |
| render_context_lifecycle | 0 | 过 |
| render_hfield_reference | 0 | 过 |
| render_mesh_bvh | 0 | 过 |
| render_per_world | 101 | 败：`InvalidArray("ray_count")`（发射前，无设备工作） |
| render_e2e | 101 | 败：183 像素（v2.2 已签清单口径未生效） |
| render_kernel_rgb | 101 | 败：183 像素 / max 73 ULP（同上） |
| render_kernel_depth_seg | 101 | 败：同上 |
| render_kernel_flags | 101 | 败：同上 |
| render_r10_multi_world | 101 | 败：`left 183 / right 0`（同上） |
| render_shadow_reference | 101 | 败：同上 |
| render_skybox_reference | 101 | 败：同上 |

## 3. 根因（两处，均为内容/构造缺陷）

1. **guards 缺 criterion v2.2 两条已签清单**：attempt1 guards 为 94 条（11 exe + 83 输入），未带 scene-a 球面轮廓清单；`run-window.ps1` 在无 `outline_lists` 时不导出 `MJWARP_RENDER_OUTLINE_LIST_DIR`，7 个回归实体按严判位级比较 → 恰好 183 像素失败（该 183 即 v2.2 已签清单的 183 条目），**不是产品回归**。
2. **新实体射线构造错**：`tests/render_per_world.rs` 用 `s7b1::precomputed_rays`（默认 2 相机 = 240,000 射线），而 perworld 参数 `cam_active=[true,false]` 仅 1 相机（76,800 射线）→ `InvalidArray("ray_count")`，发生在首个发射前，无设备工作。

## 4. 修复（不进本 attempt，见材料 v2 与实现卡后续提交）

- guards 94 → **96**（接入 v2.2 两条已签清单，复用 attempt5 归档字节；修复后曾以 attempt99 临时冻结演练 `GUARDS_OK=96/96` + `OUTLINE_LISTS_OK=2/2`——该演练的 guards 文件已删除、**没有归档日志**，此处仅记录控制台输出，不作证据；正式 attempt2 冻结已完整归档于 `window-attempt2-invalid/`），runner 自动导出豁免 env；
- `render_per_world` 的 RayBuffer 改为按自身 `resolved.plan` 构造（76,800 射线）；
- 判据、容差、参考、内核身份均未改。

## 5. 本目录内容

- `facts.txt` / `baseline.txt` / `machine-load.txt` / `pre-window-snapshot.txt`：窗口事实（原样）
- `samples.csv`（161 窗口采样）、`entity-results.csv`（11 实体 exit/wall）
- `sha-guards.txt`：94 条 SHA_OK 清单（含 11 个冻结 exe 的 sha/bytes）
- `guard.log`、`guard_aggregate.json`：`PRECHECK_OK` / `GUARDS_OK=94/94` / `INVALID_ROUND exit10`
- `logs/*.log`：11 个实体日志（原样）
- `.abort` / `.stop-sampler`：runner 控制文件（原样）

**已知证据缺口（如实记录）**：attempt1 的 guards JSON 本体在修复演练时被 attempt99 演练冻结覆盖并随后删除，未保留原文件；其身份由 `facts.txt` 的 `guards_aggregate_sha256=62ebcd7a…` 与 `sha-guards.txt` 的 94 条逐条 sha 佐证。attempt2 冻结与窗口证据将完整保留 guards 本体。
