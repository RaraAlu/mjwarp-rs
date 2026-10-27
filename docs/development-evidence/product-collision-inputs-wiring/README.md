# 产品碰撞输入接线 S1：独立 data 快照探针（宿主侧，无 GPU）

- 任务：`2026-10-27-product-collision-inputs-s1-data-snapshot`（owner Backend；HEAD 起点 `1f478513`）
- 父任务（伞，保持 backlog）：`2026-10-26-product-collision-inputs-wiring`
- 上游 pinned revision：`71da24d956378a87a703b6e1442b13aec0c4ac29`
- 本片范围：**只新增**独立 collision-inputs 探针与 Rust 侧拥有式快照；不做产品接线（S3 才接）。
- 结果一句话：新探针给出 `CollisionGpuInputs` 32 项字段的模型快照与 mjData `geom_xpos/geom_xmat` 帧快照；
  4 项 host 定向测试全过（32 项逐元素/oracle、2 组 qpos 帧位型、octree 与体积桥交叉、原始 ABI 负例）；
  fmt/clippy/clippy-cuda 通过、unit **994 过 / 0 败 / 376 ignored**；**18 个 pinned 源与 HEAD 逐字节一致（零改动）**。

## 1. 契约边界（硬条件，逐条）

1. **只新增文件**：新增 `include/mjwarp_collision_inputs_probe.h`、`native/model_probe_collision_inputs.cpp`、
   `src/model/collision_inputs.rs`、`src/io/collision_inputs.rs`、`tests/g10_p5_collision_inputs_snapshot.rs`
   与 `tests/support/collision_inputs_*.rs`；只改 `native/model_probe_sparse.cpp`（+1 行 include）与
   `build.rs`（只加 `rerun-if-changed`）。
2. **禁改 18 个 pinned 源**（`fixtures/precision/g01-inputs.json` 清单）：`git status` 与本片改动集不含其中任何
   一个；任务 machine check `unchanged` 会逐字节确认。
3. **既有 ABI 不动**：`include/mjwarp_native_probe.h` 的 `mjwarp_native_*` 声明、`mjwarp_collision_args`
   （504 B，`src/runtime/collision_gpu/abi.rs` 的静态布局注释与 `abi_tests`）、`DEVICE_SOURCES`=28
   （`multicontact_tests::device_sources_place_multicontact_and_boxbox_in_order`）、产品入口 6
   （`abi_tests`「entries 6→6，0 新增入口」）全部未触碰。本片新 ABI 使用**独立 header 与独立符号名**。
4. **不做新 feature**：沿用既有 `native-model-probe` feature；不新增依赖。
5. **无 GPU / 不开窗**：本片全部验证为宿主侧（可信 DLL + 冻结 MJB），不建 CUDA 上下文、不使用 GPU。
6. **状态输入只有 qpos**：`geom_xpos`/`geom_xmat` 的权威来源 = 调用方传 qpos 后执行一次 `mj_kinematics` 的
   mjData；mocap / act / ctrl **不在默认支持面**（mocap 模型另立任务评审 state 读取）。
7. 本片**不做产品接线**，不宣称 host↔device 等价、不宣称精度或验收结论；不写「与 forward 全流程等价」。

## 2. 原生 ABI（`include/mjwarp_collision_inputs_probe.h`）

独立 schema；字段号与 `src/runtime/collision_gpu/types.rs::CollisionGpuInputs` 的 32 项 1:1。
本 schema 的 kind 编号：**1 = int32，2 = double(mjtNum)，4 = float32**。

| 结构 | 大小/对齐 | 内容 |
| --- | --- | --- |
| `mjwarp_collision_inputs_field_info` | 16 B / 8 | `kind`、`reserved`、`count`（标量数） |
| `mjwarp_collision_inputs_info` | 96 B / 8 | `schema`、`reserved` + 11 个计数：`nq ngeom nmesh nmeshvert nmeshgraph nmeshpoly nmeshpolyvert nmeshpolymap nhfield nhfielddata noct` |
| `mjwarp_collision_inputs_data` | 不透明句柄 | `owner`、`mjData*`、`mj_kinematics`/`mj_deleteData` 符号 |

入口（错误码沿用现有约定：**0 = ok，8 = invalid**）：

- `mjwarp_native_collision_inputs_info(owner, info)`：写出 11 个非负计数。
- `mjwarp_native_collision_inputs_field_info(owner, id, info)`：32 个 ID 的 kind/count 描述（不分配）。
- `mjwarp_native_copy_collision_inputs_field(owner, id, info, target)`：复制**模型侧**字段（ID 2..31）；
  ID 0/1 是 mjData 字段，本入口返回 8，由 data 快照服务。所有 kind/count/来源/目标先 preflight，再写第一个字节。
- `mjwarp_native_collision_inputs_create(owner, out)`：`library_symbol` 从 owner 的 DLL 取
  `mj_makeData/mj_deleteData/mj_kinematics`，创建**一次** mjData 并缓存（只重跑运动学，不重复 makeData）。
- `mjwarp_native_collision_inputs_set_qpos(owner, data, qpos, nq)`：长度必须等于模型 `nq`；写 `d->qpos` 后
  执行 `mj_kinematics`。
- `mjwarp_native_collision_inputs_copy_xpos_xmat(owner, data, xpos, xmat, ngeom)`：把 `d->geom_xpos`（ngeom×3）
  与 `d->geom_xmat`（ngeom×9）转成 float32 写出；两个目标缓冲先 preflight。
- `mjwarp_native_collision_inputs_destroy(owner, data)`：防御性校验 `data->owner == owner`（外部句柄返回 8
  且不删除）；**必须在 owner close 之前 destroy**（Rust 侧用借用类型强制该顺序）。

32 项字段表（kind：1 int32 / 2 double / 4 float32；unit 即计数基数，count = unit × width）：

| id | 字段 | kind | unit | width | 来源 |
| --- | --- | --- | --- | --- | --- |
| 0 | `geom_xpos` | 4 | ngeom | 3 | mjData |
| 1 | `geom_xmat` | 4 | ngeom | 9 | mjData |
| 2 | `geom_size` | 2 | ngeom | 3 | mjModel |
| 3 | `geom_margin` | 2 | ngeom | 1 | mjModel |
| 4 | `geom_gap` | 2 | ngeom | 1 | mjModel |
| 5 | `geom_rbound` | 2 | ngeom | 1 | mjModel |
| 6 | `geom_aabb` | 2 | ngeom | 6 | mjModel |
| 7 | `geom_type` | 1 | ngeom | 1 | mjModel |
| 8 | `geom_dataid` | 1 | ngeom | 1 | mjModel |
| 9 | `mesh_vertadr` | 1 | nmesh | 1 | mjModel |
| 10 | `mesh_vertnum` | 1 | nmesh | 1 | mjModel |
| 11 | `mesh_graphadr` | 1 | nmesh | 1 | mjModel |
| 12 | `mesh_octadr` | 1 | nmesh | 1 | mjModel |
| 13 | `mesh_vert` | 4 | nmeshvert | 3 | mjModel |
| 14 | `mesh_graph` | 1 | nmeshgraph | 1 | mjModel |
| 15 | `mesh_polynum` | 1 | nmesh | 1 | mjModel |
| 16 | `mesh_polyadr` | 1 | nmesh | 1 | mjModel |
| 17 | `mesh_polynormal` | 2 | nmeshpoly | 3 | mjModel |
| 18 | `mesh_polyvertadr` | 1 | nmeshpoly | 1 | mjModel |
| 19 | `mesh_polyvertnum` | 1 | nmeshpoly | 1 | mjModel |
| 20 | `mesh_polyvert` | 1 | nmeshpolyvert | 1 | mjModel |
| 21 | `mesh_polymapadr` | 1 | nmeshvert | 1 | mjModel |
| 22 | `mesh_polymapnum` | 1 | nmeshvert | 1 | mjModel |
| 23 | `mesh_polymap` | 1 | nmeshpolymap | 1 | mjModel |
| 24 | `hfield_size` | 2 | nhfield | 4 | mjModel |
| 25 | `hfield_nrow` | 1 | nhfield | 1 | mjModel |
| 26 | `hfield_ncol` | 1 | nhfield | 1 | mjModel |
| 27 | `hfield_adr` | 1 | nhfield | 1 | mjModel |
| 28 | `hfield_data` | 4 | nhfielddata | 1 | mjModel |
| 29 | `oct_aabb` | 2 | noct | 6 | mjModel |
| 30 | `oct_child` | 1 | noct | 8 | mjModel |
| 31 | `oct_coeff` | 2 | noct | 8 | mjModel |

注：`mjModel` 3.12 没有 `nmeshpolycount` 字段；本 schema 用真实存在的计数覆盖全部 unit 需求
（`nmeshgraph` 与 `nmeshpolymap` 已显式列出，分别服务 `mesh_graph` 与 `mesh_polymap`）。

## 3. Rust 侧

- `src/model/collision_inputs.rs`：`CollisionInputsFieldKind`（Index/Num/Float，code 1/2/4）、
  `CollisionInputsUnit`、`CollisionInputsFieldSource`、`CollisionInputsFieldSpec`、32 项
  `COLLISION_INPUTS_FIELDS`、`CollisionInputsCounts::validate/expected`（负计数与溢出先拒绝）。
- `src/io/collision_inputs.rs`：
  - `NativeCollisionInputsSnapshot::capture`：查 info → 校验 11 计数 → 逐 ID 查 field info、校验
    kind/count/来源 → 拥有式 `Vec` 快照（30 项模型字段：`Index/Num/Float`）。
  - `into_inputs()`：`Num` 走**既有唯一转换入口** `io::convert_f64_to_f32_into`（拒绝非有限/f32 溢出，
    错误映射到字段名），`Float` 原样保留，`Index` 原样；产出宿主侧 `CollisionInputsModelInput`。
  - `CollisionInputsData<'a>`：借用 `NativeModelProbe` 的 mjData 句柄（RAII，Drop 调用 destroy），
    `set_qpos` 校验长度，`capture_frame()` 得到 `geom_xpos`/`geom_xmat` 的 f32 快照（ngeom×3 / ngeom×9）。
  - `io::NativeModelProbe` 新增 `collision_inputs_snapshot()` / `collision_inputs_data()` 两个方法。
- 新 cpp 从 `native/model_probe_sparse.cpp` include（放在 `model_probe_collision.cpp` 之后，同 TU 复用
  owner/library_symbol）；**新 cpp 不再次 include `model_probe.cpp`**（避免同 TU 重复定义）。

## 4. 数据源与 oracle

三份冻结 MJB（只读，未新造；均为仓库既有 fixture）覆盖 mesh / hfield / octree：

| oracle 名称 | MJB | bytes | sha256 | 关键计数 |
| --- | --- | --- | --- | --- |
| hfield-box | `fixtures/g10-p5/mjb-312/hfield-box.mjb` | 5267 | `39e74291327e71cfe9c9ab42f7f15f781a2cf26b6050fe498146d2a0f1985082` | nq 7, ngeom 2, nhfield 1, nhfielddata 9 |
| scene-312 | `fixtures/render-bvh/scene-a/scene-312.mjb` | 101328 | `c6f5f9f17118067ad39fb2ea6be4f1c274e4d932b553bbe9a4e59b464c56aad1` | nq 8, ngeom 7, nmesh 1, nmeshpoly 4 |
| sdf-cylinder | `fixtures/g10-p5/mjb-312/sdf-cylinder.mjb` | 9405930 | `9dfd077ed4ab40877ea9380ae57e317200da70c1645c603d22b5896d5aefec43` | nq 7, ngeom 2, nmesh 1, noct 63497 |

- 生成脚本（authoring-only，入库）：`scripts/generate_collision_inputs_oracle.py`，环境为 pinned
  Python 3.12.13 + **mujoco 3.12.0（python wheel）** + numpy 2.5.4；产出
  `tests/support/collision_inputs_oracle.json`（12545 B / sha256 `b5a3fd29…`）。
- oracle 内容：11 计数、30 项模型字段的原生值、2 组 qpos 的 `geom_xpos`/`geom_xmat` f32 位型；
  sdf 模型的 oct_* 因体量过大只存计数，逐元素改由**独立 G01 体积桥**（同一 pinned 模型的另一条读取路径）
  交叉核对。
- **f64 期望值以 u64 位型存储**（`f64_bits`），测试用 `f64::from_bits` 还原：本仓库 serde_json 未启用
  `float_roundtrip`，浮点文本解析可能丢 1 ULP（实测 scene-312 `geom_size[10]` 差 1 ULP），位型存储消除该误差。
- f32 期望以 `f32_bits` 存储（numpy `astype(float32)` 同舍入），测试按 `to_bits()` 逐元素比较。

## 5. 验证（全部宿主侧，无 GPU）

| 项 | 命令 | 结果 |
| --- | --- | --- |
| 定向测试 | `cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features native-model-probe --test g10_p5_collision_inputs_snapshot -- --ignored --test-threads=1` | **4 passed / 0 failed**：`snapshot_matches_pinned_mjmodel_oracle_for_all_32_fields`、`data_snapshot_matches_pinned_mjdata_oracle_and_reuses_mjdata`、`snapshot_outlives_owner_and_octree_matches_volume_bridge`、`rejects_invalid_native_arguments_without_partial_writes` |
| oracle 一致性 | 同上 | 32 项计数与模型字段逐元素一致；宿主收窄后 f32 位型与 numpy astype 一致（0 差异）；2 组 qpos 的 `geom_xpos/geom_xmat` 位型一致；mjData 复用后同一 qpos 复现同一帧 |
| octree 交叉 | 同上 | sdf-cylinder：`noct=63497`，`oct_aabb/oct_child/oct_coeff` 与 G01 体积桥逐元素相等 |
| 既有 native_g01 期望 | `cargo ... test --locked --features native-model-probe --test native_g01 -- --ignored --exact converts_complete_native_g01_fields_after_owner_drop` | **1 passed / 0 failed**（78 项 G01 字段对冻结参考不变） |
| fmt / clippy | `cargo fmt --all -- --check`；`cargo clippy --locked --all-targets --features native-model-probe -- -D warnings` | exit 0 / exit 0 |
| C++ `/W4 /WX` | build.rs `warnings_into_errors(true)`，cc 1.6 对 MSVC 映射 `/W4` + `/WX` | 新 TU 编译通过、**零警告**（`logs/build-w4.log`） |
| 门禁（最终源） | `normify gate 2026-10-27-product-collision-inputs-s1-data-snapshot` | exit 0：fmt 通过、clippy 通过、clippy-cuda 通过、**unit 994 过 / 0 败 / 376 ignored**（与本片前基线一致，新测试在 `native-model-probe` 下不进默认 unit） |

定向测试日志：`logs/targeted-test.log`；native_g01 日志：`logs/native-g01.log`；fmt/clippy：`logs/fmt-clippy.log`；
oracle 生成：`logs/oracle-generate.log`；门禁：`logs/gate.log`（run 目录
`.normify/runs/2026-10-27-product-collision-inputs-s1-data-snapshot/`）。

## 6. 边界与不宣称项

- 本片**不做产品接线**（S3 才接）；不宣称 host↔device 等价、不宣称精度或验收结论、不宣称与上游等价。
- `geom_xpos/geom_xmat` 权威来源仅指「调用方 qpos → 一次 `mj_kinematics` 后的 mjData」；**不代表 forward
  全流程等价**（未跑 position/velocity/constraint 等阶段）。
- 状态输入只有 qpos；mocap/act/ctrl 不在默认支持面。
- `sdf-cylinder` 的 oct_* 只与同 pinned 模型的 G01 体积桥交叉核对（不是第二份独立实现的外部参考）；
  oct_* 的 f32 收窄位型在定向测试中只做有限/非空校验。
- **历史 pinned 快照漂移说明**：`fixtures/precision/g01-inputs.json` 记录的 pinned sha 属于更早的
  product_revision；其中 `include/mjwarp_native_probe.h`、`native/model_probe.cpp` 在该快照之后
  已被既有 render/volume 片更新（非本片改动，本片对这 18 个路径零改动，machine check `unchanged` 以 HEAD 为准）。
- 本 README/代码均 ≤500 行；`receipt.json` 为机器可读证据（可超 500 行）。

## 7. 交付文件

| 文件 | 行/字节 | 说明 |
| --- | --- | --- |
| `include/mjwarp_collision_inputs_probe.h` | 69 行 | 独立 schema + data 快照入口 |
| `native/model_probe_collision_inputs.cpp` | 236 行 | 模型字段复制 + mjData 句柄（同 TU，复用 owner/library_symbol） |
| `native/model_probe_sparse.cpp` | +1 行 | 只加 include |
| `build.rs` | +2 行 | 只加两个 `rerun-if-changed`（新 cpp / 新 header），不加 `.file()`/feature |
| `src/model/collision_inputs.rs` | 211 行 | 32 项字段规范 + 11 计数 |
| `src/io/collision_inputs.rs` | 474 行 | 拥有式快照 + mjData 句柄 + 唯一 f64→f32 入口 |
| `src/model/mod.rs` / `src/io/mod.rs` | +6 / +32 行 | 模块声明与 probe 方法 |
| `tests/g10_p5_collision_inputs_snapshot.rs` | 234 行 | 4 项 host 定向测试（≤500） |
| `tests/support/collision_inputs_oracle_support.rs` | 105 行 | oracle 解析/位型工具 |
| `tests/support/collision_inputs_preflight.rs` | 313 行 | 原始 ABI 负例预检 |
| `tests/support/collision_inputs_oracle.json` | 12545 B | pinned oracle（机器可读） |
| `scripts/generate_collision_inputs_oracle.py` | 159 行 | authoring-only oracle 生成脚本 |
