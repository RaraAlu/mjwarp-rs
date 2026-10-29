# R08-RGB 验证窗材料（离线；未开窗、未执行真冻结）

任务：`2026-10-29-render-r08-rgb-hetero-window-material`（Arch 2026-10-29 登记；Chief 批准 S2 窗口材料片）。
性质：只写窗口材料与 QA O1 测试用例；**不开 GPU 窗口、不执行真冻结**。真实冻结与窗口执行归 `2026-10-29-render-r08-rgb-hetero-validate`。

## 1. 交付物

| 交付 | 路径 | 说明 |
| --- | --- | --- |
| 新冻结器 | `scripts/validate/render-bvh/freeze-guards-r08-rgb-hetero.ps1` | ASCII、165 LF、PSParser 0 错；suite `render-r08-rgb-hetero`、`-Attempt`；断言 9 exe + 68 输入 = 77 条目 |
| 实体表 | `materials/entity-table.json` | 9 实体：`render_r08_cameras`（new，含 O1 三通道全关）＋ 8 回归；判据沿用、无新参考 |
| 窗口计划 | `materials/window-plan.txt` | 逐字命令、`-GuardsOnly 77/77`、floor 公式、常规 util<40 无例外、时长/失败回滚 |
| 基线快照 | `materials/gpu-baseline.txt` | 只读 GPU/工具链/HEAD 快照；注明签署基线需开窗前 5 分钟重取 |
| QA O1 用例 | `tests/support/render_bvh/r08_rgb_alloff.rs`（69 LF）＋ `tests/render_r08_cameras.rs` 入口（334 LF） | cam1 三通道全关（rgb/depth/seg 均 `[true,false]`），覆盖内核 `render_megakernel.cu:131` all-off early-return；`#[ignore]`，窗口内执行 |

## 2. 冻结器 ASCII/静态自检（`logs/freeze-script-checks.txt`）

- sha256 `0c66fdbb3b5a8101e6e3fd1618920526aed1231552aafc13c92259011278c3ac`、bytes 8 761、LF 165（≤200）、UTF-8 行数 165。
- nonascii=0、CR=0、BOM=false、注释含 `$` 行=0、PSParser errors=0、bounds_ok=true。
- 本片**未执行** `-Attempt 1` 真冻结；窗口前才执行。

## 3. 实体表与判据

- 顺序：`render_r08_cameras`（new）→ `render_skybox_reference`、`render_camera_rays`、`render_context_lifecycle`、`render_e2e`、`render_kernel_depth_seg`、`render_kernel_flags`、`render_kernel_rgb`、`render_r10_multi_world`（8 回归）。
- 判据零放宽：RGB `mismatched=0/max_byte_diff=0/beyond_1_lsb=0`；depth `mismatched=0/max_ulp≤4/beyond=0`；seg 0；新 RGB 用例两跑 sha 相等；8 回归现有位级基线零漂移。
- **无新参考文件**：复用 S7b2 same-recipe RGB 与 S7b1 depth/seg 参考。
- 守卫预期：9 exe（产品+测试改动，全部 exe 变化）+ 68 输入（内容不变）= 77 条目；R05/R02 两套 guards 不动。

## 4. 窗口计划要点（`materials/window-plan.txt`）

- suite `render-r08-rgb-hetero`、attempt=1、guards 文件 `scripts/validate/render-bvh/guards-r08-rgb-hetero.json`。
- 逐字命令：常规 monitor（`-Required 60 -MaxMinutes 30`，默认 MaxUtil=40，不带 `-OpenUtilLimit`）→ freeze `-Attempt 1` → `-GuardsOnly`（期望 `GUARDS_OK=77/77` + stash 9/9）→ stage B（不带 `-OpenUtilLimit`）。
- floor 公式：`floor_mib = 开窗前 baseline free - 3072 (BudgetMiB)`（R02 先例 12 858−3 072=9 786）；free_min 低于 floor 或 used 增量 >3 072 MiB 即中止。
- 时长：monitor 预期 5–10 min（硬顶 30）＋冻结/自检 1–2 min＋实体体 60–120 s → 整窗 30–60 min。
- 失败/回滚：monitor 不达标、freeze 不符、GuardsOnly≠77/77、任一实体红 → **停手报 Chief**；不改阈值、不预批例外、不重试连刷；invalid round 留档后按文档修复重登记 attempt=2。
- 窗口最早 2026-10-30 12:00–13:50（用户醒后双签：计划＋开窗前基线＋冻结后的 77 条目身份）；未签不开窗。

## 5. QA O1 用例

- `tests/support/render_bvh/r08_rgb_alloff.rs::cam1_all_channels_off`：rgb/depth/seg 均 `[true,false]`；断言 cam0 三通道对参考、cam1 三通道读取全部拒绝、两跑输出 SHA-256 相等。
- `tests/render_r08_cameras.rs` 加 `gpu_cam1_all_channels_off`（`#[ignore]`）；本片仅编译（`--no-run`），不执行。

## 6. 门禁

- `normify gate`：fmt / clippy / clippy-cuda / unit 全过；unit **1106 passed / 0 failed**（无新增单测，O1 为 GPU 用例）。
- `cargo test --locked --features cuda-probe --no-run --test render_r08_cameras` 编译通过（含新 support 文件）。

## 7. 非声明与遗留

- 本片不做任何数值验收：窗口材料只准备、不执行；真实冻结/开窗须用户双签。
- 不改判据/容差/参考/夹具；不改产品代码（O1 仅新增测试文件与测试入口）。

## 8. 证据文件

| 文件 | 内容 |
| --- | --- |
| `materials/entity-table.json` | 9 实体 + 判据 + 期望守卫数 |
| `materials/window-plan.txt` | 一页窗口计划（ASCII） |
| `materials/gpu-baseline.txt` | 只读基线快照与 floor 公式 |
| `logs/freeze-script-checks.txt` | 冻结器 sha/bytes/LF/ASCII 三项/PSParser |
