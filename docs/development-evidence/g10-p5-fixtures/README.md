# g10-p5 G0b：双版本 MJB、octree parity 与 GJK/EPA 直达参考补采

任务：`2026-10-25-g10-p5-fixtures`（G0b）
冻结上游：`71da24d956378a87a703b6e1442b13aec0c4ac29`（MJWarp 3.15）
拍板：1A 只做物理、2A `worlds=2` 严格验收、3A 本批次 authoring 生成 MJB、5A GJK/EPA 先独立交付。
边界：只写 `scripts/authoring/g10-p5/`、`fixtures/g10-p5/`、证据目录；未改产品代码/测试/夹具/`Cargo.*`，
未修改 `fixtures/mjwarp-complex-physics/` 既有 XML。

## 1. 结论

- **双版本 MJB 与 octree parity：两模型全部字段位级全等**（`all_equal=true`）：
  `hfield-box`（noct 0=0）、`sdf-cylinder`（noct 63,497=63,497），
  `oct_aabb`/`oct_coeff` **f64 位型逐字节全等**、`oct_depth`/`oct_child`/`mesh_octadr`/`mesh_octnum`、
  `hfield_nrow/ncol/adr`、`hfield_data`(f32)/`hfield_size`(f64)、`opt_sdf`、`geom_*` 全等。
  → **SDF/hfield 数值验收可走位级**（不需要「统一 3.15 oct_* 夹具」的退路）。
- **既有参考审计**：5 个可复用（`box-box-multiccd`、`convex-mesh`、`degenerate-mesh`、`hfield-box`、
  `sdf-cylinder`）+ `sphere-sphere` 仅作 G08 回归；真实缺口 9 项已列（见 §3）。
- **nworld=2 spike：产品链路可用**（broadphase 按 world 外层循环、geom 按 `world*ngeom` 索引、
  narrowphase 校验 `worldid < nworld`、接触带 worldid）→ 2A 无需降级 2B；
  G1 需为 `hfield_data`/`oct_*` 补 world 维上传，G5 需对齐 `naconmax` 与接触写入顺序。
- **补采参考 10 个 case 全部成功**（worlds=2、两次捕获、contact 17 字段）：
  **通过判据（worldid 分组 + 排序后位级比较）全部 True**；原始顺序观察项记录：
  `sdf-box` 原始顺序两次捕获不位等（atomic 写入非确定）、5 个多接触 case 原始顺序非按 world 分组。
- GPU 短窗口：开窗 4,156 MiB → 窗口结束 4,156 MiB（回基线、无本任务计算进程）；
  kernel 缓存命中的最终运行 15:22:42–15:22:48。

## 2. 双版本 MJB 与 octree parity（先测后定）

产物：
- `fixtures/g10-p5/mjb-312/`：3.12 C API 离线编译（`dump_collision_fields_312.cpp`），同版本重载校验；
- `fixtures/g10-p5/mjb-315/`：pinned 3.15 `mujoco.mj_saveModel`，同版本重载校验；
- 3.15 不能加载 3.12 MJB（实测 `different number of sizes`）→ 两侧各自生成、各自校验。

| 模型 | 3.12 MJB (B / sha256 前 8) | 3.15 MJB (B / sha256 前 8) | noct | octree f64 全等 | hfield 全等 | 结构全等 |
| --- | --- | --- | --- | --- | --- | --- |
| hfield-box | 5,267 / `39e74291` | 5,259 / `cf60c77e` | 0 = 0 | ✅ | ✅ | ✅ |
| sdf-cylinder | 9,405,930 / `9dfd077e` | 9,405,922 / `f1df72cf` | 63,497 = 63,497 | ✅ | ✅ | ✅ |

- 逐字段来源：`fixtures/g10-p5/octree-parity.json`（每字段 dtype/count/bytes/sha256 312/315/equal）。
- 两层 parity：① `oct_aabb`/`oct_coeff` f64 位型逐字段（octree 原始数据）；② `hfield_data` f32 位型 +
  `hfield_size` f64 位型（上传口径一次转换后）。**全等** → G1 解析 3.12 MJB 时按「f64→f32 一次转换」
  对齐 mjw `put_model`，数值验收走位级。
- dump 中间字节：`target/g10-p5/dump-312|315/`（可重跑生成，不进 git）；fixtures 只留 MJB + parity。

## 3. 既有参考审计（只审计不重采）

`fixtures/g10-p5/existing-reference-audit.json`（d04 run-5，worlds=2）：

| case | count | 每 world | geom 对 | 覆盖能力 | 结论 |
| --- | --- | --- | --- | --- | --- |
| `box-box-multiccd` | 10 | 5/5 | (0,1) | box support/EPA/`multicontact`/`is_discrete` | **G2 可复用** |
| `convex-mesh` | 2 | 1/1 | (1,0) | mesh 支撑（graph/穷举）+ box-mesh 多接触 | **G2 可复用** |
| `degenerate-mesh` | 2 | 1/1 | (0,1) | mesh-mesh 退化 / EPA 退化 | **G2 可复用** |
| `hfield-box` | 8 | 4/4 | (0,1) | hfield prism-CCD + 四接触后处理 | **G4 可复用** |
| `sdf-cylinder` | 80 | 40/40 | (0,1) | SDF narrowphase/Halton/gradient_descent | **G3 可复用** |
| `sphere-sphere` | 2 | 1/1 | (0,1) | G08 primitive 解析（非 GJK） | 仅回归 |
| `plane-sphere-*` / `flex-*` | 4/8 | — | — | G08/G11 | 非 G2 范围 |

真实缺口（由本片 10 个新 case 补齐）：`sphere-box` 浅/深穿透、`capsule-box`、`cylinder-box`、
`ellipsoid-box`、`hfield-sphere`、`hfield-capsule`、`sdf-box`、非退化 `mesh-mesh`、`box-box-margin`
（margin≠0 非离散路径）。

## 4. nworld=2 spike（只读结论）

- 产品 `src/physics/collision/`：`filter.rs`/`sap.rs`/`nxn.rs` 均按 `world in 0..state.nworld`
  外层循环、`world * model.ngeom` 索引；`narrowphase.rs` 校验 `worldid >= state.nworld` 拒绝、
  `index_base = worldid * model.ngeom`、接触写 `worldid`。
- 结论：多世界状态打包对碰撞链路**可用**（2A 不降级）；缺口具体化：
  1. `hfield_data`/`oct_*` 的 world 维上传（G1，参数批 world 维机制已有但新字段未接）；
  2. 接触写入顺序：参考中 `sdf-box` 原始顺序两次捕获不位等（atomic 写入非确定），
     多接触 case 原始顺序非按 world 分组 → G5 验收用排序比较并记录观察项；
  3. `naconmax` 维度（G5 对齐参考滑动窗口语义）。
- 未发现需要降级 2B 的阻塞项，未报 Arch 阻塞。

## 5. 补采参考（10 个 case，worlds=2）

`fixtures/g10-p5/references/`（每 case 一个 JSON，含 qpos/qvel 位型、contact 17 字段、
两次捕获确定性、原始顺序观察、provenance）。判定：**通过判据 = worldid 分组 + 每 world 内
`(dist,pos)` 稳定排序后位级比较**；原始顺序是否相等作观察项记录。

| case | count | 每 world | 覆盖能力 | 排序后位等 | 原始顺序位等 | 原始顺序按 world 分组 |
| --- | --- | --- | --- | --- | --- | --- |
| sphere-box-shallow | 2 | 1/1 | GJK 球 shrink + `_inflate` 浅穿透 | ✅ | ✅ | ✅ |
| sphere-box-deep | 2 | 1/1 | 深穿透 → EPA | ✅ | ✅ | ✅ |
| capsule-box | 2 | 1/1 | capsule shrink/support | ✅ | ✅ | ✅ |
| cylinder-box | 8 | 4/4 | cylinder support + 多接触限制 | ✅ | ✅ | ❌（world 交错） |
| ellipsoid-box | 2 | 1/1 | ellipsoid support/normalize | ✅ | ✅ | ✅ |
| hfield-sphere | 8 | 4/4 | hfield prism-CCD + 球 shrink 组合 | ✅ | ✅ | ❌（world 交错） |
| hfield-capsule | 7 | 4/3 | hfield 与第二类型配对 | ✅ | ✅ | ❌（world 交错） |
| sdf-box | 80 | 40/40 | SDF narrowphase + Halton initpoints | ✅ | **❌（atomic 非确定）** | ❌ |
| mesh-mesh | 2 | 1/1 | mesh graph/穷举 + 非退化 | ✅ | ✅ | ✅ |
| box-box-margin | 10 | 5/5 | margin≠0 非 `is_discrete` 路径 + multiccd | ✅ | ✅ | ❌（world 交错） |

**原始顺序观察项（强制记录，不得静默忽略）**：
- `sdf-box`：两次捕获原始顺序不位等，原因是 SDF narrowphase 每个 initpoint 由 `write_contact`
  的 atomic 计数顺序写入（40 接触/world），排序后位等 → G3/G5 判据必须排序比较；
- `cylinder-box`/`hfield-sphere`/`hfield-capsule`/`box-box-margin`：两次捕获原始顺序位等，
  但顺序为 world 交错（非 world 分组）→ broadphase/multiccd 写入顺序效应；
- 其余 5 case 原始顺序按 world 分组且位等。

## 6. 产物与脚本

| 文件 | 说明 |
| --- | --- |
| `scripts/authoring/g10-p5/dump_collision_fields_312.cpp`（188 行） | 3.12 C API 编译 MJB + dump 原始字节（f64/f32/i32/i64 小端） |
| `scripts/authoring/g10-p5/run_g0b_312.ps1`（75 行） | MSVC 构建（`/utf-8`、`/Fo` 指向 target）+ 双模型编译 dump |
| `scripts/authoring/g10-p5/dump_fields_315.py`（141 行） | pinned 3.15 `mj_saveModel` + 同字段 dump |
| `scripts/authoring/g10-p5/octree_parity.py`（153 行） | 两层 parity（f64 位型 + f32 一次转换）→ `octree-parity.json` |
| `scripts/authoring/g10-p5/audit_existing_references.py`（131 行） | 既有参考审计 + 缺口清单 |
| `scripts/authoring/g10-p5/author_g0b_models.py`（212 行） | 生成 10 个补采模型 XML（可复现） |
| `scripts/authoring/g10-p5/capture_g0b_references.py`（282 行） | GPU 短窗口捕获（worlds=2、两次位等、contact 17 字段、顺序观察） |
| `fixtures/g10-p5/models/` | 10 个新 XML + `index.json`（含 offset） |
| `fixtures/g10-p5/references/` | 10 个参考 JSON + `index.json` |
| `fixtures/g10-p5/mjb-312|mjb-315/` | 双版本 MJB（G1 输入/参考） |
| `fixtures/g10-p5/octree-parity.json` | 两层 parity 结论（每字段 sha） |
| `fixtures/g10-p5/existing-reference-audit.json` | 既有参考审计 + 缺口 |
| `docs/development-evidence/g10-p5-fixtures/gpu-run.log` | GPU 窗口起止、设备、显存、捕获输出 |

provenance（references `index.json` 与逐 case）：源 XML sha、模型 index sha、脚本 sha、
mujoco/warp/mujoco_warp 版本、GPU、`authoring_only`、`reads_product_tree=false`、
`product_python_dependency=false`。

## 7. 未满足项

- 本片只做夹具/MJB/参考补采；**未实现产品侧 hfield/SDF 碰撞**（G1–G6）。
- `sdf-box` 的接触原始顺序非确定（atomic 写入）——验收必须排序比较，G5 需对齐写入顺序或保持排序判据。
- nworld=2 spike 是**只读代码结论**，未写产品代码验证上传链路（G1/G5 落地验证）。
- 10 个补采 case 只覆盖真实缺口，不是全组合矩阵；更多 geom 组合/位姿未采。
- hfield 与 sphere/capsule 的接触顺序与四接触选择语义在 G4 实现前未与产品对齐。
- GPU 性能未测（归 G6 perf 片）。
- Linux/T4 未覆盖（用户 2026-10-24 裁决终止 Linux 任务）。

## 8. 复现

```powershell
# 纯 CPU（3.12 C++ 需 MSVC + MuJoCo 3.12.0 package）
& scripts/authoring/g10-p5/run_g0b_312.ps1
target\mjwarp-parallel\venv\Scripts\python.exe -B scripts/authoring/g10-p5/dump_fields_315.py
target\mjwarp-parallel\venv\Scripts\python.exe -B scripts/authoring/g10-p5/octree_parity.py
target\mjwarp-parallel\venv\Scripts\python.exe -B scripts/authoring/g10-p5/audit_existing_references.py
# 生成模型（CPU）
target\mjwarp-parallel\venv\Scripts\python.exe -B scripts/authoring/g10-p5/author_g0b_models.py
# GPU 短窗口（pinned venv；warp 自带 CUDA 12.9 NVRTC）
target\mjwarp-parallel\venv\Scripts\python.exe -B scripts/authoring/g10-p5/capture_g0b_references.py
```

## 更正（2026-10-25，normify fix 2026-10-25-g10-p5-fixtures）

以本节为准；前文凡与之冲突的表述作废。

1. **PRIMITIVE 分类更正**：本批补采的 10 项并非全部 GJK/EPA 直达参考。按上游 collision_driver.MJ_COLLISION_TABLE 与实际模型 flags 复核：
   - GJK/EPA **CONVEX** 参考仅 **5 项**：cylinder-box、ellipsoid-box、mesh-mesh、convex-mesh、degenerate-mesh；
   - 另 5 项（sphere-box-shallow、sphere-box-deep、capsule-box、ox-box-margin、hfield-*）与 d04 ox-box-multiccd 中部分实际走 **PRIMITIVE** 路径，仅供其他切片复用，**不是 GJK/EPA 直达参考**。
   - 判定依据：(SPHERE,BOX)/(CAPSULE,BOX) 在表中即 PRIMITIVE；ox-box 模型带 
ativeccd=disable（mjDSBL_NATIVECCD）时驱动把 (BOX,BOX) 覆盖为 PRIMITIVE。
2. **ox-box-margin 表述更正**：该 case **不是 multiccd**。模型 <flag nativeccd="disable" multiccd="disable"/>，5 接触/world 来自 **margin≠0 的非 is_discrete 多接触路径**（is_discrete = _discrete_geoms && margins==0）；multiccd 的真实覆盖在既有 d04 ox-box-multiccd（该模型 multiccd=enable，本轮亦为 PRIMITIVE 表）。
3. **hfield 重复接触点性质**：hfield-sphere 8 行中只有 6 个不同 (dist,pos)（2 行重复），hfield-capsule 7 行中只有 3 个不同点（world1 3 行同点）→ **不得当"4 个不同接触点"使用**；比较/排序必须按全字段（含 geomcollisionid）稳定排序。
4. **原始顺序范围**：	wo_captures_raw_bitwise_identical / grouped_by_world_in_raw_order 等观察项**只对本次 CUDA 运行/本设备/本后端成立**（QA 用 warp CPU 后端重采时 cylinder-box/hfield-sphere/ox-box-margin 顺序不同，sdf-box raw 位等性也不同）→ **不得把"原始顺序位等"写成固定判据或验收条件**；判据仍是 worldid 分组 + 全字段（含 geomcollisionid）稳定排序后位级比较。