# 渲染/BVH S6b：render context 生命周期、资源所有权与显式 refit · 证据

任务：`2026-10-24-render-bvh-s6b`（plan.json 切片 S6b，依赖 S6a/S2/S3/S4，均 verified）。
参考：冻结上游 MJWarp 3.15 `71da24d956378a87a703b6e1442b13aec0c4ac29`
（`mujoco_warp/_src/render.py`、`render_util.py`；`types.RenderContext`）。

## 1. 范围与结论

本片给产品侧一个显式的 render context 生命周期（无像素内核，渲染内核属 S7b）：

| 验收（plan S6b） | 结果 |
| --- | --- |
| 100 次 create/destroy 显存稳定、阈值记录、无 UAF | **通过**：10 次采样全部等于基线，增长 **0 B**（阈值 16,777,216 B），create/destroy 计数各 +100，elapsed 6,575 ms |
| 移动几何后 render 用旧 BVH；`refit_bvh()` 后用新 BVH | **通过**：A 命中 d=4.0；移到 B 不 refit 时 B 与 A 均 miss；显式 refit 后 B 命中 d=4.0、A miss，refit 计数 1 |
| render 不触发物理/不隐式刷新 | **通过**：render 路径前后 `refits()` 恒为 0/1，当前位姿与 AABB 输入不被渲染修改 |
| 禁用输出不可读；布局/寿命检查 | **通过**：rgb 480,000 B 读回 120,000 像素；depth/seg 禁用时 `byte_len`/`read` 均 `InputError::DisabledOutput` |
| drop 释放 BVH/网格/纹理/输出资源 | **通过**：预热后 context 占用 2,097,152 B，`destroy()` 后显存回到基线（retained = 0 B）；销毁后可再次创建 |

GPU 用例 5 passed / 0 failed（测试体内 7.56 s）；主机用例 2 passed / 0 failed。
窗口：RTX 4070 Ti SUPER / 驱动 596.36 / SM89；开窗 2,870 MiB 桌面基线，跑完回 **2,870 MiB**。

## 2. 语义与设计（对齐冻结上游）

- **所有权**：`RenderContext` 持有 `SceneResources`（S2a `SceneBvh` + S2b GPU 查询器）、
  `MeshResources`（S3 三角 BVH + 查询器）、`TextureResources`（S4 CUDA 纹理对象）、
  `OutputResources`（rgb/depth/seg 设备缓冲）。`create`/`destroy` 成对，`Drop` 兜底；
  字段顺序即释放顺序（输出缓冲 → 纹理对象 → 网格/场景）。
- **显式 refit**：上游 `create_render_context` 建好 `rc.bvh` 后，物理只写 `geom_xpos/xmat`；
  渲染内核读当前位姿 + 旧 BVH 边界（陈旧 BVH 仍可渲染），`refit_bvh` 是显式 API。
  产品侧 `RenderContext::refit_bvh()` 是**唯一**重拟合入口（`SceneResources::refits` 计数），
  `render_rays` 只读当前 BVH，不触发物理、不改位姿。
- **陈旧语义的可观测定义**：`render_rays` 用当前位姿的 leaf 几何 + 当前拥有的 BVH 做 S2b 查询。
  位姿移到 B 但不 refit 时：射线 A（旧位置）在 BVH 内但几何已移走 → miss；射线 B 被旧根包围盒拒绝 → miss。
  显式 refit 后 B 命中。这与上游「陈旧 BVH 可渲染、命中反映旧加速结构」一致。
- **输出**：`OutputResources` 按 S6a `OutputLayout` 分配（`rgb_pixels/depth_pixels/seg_slots` × nworld，
  rgb u32 4 B、depth f32 4 B、seg vec2i 8 B）；未启用通道不分配且不可读，错误类别与
  `ImageBatchLayout` 一致（`InputError::DisabledOutput`）。
- **纹理/网格**：`TextureResources` 复用 S4 硬件纹理对象路径（`TextureObjects` 析构按
  `cuTexObjectDestroy` → `cuArrayDestroy`）；`MeshResources` 复用 S3 `set_points`/`refit` 语义
  （`set_points` 后不自动重拟合，测试验证 refit 前陈旧、refit 后命中）。
- **无 UAF（结构性）**：资源不以裸句柄逃逸，全部访问经 `RenderContext` 借用；`destroy(self)`
  消费 context，编译器禁止释放后使用；`TransferBuffer`/`TextureObjects`/查询器析构释放设备资源。

## 3. GPU 实测

环境：`nvidia-smi` RTX 4070 Ti SUPER / 596.36；开窗 2,870 MiB（桌面基线、无满载计算进程），
跑完 2,870 MiB。命令：

```text
$env:CUDA_PATH = (Resolve-Path 'target\toolchains\cuda-12.8.1').Path
$env:PATH = "$env:CUDA_PATH\bin;$env:PATH"
cargo test --locked --features cuda-probe --test render_context_lifecycle -- --ignored --test-threads=1 --nocapture
```

| 用例 | 关键数字 |
| --- | --- |
| `stale_bvh_until_explicit_refit_and_render_never_refits` | 初始 `geom_id=0`、distance `4.000000000`；不 refit 时 B/A 均 `geom_id=-1`；refit 后 B `geom_id=0`、distance `4.000000000`、A `-1`；`refits=1` |
| `disabled_outputs_cannot_be_read_and_sizes_follow_layout` | `rgb_pixels=120000`、`rgb_bytes=480000`；depth/seg `enabled=false` 且 byte_len/read 均拒绝 |
| `hundred_create_destroy_cycles_keep_memory_stable` | 基线 free `15,811,477,504 B`；10 次采样全部相同；`growth=0 B`（阈值 `16,777,216 B`）；elapsed `6,575 ms` |
| `drop_releases_mesh_texture_and_output_resources` | 预热一次性驱动保留 `33,554,432 B`（32 MiB，首次加载 scene/mesh/sample 三个模块的驱动代码缓存，见 §4）；预热后基线 `15,811,477,504 B`；context 占用 `2,097,152 B`；destroy 后 free 回基线、retained `0 B`；`textures=3`；网格 `stale=-1 → fresh=0`、`mesh_refits=1`；纹理采样 `[0.49803925, 0.14901961, 0.49803925, 1.0]` |
| `world_count_mismatch_is_rejected_before_buffers` | 场景 world 数与 context 不一致 → `OwnerError::WorldsMismatch`（分配前） |

主机用例：

- `scene_state_bounds_match_s2a_leaf_aabb`：`SceneState::bounds()` 与 S2a `compute_leaf_aabb` 逐 leaf 逐位相等；
- `scene_state_rejects_invalid_inputs_before_allocation`：空场景/world 0/叶-位姿长度不符/`set_poses` 长度不符/
  hfield（geom_type=1）/非有限位姿，均在分配前拒绝。

## 4. 观察项（非阻断）

1. **驱动代码缓存一次性保留 32 MiB**：同一进程首次加载 scene/mesh/sample 三个内核模块后，
   free 显存下降恰为 `33,554,432 B`（32 MiB，驱动 JIT/代码缓存口径），此后不再增长：
   - 100 次循环（同一 module 反复加载）基线后的增长为 **0 B**；
   - drop 用例的预热后 destroy → retained **0 B**。
   因此「drop 是否释放」用**预热后第二轮**判定，一次性驱动缓存不计入 context 泄漏；
   证据保留原始数字，不放宽阈值（阈值仍是 16 MiB，实测 0）。
2. context 占用 2,097,152 B：默认两相机共 120,000 像素，S0b `context.json` 三通道全启用 →
   rgb 480,000 B + depth 480,000 B + seg 960,000 B + 3 个模型纹理的 CUDA array；
   free 显存在 create 与 destroy 之间恰好变化这个值。

## 5. 门禁

`normify gate 2026-10-24-render-bvh-s6b`（2026-10-25，本提交前复跑）：

| 门禁 | 结果 |
| --- | --- |
| fmt | 通过 1259 ms |
| clippy | 通过 2873 ms（`--all-targets`，0 警告） |
| clippy-cuda | 通过 5442 ms（`--all-targets --features cuda-probe`，0 警告） |
| unit | 通过 22561 ms，测试 **531 过 0 败**（默认 feature） |

本片 host 用例（2 个）与 GPU 用例（5 个）在 `--features cuda-probe` 下执行，见 §1/§3；
默认 feature 的 integration 测试文件被 `#![cfg(feature = "cuda-probe")]` 排除，因此 unit 计数与 S6a 相同（531）。

## 6. 未满足项与 deferral

- **像素渲染内核未实现**：RGB/depth/seg 着色、AA resolve、阴影/天空盒/聚光灯、纹理/顶点法线/镜面/自发光
  的实际求值属 S7b（并依赖 S7a 发射 ABI）；本片只做输出缓冲所有权与 BVH 生命周期，
  `render_rays` 是「渲染消费 BVH」的查询代理，不宣称图像验收。
- **预计算射线缓冲**（`use_precomputed_rays=true` 时上游 context 里的 ray/ray_offset）仍由 S5
  `RayBuilder` 按需构建，不在 S6b 的 owner 内；接入渲染 context 留 S7b/S8。
- **多 world 场景**：GPU 用例用 `worlds=1`（多 world BVH 分组已由 S2a/S2b 验收）；S6b 不新增多 world 夹具。
- **上游上下文 retain/push/pop**：仅 S4 纹理对象路径需要（`TextureObjects` 内部成对 retain/release），
  S6b 不新增上下文操作；证据沿用 S4 结论。
- 未跑全量 G01（按 fast-loop 口径，用户授权），只跑本切片 GPU 用例 + 四门禁。
- 未改 S2/S3/S4 证据与夹具；未启动 Python；不新增参考夹具。

## 7. 产物

| 路径 | 行数 | bytes | sha256 |
| --- | --- | --- | --- |
| `src/render/context/owner.rs` | 364 | 13,054 | `717628612a85b2037d0f51f874154c02432671598ca53f2648d679bdf6f9636d` |
| `src/render/context/lifecycle.rs` | 322 | 12,157 | `4e4aae9726081e3eaf90000cefc85b513ca2c190d9bb3a893d5649b35051c71b` |
| `src/render/context.rs`（改） | 397 | 14,743 | `c4838c2ebb76b3c056784d9eecc9ff135fc7b5f62f2d25147eea10c609f17bed` |
| `src/render/mod.rs`（改） | 359 | 11,393 | `4081dbd27764c2889fc43baeeab44645f00f0d1bdfde24f70bfe1b7d916b9ddd` |
| `tests/render_context_lifecycle.rs` | 477 | 18,258 | `d965753722720e92c6ae466ed9ee02dc0ece10d25d71ed477e3c72bf9374cfe0` |

新文件行数全部 ≤500；`src/render/context.rs` 与 `src/render/mod.rs` 保持 <500 行且只增少量导出行。
