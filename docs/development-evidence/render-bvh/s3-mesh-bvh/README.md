# 渲染/BVH S3：网格三角 BVH 构建/重拟合与射线 uv/face 语义命中

- 任务：`2026-10-24-render-bvh-s3`（depends_on `2026-10-24-render-bvh-s2-close` 已 verified）
- 日期：2026-10-25
- 结论：**pass**（覆盖范围内）。5 个场景用例 × 436 条射线逐条对照：geomid/face 全等、
  distance max ULP **0**、u/v max ULP **0**、normal max **4.0e-8 rad**（≤1e-4）；`no_cull` 40 命中
  覆盖背面案例；`set_points`+refit 后变化射线集合与参考完全一致（32 条）；`tight_above` 29 命中 /
  `tight_below` 0 命中验证 `max_t` 严格边界；两 world 0 跨；退化三角网格 16 条射线 9 命中、
  distance max 2 ULP、无 panic。冻结 S2b 交叉校验全部成立（mesh ray 91/92 的 distance/normal
  与 S2b 位型逐位相等）。hfield/flex/SDF/纹理/相机不在本片。

## 1. 范围

覆盖：

- `src/render/bvh/mesh.rs`：从局部系 `mesh_vert`/`mesh_face` 构建三角 BVH（分箱 SAH、叶容量 8、
  节点先子后父），`refit()` 与 `set_points()` 支持动态顶点更新（对应上游 `mesh.points` 赋值后
  `mesh.refit()` 的语义）；构建/更新输入在分配与写入前校验（空网格、索引越界、非有限顶点、
  长度不符、计数溢出）。
- `src/render/bvh/mesh_query.rs`：宿主把 `MeshBvh`、geom 位姿与逐射线 `max_t` 打包成四个设备缓冲，
  经既有 `SynchronousKernel` 编译/发射/回读；打包前校验射线数、`max_t` 有限且非负、位姿有限、
  geom_id、栈深与计数溢出。
- `src/render/bvh/cuda/mesh.cu`：手写设备内核，语义翻译冻结上游渲染路径
  `ray_mesh_with_bvh` → Warp `mesh_query_ray`：`_ray_map` 到局部系、Woop 三角求交
  （`intersect_ray_tri_woop`，含 fma 差分乘积）、最近命中、u/v 重心坐标、face 索引、
  `normal = normalize(mat @ n)`、最多 8 次背面剔除步进、distance = offset + t。
- `tests/render_mesh_bvh.rs` + `tests/support/render_bvh/mesh.rs`：主机验收（参考夹具解析、
  与 `model-312.json` 的 f64→f32 顶点一致性、构建/refit 不变量、宿主候选覆盖、退化三角形）
  与 GPU 验收（默认 ignore，`--ignored --test-threads=1` 显式运行）。
- `scripts/authoring/render-bvh/capture_s3_mesh.py` + `s3_mesh_capture_support.py` +
  `s3_mesh_replay.py`：离线参考捕获与静态重放（产品测试不启动 Python）。
- 参考夹具：`fixtures/render-bvh/scene-a/s3/mesh_query.json`（设备侧 f32 位型）。

不覆盖（显式）：

- hfield / flex / SDF / splat 的网格求交：不实现（hfield 类型在数据层显式拒绝）。
- 纹理、材质、光照、相机射线/成像：S4/S5/S7b；本切片只做网格射线语义。
- 渲染整体验收（R01）不在本片范围：只验证网格 BVH 的语义命中，不宣称整片渲染验收完成。

## 2. 接线方式（复用既有发射层，不新造）

产品 GPU 内核路径是 `src/runtime/transfer/kernel` 的 `SynchronousKernel`（NVRTC 编译 +
`fmad=true`、`fast_math=false`、`--std=c++17`、compute_70，PTX 精度门禁只允许 f32），
复用其四缓冲 `launch`（元数据 i32 / 参数 f32 / 射线 f32 / 输出 f32 + 4 个 u32 维度，256 线程，
grid 覆盖射线数）；未改动任何 runtime 文件。设备侧是真 `fma.rn.f32`；S2a 主机侧为位等使用的
f64 复刻路径不用于 device 路径。

设备内核 ABI（`mesh_bvh_ray`）：

```text
meta:   [n_nodes, n_tri, n_ray, root, cull_backfaces, geom_id, n_vert,
         links(2*n_nodes), order(n_tri), indices(3*n_tri)]
params: [node bounds 6*n_nodes, vertices 3*n_vert, geom_xpos 3, geom_xmat 9]
rays:   [origin 3, direction 3, max_t] per ray
out:    [distance, normal 3, u, v, face bits, geomid bits] per ray
```

遍历与节点布局是自建实现，验收只断言语义命中（geomid/face/distance/u/v/normal），不做节点布局断言。

**f32-only 与上游 f64 兜底分支**：上游 Woop 在任一重心分母恰为 0 时会用 double 重算 U/V/W；
产品内核受 PTX 精度门禁限制保持 f32。捕获脚本用 f64-refined 静态重放（`s3_mesh_replay.py`，
按 NVRTC fmad 收缩语义模拟）对夹具每条射线/每次剔除步进逐三角复算：场景射线
`触发次数 = 0`、全体 `判定变化次数 = 0`；退化三角网格用例触发 80 次，但落在「上游与 f32 路径
都拒绝该三角形」的输入上，判定不变。因此 f32-only 翻译在本夹具上与冻结参考语义等价。

## 3. 参考捕获（authoring，GPU）

- 语义来源：冻结上游（`71da24d…`）`mujoco_warp/_src/ray.py::ray_mesh_with_bvh`（`@wp.func`），
  即渲染路径同一函数；静态网格用例直接用渲染 context 构建的 `cubql` mesh BVH。
- 设备侧 f32 位姿：沿 S2b 口径记录送进内核的 `d.geom_xpos`/`d.geom_xmat` 位型；初始/移动位姿
  与 S2b 捕获逐位交叉校验通过。
- 动态 refit 参考：`set_points` 用例使用独立 `sah` mesh（上游自己的动态 refit 路径
  `refit_flex_bvh` 也是 `sah` + `mesh.refit()`）；冻结 render context 的 `cubql` BVH 只用于静态用例。
- 射线集：384 条 S0b 冻结射线（覆盖 mesh ray 91/92）+ 52 条确定性网格射线
  （每面 oblique front/back、近边缘擦边、离面 miss、世界轴射线；方向刻意避开面法线/面平面/
  边方向，避免 f32 退化到上游 f64 分支）。
- 用例：`initial` / `moved`（body 移动后重拟合）/ `no_cull`（cull_backfaces=false，背面命中）/
  `tight_above` / `tight_below`（逐射线 max_t = 1.01×/0.99× 参考命中距离，验证 max_t 严格边界）/
  `dynamic` / `dynamic_moved`（sah mesh `set_points` + refit）+ 独立退化三角网格用例
  （16 条射线：3 个有效面中心 × 4 方向 + 边缘/线状/对角射线）。
- 退化网格射线规则：射线集刻意避开「精确穿过共享顶点/边」的输入——那类命中在相同 t 上出现
  多个候选三角形，face 结果取决于 BVH 遍历顺序（布局相关），而验收明确不做布局断言。
  首版捕获中有一条射线精确穿过顶点 (0,0,0)，参考取 face 2、产品取 face 1；随后改为不穿过
  顶点/边的确定性射线并重采，未放宽容差、未跳过射线。
- 交叉校验（任一不符即捕获失败，全部 true）：初始/移动位姿位型 == S2b 设备位姿捕获；384 条
  S0b 射线的命中/未命中集合 == S2b `geomid == 3`（initial `[91]`、moved `[91, 92]`）；mesh
  命中射线的 distance/normal 位型 == S2b；3.12 `model-312.json` 的 `mesh_vert` f64→f32 == 捕获的
  3.15 网格点 f32 位型；两次完整捕获逐位一致（第二遍 0.239 s）。
- 产物：`fixtures/render-bvh/scene-a/s3/mesh_query.json`（434,588 B，31,331 行，
  sha256 `4cf448c187c746f0f6a0bc7fe43faa7c01a81f8845fca33fe2396dd1c065a753`；f32 位型，
  产品测试只读位型、不启 Python）。
- GPU：RTX 4070 Ti SUPER / 驱动 596.36；warp 1.15.0、mujoco 3.15.0、mujoco_warp 3.15.0、
  numpy 2.5.4；GPU 空窗后独占，捕获显存基线 2869 MiB、进程内 3112 MiB。

## 4. GPU 验收结果（命令与原始输出）

```text
set CUDA_PATH=target\toolchains\cuda-12.8.1; PATH=%CUDA_PATH%\bin;%PATH%
cargo test --locked --features cuda-probe --test render_mesh_bvh -- --ignored --test-threads=1
```

结果：`6 passed / 0 failed`（测试体内 0.78 s；NVRTC 首次编译在首条测试内，模块缓存后复用）。

```text
initial:       compared=436 hits=29 misses=407 max_distance_ulp=0 max_uv_ulp=0 max_normal_angle=4.000e-8
moved:         compared=436 hits=27 misses=409 max_distance_ulp=0 max_uv_ulp=0 max_normal_angle=3.440e-8
no_cull:       compared=436 hits=40 misses=396 max_distance_ulp=0 max_uv_ulp=0 max_normal_angle=4.000e-8
dynamic:       compared=436 hits=28 misses=408 max_distance_ulp=0 max_uv_ulp=0 max_normal_angle=0.000e0
dynamic_moved: compared=436 hits=24 misses=412 max_distance_ulp=0 max_uv_ulp=0 max_normal_angle=3.228e-8
tight_above:   compared=436 hits=29 misses=407 max_distance_ulp=0 max_uv_ulp=0 max_normal_angle=4.000e-8
tight_below:   compared=436 hits=0  misses=436 max_distance_ulp=0 max_uv_ulp=0 max_normal_angle=0.000e0
degenerate:    compared=16  hits=9  misses=7   max_distance_ulp=2 max_uv_ulp=0 max_normal_angle=0.000e0
set_points changed rays: [91, 384, 386, 387, 388, 392, 394, 396, 397, 398, 399, 401, 404, 405,
 406, 407, 410, 412, 414, 417, 418, 420, 422, 423, 425, 426, 427, 430, 431, 433, 434, 435]
two-world query: 436 rays per world, 0 cross-world candidate
rejection surface: empty rays / max_t / pose / geom_id
```

- ① 固定网格射线：436 条在 5 个场景用例上 geomid/face 逐条精确；u/v max ULP **0**（预算 4）；
  normal max **4.0e-8 rad**（预算 1e-4，夹角用 f64 `atan2(|cross|,dot)` 度量，不是放宽容差）；
  distance 另测 max ULP **0**（与 S2 同口径）。
- ② 命中/未命中与背面：`no_cull`（cull=false）40 命中 vs `initial` 29 命中，多出的 11 条是背面
  命中；`back` 定向射线在 cull=true 下走最多 8 次剔除步进后仅 1/4 命中远面，其余 miss，均与
  参考逐条一致；S0b mesh ray 91/92 的命中/未命中与其 distance/normal 位型 == S2b 冻结参考。
- ③ `set_points`+refit：动态顶点更新走 `set_points` + `refit`（不重建树）；变化射线集合
  （hit/face 变化或 distance 差 >4 ULP）与参考**完全一致**（32 条，见上）；`dynamic` 28 命中
  vs `initial` 29 命中、`dynamic_moved` 24 命中 vs `moved` 27 命中。
- ④ 退化三角形：主机单测覆盖零面积/重复索引构建、refit 与宿主候选不 panic；GPU 退化网格用例
  16 条射线 9 命中逐条一致（face 精确、u/v 0 ULP、distance ≤2 ULP），无 panic／无挂起。
- ⑤ 无节点布局断言：测试只通过设备语义输出与「命中三角形是否在宿主候选内」断言，不检查
  节点编号/顺序/内部包围盒布局。

## 5. 门禁与测试

```text
normify gate 2026-10-24-render-bvh-s3
fmt 通过 1209ms
clippy 通过 304ms
clippy-cuda 通过 4981ms
unit 通过 6034ms 测试512过0败
```

- `cargo fmt --check`、`cargo clippy --all-targets --locked -- -D warnings`、
  `cargo clippy --all-targets --locked --features cuda-probe -- -D warnings` 均通过。
- 默认 `cargo test --locked`（无 cuda-probe）不含 GPU 测试（`#![cfg(feature = "cuda-probe")]`）；
  `unit` 门禁 512 过 0 败（含本片 3 个主机测试与 3 个打包校验单测）。
- 新文件行数（≤500）：`mesh.rs` 485、`mesh_query.rs` 371、`cuda/mesh.cu` 383、
  `tests/render_mesh_bvh.rs` 493、`tests/support/render_bvh/mesh.rs` 320、
  `capture_s3_mesh.py` 498、`s3_mesh_capture_support.py` 199、`s3_mesh_replay.py` 146。
- 离线 NVRTC 预检（与产品同 recipe，ctypes 直调本机 nvrtc DLL，不占 GPU）：`mesh.cu` 编译通过、
  PTX 无 f64/f16 等禁用 token、entry `mesh_bvh_ray` 正确。

## 6. 产物哈希（提交 blob / LF 口径）

| 文件 | 行数 | sha256 |
| --- | --- | --- |
| `src/render/bvh/mesh.rs` | 485 | `f63aab10844ef72554dd1eec3b34d34a30cb52757e2f20f46a5c3279e90c4059` |
| `src/render/bvh/mesh_query.rs` | 371 | `2bd3e8bbc04778680203703c2ba91f4710e19d6a6c7708ed1ef83c975b1bb7e5` |
| `src/render/bvh/cuda/mesh.cu` | 383 | `75ae35acc44125e4c4fa18eac7086f68be325167c22b5a3443ca841eef42fd46` |
| `src/render/bvh/mod.rs` | 244 | `5f1f225be9e5fe97cd61c942cc5eb7bccdc4c95d73f9285cdc1f13102011671a` |
| `tests/render_mesh_bvh.rs` | 493 | `ad1edfb480fcd43132a2594a7ba17bb7f2b702261e7f473c8498e5dc1c94cb9d` |
| `tests/support/render_bvh/mesh.rs` | 320 | `0dc1e865ca43ce52a92ec23607bc84473ce97e38b13d2981609973c4c52ed88e` |
| `scripts/authoring/render-bvh/capture_s3_mesh.py` | 498 | `a224d564cba7e157e8358f27dabafd34776891c2b7f1ed3e55e7ecceb9a223e2` |
| `scripts/authoring/render-bvh/s3_mesh_capture_support.py` | 199 | `68f924c45be9012034ef5422cd3e8271f4f444122b6ef9621c8032ab1deda014` |
| `scripts/authoring/render-bvh/s3_mesh_replay.py` | 146 | `d8f5042bead9a3d391cd78e81d15a49e297f7ac265218b887762283522002d2d` |
| `fixtures/render-bvh/scene-a/s3/mesh_query.json` | 31331 | `4cf448c187c746f0f6a0bc7fe43faa7c01a81f8845fca33fe2396dd1c065a753` |

`src/render/bvh/mod.rs` 相对 S2b 提交（`c7b5d7f`）的 188 行增加 mesh 子模块挂载、`vertices_from_f64`
与 `MeshBvhError` 的 Display/Error 实现（避免 `mesh.rs` 超 500 行）；语义未变。

## 7. 验收映射（plan S3 五条）

| plan S3 验收 | 本切片 | 状态 |
| --- | --- | --- |
| ① 固定网格射线：三角形/face 精确；u/v ≤4 ULP；normal ≤1e-4 rad | 5 用例 × 436 条：face/geomid 全等、u/v max ULP 0、normal 4.0e-8 rad（distance 另测 max ULP 0） | ✅ |
| ② 命中/未命中与参考一致，含背面案例 | 5 用例逐条一致；`no_cull` 40 命中含 11 条背面；S0b mesh ray 91/92 与 S2b 位型相等 | ✅ |
| ③ `set_points + refit` 后命中变化与参考一致 | 变化集合 32 条与参考完全相等（不重建树路径） | ✅ |
| ④ 退化三角形不 panic | 主机单测 + 退化网格 GPU 用例 16 条/9 命中逐条一致 | ✅ |
| ⑤ 不做 BVH 节点布局断言 | 只断言语义输出与宿主候选覆盖 | ✅ |

## 8. 未满足项 / 限制

- 上游 Woop 的 f64 兜底分支未翻译：产品内核 f32-only；本夹具上静态重放证明判定不变
  （场景射线 0 次触发；退化网格触发 80 次但判定变化 0），但「分母恰为 0 且 f64 兜底会改变
  判定」的输入不在覆盖范围内，不宣称泛化等价。
- 单 fixture（scene-a 四面体网格 + 436 条射线），无 fuzz/大网格/随机射线集；深树（>128 层）
  栈限制未实测（构建侧深度超限在打包前拒绝）。
- hfield / flex / SDF / splat 的 BVH 与命中不实现；纹理、材质、光照、相机不在本片。
- 参考为 3.15 冻结产物（MJWarp 3.15 捕获、产品 native 3.12 输入）；动态 refit 参考用上游
  默认 `sah` 构造，静态参考用 render context 的 `cubql` 构造——两者语义（最近命中）一致，
  但节点布局不同，按验收明确不做布局断言；退化网格用例的射线集刻意避开布局相关平局
  （精确穿共享顶点/边的输入不在覆盖内）。
- 未跑全量 G01；本片按 fast-loop 口径（fmt/clippy/clippy-cuda/unit + 定向 GPU 测试）。
