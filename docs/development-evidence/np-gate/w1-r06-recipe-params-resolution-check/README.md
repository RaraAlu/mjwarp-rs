# QA 复核 W1 核查：R06 多帧 recipe 比对 7→28 既有实现范围核查（`2026-11-11-w1-r06-recipe-params-resolution`，提交 `47c28f5e`）

- 复核对象：卡 `2026-11-11-w1-r06-recipe-params-resolution`（`docs/development-evidence/plan/w1-r06-recipe-params-resolution/`）；内容提交 **`47c28f5e`**。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 0 低 + 2 观察）**。

## 1. 独立核验

- **实现范围 ✅**：实现提交 `20015625`/`8618e447`/`a20efe2a` 我逐个 `git show` 核路径——**只动 `.normify/`、`docs/development-evidence/`、`tests/`**，`src/`/`fixtures/`/`scripts/` 与 `.gitattributes` **零改动**（无产品 recipe/判据/夹具/eol 触碰）。
- **测试侧事实 ✅**：`tests/support/render_bvh/multiframe_recipe.rs` 188 行、`assert_params_match_capture` 在位、`RECIPE_PARAM_KEYS` 26 项数组 + nworld + cam_res = 28 项口径、键集合守卫（capture.parameters == RECIPE_PARAM_KEYS）在位。
- **CPU 自证 ✅（我重跑）**：`cargo test --locked --features cuda-probe --test render_flex_multiframe recipe -- --test-threads=1` → **5 passed / 0 failed / 0 ignored / 37 filtered out**（含 4 个 multiframe_recipe 用例 + 1 个 s7b1 用例），与卡内声明逐数字一致（实测输出见 log 末段）。
- **卡面事实 ✅**：复核时点 `2026-11-05-render-r06-flex-multiframe-recipe-params-backlog` 为 `backlog/0`（我在 `47c28f5e`/`76bd3ccc` blob 上核实，acceptance 2 条）；「建议治理片按既有实现关闭」与后续治理链（`8a04f15a`→`f2ad1ce2`）一致。
- **边界/证据 ✅**：提交 `47c28f5e` 仅本目录 + `.normify`；零代码、零 GPU、零窗；manifest 4/4、selfcheck RESULT: PASS、policy_scan 0 命中、四门禁记录齐、UTF-8/LF 无 BOM/裸 CR/NUL。

## 2. 发现

### 观察
- **O1**：被核查卡在复核时点为 `backlog/0`（符合两片自述）；其后已由治理片 promote 并关闭（`e414eb9b`/`f2ad1ce2`），属后续片，我将在治理关闭片复核时核对 acceptance 2/2 与测试输出。
- **O2**：「28 项 = 26 recorded + nworld + cam_res」中 cam_res 为名义第 28 项（多帧 capture 无 context.resolved 段、无字面断言），逐项断言实际 27 项——W1 已如实写明，未夸大成 28 项全断言。

## 3. 非声明

本复核只对诊断/核查的事实与范围负责；**不宣称**任何门禁或阶段准入通过；不宣称 NP 通过；
不宣称上游等价；不宣称位级等价；不宣称双平台或发布；零 GPU、零窗、不改判据/参考/容差/历史字节。
