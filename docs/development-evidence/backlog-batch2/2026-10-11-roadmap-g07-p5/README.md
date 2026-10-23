# G07@P5 宽相碰撞切片：NXN 与 SAP 候选生成

任务：`2026-10-11-roadmap-g07-p5`。日期：2026-10-23。执行：Backend（批次 2）。
矩阵：G07 宽相碰撞（`NXN、两类SAP、过滤` / `候选完整性与容量边界` / P5）。

## 1. 阶段语义与细化口径

旧计划 G07@P5 只有粗切片（NXN、两类 SAP、过滤；验收「候选完整性与容量
边界」）。按当前代码布局，本片交付计划 API `nxn_broadphase`（U007）与
`sap_broadphase`（U008）的**宿主侧候选生成**：

- 输入：冻结模型字段（`geom_type`、`geom_bodyid`、世界批量 `geom_dataid`、
  AABB center/size、`geom_rbound`、`geom_margin`、`geom_gap`）、上游
  `put_model` 预计算的过滤后 geom 对表（`nxn_geom_pair_filtered` +
  `nxn_pairid_filtered`）与全量 `nxn_pairid` 上三角、冻结每世界
  `geom_xpos`/`geom_xmat`，以及 `BroadphaseFilter` 位模式与 `naconmax` 容量。
- 输出：候选 `pair`（按 geom 类型排序）、`[contact_pairid, collision_pairid]`、
  `worldid` 与 `ncollision`。`ncollision` 统计全部通过候选（可超容量），
  写出截断到容量（对齐 `_add_geom_pair` 的原子计数）。
- 语义：平面 `rbound == 0` 只走 plane filter；非平面按 SPHERE/AABB/OBB 位
  逐项预筛（未置位则不检查 → 全部通过）；NXN 用过滤后对表，SAP 用全量
  上三角并跳过 `pairid[0] < -1` 且 `pairid[1] < 0` 的排除对；collision
  sensor 对（`pairid[1] >= 0`）绕过过滤强制包含；世界批量 `geom_dataid`
  缺失 mesh（`dataid < 0`）两条路径都跳过；SAP 方向为上游固定随机方向
  `normalize(0.5935, 0.7790, 0.1235)`（f32），平面投影半径取 `MJ_MAXVAL`，
  扫描范围对齐 `sap_binary_search`（`>` 边界）与 `sap_range` 的 `limit`。

## 2. 实现

| 文件 | 内容 |
| --- | --- |
| `src/physics/collision.rs` | 公开类型、`BroadphaseFilter`/`BroadphaseType`、`GeomPair`、`CandidateWriter`、`upper_tri_index`、输入校验、两个入口 |
| `src/physics/collision/filter.rs` | `_plane_filter`/`_sphere_filter`/`_aabb_filter`/`_obb_filter`/`_broadphase_filter` 的 f32 逐分支转写 |
| `src/physics/collision/nxn.rs` | NXN 候选生成（对表遍历、缺失 mesh、强制包含、容量截断） |
| `src/physics/collision/sap.rs` | SAP 投影/排序/扫描（`first_greater` 对齐上游二分边界） |
| `src/physics/collision/tests.rs` | 冻结上游逐案、NXN/SAP 一致性、原生 f64 一级判定、校验面 |

## 3. 参考来源与生成方式

| 项 | 值 |
| --- | --- |
| 模型源 | `fixtures/broadphase/mixed.xml`（7 geom/4 keyframe，含同体 geom 与 collision sensor 强制对、父子排除对）、`filters.xml`（平面 + 两胶囊/4 keyframe）、`margin.xml`（球-球边距）、`mesh.xml`（平面 + 四面体 mesh，用于世界批量 dataid 缺失） |
| 模型变体 | 10 个：4 个 XML + 6 个边距变体（margin 0/.011/.00999 组合） |
| 用例 | 47 个：模式 NXN 34 / SAP_TILE 7 / SAP_SEGMENTED 6；过滤 ALL 31 / PLANE 4 / PLANE+SPHERE 4 / PLANE+AABB 4 / PLANE+OBB 4；2 个容量截断（`naconmax=1`）；4 个多世界（`nworld=2`） |
| 原生参考 | MuJoCo 3.15.0（pinned venv）逐世界 `mj_kinematics`+`mj_collision`（f64，NP03 一级） |
| 上游参考 | MJWarp `71da24d956…`，Warp 1.15.0，`nxn_broadphase`/`sap_broadphase`（二级） |
| 生成脚本 | `scripts/authoring/author-g07-broadphase.ps1` + `.py`（离线；产品测试不启动 Python） |
| 产物 | `fixtures/broadphase/broadphase-cases.json`、本目录 `authoring.json` |

## 4. 对照结果（本次实测）

| 判定 | 结果 |
| --- | --- |
| 产品 vs 冻结上游（47 用例） | `ncollision` 全部相等；候选 `(worldid, pair, pairid)` 多集合规范化后**逐案相等**（上游写出顺序由原子计数决定，非确定性） |
| 容量边界（2 用例，`naconmax=1`） | `ncollision=4`（不截断计数）；写出 1 条且必属全容量候选集合；三条数组长度同步为 1 |
| NXN vs SAP 一致（4 keyframe × 3 模式） | 候选集合逐 keyframe 相等；`filters-near` 的 NXN 与两类 SAP 相等；mesh 缺失的 NXN/SAP_SEGMENTED 相等 |
| 产品 vs 原生 f64 一级判定 | 27 例含原生对照；原生接触对 **27 条全部包含在候选集中**（0 缺失） |
| 边距边界（6 用例） | 候选计数与原生 `mj_collision` 接触计数**逐案相等**（0/1/1/0/0/1） |
| 缺失 mesh（2 用例，世界批量 dataid） | 世界 0（`dataid=-1`）不产生候选、世界 1 产生 `[0,1]`；`ncollision=1`，与上游一致 |
| 平面/过滤分位（20 用例） | `PLANE` 位缺失时非平面对全部通过；`PLANE+SPHERE`/`PLANE+AABB`/`PLANE+OBB` 各自逐案候选与上游相等 |

复跑：`cargo test --locked --lib collision` → **5 过 0 败**；全量 `cargo test --locked --lib`
→ **322 过 0 败**；`cargo fmt --check` 通过；`cargo clippy --all-targets -- -D warnings` 通过。

## 5. 验收指标映射（NP01–NP04）

- **NP01**：新增路径全 f32（模型字段、位姿、过滤标量、投影、容量计数），无 f64 中间量。
- **NP02**：记录冻结上游提交、`nxn_broadphase`/`sap_broadphase` 函数名、四个模型 XML 的 sha256、环境与用例形状。
- **NP03**：两级判定——原生 f64 `mj_kinematics`+`mj_collision` 先做一级判定（接触对包含性 27/27、边距计数 6/6），再与 MJWarp f32 参考做逐案候选集合相等比较。
- **NP04**：**不适用/未测**——不发射内核、无批量耗时与显存记录。

## 6. 未验收边界

- 未实现 SLEEP 睡眠跳过（`enableflags.SLEEP` + `body_awake` 状态）与 incremental
  awake 复跑（`awake_prev`）。
- AABB/rbound/margin/gap 的世界批量语义未验收（本片按 `world % batches` 取批，
  参考只用单批）；`geom_dataid` 批量按上游语义实现。
- 未实现 `collision()` 完整驱动、窄相（primitive/convex/SDF/flex）、接触写入
  （`dist/pos/frame/...`）、`naconmax` 溢出告警与 CUDA 图条件。
- 无 GPU 内核/图执行；不宣称完整 G07 或 P5 验收通过；NP04 未测。

## 7. 复现

```powershell
scripts/authoring/author-g07-broadphase.ps1
cargo test --locked --lib collision
```
