# QA 复核：R06 多帧 recipe 参数比对补强（`20015625`，卡 C）

- 任务：`2026-11-11-render-r06-flex-multiframe-recipe-params-fill-check`（owner QA）。复核对象：提交 **`20015625`** + 卡 `2026-11-11-render-r06-flex-multiframe-recipe-params-fill`。零 GPU、只读复核。
- 基线：`8935d637`（卡 C 登记/start）。

## 结论

**复核通过（0 阻断，2 低 + 2 观察）。** 7→28 口径的补强**是真实比对**（26 个 recorded 键与 capture 逐项对照 + 防漏集合守卫；既有键反例已证，新增 21 项非空转待卡 E 补证），不是凑数；行数棘轮/identity/边界/门禁/证据均成立。低项是「单帧亦为隐式」的措辞不准确（单帧其实显式对照 cam_res；多帧 capture 无对照源，实际逐项断言 27 项）。

## 一、补强真实性（派工核心）

- **26 recorded 键 = capture 侧键集（我独立比对）**：`RECIPE_PARAM_KEYS: [&str; 26]` 与 `fixtures/render-bvh/scene-flex/multiframe/capture.json` 的 `parameters` 键集**集合完全相等**（两侧各 26，`Compare-Object` 无差异）；capture.json sha `767f8c1b…` 与测试内 `TEXT_FILES` 常量的 `multiframe/capture.json` 锚定值一致、本卡未改该 fixture。
- **28 = 26 recorded + nworld + cam_res**：3 render flag + 12 布尔 + `enabled_geom_groups` + `cam_active` + `background_color` + `shadow_light_fraction` + `samples_per_pixel` + 6 splat = **26** ✅；`nworld` 单独断言（capture 不记录，多帧固定 1）；`cam_res` 隐式（见低项）。
- **新增 21 项断言**：`nworld`(1) + 3 flag + 10 布尔 + `shadow_light_fraction`(1) + 6 splat(6) = 21 ✅；既有 6 项保留（`use_shadows`/`use_fast_math`/`samples_per_pixel`/`enabled_geom_groups`/`background_color`/`cam_active`），`cam_active` 另补 `recipe[...].is_null()`（与单帧写法一致）。
- **逐项都是真比对**：布尔/flags 用 `recipe[key].as_bool()` 对照（非仅存在性），`enabled_geom_groups`/`background_color` 做数组逐元素等值，`samples_per_pixel`/`shadow_light_fraction` 精确等值，6 个 splat 断言「recipe 为 null **且** `params.splat.any_present()==false`」——是「确认未启用」的真实对照，不是空占位。
- **防漏守卫（两层）**：①`assert_params_match_capture` 末尾把 `compared` 排序后与 `RECIPE_PARAM_KEYS` 排序集**等值断言**（漏比/多比即失败）；②`capture_parameter_keys_match_28_item_scope` 断言 capture 侧键集 == 表、且 `26+2==28`。我核过 `compared.push` 覆盖全部 26 键（3 flags + 12 booleans + 10 单项）。
- **反例证明非空转**：`recipe_param_mismatch_is_detected` 翻转 `use_shadows`、改 `samples_per_pixel` 各一次，`catch_unwind` **必须捕获 panic**，末尾再跑未改动正例——证明**既有键**断言链不是恒真（21 项新增见低 L2）。

## 二、独立执行（零 GPU）

- `cargo test --locked --features cuda-probe --test render_flex_multiframe -- recipe` ⇒ 命中 5 个用例全过（4 个 recipe/键集用例 + 1 个 s7b1 名含 recipe 的既有用例）、0 failed。
- 证据 `logs/host-params-tests.log` 与卡声明一致：整 target **41 passed / 0 failed / 1 ignored**（ignored = `multiframe_renders_match_captured_reference`，需 GPU+NVRTC）；本卡前 38 passed ⇒ +3 新用例自洽。
- 测试仍由同一 test target 执行（`#[path] mod multiframe_recipe;`），GPU 用例调用点经 `pub use` 保持不变；我实编通过。

## 三、行数棘轮 / 身份 / 边界（✅）

- 物理行数（我重新按 LF 计数）：`tests/render_flex_multiframe.rs` **494**（前 499）、`tests/support/render_bvh/multiframe.rs` **490**（前 496）、`multiframe_recipe.rs` 新增 **188** —— 均 <500 ✅，与 `logs/line-counts.txt` 一致。
- 测试 exe 身份：我独立重编后实测 `118,588,392 B / d226bb76dfa22c42b45c4ab49637ec5a01ea336f9949994f293b26e826d21488`，与卡声明的 post-card live **逐位一致**；pre-card live `3d482e56…`/118,291,924 与既有 frozen stash `0f76f9e1…`/118,098,954（**先于本卡已 drift**，卡已如实记录）与 `logs/identity-old-new.txt` 一致。
- 边界：提交 16 files = 3 个 test 文件 + 证据目录 8 文件 + `.normify` 5 文件；`git diff 8935d637..20015625` 无 `src/`/`fixtures/`/`scripts/` 路径 ⇒ 数值路径与 `capture.json` 未动（`tests/render_flex_validate.rs` 亦零 diff）；guards `94478b3d…`/43,729 B 与 `guards-r06-multiframe.json` `64bfecdf…`/11,899 B 只读未变（我复核哈希一致）；不改判据/参考/容差/相机/recipe；零 GPU、零窗口、不重冻。
- 证据：`r06-flex-multiframe-recipe-params-fill/` manifest **7/7** 独立重算全对（bad=0/missing=0/extra=0、不自列）；README/receipt/logs 齐；编码均为 UTF-8/LF、无 BOM/CR/NUL。
- 门禁：`.normify/runs/…-fill/` fmt/clippy/clippy-cuda exit 0、unit **1,406/0/402**（本卡前 1,403，+3 自洽）。

## 四、低项 / 观察

- **低 L2（测试强度/证据链缺口；Arch 复核发现，由卡 E 修）**：现有反例只覆盖 2 个**既有**键（`use_shadows`、`samples_per_pixel`），**21 项新增断言**（3 render flag、10 布尔、`shadow_light_fraction`、6 splat、`nworld`）没有任何负例证明其非空转——若某个新增断言的取值对照写错（取错键/恒真），当前反例不会暴露。Chief 已让 Backend 开**卡 E** 补「1 个新增键负例 + 保留 1 个既有项负例」；本卡结论不受影响（正向覆盖真实、双层防漏守卫在位），但 21 项新增的非空转证据链在卡 E 落地前不完整。

- **低 L1（口径措辞不准确）**：新模块注释/README 写「`cam_res` 由 model/camera 隐式推导；**单帧验收亦为隐式比对**，本模块不新增字面断言」。实测单帧验收 `tests/render_flex_validate.rs:421-422` **显式对照** `capture.context.resolved.cam_res`（并非隐式）；真正的理由是**多帧 capture.json 没有 `context` 段**（我核实 multi-frame capture 顶层无 `context`，无法从该 fixture 取 cam_res 对照）。因此多帧侧实际逐项断言为 **27 项**（26 recorded + nworld），cam_res 只是 28 口径中的名义项。建议措辞改为「多帧 capture 无 `context.resolved` 段，cam_res 无独立对照源，故仅计入口径、不新增断言」；如要真补，可对 model 的 `cam_resolution` 做推导一致性断言（注意：这属自证，非参考对照）。不影响本卡主要目标的成立。
- **观察 O1**：`cam_res` 的 model 数组名实为 `cam_resolution`（非 `cam_res`），若后续要补断言需用该键。
- **观察 O2**：GPU ignored 用例仍由同一 target 编译、经 re-export 使用新断言函数（我实编通过），但零 GPU 未执行——本卡不判定 GPU 行为，符合零 GPU 口径；后续 R06 窗口须重冻 exe（卡已写明）。

## 五、口径

不宣称渲染数值/等价/发布结论；不改判据/参考/容差/相机/recipe；`capture.json` 只读；零 GPU、零窗口、不重冻。
