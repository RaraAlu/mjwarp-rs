# R06 多帧实体 A：recipe 参数比对补强（7 → 28 项口径；CPU 自证 + 防漏守卫；零 GPU）

- 任务：`2026-11-11-render-r06-flex-multiframe-recipe-params-fill`（owner Backend；p20；P8-渲染与BVH）。
  来源 backlog 卡 `2026-11-05-render-r06-flex-multiframe-recipe-params-backlog`（p20：实体 A 比对 7 项 vs
  单帧验收 28 项，风险低、后置）；规格：Arch `target/render-bvh/gl2-ptx-diff/r06-recipe-params-spec.md`
  （`target/` **工作材料，非证据**）。
- 边界：**仅测试支持代码与测试断言**——不改渲染/数值路径、不改 `capture.json`、不改 `src/`、`fixtures/`、
  `scripts/`、判据/参考/容差/相机/recipe；零 GPU、零窗口、不重冻 guards/stash。

## 1. 结论

- 多帧实体 A 的 recipe 比对由 **7 项口径**（6 项 recorded + `resolved.nworld` 校验）补到与单帧验收
  （`tests/render_flex_validate.rs`）同口径的 **28 项** = capture `parameters` 的 **26 recorded 键**
  + `nworld`（capture 未记录，多帧固定单 world）+ `cam_res`（隐式，由 model/camera 推导；本卡不新增字面断言）。
- 新增 **21 项断言**（§3）；保留既有 6 项中 `cam_active` 补 `recipe["cam_active"].is_null()`（单帧同口径），
  其余保持。
- **防漏守卫**：`RECIPE_PARAM_KEYS: [&str; 26]`（字典序全部 recorded 键）；`assert_params_match_capture`
  末尾断言「已比对键集合 == RECIPE_PARAM_KEYS」（漏比/多比即失败）；`capture_parameter_keys_match_28_item_scope`
  断言 capture 侧键集 == 该表。
- **CPU 自证 4 用例**（本卡新增 3 + 既有 1 迁移）：`cargo test --locked --features cuda-probe --test
  render_flex_multiframe` ⇒ **41 passed / 0 failed / 1 ignored**（本卡前 38 passed；+3 新增，既有用例随
  recipe 子模块迁移不变）。
- 身份：测试 exe 变化（断言增加）`3d482e56…`/118,291,924 B → **`d226bb76…`/118,588,392 B**；既有 frozen stash
  `0f76f9e1…`/118,098,954 B **先于本卡已 drift**。数值路径未变；**R06 后续窗需重冻，本卡不重冻、不占窗**。

## 2. 改动与行数（按功能抽出子模块，守 500 行棘轮）

| 文件 | 改动 | 物理行数 |
| --- | --- | --- |
| `tests/support/render_bvh/multiframe_recipe.rs`（新增） | 28 项 `assert_params_match_capture` + `RECIPE_PARAM_KEYS` + `assert_capture_key_set` + 4 个 CPU 用例（3 新 + 1 迁移） | **188** |
| `tests/support/render_bvh/multiframe.rs` | 删除旧 6 项 `assert_params_match_capture`，改 `pub use crate::multiframe_recipe::assert_params_match_capture;`（GPU 用例调用点不变） | 496 → **490** |
| `tests/render_flex_multiframe.rs` | 声明 `#[path] mod multiframe_recipe;`；`context_params_match_capture_recipe` 随 recipe 用例迁入子模块 | 499 → **494** |

抽出原因：两文件原已 496/499 行，贴近单文件 500 行棘轮；28 项断言与 recipe CPU 用例按功能抽出，测试仍由
同一 test target 执行（`#[test]` 在 `#[path]` 子模块内同样被收集）。`tests/render_flex_validate.rs` 零 diff。

## 3. 7 → 28 差异表（新增 21 项，全部按单帧口径）

| # | recipe 键 | 断言（多帧） |
| --- | --- | --- |
| 1 | `nworld`（capture 未记录） | `assert_eq!(params.nworld, 1)`；注释写明 capture 不记录、固定单 world（`resolved.nworld==1` 另有用例覆盖） |
| 2–4 | `render_rgb` / `render_depth` / `render_seg` | `assert_eq!(*spec, Some(RenderFlagSpec::Bool(recipe[key])))`（与单帧同写法） |
| 5–14 | `use_textures`、`use_ambient_lighting`、`flex_render_smooth`、`use_precomputed_rays`、`render_skybox`、`enable_backface_culling`、`enable_vertex_normals`、`enable_specular`、`enable_emission`、`enable_per_light_ambient`（10 个布尔） | `assert_eq!(params.<field>, recipe[key].as_bool())`，逐项带字段名 label |
| 15 | `shadow_light_fraction` | `assert_eq!(params.shadow_light_fraction, recipe[key].as_f64())`（精确相等，与单帧一致；当前 0.3） |
| 16–21 | `splat_position` / `splat_rotation` / `splat_scale` / `splat_rgba` / `splat_adr` / `splat_group_id` | recipe 六项均为 `null`，且 `params.splat.any_present() == false` |
| 保留 | `use_shadows`、`use_fast_math`、`samples_per_pixel`、`enabled_geom_groups`、`background_color` | 原比对保持 |
| 保留+补强 | `cam_active` | 补 `recipe["cam_active"].is_null()`（原只断言 `CameraActive::All`） |
| **口径** | `cam_res`（第 28 个参数） | **不新增字面断言**（capture 不记录、单帧亦隐式）；README/注释写明 28 = 26 recorded + nworld + cam_res |

## 4. CPU 自证与反例证明（`logs/host-params-tests.log`）

| 用例（模块 `multiframe_recipe`） | 作用 |
| --- | --- |
| `recipe_params_match_capture_28_item_cpu_self_check` | 正例：`context_params()` 对 capture 全过 |
| `recipe_param_mismatch_is_detected` | **反例/非空转证明**：翻转 `use_shadows`、改 `samples_per_pixel` 各一次，`catch_unwind` **必须捕获 panic**；未改动时必须通过 |
| `capture_parameter_keys_match_28_item_scope` | 键集守卫：capture 26 键 == `RECIPE_PARAM_KEYS`；28 = 26 + nworld + cam_res |
| `context_params_match_capture_recipe`（迁移） | 28 项比对 + `resolve_context` 的 `nworld==1`、4 个 geom 全开 |

## 5. 测试 exe 身份（`logs/identity-old-new.txt`）

| 项 | sha256 | bytes |
| --- | --- | --- |
| 本卡前 live | `3d482e56cb70075fd0946d7d97622ce2c749b991349b0d1f78697976ade2feac` | 118,291,924 |
| 既有 frozen stash（**先于本卡已 drift**） | `0f76f9e14d1e0bd685f6331655e8b41c18ef6e38a97c0d3d03c4c8365412b0f2` | 118,098,954 |
| 本卡后 live（最终构建） | **`d226bb76dfa22c42b45c4ab49637ec5a01ea336f9949994f293b26e826d21488`** | 118,588,392 |

- 变化原因：测试断言增加（非数值路径变化）；`src/` 零改动。
- **冻结口径**：后续任何 R06 窗口必须重新冻结 exe/stash；本卡不重冻、不占 GPU 窗口。

## 6. 验证 / 边界 / 非声明

- CPU 测试：**41 passed / 0 failed / 1 ignored**（exit 0；ignored 为 GPU 用例）。
- 四门禁 fmt/clippy/clippy-cuda/unit 全过（unit **1,406 过 / 0 败 / 402 ignored**；本卡前 1,403，+3 新用例）；
  `normify check` 全过 → done；runs 记录在 `.normify/runs/2026-11-11-render-r06-flex-multiframe-recipe-params-fill/`。
- 边界：`unchanged src/`、`fixtures/`、`scripts/`；`guards-perworld-l2-light.json` `94478b3d…`/43,729 B 只读核对未变；
  `guards-r06-multiframe.json` `64bfecdf…`/11,899 B 只读未变（`logs/guards-untouched.txt`）。
- 非声明：不宣称渲染数值/等价/发布结论；不改任何判据/参考/容差/相机/recipe；`capture.json` 只读；零 GPU。