# W1 清理项范围核查：R06 多帧 recipe 比对 7→28（证据/建议片，零代码）

- 任务：`2026-11-11-w1-r06-recipe-params-resolution`（owner Backend）；性质：**只读范围核查 + 证据登记**，不改卡、不改代码。
- 复核对象：`2026-11-05-render-r06-flex-multiframe-recipe-params-backlog`（status=backlog，W1 p20 清理项）。

## 1. 范围结论（先看后动）

- backlog 卡 acceptance：不排期；若补齐 → 对齐到单帧同口径 28 项、CPU-only、不改 spec/判据/参考、不改 G-2 夹具。
- 核查结果：**该 7→28 对齐已由既有测试侧实现完成**，实现提交只动 `.normify/`、`docs/development-evidence/`、`tests/`；
  `src/`/`fixtures/`/`scripts/` 零改动，**未触碰产品 recipe 常量/判据/容差，未涉及 eol/.gitattributes**。

| 提交 | 内容 |
| --- | --- |
| `20015625` | 28 项口径 assert_params_match_capture（26 recorded + nworld；cam_res 隐式）+ RECIPE_PARAM_KEYS + 防漏守卫 + 4 个 CPU 用例；测试侧 |
| `8618e447` | 新增键负例 use_textures（证明 21 项新增断言非空转）；测试侧 |
| `a20efe2a` | 卡 L 注释措辞修正（cam_res 口径），注释 only、断言零改动 |

- 实现：`tests/support/render_bvh/multiframe_recipe.rs`（188 行）——28 项 = 26 个 capture recorded 键（RECIPE_PARAM_KEYS）+ nworld + cam_res（名义第 28 项；多帧 capture 无 context.resolved 段，无字面断言）；
  实际逐项断言 **27** 项；防漏守卫：
  已比对键集合 == RECIPE_PARAM_KEYS 且 capture.parameters 键集合 == RECIPE_PARAM_KEYS（防漏/防漂移）。

## 2. CPU 自证（零 GPU）

cargo test --locked --features cuda-probe --test render_flex_multiframe recipe -- --test-threads=1

结果：**5 passed / 0 failed / 0 ignored / 37 filtered out（含 4 个 multiframe_recipe CPU 用例 + 1 个 filter 命中的 s7b1 用例）**；4 个 recipe CPU 用例：

- `recipe_params_match_capture_28_item_cpu_self_check`
- `recipe_param_mismatch_is_detected（既有键 samples_per_pixel + 新增键 use_textures 两个反例）`
- `capture_parameter_keys_match_28_item_scope`
- `context_params_match_capture_recipe`

## 3. 结论与建议

W1 p20 清理项（R06 多帧 recipe 比对 7→28）**已由既有测试侧实现完成**：28 项口径 + 防漏守卫 + CPU 自证在 `tests/support/render_bvh/multiframe_recipe.rs`；实现提交仅动 `.normify/`、`docs/development-evidence/`、`tests/`，未触碰产品 recipe 常量/判据/容差、未改夹具、未涉及 eol。backlog 卡的建议后续动作=由治理片按既有实现关闭（本片只登记证据与建议，不关卡、不改卡）。

## 4. 验证

- selfcheck（`logs/selfcheck.txt`）：范围/实现事实/测试结果在案、`zero_code=true`、`product_recipe_constants_touched=false`、
  manifest 4 文件自核、policy_scan 0 命中、编码 UTF-8/LF 无 BOM、边界仅本目录 + `.normify`。
- 四门禁：fmt/clippy/clippy-cuda/unit。

## 5. 边界与非声明

- 零代码、零 GPU、零窗；只写本目录 + `.normify/`；不改卡/代码/判据/参考/容差；eol 相关一律不动（等用户确认范围）。
- 不宣称任何门禁/阶段准入通过、不宣称渲染数值/等价/发布结论。
