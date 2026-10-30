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

## 2. 窗口纪律（先常规 util<40；仅当失败才启用已签条件式 validation-only 例外）

- 新 suite `render-sdf-validate`、attempt=1、新冻结器 `freeze-guards-sdf.ps1`（ASCII、≤200 行、PowerShell 5.1）；**不复用 R05/R02/R08 guards**（U1/U1b 改了产品代码 ⇒ 所有 exe 哈希都变）。
- 预期冻结条数：**9 exe + 83 inputs = 92**（scene-sdf 10 + scene-hfield 10 + legacy 47 + scene-shadow 16），冻结脚本硬断言，且**逐条核 guards 文件**（不只核总数）。
  - 与 S3 窗的 68 inputs 对账：**+10 scene-sdf**、legacy **42 → 47**（+5 perworld）；scene-hfield 10、scene-shadow 16 不变。
- **条件式执行（用户 2026-10-31 签）**：先常规 `util<40`（三点基线 → floor → monitor 默认 MaxUtil=40，不加 `-OpenUtilLimit`）；60/60 则正常开窗（`not_a_normal_threshold_pass=false`、`authoring_exception_used=false`）。**仅当常规失败**（保留失败 CSV/快照/判定）才启用已签的 `validation_only_exception_util_lt_60`：五要素 + 重取基线 + `-MaxUtil 60`（新 CSV、PowerShell 5.1）+ 60/60 才开窗；例外生效证据 = 重算 util≥40 仍 clean 的样本/连续段、`ge60=0`、`util_max<60`，全 <40 则记 `exception_effectively_not_load_bearing=true`。**现场按序执行、不预设路径；绝不可写成常规通过。**
- 五条命令（4 常规 + 1 条件式例外 monitor）逐字见 `materials/window-plan.txt`；另见其中 `CONDITIONAL EXECUTION PROTOCOL`。
- **编译口径**：freeze 必然跑 `cargo test --no-run`（9 个新 exe，属预期编译）⇒ **不宣称「窗口零 Rust 编译」**；证据口径为「**body 开始后零编译**」（deps 快照取在 body 之前）。

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

## 8. 用户签字（2026-10-31，via Chief）

- 用户签：**常规 `util<40` 窗口 + 条件式 `validation-only util<60` 例外**（二选一按现场结果，先常规）。
- 签署时间：`2026-10-31T00:55+08:00`（via Chief 转达；user 卡片答复后 Chief 即时转达；卡片 UI 精确时间戳未暴露，转达时点 ±5 分钟；用户批准的是协议内容（regular + conditional），amend 文本由 Arch 按 Chief 转达落字）；材料 amend 后 pin 新提交。
- 协议细节：`materials/window-plan.txt` 的 `CONDITIONAL EXECUTION PROTOCOL` 与 `materials/expected-products.json` 的 `conditional_protocol`。
- 两条 QA 口径已落：①`inputs 83` 分解 + 与 S3 的 68 对账（+10 sdf、legacy 42→47）并逐条核 guards；②「body 开始后零编译」替代「窗口零编译」（freeze 编译为预期步骤）。

## 9. QA 需修修复（2026-10-31，freeze 前）

| # | 需修 | 修法 |
| --- | --- | --- |
| P1 | `receipt.files.receipt.json` 自条目过期（`4ef24623…/2078 B`；amend 前实测 `87631dba…/3825 B`） | **删除自引用条目**（manifest 本就不登记自己，自引用哈希无法自洽）；receipt 自身哈希只由 `manifest.txt` 记录，并在 receipt 加 `files_note` 留痕 |
| P2 | 签字时间占位（原先为不完整时刻写法，4 文件共 10 处） | 统一改为 **`2026-10-31T00:55+08:00`**，并在签名块/receipt 注明来源：Chief 即时转达、卡片 UI 精确时间戳未暴露、±5 分钟；用户批准的是协议内容 |
| P3 | README §2 标题「常规 util<40，无例外」与条件式内容矛盾 | 改为「先常规 util<40；仅当失败才启用已签条件式 validation-only 例外」 |
| P4 | 「5 条命令逐字」不成立（实际 4 条） | 补第 5 条**条件式例外 monitor 逐字命令**与新 CSV 名 `monitor-pre-window-exception.csv`（不覆盖常规失败 CSV），口径改「5 条（4 常规 + 1 条件式例外）」 |
| O1 | 测试文件行预算 260–360 贴线 | 改「≤400」 |
