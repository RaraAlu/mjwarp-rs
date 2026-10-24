# 渲染/BVH S5：相机筛选/射线生成/AA 采样与拒绝面

- 任务：`2026-10-24-render-bvh-s5`（depends_on S1 已 verified）
- 日期：2026-10-25
- 结论：**pass**（覆盖范围内）。17 个相机 context × 全网格射线与冻结上游 `rc.ray`/`rc.ray_offset`
  **逐位相等**（SHA-256 全等 + 每相机 128 抽样位型全等）：透视 intrinsic 分支、透视 fovy 分支、
  正交（单正交 + 正交/透视混合）、`samples_per_pixel=2`（4 子样本）、相机选择 bool/name/index/空列表、
  异构分辨率、znear 变体、动态射线模式（`use_precomputed_rays=false`，与预计算位等）。
  拒绝面 7 项映射到产品错误面。**AA 图像级验收按 Arch 裁决延后 S7b/S8**，本片只验射线层；
  世界系变换（`cam_xmat @ dir`）归 S7b。

## 1. 范围

覆盖：

- `src/render/camera.rs`：相机选择（`None`/`[]`/bool 掩码/名称/id 列表）、`cam_res` 解析
  （None→模型值、单元素复制、异构列表、数量校验）、`render_rgb/render_depth/render_seg`
  推导（None→`cam_output` 位；bool/单元素/全量列表）、`total_rays`、`rgb_pixels`、
  `samples_per_pixel²`、`znear/zfar = f32(f64(vis.map.znear|zfar) * stat.extent)`，
  以及写前拒绝面（空相机列表合法）。
- `src/render/rays.rs`：相机空间射线缓冲 API（`RayBuffer` 的 dir/offset + 规范字节序）、
  射线条数/布局（`[sample][camera][row][column]`，x 最快）、`RayMode::{Precomputed,Dynamic}`、
  设备内核封装 `RayBuilder`（打包/发射/回读，复用既有 `SynchronousKernel` 四缓冲 ABI）。
- `src/render/camera/cuda/build_rays.cu`：`compute_ray` 的 f32 等价翻译（透视 intrinsic/fovy
  两分支 + 正交分支），布局与子样本偏移与上游 `_build_rays` 一致；输出每射线 6 个 f32
  （dir3+offset3 交错），显式 `fmaf` 钉死与冻结参考一致的收缩顺序。
- `tests/render_camera_rays.rs`：主机验收（夹具解析、选择/分辨率/输出解析、布局下标、
  空列表、动态=预计算位等、拒绝面映射）与 GPU 验收（全网格 SHA-256 + 抽样位型，默认 ignore）。
- `scripts/authoring/render-bvh-s5/`：离线参考捕获与支持（环境/哈希/设备数组读取/SHA-256/抽样）。
- 参考夹具：`fixtures/render-bvh/scene-a/s5/camera_rays.json`。

不覆盖（显式）：

- 世界系射线（`ray_dir_world = cam_xmat @ dir`、正交 `origin + cam_xmat @ offset`）与
  `max_cam_dist = zfar/max(-dir_z,1e-6)`：属渲染内核 S7b。
- AA 合成图像：产品尚无渲染内核，图像级验收延后 S7b/S8（本片验射线层）。
- 纹理/材质/光照/阴影/完整 R01：后续切片。

## 2. 语义与布局（冻结上游 71da24d）

- `compute_ray(projection, fovy, sensorsize, intrinsic, img_w, img_h, px, py, znear)`：
  - 正交：`dir=(0,0,-1)`；`sensor_h=fovy`（直接作为传感器高度）、`aspect=img_w/img_h`、
    `sensor_w=sensor_h*aspect`、`offset=(left+sensor_w*u, top+(bottom-top)*v, 0)`，
    `u=(px+0.5)/img_w`、`v=(py+0.5)/img_h`。
  - 透视 intrinsic 分支（`sensorsize[1]!=0`）：aspect 修正 sensor_w/h 后
    `inv_fx_znear=znear/fx` 等构造 left/right/top/bottom，`dir=normalize(x,y,-znear)`；
    fovy 分支：`half_height=znear*tan(0.5*fovy*deg2rad)`、`half_width=half_height*aspect`。
- 预计算布局：`nsamples=spp²`；`[sample][camera(active 顺序)][row][column]`，x 最快；
  sample 内 `px=xid*spp + sample%spp`、`py=yid*spp + sample//spp`，图像尺寸乘 spp。
- **动态模式**：渲染内核逐像素以同参数调用 `compute_ray`；spp=1 时与预计算版参数完全相同，
  因此 `dynamic_default`/`dynamic_ortho` 与对应预计算 context 位型相同（夹具记录并断言）。
- 相机 id vs 输出序号：`cameras`（`cam_id_map` 顺序）与输出位置分离；`indices_reversed`
  （模型相机 id `[1,0]`）验证顺序映射。
- `znear/zfar`：MuJoCo 3.12 `vis.map.znear/zfar` 原生 float32，`stat.extent` 为 double；
  上游 `float(f32 * f64)`，产品按同口径计算后 f32。

## 3. 参考捕获（authoring，GPU）

- 直接读取冻结上游 `RenderContext` 的 `rc.ray`/`rc.ray_offset`（f32 vec3）位型 + 相机元数据
  （`cam_id_map`/`cam_res`/`render_*`/`total_rays`/`samples_per_pixel`/`znear`/`zfar`），
  不经过 Python 侧再计算。
- 模型：`default`（scene-a 2 个透视相机：intrinsic 分支 320×240、fovy 分支 240×180）；
  `ortho`（3.15 MjModel 副本，`cam_projection[1]=ORTHOGRAPHIC`；Arch 批准，provenance 记录
  原始 MJB sha 与覆盖字段/值）；`znear_x2`（`vis.map.znear` ×2，f32 0.02）。
- context 矩阵 17 个：default_all / mask_first / mask_second / names_track（`cam-track`）/
  indices_reversed / empty_active / empty_mask / res_tuple（100×50 复制）/ res_hetero（100×50, 7×9）/
  res_single_list（160×120 复制）/ render_rgb_first（rgb=[T,F]、depth=[F,T]）/ spp2_default /
  ortho_single / ortho_mixed / znear_x2_default / dynamic_default / dynamic_ortho。
- 全网格摘要：`SHA-256(dir[0..n] LE f32 ‖ offset[0..n] LE f32)`；每相机 128 个等距像素 ×
  全部子样本存原始 dir/offset f32 位型（含全局下标）。
- 两次完整捕获逐位一致（第二遍 1.646 s）；空相机列表 context 的射线数组为空、SHA-256 为
  `e3b0c442…`（空字节）。
- 拒绝面 7 项由上游异常类型记录：`spp=0`(ValueError) / `spp>1`+dynamic(ValueError) /
  `spp>1` 无 RGB(ValueError) / `cam_res` 数量不符(ValueError) / bool 掩码长度不符(AssertionError) /
  相机名不存在(ValueError) / 相机 id 越界(**IndexError**——上游未校验、直接下标崩溃；产品按
  「非法 camera 选择写前拒绝」验收要求显式拒绝，属记录在案的硬化差异)。
- GPU：RTX 4070 Ti SUPER / 驱动 596.36；捕获显存基线 2871 MiB、进程内 3111 MiB；
  warp 1.15.0 / mujoco 3.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4；上游 `71da24d…`。

## 4. GPU 验收结果（命令与原始输出）

```text
set CUDA_PATH=target\toolchains\cuda-12.8.1; PATH=%CUDA_PATH%\bin;%PATH%
cargo test --locked --features cuda-probe --test render_camera_rays -- --ignored --test-threads=1
```

结果：`2 passed / 0 failed`（测试体内 1.11 s）。

```text
camera-ray contexts checked: 17
builder rejection surface ok
```

- 全网格位等：17 个 context 的 `RayBuffer::canonical_bytes()` SHA-256 与夹具全等；
  抽样位型（每相机 128 像素 × 全部子样本的 dir/offset 位型）逐项全等。
- 覆盖：透视 intrinsic 分支、透视 fovy 分支、正交单相机/混合、spp=2 四子样本、
  选择 bool/name/index/空列表、分辨率 None/复制/异构、znear ×2、dynamic 模式。
- 拒绝面（设备侧打包前）：samples 不一致、相机 id 越界、零分辨率、动态模式 spp>1 均拒绝。
- 主机测试：`cargo test --locked --test render_camera_rays` → 4 passed（解析/布局/空列表/
  拒绝面映射，不占 GPU、不启 Python）。

## 5. 门禁与测试

```text
normify gate 2026-10-24-render-bvh-s5
fmt 通过
clippy 通过
clippy-cuda 通过
unit 通过（测试 516 过 0 败）
```

- `cargo fmt --check`、`cargo clippy --all-targets --locked -- -D warnings`、
  `cargo clippy --all-targets --locked --features cuda-probe -- -D warnings` 均通过。
- 默认 `cargo test --locked` 不含 GPU 测试（`#![cfg(feature = "cuda-probe")]`）。
- 新文件行数（≤500）：`camera.rs` 469、`rays.rs` 299、`camera/cuda/build_rays.cu` 169、
  `tests/render_camera_rays.rs` 498、`capture_s5_camera_rays.py` 345、
  `s5_capture_support.py` 184；`src/render/mod.rs` 357（既有文件，新增模块挂载与错误 Display）。
- 依赖：`Cargo.toml` 新增 dev-dependencies `sha2 = "=0.10.9"`（同 crate 可选依赖版本）与
  `serde = "=1.0.229"`（同 crate 可选依赖版本，用于夹具反序列化）；`Cargo.lock` **无变化**
  （两依赖已在 lock 中）。

## 6. 产物哈希（提交 blob / LF 口径）

| 文件 | 行数 | sha256 |
| --- | --- | --- |
| `src/render/camera.rs` | 469 | `e2295f169161dca48a2b88a8808480487ef7fd1f518b8ba0d83cd734ce019b14` |
| `src/render/rays.rs` | 299 | `fdec43c79b1547a6083c468ddf78a56cdfa2c4b125f1753e82d8194a888d7697` |
| `src/render/camera/cuda/build_rays.cu` | 169 | `e38cdf2a29d529ee3cae76f88db476fc999ff555f992b8219c938664ac063577` |
| `src/render/mod.rs` | 357 | `231c119668cb935ab47db75446242edbd02a08228c4211e5a2aa536dab018a13` |
| `tests/render_camera_rays.rs` | 498 | `c4d3ea5402869f79dcd14283b5f9902b3e4b7ecb696ce27819472909387121d8` |
| `scripts/authoring/render-bvh-s5/capture_s5_camera_rays.py` | 345 | `57f6fa6b716bdc7f0b86227dffba25e3114cb43bb2b5c5b2cac7927b66a864a6` |
| `scripts/authoring/render-bvh-s5/s5_capture_support.py` | 184 | `ce1deef3ed99141f1813f21a4504788352857d84c1438d7ede971d3cbde86943` |
| `fixtures/render-bvh/scene-a/s5/camera_rays.json` | 69762 | `545a1442fc5ac67d8de9abd6d7f7cf0a24e5a86baa3b473f660d8aae83842022` |

`Cargo.toml` 的 dev-dependency 变更随任务提交；`Cargo.lock` 无差异（见 §5）。

## 7. 验收映射（plan S5 四条 + 任务五条）

| 验收 | 本切片 | 状态 |
| --- | --- | --- |
| 每相机/投影像素 dir/offset 与参考一致（优先逐位） | 17 context 全网格 SHA-256 全等 + 抽样位型全等（位等，非容差） | ✅ |
| 透视 + 正交均覆盖 | 透视 intrinsic/fovy 两分支 + 正交单/混合 context | ✅ |
| `samples_per_pixel=1/2` 射线数/偏移/顺序正确、AA 图像 | spp=1 与 spp=2（4 子样本）全网格位等；**AA 图像延后 S7b/S8（Arch 裁决）** | ✅（射线层）/ 延后（图像） |
| `use_precomputed_rays=false` 动态路径一致 | dynamic 两个 context 与预计算对应 context 位型相等 | ✅ |
| 相机筛选（bool/name/index/空列表）、id vs 输出序号、异构/空列表不越界 | 选择矩阵 + `indices_reversed` + 异构分辨率 + 空列表；产品显式校验越界 | ✅ |
| 拒绝面写前拒绝 | 7 项上游异常→产品错误面映射；设备打包前 4 项校验 | ✅ |
| 证据/门禁（README+receipt、四门禁、≤500 行、GPU 独占/显存、不启 Python） | 见 §3/§5；GPU 跑完释放回 2865 MiB 桌面基线 | ✅ |

## 8. 未满足项 / 限制

- **AA 图像级验收延后 S7b/S8**（产品无渲染内核）；本片不宣称 AA 图像一致。
- 世界系射线与 `max_cam_dist`（zfar 截断）不在本片，属 S7b。
- 上游 `cam_active` 整数越界是未校验的 `IndexError` 崩溃；产品按验收「非法选择写前拒绝」
  显式拒绝（硬化差异，已记录）。
- `znear_x2` 与默认 context 位型相同：归一化方向对 znear 的整体缩放不变（factor 2 精确缩放），
  变体用于验证产品确实按 znear 公式计算（错误实现会改变结果）；不构成独立数值证据。
- 单 fixture（scene-a 2 相机 + 17 context），无 fuzz/任意相机属性组合；相机名仅 ASCII 夹具名。
- 参考为 3.15 冻结产物；正交分支的参考来自 3.15 MjModel 副本覆盖（provenance 在夹具内），
  产品输入来自 S1 已验收的 3.12 字段（默认模型字段与 S1 model-312.json 位型逐项相等，捕获脚本
  交叉校验）。
- 未跑全量 G01；按 fast-loop 口径（fmt/clippy/clippy-cuda/unit + 定向 GPU 测试）。
