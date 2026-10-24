# 渲染/BVH S2a：主机侧 primitive AABB 位等、场景 BVH 构建/重拟合与分组拒绝面

- 任务：`2026-10-24-render-bvh-s2a`（depends_on S1，无 GPU；产品侧测试纯主机）
- 日期：2026-10-24
- 结论：**pass**。7 类 leaf × 8 个位姿共 56 个 AABB 与冻结 MJWarp 参考逐位相等（max ULP 0）；
  分组/重拟合语义与拒绝面按验收覆盖。GPU 射线命中逐条对照属 S2b，本切片不宣称。

## 1. 范围

覆盖：

- primitive geom（plane/sphere/capsule/ellipsoid/cylinder/box）与 mesh/SDF 的 **leaf 包围盒**
  （Mesh/SDF 用上游 `mesh_bounds_size` 的 box 公式；网格三角形 BVH 属 S3）。
- 主机侧场景 BVH：per-world 分组、分箱 SAH 构建、重拟合、宿主语义查询、拒绝面。

不覆盖（显式）：

- CUDA 遍历、射线-primitive 求交、geomid/distance/normal 逐条对照 → S2b。
- mesh 三角形 BVH / mesh 命中语义 → S3；本切片不建、不宣称。
- hfield / 3.15 SDF 的完整语义：当前 scene-a 无 hfield；SDF 只按上游 MESH 分支用 mesh bounds。

## 2. 参考来源（离线 authoring，非产品测试链）

- 捕获脚本：`scripts/authoring/render-bvh/capture_s2a_bounds.py`（372 行，离线专用；产品构建/测试/门禁
  都不运行 Python）。
- 环境（脚本逐项校验，不符即失败退出）：pinned venv Python 3.12.13 / Warp 1.15.0 / MuJoCo 3.15.0 /
  MJWarp 3.15.0 / numpy 2.5.4；上游 `71da24d956378a87a703b6e1442b13aec0c4ac29` clean；
  GPU RTX 4070 Ti SUPER / 驱动 596.36；`scene-315.mjb` 与 `scene.xml` 哈希锁定。
- 位姿：`initial`（构建路径）+ `moved-base`（与 S2 参考相同的确定性位移：平移 0.05/-0.04/0.03、
  绕 z 0.18 rad，`mjw.refit_bvh`）+ 6 个确定性校准位姿（平移 + 归一化四元数 + `tri-hinge` 角），
  共 8 个位姿。
- **关键输入口径**：refit 的 AABB 参考来自 MJWarp **设备侧 f32 geom 位姿**（`d.geom_xpos`/`d.geom_xmat`），
  它与 CPU MuJoCo f64 位姿转 f32 可能差 1 ULP；因此 fixture 记录的是「送进 kernel 的 f32 位姿位型」，
  而不是 CPU 位姿。`initial` 构建路径的输入是 `create_render_context` 内部 CPU `mj_forward` 后
  f64→f32 的位姿，同样按位型记录。
- 交叉校验：与已提交的 S2 捕获 `s2/bvh_initial.npz`、`s2/bvh_moved.npz` 的 lower/upper 位型
  4/4 相等（同一 context 参数与同一确定性位移）。
- 确定性：两次完整捕获（新模型/数据/context）逐位一致；第二次 0.078 s。
- GPU 显存：基线 3536 MiB、进程内 3669 MiB，进程退出后已释放；仅此一次性捕获使用 GPU。

产物：`fixtures/render-bvh/scene-a/s2a/bounds.json`
sha256 `aad9e7e5690b17979da8f2a6c5ebb08dcd5e4937bad1af3598b666c4af7b8454`（27738 B，1353 行）。
fixture 含：model 侧输入（geom_type/geom_dataid/geom_size f32 位型/mesh bounds 位型）、
每个位姿的输入 f32 位姿位型、参考 lower/upper 位型、context 参数、环境、两次捕获一致性。

## 3. AABB 位等结论

参考 kernel：`mujoco_warp/_src/bvh.py::_compute_bvh_bounds`（冻结上游）。NVRTC 编译把 warp 的
`wp::mul(mat33, vec3)` / `wp::dot(vec3, vec3)` 收缩成固定乘加顺序：

```text
3 项和：fma(c2, b2, fma(c0, b0, round(c1*b1)))
2 项和：fma(c0, b0, round(c1*b1))
pos ± z*half：fma(±z, half, pos)
```

该顺序由 8 位姿 × 7 类 geom 的 56 个 leaf AABB 逐位对照钉死；离线复核脚本
`scripts/authoring/render-bvh/verify_s2a_aabb.py` 可重算并生成 `aabb-parity.json`
（sha256 `297e310a0a533e48a632c7ba21597f2983270f8db16e4ddc8912ede8c4e98e14`）。

| 类型 | geom id | 检查数 | 位差 | 最差 ULP |
| --- | --- | --- | --- | --- |
| plane | 0 | 8 | 0 | 0 |
| box | 1 | 8 | 0 | 0 |
| sphere | 2 | 8 | 0 | 0 |
| mesh（mesh bounds box） | 3 | 8 | 0 | 0 |
| capsule | 4 | 8 | 0 | 0 |
| cylinder | 5 | 8 | 0 | 0 |
| ellipsoid | 6 | 8 | 0 | 0 |
| **合计** | — | **56** | **0** | **0** |

模型侧输入对照：3.12 `model-312.json` 的 `geom_size`（f64→f32）与 mesh_vert 计算的 mesh bounds
逐位等于 3.15 fixture 中对应输入位型（S0a 渲染相关数组 f32 位等前提在 AABB 输入层再次成立）。

### 宿主 fma 的实现口径

产品主机侧不用 `f32::mul_add`：本目标 debug 构建会落到不保证单次舍入的回退实现，与 release 位型
可能差 1 ULP（实测会破坏位等与调试/发布一致性）。实现改为 f64 计算
`(a as f64 * b as f64 + c as f64) as f32`（f32*f32 在 f64 中精确），在全部 56 个体上与冻结参考
逐位一致，且 debug/release 一致。AABB 只作为加速结构，不进入产品物理数值链；该口径与限制
在 receipt 的 `not_claimed`/`unmet` 中登记。

## 4. 实现

| 文件 | 行数 | 内容 |
| --- | --- | --- |
| `src/render/bvh/mod.rs` | 184 | `Aabb`（union/包含/slab 相交）、`BvhError` 与拒绝面类型 |
| `src/render/bvh/aabb.rs` | 400 | 7 类 leaf AABB 公式（冻结 f32 收缩顺序）、mesh bounds、输入校验 |
| `src/render/bvh/builder.rs` | 440 | 校验 + per-world 分箱 SAH 构建（12 箱）、group root、节点数组 |
| `src/render/bvh/refit.rs` | 123 | 重拟合 leaf 与内部节点（子先父后，索引正序一遍） |
| `src/render/bvh/query.rs` | 111 | 宿主语义查询：从 group root 出发的射线-AABB 候选 leaf |
| `src/render/mod.rs` | 242（原 240） | 只加 `pub mod bvh;` 与文档说明 |

语义约定：

- 每个 world 一个独立 group（上游 `group == worldid`），各自建树；group root 只指向本组树根，
  因此从某 world 的 root 出发不会跨 world。
- 节点先子后父追加，refit 按索引正序一遍完成；构建/refit 都是确定性的。
- 节点布局是本实现细节，验收只断言语义不变量（每个 leaf 恰好可达一次、内部节点包围盒等于
  子节点并集、root 等于组内 leaf 并集），不对节点编号/顺序做断言（plan S2 验收第 6 条）。

## 5. 构建 / 重拟合语义测试

`tests/render_scene_bvh_host.rs`（472 行，纯主机，不启 Python/GPU）：

1. 模型输入 f32 位型对照（geom_size、mesh bounds）。
2. 56/56 leaf AABB 逐位对照（8 位姿 × 7 类）。
3. `build → refit`：初始与 moved 两套 leaf 位等；group root 包围盒等于组内 leaf 并集；
   每个 leaf 恰好可达一次；初始 vs moved 变化 leaf 集合 = `{1,2,3,4,5,6}`，floor(geom 0) 静态。
4. 两个 world 复制同一场景（MuJoCo `nworld` 语义：各 world 几何同位）后，32 条射线 × 2 world
   查询：候选 leaf 的 group 全部等于查询 world；两 world 的候选 geom 集合一致；命中 64/64。
5. 拒绝面：空场景、world 数 0、输入长度不一致、group 越界、空 world 分组、AABB 非法
   （下界>上界）、refit 长度不符、geom 类型 hfield、非有限位姿、空 mesh bounds。

## 6. 门禁与测试

```text
normify gate 2026-10-24-render-bvh-s2a
clippy 通过 354ms
clippy-cuda 通过 8099ms
fmt 通过 1262ms
unit 通过 16060ms 测试506过0败
```

- 定向测试：`cargo test --locked --test render_scene_bvh_host` → 5 passed / 0 failed。
- 模块内单测：`cargo test --locked --lib render::bvh` → 16 passed / 0 failed（Aabb、公式、
  容量溢出、构建、refit、查询、拒绝面）。
- `cargo fmt --check`、`cargo clippy --all-targets --locked -- -D warnings`、
  `cargo clippy --all-targets --locked --features cuda-probe -- -D warnings` 均通过。

## 7. 产物哈希（提交 blob / LF 口径）

| 文件 | 行数 | sha256 |
| --- | --- | --- |
| `src/render/bvh/mod.rs` | 184 | `1581aa460e579de5e25301238943db93f32ea4450322a69662b2d62c40f981d1` |
| `src/render/bvh/aabb.rs` | 400 | `d16307daf64c3a253bd21743272a8084ae448f1c17b201ae7448d7e98c35c537` |
| `src/render/bvh/builder.rs` | 440 | `d8cadd95b9407d5dfd07f67f08ec953dbc2e01688abbdcf395892126484a9435` |
| `src/render/bvh/refit.rs` | 123 | `4bfc0ddb33d06204a7674a273932228483a457d37822f9684c94f2db14411d54` |
| `src/render/bvh/query.rs` | 111 | `42363f668890ea2d9381315606c4a8ad51fccb7ba12eee0f0f9fc30da20b0809` |
| `src/render/mod.rs` | 242 | `0eaf74a668ea148b89ef25d91ddbe13da23fce180f8fd69fdab95d26ae3190b6` |
| `tests/render_scene_bvh_host.rs` | 472 | `8150a63e7664a985efba4bd720ae87406b77ff14676eb66a435402ce0ce0f617` |
| `capture_s2a_bounds.py` | 372 | `17eb48044439ed6c9f5f337a983ce910450b63ccef3331a7e0794ecc237672dd` |
| `verify_s2a_aabb.py` | 307 | `3eeea5be4a6c853ee5add3b16a1ff88825cb8a08773ee2fa551d226461471eb5` |
| `fixtures/.../s2a/bounds.json` | 1353 | `aad9e7e5690b17979da8f2a6c5ebb08dcd5e4937bad1af3598b666c4af7b8454` |
| `aabb-parity.json` | 216 | `297e310a0a533e48a632c7ba21597f2983270f8db16e4ddc8912ede8c4e98e14` |

## 8. 验收映射（plan S2 六条）

| plan S2 验收 | 本切片 | 状态 |
| --- | --- | --- |
| ① 固定射线 geomid 精确 / distance ≤4 ULP / normal ≤1e-4 rad | 不在本切片（S2b） | 待 S2b |
| ② 构建前 AABB 逐元素位等 | §3，56/56 位等 | ✅ 本切片 |
| ③ 移动 body refit 后同射线集变化一致 | AABB 层已位等（§5-3）；射线层 S2b | AABB 层 ✅，射线层待 S2b |
| ④ per-world group root，world groups 不跨命中 | §5-4，宿主语义查询 | ✅ 本切片（宿主层；GPU 命中层 S2b） |
| ⑤ 空场景/越界 group 被拒绝 | §5-5 | ✅ 本切片 |
| ⑥ 不对 BVH 节点布局做断言 | §5 只做语义不变量 | ✅ 本切片 |

## 9. 未满足项 / 限制

- 无 GPU 遍历与命中，因此不宣称 S2 六条全部通过（①③ 的射线层属 S2b）。
- mesh 三角形 BVH 与 mesh 命中的 distance/normal 语义留 S3；本切片 mesh 只到包围盒。
- `scene-a` 的 floor 是静态恒等位姿，参考没有覆盖「旋转 plane」的 AABB 路径；该路径由公式的
  收缩顺序与 box 共用实现覆盖，仅单测与公式一致性支撑，无 GPU 位型参考。
- 产品 host fma 用 f64 产品+求和路径复刻（见 §3），不是平台无关的 IEEE fma；对任意输入
  可能出现 ≤1 ULP 与 CUDA `fma.rn` 的差异；本切片验收为冻结 fixture 的 56 个个体。
- AABB 参考输入是 MJWarp 设备侧 f32 位姿（与 CPU 位姿可能差 1 ULP），产品集成时位姿来源
  必须与 S2b 的 GPU 运动学路径一致，不能改用 CPU f64 位姿反推。
- 未做 fuzz/任意 MJB 的性质测试；未覆盖 hfield/SDF 完整语义。
