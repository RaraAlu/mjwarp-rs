# G08@P5 基本几何碰撞切片：plane-sphere / sphere-sphere 窄相与接触写出

任务：`2026-10-11-roadmap-g08-p5`。日期：2026-10-23。执行：Backend（批次 3）。
矩阵：G08 基本几何碰撞（`上游几何对与接触规则` / `深度、法线、接触数量` / P5）。

## 1. 阶段语义与细化口径

旧计划 G08@P5 只有粗切片（上游几何对与接触规则；验收「深度、法线、接触数量」）。
本片接 G07 宽相候选（依赖 `g07-p5` verified），交付计划 API
`primitive_narrowphase` 的**两类基础几何对**：

- `plane_sphere`：`dist = dot(sphere_pos - plane_pos, normal) - radius`、
  `pos = sphere_pos - normal*(radius + 0.5*dist)`；平面法线取旋转矩阵第三列。
- `sphere_sphere`：中心距减半径和；`pos` 在连心线上；法线为单位连心向量（零距离退化）。
- `make_frame`：法线为第一行，`orthogonals` 正交基（b 取 y/z 轴投影归一化，c = a×b）。
- 接触写出对齐 `write_contact`/`contact_params`：`margin/gap` 检测（`detected =
  dist < margin+gap`）、`active` 与 `contact_type` 位、`solmix`/`priority` 材质混合、
  `condim` 取大、`adhesion` 降维（dist≥margin 时 dim=1）、`MJ_MINMU` 摩擦下限、
  `solref/solimp` 混合与 `solreffriction=[0,0]`、`geomcollisionid=0`。

## 2. 实现

| 文件 | 内容 |
| --- | --- |
| `src/physics/collision/narrowphase.rs` | `PrimitiveModel`/`PrimitiveState`、`plane_sphere`/`sphere_sphere`/`make_frame`、`material_params`、`write_contact`、`primitive_narrowphase`、`GeomContact` |
| `src/physics/collision/narrowphase/tests.rs` | G07 候选一致、冻结上游接触逐字段、原生 f64 `mj_collision`、公式/拒绝面 |
| `src/physics/collision.rs` / `src/physics/mod.rs` | 模块挂接与公开导出（含 `GEOM_SPHERE`） |

## 3. 参考来源与生成方式

| 项 | 值 |
| --- | --- |
| 模型源 | `fixtures/narrowphase/model.xml`（sha256 `686a4ff5…`；平面 + 3 球，ngeom=4/nworld=2；`sphere_a` condim4/solmix2/margin.01、`sphere_b` condim6/priority1/margin.02、`sphere_c` solmix0 远处） |
| 用例 | 2 世界 qpos 冻结；宽相 NXN + ALL 过滤候选 5（世界 0:3、世界 1:2） |
| 上游参考 | MJWarp `71da24d956…`，Warp 1.15.0：`kinematics` → `nxn_broadphase` → `collision()` 接触（顺序由原子计数决定，按 `(worldid,geom)` 规范化） |
| 原生参考 | MuJoCo 3.15.0（pinned venv）f64 `mj_kinematics` + `mj_collision`（一级判定） |
| 生成脚本 | `scripts/authoring/author-g08-narrowphase.ps1` + `.py`（离线；产品测试不启动 Python） |
| 产物 | `fixtures/narrowphase/cases.json`、本目录 `authoring.json` |

## 4. 对照结果（本次实测）

| 判定 | 结果 |
| --- | --- |
| G07 候选一致 | 产品 `nxn_broadphase` 候选 5 条与冻结上游逐条一致（world/pair/pairid 集合） |
| 接触数量 | 产品 5 条 = 上游 `nacon` 5 = 原生 `ncon` 之和 5（世界 0:3、世界 1:2） |
| 深度/法线 vs 冻结上游 | `dist`/`pos`/`frame`/`includemargin`/`friction`/`solref`/`solimp`/`adhesion` max abs **7.45e-09** ≤ 1e-6；`dim`/`geom`/`type`/`worldid` 逐位一致 |
| 原生 f64 一级判定 | `dist`/`pos`/`frame`/`includemargin` max abs **5.96e-08** ≤ 1e-5；`dim` 逐位一致 |
| 材质混合 | 平面-A（solmix 1/2 等权 → condim 4）、平面-B（priority 1 → condim 6/摩擦取 B）、A-B（condim max 6）与上游一致；摩擦均 ≥ `MJ_MINMU`(1e-5) |

复跑：`cargo test --locked --lib narrowphase` → **3 过 0 败**；全量 lib 通过；
`cargo fmt --check` 与 `cargo clippy --all-targets -- -D warnings` 通过。

## 5. 验收指标映射（NP01–NP04）

- **NP01**：新增路径全 f32（位姿、几何公式、材质混合、接触结构）。
- **NP02**：记录冻结上游提交、`nxn_broadphase`/`collision` 函数名、模型 sha256、
  geometry/contact 参数布局与常量（`MJ_MINVAL`/`MJ_MINMU`）、环境。
- **NP03**：两级判定——原生 f64 `mj_collision` 先行（5.96e-08），再与 MJWarp f32 二级对照（7.45e-09）。
- **NP04**：**不适用/未测**——不发射内核、无批量耗时与显存记录。

## 6. 未验收边界

- 未实现其余几何对（capsule/box/ellipsoid/cylinder/mesh/hfield/SDF）与 convex/CCD/multiccd。
- 未实现 flex 接触、显式 `<pair>` 覆盖（pair_dim/pair_solref/pair_friction 等）与
  collision sensor 接触类型位（本片 sensor 恒 0）。
- 未实现 EFC 行地址、约束求解、`naconmax` 溢出告警、世界批量参数、GPU 内核与图执行。
- 不宣称完整 G08 或 P5 验收通过；NP04 未测。

## 7. 复现

```powershell
scripts/authoring/author-g08-narrowphase.ps1
cargo test --locked --lib narrowphase -- --nocapture
```
