# QA 复核：R06 recipe-params 反例非空转补强（`8618e447`，卡 E）

- 任务：`2026-11-11-render-r06-flex-multiframe-recipe-params-negative-new-key-check`（owner QA）。复核对象：提交 **`8618e447`** + 卡 `2026-11-11-render-r06-flex-multiframe-recipe-params-negative-new-key`。零 GPU、只读复核。
- 基线：`e4f1c71f`（卡 E 登记/start）。

## 结论

**复核通过（0 阻断，0 低 + 3 观察）。** 卡 C 的 L2（21 项新增断言非空转未证）已按零偏差最小规格补上：新增键负例 `use_textures` 直接证明 **bool 形态**（10 项同数据驱动路径）非空转；既有键负例保留；21 项比对口径逐行未动；证据如实标注未覆盖的 4 组形态（非声明）。

## 一、改动与正确性（✅）

- `tests/support/render_bvh/multiframe_recipe.rs`：**+11/−11，全部在 `recipe_param_mismatch_is_detected` 的负例块内**（函数外零改动）；`assert_params_match_capture` 与 21 项比对逐行未动；文件仍 **188 物理行**（<500）。
- 负例 1（**既有键**，保留）：改 `samples_per_pixel`（7 项口径时代已比对）→ `catch_unwind` 必须 panic；负例 2（**新增键**，新）: 翻转 `use_textures`（21 项之一，10 布尔数据驱动循环内）→ `catch_unwind` 必须 panic；断言消息分别标注「既有键」/「新增键」；正例对照保留。
- 我独立重跑：`cargo test --locked --features cuda-probe --test render_flex_multiframe -- --test-threads=1` ⇒ **41 passed / 0 failed / 1 ignored**（含 `recipe_param_mismatch_is_detected` 通过、正例通过），与证据一致；用例数未变（本卡改负例内容、不增删用例）。
- 顺带修正卡 C 的错标：卡 C 注释把 `use_shadows` 写成「本次**新增**的布尔项」，但 `use_shadows` 是 7 项口径就有**既有**键；卡 E 已改为「既有键 = `samples_per_pixel`、新增键 = `use_textures`」，与我复核卡 C 的 L2 记录一致。

## 二、覆盖映射的诚实度（✅ 我逐项核）

- `logs/coverage-21-mapping.txt`：21 = `nworld`(1) + render flags(3) + 10 布尔 + `shadow_light_fraction`(1) + splat null(6)；与实现分组一致（我核过实现里的 1+3+10+1+6 结构）。
- **直接负例只覆盖 bool 形态**（`use_textures` 与同组 9 项同一数据驱动断言循环）；`nworld`/render flag/f64/splat-null 四组（共 11 项）**没有**定向负例——README/receipt 均明确写为**非声明**（"does not claim negative coverage for the non-bool assertion forms"），没有把「21 项全部被负例证明」写成结论。符合 Chief「1 新增键负例 + 保留 1 既有项负例」的最小规格。

## 三、证据 / 身份 / 边界 / 门禁（✅）

- 卡 C 证据 README 追加 **§7（append-only）**：我实测 6,779 → 7,940 B、**共同前缀 6,779 = 旧长度**（纯追加，原字节未动）；§7 含规格 sha **`d7fcf156…`/7,608 B**（我实测 target 规格文件 7,608 B/`d7fcf156d7ce7f1fec0f30d13d96a53d88b2045dcfc7eef23aafa1f0a9656638` 一致）、新键负例说明、exe 身份 supersession；卡 C 目录 manifest 我重算 **7/7** 全对。
- 新证据目录 manifest **6/6** 重算全对（bad=0/missing=0/extra=0、不自列）；README/receipt/logs（diff、host tests、identity、coverage mapping）齐。
- 身份：multiframe 测试 exe 卡 C 点值 `d226bb76…`/118,588,392 B → 卡 E 后 **`0966bf4c…`/118,588,904 B（+512 B）**；我实测 on-disk exe **118,588,904 B / `0966bf4c…`** 与声明一致；证据引用卡 D 的「同源重建不可复现」caveat，并写明「R06 后续窗需重冻、本卡不重冻」。
- 边界：`git diff e4f1c71f..8618e447` 仅 `tests/support/render_bvh/multiframe_recipe.rs` + 证据/`.normify`；`src/`/`fixtures/`/`scripts/` 零改动、`*.cu` 空；capture JSON 未动（`767f8c1b…` 只读）；`git ls-files --eol` = `i/lf w/lf`；不改判据/参考/容差/相机/recipe；零 GPU、未执行 `--ignored` GPU 用例。
- 门禁 `.normify/runs/…-negative-new-key/`：fmt/clippy/clippy-cuda exit 0、unit **1,406/0/402**（与卡 C/D 后基线一致，未增删用例）。

## 四、观察（不阻断）

- **O1（剩余覆盖缺口，已如实非声明）**：4 组形态（`nworld` 1、render flag 3、`shadow_light_fraction` 1、splat null 6）仍无定向负例；卡 C 的 L2 因此只是**部分关闭**（bool 形态已证）。按 Chief 的最小规格这是可接受的，证据已明确非声明；若后续要继续收口，可对 render flag 或 splat-null 各加 1 个负例（每加一个负例 test exe 会再变，需重冻）。
- **O2（错标修正）**：卡 C 注释「本次新增的布尔项 use_shadows」是错标（`use_shadows` 为既有键）；卡 E 已修正为既有/新增的正确区分——本卡顺带清了这条记录面问题。
- **O3（身份 caveat 延续）**：exe sha 为构建时点测量值；同源重建不保证逐字节一致（卡 D 已实证），后续 R06 窗口以冻结产物为准。

## 五、口径

不宣称渲染数值/等价/设备端行为变化；不宣称非 bool 形态被负例覆盖；不改判据/参考/容差；不重冻、不占窗；零 GPU。
