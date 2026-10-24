# 渲染/BVH S2b：CUDA 场景 BVH 遍历与语义射线命中逐条对照

- 任务：`2026-10-24-render-bvh-s2b`（depends_on S2a 已 verified；GPU 窗口由 Arch 排定）
- 日期：2026-10-24
- 结论：**pass**（覆盖范围内）。383 条 initial 射线与 382 条 moved 射线逐条对照：geomid 全等、
  distance 全部 0 ULP（f32 位型相同）、normal 最大夹角 4.6e-8 rad ≤ 1e-4 rad；refit 前后 geomid
  变化集合与参考一致；双 world 0 跨命中；拒绝面覆盖。mesh 命中（initial 1 条、moved 2 条含 1 条
  由 cylinder 变为 mesh）按 plan 明确延后 S3，不含在通过口径内。

## 1. 范围

覆盖：

- `scene.cu`：手写 CUDA 场景 BVH 遍历（栈式、slab 剪枝）+ 6 类 primitive 求交
  （plane/sphere/capsule/ellipsoid/cylinder/box），语义翻译自冻结上游 `ray.py`
  （`_ray_map` / `_ray_quad` / `safe_div` / `ray_plane` / `ray_sphere` / `ray_capsule` /
  `ray_ellipsoid` / `ray_cylinder` / `ray_box` / `_ray_bvh` 的最小距离选择）。
- `src/render/bvh/cuda/mod.rs`：宿主打包 S2a 的 `SceneBvh`、leaf geom 位姿与射线集，编译并发射
  内核，回读 dist/geomid/normal；打包前做长度、world、geom 类型、栈深度校验。
- `tests/render_scene_bvh_gpu.rs` + `tests/support/render_bvh/`：GPU 验收测试（默认 ignore，
  显式 `--ignored --test-threads=1` 运行）。

不覆盖（显式，plan 归 S3）：

- mesh 三角形求交与 mesh 命中的 distance/normal（`scene.cu` 对 geom_type 7/8 返回 miss）。
- hfield 求交（拒绝面显式报错）。
- ray.py 的剔除语义参数化（bodyexclude / geomgroup / flg_static / 不可见材质）：本切片实现的是
  冻结捕获所用默认调用（bodyexclude=-1、全部可见、flg_static=true、geomgroup 全 -1）。

## 2. 接线方式（既有 launcher 模式，不新造发射层）

产品 GPU 内核路径是 `src/runtime/transfer/kernel` 的 `SynchronousKernel`（NVRTC 编译 +
`fmad=true`、`fast_math=false`、`--std=c++17`、compute_70，PTX 精度门禁仅允许 f32），不是
`src/runtime/launcher`（统一发射器的 M1/M2 产物/缓存层；其 NVRTC 路径目前只被冻结 PTX 探针使用）。
`SceneBvhQuery` 复用 `SynchronousKernel::compile` + 既有四缓冲 `launch`（元数据 i32 / 参数 f32 /
射线 f32 / 输出 f32 + 4 个 u32 维度，256 线程，grid 覆盖射线数），未改动 runtime 文件。

设备侧是**真 fma.rn.f32**（NVRTC fmad=true 编译）；S2a 主机侧因 `f32::mul_add` debug 回退不保证
单次舍入而使用的 f64 复刻路径**不用于** device 路径。W15 `include/warp-headers/bvh.h` 已读：
其 `bvh_query_ray/bvh_query_next` 消费 Warp 内部 BVH 布局/描述符，与本产品自建节点布局不兼容；
本切片按 plan 第 6 条（不对节点布局断言）使用自己的遍历，命中语义与布局无关。

## 3. 参考来源修正：必须用设备侧 f32 位姿

S0b/S2 的命中由 MJWarp `_ray_bvh` 用**设备侧 f32 geom 位姿**（`d.geom_xpos`/`d.geom_xmat`）计算；
S2 npz 里存的 `geom_xpos` 是 CPU MuJoCo 位姿。最初用 CPU 位姿喂产品内核时，cylinder 类命中出现
**124 ULP** 距离偏差（initial ray 92），超出 4 ULP 预算。修正方式：新增离线 authoring 捕获
`scripts/authoring/render-bvh/capture_s2b_query.py`，在同一 pinned 环境下记录设备侧位姿位型与
参考命中，并做 7/7 交叉校验：

```text
initial_geomid_vs_s0b / initial_dist_bits_vs_s0b / initial_normal_bits_vs_s0b   true
moved_geomid_vs_s2 / moved_dist_bits_vs_s2 / moved_normal_bits_vs_s2            true
initial_geomid_vs_s2                                                            true
```

两次独立捕获逐位一致（第二遍 0.216 s）；环境/版本/上游/夹具哈希/GPU 不符即失败退出；产物
`fixtures/render-bvh/scene-a/s2/reference_hits.json`（f32 位型，产品测试不启 Python）。
GPU：RTX 4070 Ti SUPER / 驱动 596.36；显存基线 3434 MiB、进程内 3692 MiB，跑完回桌面基线；
捕获总耗时 15.9 s（含首次 NVRTC 编译）。

## 4. GPU 验收结果（命令与原始输出）

```text
set CUDA_PATH=target\toolchains\cuda-12.8.1; PATH=%CUDA_PATH%\bin;%PATH%
cargo test --locked --features cuda-probe --test render_scene_bvh_gpu -- --ignored --test-threads=1
```

结果：`3 passed / 0 failed`（测试体内 0.53 s；含 3 次 NVRTC 编译的整条命令 5.17 s）。

```text
initial: compared=383 hits=157 misses=226 skipped_mesh=1 max_ulp=0 max_normal_angle=4.592e-8
moved:   compared=382 hits=155 misses=227 skipped_mesh=2 max_ulp=0 max_normal_angle=2.872e-8
refit changed rays: [99, 109, 116, 117, 155]
two-world query: 384 rays per world, 0 cross-world candidate
rejection surface: empty rays / world out of range / length mismatch / hfield rejected
```

- ① 射线：initial 383/384 条比较（跳过 mesh 命中 ray 91）；moved 382/384（跳过 mesh ray 91、92）。
  参考 initial 158 命中含 1 条 mesh → 对照范围内 157 primitive 命中 + 226 未命中；
  moved 157 命中含 2 条 mesh → 对照范围内 155 primitive 命中 + 227 未命中。
  geomid 精确、distance **max ULP 0**、normal **max 4.6e-8 rad**（夹角用 f64 `atan2(|cross|,dot)`
  度量；f32 dot+acos 在 1e-7 rad 量级会放大到 1e-3，不能反映真实角度——这是度量方式，不是放宽容差）。
- ③ refit：moved 位姿重拟合后，同射线集 geomid 变化集合与参考一致
  `[99,109,116,117,155]`；参考全部 6 条 geomid 变化中 ray 92 变为 mesh 命中（S3 延后），其余 5 条
  在产品与参考中逐条一致。
- ④ 双 world：复制同一场景、world 1 的 geom id 偏移 100，各自独立 group root；world 0 结果与
  单 world 结果逐条一致且 geom id 全部 <100，world 1 命中 id 恰为 world 0 命中 +100、distance
  位型相同，384×2 次查询 0 跨 world。
- ⑤ 拒绝面：空射线集、world 越界、leaf 几何长度不符、hfield 类型均被拒绝；空场景/越界 group 在
  S2a 构建层已覆盖。
- ⑥ 无节点布局断言：测试只通过 `SceneBvhQuery` 的语义输出（geomid/distance/normal）与分组结果
  断言，不检查节点编号/顺序/内部包围盒布局。

## 5. 门禁与测试

```text
normify gate 2026-10-24-render-bvh-s2b
fmt 通过 1260ms
clippy 通过 354ms
clippy-cuda 通过 5033ms
unit 通过 16856ms 测试506过0败
```

- `cargo fmt --check`、`cargo clippy --all-targets --locked -- -D warnings`、
  `cargo clippy --all-targets --locked --features cuda-probe -- -D warnings` 均通过。
- 默认 `cargo test --locked`（无 cuda-probe）不含 GPU 测试（`#![cfg(feature = "cuda-probe")]`），
  S2a 主机测试照常运行。

## 6. 产物哈希（提交 blob / LF 口径）

| 文件 | 行数 | sha256 |
| --- | --- | --- |
| `src/render/bvh/cuda/scene.cu` | 462 | `8dad4d61235a8686d8b43fecef450535816c5a9423f15d256be4aab9a714d56d` |
| `src/render/bvh/cuda/mod.rs` | 283 | `a947d0e3851f6c74bd7de21948c901cb160cc5599486274bc0db68154e4a4a88` |
| `src/render/bvh/mod.rs` | 188 | `cc847264e10469920d07a84041d000160ad07a474f6397911283d64bd5936b73` |
| `tests/render_scene_bvh_gpu.rs` | 336 | `b40b71d2f83c60d813cf11fcd41c913460ca341b843623fc2f26c35944e92829` |
| `tests/support/render_bvh/mod.rs` | 251 | `dcbc11df9360191147018adb57a031322e5ede475f147a75166e0cbd340847e1` |
| `scripts/authoring/render-bvh/capture_s2b_query.py` | 368 | `2de6f3f59b7c270ea8e2eba74e939af86fbf09ab4e9f690f922847b840f2df11` |
| `fixtures/render-bvh/scene-a/s2/reference_hits.json` | 6453 | `609ffc527e8216654cb7b6576f998659026d0e5324429c31c7a67f058c225f84` |

`src/render/bvh/mod.rs` 相对 S2a 提交（`f9adf14`）只增加 `pub mod cuda` 与条件 re-export（+4 行）；
S2a receipt 里的该文件哈希指向 S2a 提交，属于历史快照，不改写。

## 7. 验收映射（plan S2 六条，与 S2a 合并口径）

| plan S2 验收 | 本切片 | 状态 |
| --- | --- | --- |
| ① 固定射线 geomid 精确 / distance ≤4 ULP / normal ≤1e-4 rad | 383+382 条逐条对照，max ULP 0、normal 4.6e-8 rad；mesh 命中 ray 延后 S3 | ✅（primitive 范围） |
| ② 构建前 AABB 逐元素位等 | S2a 已验收（56/56，max ULP 0） | ✅（S2a） |
| ③ 移动 body refit 后同射线集变化一致 | geomid 变化集合一致（5/6 条可对照；ray 92 变 mesh 延后 S3），distance 逐条 0 ULP | ✅（primitive 范围） |
| ④ per-world group root，world groups 不跨命中 | world 0/1 各 384 条查询，0 跨；id 偏移映射与 distance 位型一致 | ✅ |
| ⑤ 空场景/越界 group 被拒绝 | S2b 拒绝面 + S2a 构建拒绝面 | ✅ |
| ⑥ 不对 BVH 节点布局做断言 | 只用语义命中/分组结果断言 | ✅ |

## 8. 未满足项 / 限制

- mesh 三角形求交与 mesh 命中语义留 S3：initial ray 91（mesh 命中）、moved ray 91/92 不计入通过
  口径；因此 ①③ 的“全部 384 条”表述限定为 primitive 命中 + 未命中。
- 单 fixture（scene-a × 384 条射线），无 fuzz/随机射线集；plane 旋转 AABB/命中路径无独立 GPU 参考。
- 内核只实现冻结捕获的默认剔除语义；bodyexclude/geomgroup/flg_static/不可见材质未参数化。
- hfield 未实现（显式拒绝）；SDF 仅按 MESH 分支返回 miss。
- 固定 128 层遍历栈；构建深度超限的输入在打包前拒绝（本 fixture 远低于上限）。
- AABB 用参考设备位姿在 S2b 内重建，可能与 S2a capture 的 initial CPU 位姿 AABB 差 ≤1 ULP；
  S2a 的位等验收口径不变（那是 CPU 位姿输入层的对照）。
- 参考是 3.15 冻结产物；产品运行时不依赖 Python/NVRTC 之外的作者链（NVRTC 由 cuda-probe 提供）。
