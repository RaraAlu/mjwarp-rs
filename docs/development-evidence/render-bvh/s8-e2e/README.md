# 渲染/BVH S8：端到端（产品 camlight → 渲染、双相机、refit、RGB/depth/seg）

任务：`2026-10-25-render-bvh-s8-e2e`（渲染/BVH S8），冻结上游：`71da24d956378a87a703b6e1442b13aec0c4ac29`（MJWarp 3.15）
精度门：`f32-render`（A′ `check_render_ptx`，编译只走 `compile_render_artifact`）
GPU：NVIDIA GeForce RTX 4070 Ti SUPER / 驱动 596.36 / SM89（Windows 11）

## 1. 结论

- **产品 camlight 真接上**：渲染输入的相机 data 位姿由产品链
  `KinematicsPlan::with_camlight` → `update_rigid`（产品 f32 FK）→ `update_com` →
  `update_camlight`（`src/physics/camlight.rs` 的 GPU 段）→ `readback` 产出；两相机
  `cam_xpos`/`cam_xmat` 与 S7b1 同配方参考 `camera_data` **逐位一致（0 origin 位差 / 0 xmat 位差）**，
  含 `cam_mode=3` TARGETBODY 跟踪相机（camera1）。**夹具 data 位姿只用于比对，不参与渲染。**
- **端到端两相机数字**（默认 28 参数 context、spp=1、precomputed rays）：RGB 对同配方
  （`use_fast_math=False`）参考 **120,000/120,000 像素 0 差异**（判据 ≤1 LSB）、depth
  **120,000/120,000 位等 max 0 ULP**（判据 ≤4 ULP）、seg **0 不一致**；两次运行输出哈希逐位相同
  （`depth_seg_hash=03efa95416bbd008`、`rgb_hash=ed50f750936c0275`）。
- **S0b fast-math 参考只作跨配方偏差记录**：cam0 max_byte_diff=1（2 像素）、cam1 全位等，
  与夹具 `cross_recipe.per_camera` 记录一致，不作通过依据、不写成产品缺陷。
- **反证成立（camlight 真接上的反证）**：
  1. 篡改本地比对副本的夹具 data 位姿（identity+原点）后，产品位姿渲染结果逐位不变，
     且产品位姿仍与**原始**参考逐位一致；
  2. 若把篡改后的夹具位姿当输入渲染，RGB 变化 **115,972/120,000 像素**（夹具回填必被检测）；
  3. 产品 camlight 输出注入扰动（camera1 绕 y 0.05 rad + `xpos.x` 平移 0.3）→ RGB 变化 18,859、
     depth 变化 19,182、seg 变化 9,464；同扰动两次逐位相同、不同扰动结果不同、
     恢复产品位姿后逐位复现。
- **生命周期**：create→render→显式 refit→render→destroy 全链路；位姿不变时 refit 后输出逐位相同
  （无冻结参考时只做结构断言）；`refit_bvh` 是唯一重拟合入口（`refits=1`）；destroy 后
  显存增长 **0 B**（阈值 16 MiB）；禁用输出 depth/seg 读回 `InputError::DisabledOutput`。
- **负向控制保持**：shadows / `use_precomputed_rays=false` / skybox 三类写前拒绝逐类可判定；
  AA（spp=2 → nsamples=4）射线数不符写前拒绝（`InvalidArray(ray_count)`）。
- **不回归基线**：S7b1 GPU 6/6（depth/seg 120,000/120,000 位等 0 ULP）、S7b2 GPU 3/3
  （RGB 同配方 0 差异、depth/seg 不回归）、S7b3 GPU 6/6（AA/flag 语义、PTX 197,583 B /
  27,536 B 不变）、S6b GPU 5/5（100 次 create/destroy growth 0 B）。
- **不宣称「完整渲染通过」「完整 R01 通过」「产品渲染验收通过」「上游完全等价」。**
  本片只给 r01-p8 渲染 MVP 的收口证据（十项映射见 §6），未满足项见 §7。

## 2. 产品 camlight 接线（数据来源与调用链）

### 2.1 输入夹具

`fixtures/render-bvh/scene-e2e/camlight-model.json`（416 行、6,499 B、sha256
`cf7af78334a52db598c34f7c582f603a3dd727c5641a318f5624481760bdc1be`）：
authoring 脚本 `scripts/authoring/render-bvh-s7b/capture_e2e_camlight.py`
（198 行、sha256 `02707f6f067d92acae8aa4495fc705db33c749425b55ff3f95c097299f65b9c5`）
用 pinned venv 的 **mujoco 3.15.0** 从冻结 `fixtures/render-bvh/scene-a/scene-315.mjb`
（sha256 `8953988f832c05b0561f04dd8b8e797a51c136fdead0ffda2a0b190bfd92e0d4`）读取模型字段：
`nq=8 nv=7 nbody=3 njnt=2 ngeom=7 nsite=0 nmocap=0 ncam=2 nlight=3`，double → f32 一次转换
（与产品上传 dtype 相同）。脚本**不初始化 warp/CUDA、不占 GPU、不读产品代码**。

### 2.2 产品链（测试内的唯一入口 `tests/support/render_bvh/e2e.rs::product_camlight`）

```text
KinematicsPlan::with_camlight(session, CamLightModelInput, KinematicsParameters::default(),
                              CamLightParameters::default())
  -> plan.create_data(1)
  -> data.write_qpos(qpos0)          # 模型默认姿态（pinned 3.15 forward 同一初始状态）
  -> plan.update_rigid(&mut data)     # 产品 f32 FK：body3 xpos/xquat
  -> plan.update_attached(&mut data)
  -> plan.update_com(&mut data)
  -> plan.update_camlight(&mut data)  # src/physics/camlight.rs 的 camlight_rigid + camlight_com
  -> data.readback()
  -> snapshot.camlight().world(0)     # cam_xpos / cam_xmat -> Vec<CameraPose>（按模型相机 id）
  -> renderer.render(context, fields, model, &product.poses, &rays)
```

调用链可追踪点：
- `tests/support/render_bvh/e2e.rs` 的 `product_camlight`（构造 `poses`）；
- `tests/render_e2e.rs::render_input_poses_come_from_camlight_not_fixture` 中
  `e2e::render_read_all(..., &product.poses, &rays)`（渲染只吃 `poses` 参数）；
- 夹具 `camera_data` 只出现在 `e2e::compare_pose_bits`（比对）与
  `s7b1::camera_data_poses`（反证②的「若吃夹具」对照）里，**从不**作为正常渲染输入。

### 2.3 位姿逐位结果

| 项 | 结果 |
| --- | --- |
| camera0（fixed, body0）origin/xmat | 与同配方参考 data 位姿 **0 位差** |
| camera1（TARGETBODY → body2）origin/xmat | 与参考 data xmat **0 位差**（look-at 产物） |
| 产品 FK body 位姿 | body1 `[0.1,-0.2,0.3]`、body2 `[0.2132049,-0.07466385,0.5897853]`（参考 data xmat 一致） |
| camera0 vs host `quat_to_mat` 辅助口径 | max **1 ULP / 9 分量中 2 个**（见下） |
| 三灯 `light_xpos`/`light_xdir` | 产品 camlight == 渲染器 `light_data_poses` == pinned 3.15 位型，**0 位差** |

camera0 的 1 ULP 说明（辅助口径，不是验收缺陷）：权威口径是 pinned 3.15 `mjwarp.forward` 的 data
位姿位型，产品 camlight 输出与之 **0 位差**；host `crate::math::quat_to_mat` 是代数展开实现，
而 warp 1.15 `math.quat_to_mat` 走 `quat_rotate` 三列路径（`warp/native/quat.h`：
`c = 2w²-1`、`d = 2(qx·x0+qy·x1+qz·x2)`），两者在末位可差 1 ULP。该 host 函数**不参与渲染输入**
（渲染输入来自产品 camlight 输出），故不影响端到端数字；测试断言记录为 `max_ulp ≤ 1`。

## 3. 端到端数字（默认 28 参数、spp=1、precomputed rays）

命令：`cargo test --locked --features cuda-probe --test render_e2e -- --ignored --test-threads=1 --nocapture`

| 项 | 相机 | 结果 |
| --- | --- | --- |
| RGB vs 同配方参考（≤1 LSB 硬指标） | cam0 320×240 | 0/76,800 像素差异，max_byte_diff=0 |
| | cam1 240×180 | 0/43,200 像素差异，max_byte_diff=0 |
| depth vs 同配方参考（≤4 ULP 硬指标） | cam0 / cam1 | 0 不一致、max **0 ULP**（120,000/120,000 位等） |
| seg vs 同配方参考（全等） | cam0 / cam1 | 0 不一致（120,000/120,000） |
| 两次运行哈希 | — | `depth_seg_hash=03efa95416bbd008`、`rgb_hash=ed50f750936c0275`，逐位相同 |
| 跨配方（S0b fast-math）偏差记录 | cam0 | max_byte_diff=1，beyond_1_lsb=0（2 像素，仅记录） |
| | cam1 | max_byte_diff=0（全位等，仅记录） |
| 产品 camlight 位姿 vs 夹具 data 位姿渲染 | — | RGB/depth/seg **逐位相同**（位姿来源替换无副作用） |

## 4. 反证：渲染输入的位姿来源 = 产品 camlight 输出

用例：`render_input_poses_come_from_camlight_not_fixture`

1. **夹具篡改不影响产品位姿渲染**：把本地比对副本的 `camera_data` 改成 identity+原点后，
   以产品 `poses` 重渲染 → RGB/depth/seg 与篡改前**逐位相同**；产品位姿仍与原始参考 0 位差。
2. **夹具回填会被检测**：若把篡改后的夹具位姿喂给渲染（这是「夹具回填」假设下的输入），
   RGB 变化 **115,972/120,000 像素**。
3. **产品位姿扰动必须响应**：camera1 绕 y 轴 0.05 rad + `xpos.x += 0.3` → RGB 18,859、
   depth 19,182、seg 9,464 像素变化（渲染内核用 `cam_xmat/cam_xpos` 把局部射线变换到世界系，
   `cuda/render_megakernel.cu` 第 148-151 行，因此位姿是必需输入）。
4. 同扰动两次渲染逐位相同；另一扰动（`xpos.z += 0.3`）结果不同；恢复产品位姿后逐位复现。

## 5. 生命周期与负向控制

| 项 | 结果 |
| --- | --- |
| create→render→显式 refit→render→destroy | 全链路通过；位姿不变时 refit 后输出逐位相同（结构断言） |
| `refit_bvh` 唯一重拟合入口 | `refits=1`；render 不隐式 refit |
| destroy 后显存 | baseline `15,756,951,552 B` → free 同值，growth **0 B**（阈值 16 MiB） |
| 禁用输出 | 只启 rgb 的 context：depth/seg `read` 返回 `InputError::DisabledOutput` |
| 负向控制 | shadows / `use_precomputed_rays=false` / skybox 三类写前拒绝逐类可判定 |
| AA 射线数 | spp=2（nsamples=4）+ spp=1 射线 → `InvalidArray("ray_count")` 写前拒绝 |

## 6. r01-p8 收口映射（十项，供 Arch 汇总；不宣称「完整 R01 通过」）

| # | 项 | 本片结论 | 可追溯路径 |
| --- | --- | --- | --- |
| 1 | 世界/相机/资产 | 产品 camlight 从冻结 mjb 模型字段产出两相机 data 位姿；灯位姿与渲染器一致 | `docs/development-evidence/render-bvh/s1-model-input/receipt.json`；本片 §2 |
| 2 | BVH（场景） | 场景 BVH 由 context 拥有；refit 显式触发；渲染消费当前 BVH | `docs/development-evidence/render-bvh/s2b-scene-bvh-gpu/receipt.json` |
| 3 | BVH（网格） | 网格三角 BVH 在 context 内；`mesh_rays` 局部系语义 | `docs/development-evidence/render-bvh/s3-mesh-bvh/README.md` |
| 4 | 28 默认参数 | 默认 context（`enabled_geom_groups=[0,1,2,3]`、rgb/depth/seg 全开、spp=1、precomputed rays） | `docs/development-evidence/render-bvh/s6a-context-params/receipt.json` |
| 5 | 生命周期 | create→render→refit→render→destroy；显存回基线（growth 0 B） | `docs/development-evidence/render-bvh/s6b-context-lifecycle/receipt.json`；本片 §5 |
| 6 | RGB | 对同配方参考 120,000/120,000 像素 0 差异（≤1 LSB） | `docs/development-evidence/render-bvh/s7b2-shading/receipt.json`；本片 §3 |
| 7 | 深度 | 对同配方参考 120,000/120,000 位等 max 0 ULP（≤4 ULP） | `docs/development-evidence/render-bvh/s7b1-depth-seg/receipt.json`；本片 §3 |
| 8 | 分割 | 对同配方参考 0 不一致（全等） | `docs/development-evidence/render-bvh/s7b1-depth-seg/receipt.json`；本片 §3 |
| 9 | 资源检查 | 显存回基线无泄漏；禁用输出拒绝读取；100 次 create/destroy 稳定（S6b 回归） | `docs/development-evidence/render-bvh/s6b-context-lifecycle/README.md`；本片 §5 |
| 10 | 拒绝面 | shadows/动态射线/skybox 三类写前拒绝；AA 射线数拒绝；flag/AA 语义由 S7b3 覆盖 | `docs/development-evidence/render-bvh/s7b2-shading/README.md`、`s7b3-flags/receipt.json`；本片 §5 |

## 7. 未满足项（如实记录）

- `nworld > 1` 未验收（本片 scene-a 单 world；多 world 渲染路径未测）。
- 阴影、天空盒、splat、柔体、高度场（hfield）、SDF、多相机选择（`cam_active`/动态射线）仍未实现，
  写前拒绝。
- Linux / T4 未在本片验收（GPU 证据仅 RTX 4070 Ti SUPER / 596.36 / SM89）。
- 上游完全等价未声明（本片只证明 scene-a 默认参数下的渲染数值对齐；其余场景/参数未覆盖）。
- 性能未验收（本片只记录用例耗时；无性能门）。
- AA 仅 `samples_per_pixel=2`（nsamples=4）有图像级参考；更高 spp 未采样。
- `camera0 vs host quat_to_mat` 有 1 ULP 辅助口径差异（见 §2.3；权威口径 0 位差，不参与渲染输入）。

## 8. 环境、命令、门禁、显存

- 设备：NVIDIA GeForce RTX 4070 Ti SUPER / 驱动 596.36 / SM89；开窗桌面基线约 5,091 MiB，
  套件跑完 `5,053 MiB`（回基线以下、无本任务计算进程）。
- 产品渲染配方：`WARP_1_15_RENDER`、`use_fast_math=false`、`compute_70`；A′ 门禁 PTX 与 S7b3 一致
  （megakernel 197,583 B、aa_resolve 27,536 B；本片不改渲染内核源码）。
- 命令：
  - GPU 套件：`cargo test --locked --features cuda-probe --test render_e2e -- --ignored --test-threads=1 --nocapture`
  - 主机用例：`cargo test --locked --features cuda-probe --test render_e2e -- --test-threads=1 --nocapture`
  - 回归：`render_kernel_depth_seg` / `render_kernel_rgb` / `render_kernel_flags` / `render_context_lifecycle`
  - 夹具生成：`target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/authoring/render-bvh-s7b/capture_e2e_camlight.py`
- 门禁：fmt / clippy / clippy-cuda / unit（见 receipt `gates`；证据以最终源码为准）。
- 日志：`gpu-run.log`（主机+GPU 套件、设备与显存前后）、`regression.log`（S7b1/S7b2/S7b3/S6b）。

## 9. 夹具与 provenance

- `fixtures/render-bvh/scene-e2e/camlight-model.json`：
  - 来源 mjb：`fixtures/render-bvh/scene-a/scene-315.mjb` sha256
    `8953988f832c05b0561f04dd8b8e797a51c136fdead0ffda2a0b190bfd92e0d4`；
  - 生成脚本 sha256 `02707f6f067d92acae8aa4495fc705db33c749425b55ff3f95c097299f65b9c5`；
  - 转换口径：mjModel double → f32 逐元素一次（产品上传 dtype）；
  - 夹具内 `provenance` 段记录 mjb sha、脚本 sha、Python/mujoco 版本、camera_data 参考来源。
- 参考 data 位姿**不重派生**：测试直接读 S7b1 已冻结同配方夹具
  `fixtures/render-bvh/scene-a/s7b/depth_seg_fast_math_false.json`
  （sha256 `d6ab6dde58034df5c1bc69ab397026a8f7c3aaf58d29d25edb032d3818000dc7`），
  夹具只用于比对。
- RGB 同配方参考：`fixtures/render-bvh/scene-a/s7b/rgb_fast_math_false.json`
  （sha256 `d0de2b6069241518513a8b6ec1f65dfb76ce43d4c86b663374e8cf32c6e85b4a`）。
- 本片未改 S1–S7 产品源码、未改渲染内核源码、未改任何 S2–S7 证据与夹具；
  `src/render/bvh/**`、S2–S6 夹具、`fixtures/render-bvh/scene-a/reference/**` 均未触碰。
