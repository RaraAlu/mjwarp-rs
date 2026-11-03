# per-world S1 窗口 attempt2 证据（INVALID_ROUND，record-only，原样留档）

被服务任务：`2026-10-30-render-perworld-impl-s1`（逐 world 渲染实现 S1，方案 L）
窗口时点：2026-11-03T22:41 用户签（计划 v2 + 基线 14,839/14,839/14,827 + **signed floor 11,755 MiB** + QA 材料 v2 pass），22:47:20 开窗。
结论：**INVALID_ROUND（runner exit 10）**。未重试、未改阈值/容差/参考；本目录为原样事实。

## 1. 环境与前置（全绿，非失败原因）

- 开窗前三点（22:41:21/24/27）：free **14,840 / 14,840 / 14,840** MiB ≥ signed floor 11,755；util 2/5/4%；filtered 0、build 0
- monitor：**60 样本、longest_clean 60/60**、util_max 5、free_min 14,839、exit 0
- freeze attempt2：**96 = 11 exe + 83 输入 + 2 条 criterion v2.2 已签清单**；guards `036e7f86b7e8596bc60e33316ff9cf5f2b6bfdc7cb65d8840e570aedd03b67c3` / 38,659 B / aggregate `a3712c2a858349979ed6e33375668bf8844e53e96fbe28c1931b0339ecba8001`
- GuardsOnly：**OUTLINE_LISTS_OK=2/2** + **GUARDS_OK=96/96**
- body：`zero_rust_compile=True`、`floor_breached=False`、`increment_exceeded=False`、post exe recheck **11/11**、residual 0、`outline_lists_post=2/2`；164 采样、used_peak 1,493 MiB、util_max 9、free_min 14,569 MiB

## 2. 实体结果（10 过 / 1 败）

- **通过 10**：`render_per_world`（**新实体首次通过**）、`render_context_lifecycle`、`render_e2e`、`render_hfield_reference`、`render_kernel_depth_seg`、`render_kernel_flags`、`render_kernel_rgb`、`render_mesh_bvh`、`render_r10_multi_world`、`render_skybox_reference`
- **失败 1**：`render_shadow_reference`（exit 101）
  - `shadow_off_bitwise_regression_to_pre_unlock_reference`：cam1 depth `mismatch=183, max_ulp=73`，first `(camera 1, pixel 28969, x=169, y=120)`
  - `shadow_on_matches_captured_reference`：cam1 depth `mismatch=183, max_ulp=73, beyond_ulp=149`，同一 first pixel；cam0 rgb/depth/seg 全 0
  - `shadow_visibility_binary_control` 通过

## 3. 根因（1 处，测试支持面）

失败值与 criterion v2.2 已签 scene-a 球面轮廓清单逐项一致（rows=183 / exact_max=73 / first_pixel=28969），且 `OUTLINE_LISTS_OK=2/2` 与 env 已生效——但 **`render_shadow_reference` 的 depth 比较实现（`tests/support/render_bvh/shadow_scene.rs:155`）直接断言位级相等，没有走 v2.2 清单豁免 helper**（其余 9 个回归与 SDF attempt5 的实体都走 `outline::diff_depth_with_outline` / `recipe_depth_diff`）。

这不是产品/内核回归：同一批 183 像素在 SDF attempt5 与本次 attempt2 的其它 7 个回归实体上均按 v2.2 清单通过。修复方向（attempt3 前，只改测试支持面、判据不变）：把 shadow_scene 的 cam1 depth 比较改为 v2.2 清单豁免（`Arm::Recipe`），清单外位级/清单内 |ULP|≤128/成员逐像素；env 未启用时保持原行为。

## 4. 未做

- 未重试、未重冻、未改判据/容差/参考/清单/源码/内核身份；未在窗口内改任何文件。
- 本次 guards 本体已随本目录归档（`guards-perworld-s1.json`，attempt=2 / 96 条），`scripts/` 下的生成物已移除以保持树干净；attempt3 会重新冻结（同 96 口径）。

## 5. 本目录内容

`facts.txt`、`baseline.txt`、`machine-load.txt`、`pre-window-snapshot.txt`、`samples.csv`、`entity-results.csv`、`sha-guards.txt`、`guard.log`、`guard_aggregate.json`、`guards-perworld-s1.json`、`cargo-artifacts.jsonl`、`logs/*.log`（11 实体）、`.abort`/`.stop-sampler`；manifest 不自引用。
