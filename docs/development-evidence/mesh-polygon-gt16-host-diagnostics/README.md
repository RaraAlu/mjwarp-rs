# mesh >16 顶点多边形宿主超界诊断（方案 C）

任务：`2026-11-03-mesh-polygon-gt16-host-diagnostics`（Arch 2026-11-03 登记，owner Backend）
性质：**只加宿主侧显式诊断**。不改设备代码/`.cu`、不改 ABI、不改判据/容差/参考/夹具、不开 GPU 窗口、不冻结。
实现时 HEAD：`dd75bf2b`。上游评估依据：`docs/development-evidence/mesh-polygon-gt16-gap/`（同卡依赖）。

## 0. 结论摘要

1. 产品碰撞 plan 入口现在会在**宿主侧**用已有 mesh 数组计算模型级 `npolygonmax`/`nmeshdegmax`，与设备
   mesh 多接触固定容量（`face≤20`、`degree≤10`）比较；超界返回结构化错误
   `CollisionGpuError::MeshMulticontactCapacity`，载荷含**维度 / 实际值 / 上限值 / 拒绝路径**，`Display` 打全字段。
   ⇒ 产品路径不再把「设备逐 feature 静默回退单接触」的近似交给下游。
2. 拒绝只放在产品组装入口：`run_product_collision` → `prepare_inputs` → `inputs_from_snapshot`
   （`src/runtime/collision_gpu/inputs_from_snapshot.rs`）。`validate_collision_inputs` 与
   `CollisionGpuProgram::run` **零改动**：直连 `run` 的探针路径仍按设备真实行为记录（见 §6 边界）。
3. 宿主上限常量镜像 `.cu` 宏，并用模块内 `include_str!` 断言守护：A 方案（face/degree 32/32）改宏后
   宿主要么同步、要么测试红，不会静默失配。
4. 测试（模块内 `#[cfg(test)]`，不碰 `tests/`）：6 条，覆盖 fan-cone-24 双触发、face=21 单维、
   degree=24 单维、face=20/degree=10 边界放行、无 sidecar 放行、`.cu` 宏一致性。
5. 不宣称 U7/U8 通过、不宣称容量缺口已清、不宣称与上游行为一致；本片零数值行为变化（只把静默近似
   改成显式拒绝）。

## 1. 设备侧事实（为什么要在宿主拒绝）

设备 mesh 多接触是**固定容量**，超界时对应助手返回 0，`g5_multicontact` 回退到 EPA 单接触
（`multicontact.cu:194-198`；调用方 `contact.cu:233` 采纳 `n=1`）：

| 限位 | 位置 | 当前值 | 超界行为 |
| --- | --- | --- | --- |
| `G5_MC_MESH_FACE_MAX`（单 polygon 顶点数） | `src/physics/collision/cuda/multicontact_geom.cu:385` | 20 | `g5_mc_mesh_face`（`multicontact_mesh.cu:180`）返回 0 → 回退单接触 |
| `G5_MC_MESH_DEG_MAX`（单顶点相邻 polygon 数） | `src/physics/collision/cuda/multicontact_geom.cu:384` | 10 | `g5_mc_mesh_normals`（`:120`）/`g5_mc_mesh_edge_normals`（`:148`）返回 0 → 回退单接触 |

上游是模型级动态容量（pinned `collision_convex.py:1279-1289`），设备固定 20/10 属语义差异——本片不改设备，
只让产品入口把「输入会触发设备回退」这件事显式化。

## 2. 产品调用链（唯一组装入口）

```
产品适配（repo 外）
  → runtime::collision_gpu::product_entry::run_product_collision   (src/runtime/collision_gpu/product_entry.rs)
      → prepare_inputs                                            (同文件；nworld=1 口径 + 组装)
      → inputs_from_snapshot                                      (src/runtime/collision_gpu/inputs_from_snapshot.rs)
          → abi::validate_inputs                                  (既有形状/长度/值域校验，零改动)
          → types::mesh_multicontact_capacity_violations          (新增容量门；src/runtime/collision_gpu/types.rs)
          → CollisionGpuError::MeshMulticontactCapacity           (超界显式拒绝；src/runtime/collision_gpu/error.rs)
```

错误传播：`prepare_inputs` 在设备选择/探测**之前**返回 `Err`，`run_product_collision` 原样上抛，
**不降级 Host、不静默继续**。`CollisionGpuError::InvalidArgument` 的既有调用面不动。

## 3. 实现点

| 文件 | 改动 | 行数 |
| --- | --- | --- |
| `src/runtime/collision_gpu/types.rs` | `MeshCapacityDimension`、`MeshCapacityViolation`、常量 `MESH_MULTICONTACT_FACE_MAX=20`/`MESH_MULTICONTACT_DEGREE_MAX=10`/`MESH_MULTICONTACT_REJECT_PATH`、扫描函数 `mesh_multicontact_capacity_violations(polyvertnum, polymapnum)`；`CollisionGpuMeta` 仅 derive `Default`（无字段变化） | 187 |
| `src/runtime/collision_gpu/error.rs` | 新增 `MeshMulticontactCapacity { violations, reject_path }` 变体 + `Display` 全字段 | 72 |
| `src/runtime/collision_gpu/inputs_from_snapshot.rs` | `validate_inputs` 后接容量门（超界返回富错误）；模块内 6 条测试 | 498 |
| `src/runtime/collision_gpu.rs` | 仅扩展 `pub use types::{…}` 再导出（+4 行） | 470 |

数值计算路径：`npolygonmax = max(mesh_polyvertnum)`（逐 polygon 顶点数）、
`nmeshdegmax = max(mesh_polymapnum)`（逐 vertex 相邻 polygon 数）；9 项 mesh sidecar 全空时两者为空，
行为与改动前一致（不拒绝）。

## 4. 测试覆盖（模块内，零 GPU）

`cargo test --locked --lib collision_gpu::inputs_from_snapshot` → **6 passed / 0 failed**：

1. `product_entry_rejects_fan_cone_24_on_both_dimensions`：真实 fixture `fan-cone-24.obj`（25 顶点 / 25 面，
   face=24 且 degree=24）→ 产品入口返回富错误，两条 violation 同时报告（face 24>20、degree 24>10），
   `Display` 含维度/实际值/上限/拒绝路径关键字。
2. `product_entry_rejects_face_only`：21 顶点 polygon → 只报 face（actual=21）。
3. `product_entry_rejects_degree_only`：24 三角扇（扇心 degree=24、最大面 3）→ 只报 degree。
4. `product_entry_accepts_face_20_degree_10_boundary`：face=20、degree=10 的边界网格 → `Ok`（放行）。
5. `product_entry_accepts_without_mesh_sidecar`：9 项 mesh_poly 全空 → `Ok`（产品行为与改动前一致）。
6. `host_limits_match_device_macros`：`include_str!` 解析 `multicontact_geom.cu`，断言宿主常量 ==
   `G5_MC_MESH_FACE_MAX`/`G5_MC_MESH_DEG_MAX`（A 方案改宏后防失配）。

## 5. 绕过入口核查（acceptance ⑤）

- `src/` 内 `CollisionGpuInputs { … }` 的构造只有 1 处非测试点：`inputs_from_snapshot` 本身；
  其余出现在 `validate_*_tests.rs` 测试模块。
- `inputs_from_snapshot(` 的 `src/` 调用点只有 `product_entry.rs`（`prepare_inputs`）；
  产品入口链上没有第二条组装路径。
- 直连 `CollisionGpuProgram::run` 的是探针/测试路径（GPU ignored），不属于产品 plan；本片未改它。

## 6. 未覆盖 / 边界（acceptance ④）

- **探针语义不变**：直连 `CollisionGpuProgram::run`（如 `tests/g10_p5_branch_closure_cuda.rs` 的 limits
  record-only 用例）仍按设备真实行为记录（U7/U8 设备 count 仍可能是 1）；**该路径不计入本片验收面**，
  本片也不改它的测试预期。
- **设备侧零改动**：`src/render/kernel/cuda` 与全部 `*.cu` 零 diff；`validate_collision_inputs`/`run` 零改动。
- **数值行为零变化**：合法输入（face≤20 且 degree≤10 或非 mesh）行为与改动前逐字节一致（同一 `run`/内核/
  ABI/自旋路径）；只有超界输入从「静默近似」变为「显式 `Err`」。
- **不重采参考**：本片不碰 fixtures/参考；上游参考与既有窗口证据不受影响（A 方案才需要设备侧重冻结）。
- 若后续做 A 方案（设备 face/degree 32/32），需同步改宿主常量（测试会先红提醒）；那属于另一张卡。

## 7. 不宣称

不宣称：U7/U8 通过、容量缺口被清除、与上游行为一致、精度验收通过；不宣称 U7/U8 覆盖；不宣称设备行为
或一般 mesh 覆盖；本片未跑 GPU、未冻结、未开窗口；无任何数值/性能结论。

## 8. 复现

```
cargo test --locked --lib collision_gpu::inputs_from_snapshot        # 6 passed / 0 failed
cargo test --locked --lib runtime::collision_gpu                    # 模块回归（48 passed / 0 failed / 1 ignored）
cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings
cargo test --locked --features cuda-probe --no-run                  # EXIT=0
normify gate 2026-11-03-mesh-polygon-gt16-host-diagnostics          # 权威门禁回执在 .normify/runs/<task>/
```
