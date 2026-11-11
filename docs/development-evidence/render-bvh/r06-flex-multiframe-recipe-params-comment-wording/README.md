# R06 多帧 recipe 模块 cam_res 口径注释措辞修正（卡 L，注释 only、零断言改动、零 GPU）

- 任务：`2026-11-11-render-r06-flex-multiframe-recipe-params-comment-wording`（owner Backend；p85；P8-渲染与BVH）。
- 来源：QA 卡 K 复核低 L1（`2026-11-11-review-low-findings-addenda` 的 C-L1 测试面 follow-up）；Chief 2026-11-11 13:02 指令。
- 边界：**仅测试注释措辞**——`tests/support/render_bvh/multiframe_recipe.rs` 4 行注释（第 8/51/131/168 行）；断言与 21/28 项比对逻辑零改动；不改 `src/`、`fixtures/`、`scripts/`、判据/参考/容差；零 GPU、零窗、不重冻。

## 1. 改动（4 处注释，行数不变 188）

| 行 | 原措辞 | 新措辞 |
| --- | --- | --- |
| 8 | `cam_res`（由 model/camera 隐式推导；单帧验收亦为隐式比对…） | `cam_res` 为**名义第 28 项**；多帧 capture 无 `context.resolved` 段、无独立对照源，故不新增字面断言，实际逐项断言 **27 项** = 26 recorded + nworld |
| 51 | 28 项口径；`cam_res` 隐式 | 28 项口径；实际逐项 27；`cam_res` 名义第 28、无字面断言 |
| 131 | 28 项口径；`cam_res` 隐式 | 实际逐项 27；28 项口径中 `cam_res` 为名义第 28 项 |
| 168 | 28 项 = 26 recorded + nworld + cam_res(隐式) | 同上 + 多帧 capture 无 `context.resolved` 段，无字面断言 |

- `git diff --numstat` = **4 insertions / 4 deletions**，全部为 `//`/`///` 注释行（`logs/diff-stat.txt` 含 `comment_only=true` 与 `-U0` 全文）。
- 断言零改动：`assert_params_match_capture`、`RECIPE_PARAM_KEYS`、21 项新增断言、防漏守卫逐字未动；文件仍 **188 行**（`logs/line-counts.txt`，<500 棘轮）。

## 2. 测试与身份

- `cargo test --locked --features cuda-probe --test render_flex_multiframe` ⇒ **42 tests: 41 passed / 0 failed / 1 ignored**（ignored = GPU 渲染用例，零 GPU 未执行）；`logs/test-log.txt`。
- multiframe 测试 exe 身份 old→new：`0966bf4cccc7312c…`/118,588,904 → **`cdd8598880b42fff…`/118,588,904**（注释改动影响 debug info）；**R06 后续窗需重冻，本卡不重冻、不占窗**；同源重建不保证逐字节可复现，身份以冻结产物为准（`logs/identity-old-new.txt`）。

## 3. 卡 K 的 L1 闭环与 backlog 计数说明

- 卡 K 复核低 L1：C-L1 的 addendum 留了测试面 follow-up（4 处注释仍写「单帧验收亦为隐式比对」「cam_res 隐式」），且 K 的 `backlog=1` 只算 F-L1、未计该 follow-up。**本卡闭环该测试面 follow-up**；K 证据（`plan/review-low-findings-addenda/README.md`）的 C-L1 处置为「文档面由 K 补、测试面由卡 L 闭环」，K 的 `backlog=1` 指 F-L1（账本拆字段/checker 断言）——口径一致，**不改 K 证据字节**（K 已 verified；如需关联由本目录引用）。
- 至此 C-L1 全闭：文档面（K）+ 测试面（L）。

## 4. 非声明

不宣称任何门禁/NP01/NP02 通过；不改判据/参考/容差/断言/行为；零 GPU、零窗、不重冻；历史证据不改写。
