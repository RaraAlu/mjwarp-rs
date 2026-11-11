# R06 recipe-params 反例非空转补强：新增「新增键」负例（use_textures）

- 任务：`2026-11-11-render-r06-flex-multiframe-recipe-params-negative-new-key`（owner Backend；p78；P8-渲染与BVH）。
  来源：Chief 2026-11-11 指令——卡 C（`20015625`）复核 PASS，但两个负例都落在**既有**键（`use_shadows`、
  `samples_per_pixel`），21 项新增断言的「非空转」未被直接证明；按**零偏差**最小改法补 1 个新增键负例。
  规格：Arch `target/render-bvh/gl2-ptx-diff/r06-recipe-params-spec.md` = **`d7fcf156…`/7,608 B**
  （`target/` 工作材料，非证据；已补入卡 C 证据 README §7）。
- 边界：只改**测试支持/用例与证据**；断言口径与 21 项比对本身不动；无 `.cu`、无 `src/` ⇒ 内核身份不变；
  零 GPU、零窗口、不重冻。

## 1. 改动（仅 `tests/support/render_bvh/multiframe_recipe.rs::recipe_param_mismatch_is_detected`）

- 负例 1（**既有键**，保留）：改动 `samples_per_pixel`（7 项口径时代就已比对）→ `catch_unwind` 必须 panic。
- 负例 2（**本片新增的 21 项断言之一**，新增）：翻转 `use_textures` → `catch_unwind` 必须 panic；断言消息标注
  「新增键 use_textures」。
- 正例对照保留（未改动时必须通过）；测试函数外无改动——`assert_params_match_capture` 与 21 项断言逐行未动。
- diff：11 insertions / 11 deletions，全部在负例块内（`logs/diff.txt`）；文件仍 188 物理行（< 500 棘轮）。

## 2. 「负例覆盖新增键」× 21 项对应关系（`logs/coverage-21-mapping.txt`）

| 组 | 数量 | 断言形态（代码位置） | 直接负例覆盖 |
| --- | --- | --- | --- |
| `nworld` | 1 | `assert_eq!(params.nworld, 1)` | 否 |
| `render_rgb/depth/seg` | 3 | `assert_eq!(*spec, Some(RenderFlagSpec::Bool(..)))` | 否 |
| `use_textures` + 9 布尔 | 10 | `assert_eq!(value, boolean(name))`（数据驱动循环） | **是：`use_textures`**（与同组其余 9 项同一断言路径） |
| `shadow_light_fraction` | 1 | `assert_eq!(params.shadow_light_fraction, f64)` | 否 |
| 6 个 `splat_*` | 6 | `assert!(recipe[name].is_null())` + `splat.any_present()==false` | 否 |
| **合计** | **21** | — | 新增键直接负例 1 项（规格最小要求） |

- 口径诚实说明：直接负例只证明 **bool 形态**（10 项同路径）；`nworld`/render flag/f64/splat-null 各自由独立
  `assert_eq!`/`assert!` 比对，本卡**未**为其加定向负例（零偏差最小规格），证据不宣称它们被负例直接覆盖。
- 既有 6 项保留比对；既有键负例 `samples_per_pixel` 保留。

## 3. 测试（`logs/host-params-tests.log`）

`cargo test --locked --features cuda-probe --test render_flex_multiframe -- --test-threads=1` ⇒
**41 passed / 0 failed / 1 ignored**（exit 0；4 个 recipe 用例含 `recipe_param_mismatch_is_detected` 全过，
计数与卡 C 相同——本卡改的是负例内容、不增删用例）。

## 4. multiframe 测试 exe 身份（`logs/identity-old-new.txt`）

| 项 | sha256 | bytes |
| --- | --- | --- |
| 卡 C 记录值 / 卡 E 前（exe 未重建，mtime 12:14:44） | `d226bb76dfa22c42b45c4ab49637ec5a01ea336f9949994f293b26e826d21488` | 118,588,392 |
| **卡 E 后（built 12:22:52）** | **`0966bf4cccc7312c26c1204a2148473c597a833b4c04bc8e7a216ca8eccdaef4`** | 118,588,904 |

- delta +512 B（多一个负例块）。**测量 caveat**：同源重建 exe sha 不保证逐字节可复现
  （见 `perworld-l-v2-two-run-sha-print/logs/identity-measurements.txt`）；exe 身份以冻结产物为准。
- **重冻含义**：R06 后续窗口必须重新冻结 exe/stash；本卡不重冻、不占窗。

## 5. 验证 / 边界 / 非声明

- 四门禁 fmt/clippy/clippy-cuda/unit（unit 1,406 过 / 0 败 / 402 ignored，与卡 C/D 后基线一致）；
  `normify check` 全过 → done。
- 边界：`unchanged src/`、`fixtures/`、`scripts/`；`git diff -- '*.cu'` 空；不改判据/参考/容差/相机/recipe；
  capture JSON 只读。
- 卡 C 证据目录 README 追加 §7（规格 sha `d7fcf156…`/7,608 B、新键负例、身份 supersession），原文字节不改，
  manifest 已重算。
- 非声明：不宣称渲染数值/等价/设备端行为变化；不重冻；零 GPU。