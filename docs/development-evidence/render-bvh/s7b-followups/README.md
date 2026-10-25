# 渲染/BVH S7b 配套修正：合成顶点法线 pinned 参考 + S7b2 断言升级

任务：`2026-10-25-render-bvh-s7b-followups`
冻结上游：`71da24d956378a87a703b6e1442b13aec0c4ac29`（MJWarp 3.15）
GPU：NVIDIA GeForce RTX 4070 Ti SUPER / 驱动 596.36 / SM89（窗口起止 5059 → 5057 MiB，无计算进程残留）

## 1. 结论

- **合成顶点法线 pinned 同配方参考已补采**（S7b3 复核缺口 #1）：in-memory 覆盖
  `mjm.mesh_normal` 为 `[1,0,0]` / `[0,0,1]` 后，pinned 3.15 + `use_fast_math=False`
  各采两相机全图 RGB；`[1,0,0]` 对默认 scene-a 生效 **653 像素**、`[0,0,1]` **9 像素**，
  两次捕获逐位一致。
- **产品对 pinned 合成参考：两相机 0 像素差异**（x/z 两组都 0）；且
  `enable_vertex_normals` on/off 在合成法线下分别 **653 / 9 像素**可判别 → 分支生效且
  与 pinned 参考一致。默认 scene-a 下 `vertex_normals_off` 无判别力的结论保留。
- **S7b2 spp=2 旧断言升级已落地**：`tests/render_kernel_rgb.rs` 的拒绝面从 4 类改为
  **3 类（shadows/动态射线/skybox）+ 注释注明 AA 验收归 S7b3**；S7b2 GPU 回归 **3/3**
  （RGB 同配方 0 差异、depth/seg 120,000/120,000 位等 0 ULP）。测试改动提交 **`3ab52cf`**
  （走 S7b2 `Normify-Fixes` 流程），S7b2 证据说明在 `s7b3-fix`/本任务伴随提交中补齐。
- 不回归：S7b1 depth/seg **6/6**、S6b **5/5**；PTX 不变（megakernel 197,583 B、aa_resolve 27,536 B）。

## 2. 合成顶点法线 pinned 参考（采集口径）

**为什么不改 mjb**：`mujoco.MjModel.from_binary_path` 后 `mjm.mesh_normal` 是
`(4, 3)` f32 且可写；直接 `mjm.mesh_normal[:] = direction` 再 `mjwarp.put_model(mjm)`
即把覆盖值上传设备，脚本可完整复现，无需改冻结场景文件。

- 采集脚本：`scripts/authoring/render-bvh-s7b/capture_synthetic_normals.py`
- 覆盖方式：`mjm.mesh_normal[:] = [1,0,0]`（4 顶点同方向）/ `[0,0,1]`；`put_model` 上传；
  `create_render_context` 用同配方 `PARAMS_28`（仅 `use_fast_math=False`）
- provenance（`synthetic_normals_index.json`）：原 mjb sha `8953988f…`、原法线数组 sha
  `95ecb321a32a05657dfc761618975fdba64b66717a8387134baa593227a9c1b2`、覆盖后数组 sha、
  覆盖方式与方向、脚本 sha、生成时间、环境版本
- **生效断言**（脚本内 fail-fast）：覆盖后两相机渲染必须与默认 scene-a 同配方参考
  （`rgb_fast_math_false.json`）不同——实测 `[1,0,0]` **653**、`[0,0,1]` **9** 像素；
- **两次捕获位等**：每组渲染两次，rgb/depth/seg 逐相机逐位一致（capture.log 有
  `determinism=two_captures_bitwise_identical` 行）。

| 文件 | bytes | sha256 |
| --- | --- | --- |
| `fixtures/render-bvh/scene-a/s7b/flags/synthetic_normals_x_rgb.json` | 1320499 | `34d123a2622a4b6f68b6a6a5fc17cb566e11c192fdcf17dd271077376aa15e15` |
| `fixtures/render-bvh/scene-a/s7b/flags/synthetic_normals_z_rgb.json` | 1320499 | `0946517cae34eb3a1f152f5cb275083c7fd6f70798ff407476986b5866cba9e8` |
| `fixtures/render-bvh/scene-a/s7b/flags/synthetic_normals_index.json` | 3228 | `6b80232180f3710d19b82c95a4f7ef65060bd42ea7230a78ee6f7906b3b72203` |

编码：packed u32 ABGR（a=255、`int()` 截断），row-major；解包
`float32(byte) * float32(1/255)`；判据「解包后每通道 ≤1 LSB」。

## 3. 产品对照结果

测试：`tests/render_kernel_flags.rs::synthetic_mesh_normal_matches_pinned_reference_and_discriminates`

| 方向 | 产品 vs pinned 参考（两相机） | on/off 可判别 | pinned 覆盖 vs 默认 scene-a |
| --- | --- | --- | --- |
| `[1,0,0]` | mismatched 0 / beyond_1_lsb 0 / max_byte_diff 0 | 653 像素不同 | 653 像素变化 |
| `[0,0,1]` | mismatched 0 / beyond_1_lsb 0 / max_byte_diff 0 | 9 像素不同 | 9 像素变化 |

- 产品侧用 `s7b3::fields_with_mesh_normal`（同一 `RenderFields.mesh_normal` 数据源）构造
  合成法线，与 pinned 覆盖值一致；因此两侧可 like-for-like 位等对照。
- 默认 scene-a 下 `vertex_normals_off` 参考无判别力的结论**保留**（由
  `flag_fixtures_are_complete_and_documented` 断言 `has_discriminating_effect=false`）。

## 4. S7b2 spp=2 断言升级（已落地）

- 提交 **`3ab52cf`**（`Normify-Fixes: 2026-10-25-render-bvh-s7b2-shading`）：
  `tests/render_kernel_rgb.rs::unsupported_configurations_are_rejected_before_launch`
  的拒绝列表从 4 类改为 **3 类**（`use_shadows`、`use_precomputed_rays=false`、
  `render_skybox`）并注明 AA 的合法配置/拒绝面验收归 S7b3（`render_kernel_flags.rs`）。
- S7b2 原验收结论与数值不变：RGB 对同配方参考 120,000/120,000 位等、depth/seg
  120,000/120,000 位等 0 ULP、灯位姿 6/6 位等、PTX 195,594 B。
- 本次 followups 复跑 S7b2 GPU **3/3**（`s7b2-regression.log`）。

## 5. S7b3 证据修正（记录于此，改动由 `s7b3` 修正提交）

按 S7b3 QA 复核非阻断问题修正（不改验收结论与数值）：

1. `host_modules.lines` 行数按实际文件更正：`megakernel.rs` **495**、`megakernel_aa.rs` **93**、
   `megakernel_arrays.rs` **233**（bytes/sha 不变，均 <500）。
2. `megakernel.rs` 模块头注释与实现相反处更正为「depth/seg 是单样本输出，只在
   `ray_base == 0` 的第一个 pass 写」（AA 首轮 78% 不匹配的根因修复，注释与代码一致）。
3. `capture.log` 增打印两次捕获位等结论（`determinism=two_captures_bitwise_identical`）；
   重跑后 8 个变体夹具 sha 全部不变，仅 `flags/index.json` 因脚本 sha 变化而更新
   （新 sha `f83f73e2f836ad26a009678c43e9b4c03747598d616a84115957e552b62bf62c`）。
4. 证据注明逐 geom 归因口径：交付的 `rgb_changed_by_geom` 按**变体/AA 的 seg**（产品侧
   seg 参考）归因；QA 复核按 **base seg** 归因——同一像素的 geom 归属可能不同（语义等价，
   不是数值分歧）。

## 6. 环境与门禁

| 项 | 值 |
| --- | --- |
| GPU / 驱动 / SM | RTX 4070 Ti SUPER / 596.36 / SM89 |
| 产品编译 | NVRTC（CUDA 12.8.1 工具链，compute_70）、`WARP_1_15_RENDER`（fast_math=false） |
| authoring | pinned venv：Python 3.12.13、warp 1.15.0（自带 NVRTC CUDA 12.9）、mujoco/mujoco_warp 3.15.0、numpy 2.5.4 |
| GPU 窗口 | 起止 5059 → 5057 MiB（桌面波动同带），结束无本任务 CUDA 计算进程 |
| 门禁 | fmt / clippy / clippy-cuda / unit（见 `unit-summary.log`） |

## 7. 未满足项

- 合成法线参考只覆盖 `[1,0,0]` / `[0,0,1]` 两个方向（与 S7b3 产品侧合成用例一致）；更多
  方向未采（同一路径，未单独验收）。
- S7b3 的 `vertex_normals_off` 在默认 scene-a 下无判别力是数据/量化结论，本任务只是补参考
  对照，不改变该结论。
- 未跑完整 G01（团队规则：开发迭代只跑定向 + fmt/clippy/clippy-cuda/unit；里程碑由用户点名）。

## 8. 证据文件

- `capture.log`：合成法线采集输出（含 determinism 与生效断言 changed_pixels）
- `gpu-run.log`：followups GPU 用例（6/6，含合成 pinned 对照）
- `s7b2-regression.log` / `s7b1-regression.log` / `s6b-regression.log`：回归
- `unit-summary.log`：`cargo test --locked`
- `receipt.json`：机器可判定回执
