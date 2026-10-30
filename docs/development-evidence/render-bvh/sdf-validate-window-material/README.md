# SDF 验证窗材料（U4；已落卡 proposed，未执行）

任务（材料）：`2026-10-31-render-sdf-validate-window-material`（Arch；离线、零 GPU）
被执行的任务：`2026-10-31-render-sdf-validate`（GPU 满载验证窗，需用户双签）
执行草稿（本材料来源）：`target/u4-validate-window-material.md`、`target/u4-entity-table.json`、`target/u4-expected-products.json`

## 0. 一句话

在用户双签的 GPU 窗口里跑 **1 个新 target `render_sdf_reference` + 8 条回归**，用已入库的上游 SDF 采集参考逐相机比对（RGB ≤1 LSB、depth bitwise 优先且 `max_ulp≤4`、seg exact、两跑 sha 相等），并做 shadow 开/关与 DepthSeg 跨路径一致性；**常规 `util<40`、不使用 authoring 例外**；失败即停。

## 1. 脚本身份与 U4a dump pin

| 项 | 值 |
| --- | --- |
| `tests/support/render_bvh/sdf/model-312.json` | **53849** B / `7de15ff458aa8f6b1c026bbed5d9b1fbaf8f5dd39b385bab0b79d5c289b40d36` |
| `tests/support/render_bvh/sdf/model-mixed-312.json` | **54205** B / `60698f6baaeff8dcc11e725e56242bd5d6f947741cb7070b4e1f32ec13765578` |
| dump 工具 | `target/render-bvh/s0a/compile_scene_312.exe`（MuJoCo 3.12.0，sha `cd91e099…`/161,280 B，离线） |
| U4a 卡 | `2026-10-31-render-sdf-model-dump`（verified） |

## 2. 窗口纪律（常规 util<40，无例外）

- 新 suite `render-sdf-validate`、attempt=1、新冻结器 `freeze-guards-sdf.ps1`（ASCII、≤200 行、PowerShell 5.1）；**不复用 R05/R02/R08 guards**（U1/U1b 改了产品代码 ⇒ 所有 exe 哈希都变）。
- 预期冻结条数：**9 exe + 83 inputs = 92**（scene-sdf 10 + scene-hfield 10 + legacy 47 + scene-shadow 16），冻结脚本硬断言。
- 开窗前三点基线 → `floor = min free − 3072`；monitor `-Required 60 -MaxMinutes 30`（默认 MaxUtil=40，**不得** `-MaxUtil 60`/`-OpenUtilLimit`）；失败即停、不重试、不换例外。
- 五条命令逐字见 `materials/window-plan.txt`。

## 3. 判据（零放宽）

RGB `beyond_1_lsb=0`/`max_byte_diff≤1`；depth `mismatched=0` 且 `max_ulp≤4`/`beyond_ulp=0`；seg `mismatched=0`；两跑 sha 相等；shadow 臂只做一致性（depth/seg 位级相等、rgb `shadow_off≥shadow_on`、差异像素>0、两臂两跑 sha 相等）；DepthSeg 跨路径位级一致。

## 4. 实体（9 条，Chief 2026-10-31 定稿）

`render_sdf_reference`（new）+ `render_e2e`、`render_kernel_rgb`、`render_kernel_depth_seg`、`render_kernel_flags`、`render_r10_multi_world`、`render_skybox_reference`、`render_context_lifecycle`、`render_mesh_bvh`；移出 `render_camera_rays`/`render_r08_cameras`。详见 `materials/entity-table.json`。

## 5. 参考产物（U3 已入库，只读）

`fixtures/render-bvh/scene-sdf/capture-sdf/{sdf-mesh,sdf-occluder}/`：两组各 `rgb.npz`/`depth.npz`/`seg.npz` + `capture.json`，共 8 文件 + 2 个场景 XML；recipe 见 `materials/expected-products.json`（nrender=2、cam-front 320×240 + cam-top 240×180、total_rays 120,000、fast_math=false、shadows=true、spp=1、repeat=2、legacy 69）。

## 6. 边界与不宣称

- 本材料**只写计划**：零 GPU、零执行、零参考写入、零判据/容差改动。
- 不宣称 SDF 求交实现、plugin SDF、per-world、AA；不宣称任何数值结论（结论只在窗口证据里按判据数字给）。
- 失败/无效轮保留 attempt 目录，只有文档化修复后才 attempt=2。

## 7. 证据清单

| 文件 | 内容 |
| --- | --- |
| `README.md` | 本文 |
| `receipt.json` | 材料 id、U4a dump pin、文件哈希、双签状态 |
| `manifest.txt` | 本目录除自身外全部文件 sha256/bytes |
| `materials/entity-table.json` | 9 实体 + 判据 + guards 预期 |
| `materials/window-plan.txt` | 窗口计划一页（ASCII）：命令逐字、基线/floor、时限、失败策略、双签栏 |
| `materials/expected-products.json` | U4a pin、参考 sha、判据、guards 预期、窗口命令、双签栏 |
