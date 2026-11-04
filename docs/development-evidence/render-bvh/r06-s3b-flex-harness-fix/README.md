# R06-S3b⑥b：flex GPU 用例 D4a/D4b 修复与 CPU 前置自证（attempt 1 + attempt 2 对应）

任务：`2026-11-04-render-r06-s3b-flex-harness-schema-fix`（p76，owner Backend；depends `…-flex-device-harness`）。
本片**零 GPU、零窗口、不做真冻结**；只改 `tests/render_flex_validate.rs` + 本证据目录。
判据/容差/参考**零改动**；`fixtures/`、`src/` 零 diff；**不宣称 flex 渲染通过/不通过**。

## 1. 窗口失败记录（两轮 INVALID，均非数值结论）

| 轮 | 证据提交 | 中止点 | 根因 |
| --- | --- | --- | --- |
| attempt 1 | `7c4a8296` | `tests/render_flex_validate.rs:56`（`ints(model,"flex_edge")`） | `model-312.json` 无 `flex_edge` 数组；GPU 用例只做过 `--no-run`，首跑即 panic |
| attempt 2 | `cccbd470` | `tests/render_flex_validate.rs:256`（`render_once`）→ `Unsupported("S7b2 要求 enabled_geom_ids 覆盖全部 geom")`（产品检查 `src/render/kernel/megakernel.rs:256`） | 用例用 `ContextParams::default()`：`enabled_geom_groups=[0,1,2]`，而场景 `geom_group=[1,2,3,1]` → enabled 只有 **3/4**；采集 recipe 是 `[0,1,2,3]`（全开） |

两轮都停在**发射前**，rgb/depth/seg 判据**一次也没进入评估**，因此本目录记录不能用于任何方向的验收结论。

## 2. 第一轮修复（attempt 1 对应，提交 `802b21de`）

- **`flex_edge` 从 `flex_elem` 派生**（dim==1 时元素就是边；上游 `mjmodel.h` 口径），四条交叉校验：
  `nflexelemdata == 2*nflexedge`、`flex_edge.len() == 2*nflexedge`、`sum(flex_edgenum) == nflexedge`、
  `flex_edgeadr[0] == 0` 且端点 < 各自 `flex_vertnum`。实际值 `[0,1, 1,2, 2,3, 0,1]`。
- **`flexedge_j_*` 同源推导**：每边一行、行内 2 个非零（`rownnz=[2;nflexedge]`、`rowadr=[0,2,4,…]`、`colind=flex_edge`），
  不再硬编码 `vec![1; len/2]`。
- 新增**普通 `cargo test` 可跑的 CPU 自证** `flex_model_dump_schema_self_check`（17 个 flex 数组逐项
  `kind`/`count`/`values` → host 组装 → mesh/BVH 输入断言）；CPU 自证当场又抓到 `FlexTopologyFields.nflex` 未填。

## 3. 第二轮修复 D4a（attempt 2 对应，本片）

**D4a（阻断）**：用例必须与参考视角同 recipe。新增单点构造 `context_params()`，GPU 用例与 CPU 前置校验**共用**：

- `enabled_geom_groups = [0,1,2,3]`（4/4 全开：`geom_group=[1,2,3,1]`，默认 `[0,1,2]` 漏 group 3 的 geom）。
- `use_shadows = true`（参考在 2 盏 `castshadow` 灯下采集；默认 `false` 会丢阴影颜色）。
- `use_fast_math = false`（与参考同配方；本移植不消费该字段，仍按 recipe 记录）。
- 其余 28 参数同 `ContextParams::default()`（与 recipe 一致）。
- GPU 用例的射线计划改为**同源于本用例 resolved**（`RayBuilder::build(&camera_model, &resolved.plan, Precomputed)`），
  不再经 `s7b1::precomputed_rays` 内部按默认参数重解析，消除隐藏配方。
- 新增 `assert_params_match_capture()`：`context_params()` 与 `capture.json.parameters` 逐参数比对
  （3 个 render 目标 + 12 个布尔 + `enabled_geom_groups` + `cam_active=All` + `background_color` +
  `shadow_light_fraction` + `samples_per_pixel` + 6 个 splat 为 `null`）。

**延伸发现（窗口材料未列，D4a 同源）**：除 geom 组外，`use_shadows` 与 `use_fast_math` 也偏离 recipe
（默认 `false`/`true` vs recipe `true`/`false`）。只修 geom 的话 attempt 3 仍会在阴影数值上红——
本片把三处一起对齐并用 CPU 断言钉死。

## 4. 第二轮修复 D4b（CPU 自证前移到「发射前」）

新增普通 `#[test]`（非 `#[ignore]`）`render_context_preflight_matches_capture_recipe`，在 CPU 走完
`render_with_flex` **发射前全部 host 可判定校验**：

| 校验段 | 断言 | 对应产品分支 |
| --- | --- | --- |
| recipe 28 参数 | 逐项 == `capture.json.parameters`；splat 六项为 `null` | 用例配方漂移 |
| `resolve_context` | nrender / cam_id_map / cam_res / use_shadows / use_fast_math / shadow_light_fraction / flex_render_smooth / samples_per_pixel / background_color_unpacked / znear / zfar / nworld / total_rays 逐项 == `capture.json.context.resolved` | `ContextError` 分支 |
| **enabled geom 覆盖** | 按 `geom_group` 与 recipe 组独立复算 id 集 == `resolved.enabled_geom_ids` == 全 4 个 geom；等价产品前置「非空且覆盖全部 geom」 | `megakernel.rs:256` `Unsupported`（attempt 2 的 D4a） |
| **负对照** | 默认参数（`[0,1,2]`）→ `enabled_geom_ids=[0,1,3]`（3/4），断言 `len != ngeom`：任何人把 `context_params()` 退回默认，普通 `cargo test` 立即红 | 同上 |
| geom 模型数组 | `ModelRenderArrays::from_fields` → `geom_type.len == ngeom` | `LauncherError::InvalidArray` |
| 灯光 | `light_data_poses` → 2 盏；xpos/xdir 与 `state.npz` 的 `light_xpos`/`light_xdir` **逐 f32 位型相等**（含 warp `normalize` 的 fma 序） | `PoseError` |
| 相机 | `CameraModel::from_fields`；`camera_poses` 数 == ncam；`world_camera_pose_arrays`、`world_active_camera_arrays` | `LauncherError::InvalidArray/Overflow` |
| 场景 | `SceneState::new` → 叶/位姿数 4、worlds 1；mesh 数 == nmesh | `BvhError` 前置 |
| 纹理 | `TextureRegistry::from_fields` | `TextureError` |
| 1D flex 场景叶 | `flex_model_dump_schema_self_check` 增加 `one_dimensional_scene_leaves` → 4 个胶囊叶、group 全 0 | `FlexLeafError`（`render_impl` 设备打包前） |

## 5. 覆盖边界（诚实口径）

- **已覆盖（CPU）**：上表全部 host 可判定校验，即参数配方、context 解析、模型/灯光/相机/纹理/场景状态、
  1D flex 场景叶构造；以及 host 侧 flex 三字段 → `FlexHostAssembly` → `FlexMeshAssembly` 输入断言。
- **未覆盖（需 CUDA 会话，仍由开窗卡执行）**：NVRTC 编译、设备缓冲分配/上传、`SceneDevice`/flex 设备打包、
  kernel 发射、读回与三相机 rgb/depth/seg 判据、两跑 sha 一致性。
- 因此本片**只**证明「setup 缺陷能在普通 `cargo test` 被发现」，**不**证明渲染数值。

## 6. 验证结果

- `cargo test --locked --test render_flex_validate` → **13 passed / 0 failed / 1 ignored**
  （ignored = `flex_renders_match_captured_reference`，需 GPU + NVRTC）。原始输出：`self-check.txt`。
- `cargo test --locked --features cuda-probe --no-run --test render_flex_validate` → **exit 0**（`compile.log`）。
- 四门禁：`fmt` / `clippy` / `clippy-cuda` / `unit` 全过（unit **1316 过 / 0 败**，新增 1 个 CPU 用例）。
- 文件行数：`tests/render_flex_validate.rs` **447 行 ≤ 500**（`#[rustfmt::skip]` 仅用于压缩既有长断言行，不规避门禁）。
- 零 GPU、零窗口、无真冻结；判据/容差/参考零改动；`fixtures/`、`src/` 零 diff。

## 7. 与窗口衔接（attempt 3）

- 修复后测试二进制会重建 → 开窗卡必须**重新真冻结**（`freeze-guards-r06-s3b-flex.ps1 -Attempt 1` 解析新 cargo 产物、
  填 exe sha/bytes、`status=frozen`、`GUARDS_OK=15/15`），attempt 3 仍需 Chief 再发 GO。
- 开窗前不必再跑 GPU 自检：本片 CPU 自证（13 passed + 负对照）不依赖 exe 哈希，重建后重跑一遍即可确认仍绿。
- 窗口命令与判据沿用窗口材料，不变：
  `cargo test --locked --features cuda-probe --test render_flex_validate -- --ignored --test-threads=1`
  （rgb bit-first ≤1 LSB、depth 位级 + ≤4 ULP 且 0 beyond、seg 全等含 cam-flex 654 个 `(flexid,9)`、两跑 sha256 相同）。
