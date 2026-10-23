# G09@P5 凸体与网格切片：plane-box 多接触与退化几何

任务：`2026-10-11-roadmap-g09-p5`。日期：2026-10-23。执行：Backend（批次 4）。
矩阵：G09 凸体与网格（`GJK、EPA、CCD等上游路径` / `退化几何与多接触` / P5）。

## 1. 阶段语义与细化口径

旧计划 G09@P5 覆盖凸体/网格（GJK/EPA/CCD），当前无 BVH、网格/凸体资产管线与
GJK/EPA 实现。按 Chief 边界，本片只交**不依赖 BVH 的凸体原语部分**：

- 对齐上游 `collision_primitive_core.plane_box`：8 个盒角点（`i&1/2/4` 位序取 ±半尺寸）
  到平面的有符号距离与接触位置；`plane_box_wrapper` 对 8 个候选依次 `write_contact`，
  经 margin/gap 检测后写出面接触 4、棱 2、角 1、间隙外 0 个接触。
- 接入 G08 窄相框架：`primitive_narrowphase` 新增 `(plane, box)` 与 `(box, plane)`
  两个方向分支，复用既有 `write_contact`/材质混合/接触字段语义。
- 退化几何：薄板（半尺寸 z=5e-4）平放退化为 4 接触、薄板在 margin+gap 外为 0 接触；
  边/角姿态的重复支撑点由 `write_contact` 检测自然降为 2/1 接触。

## 2. 实现

| 文件 | 内容 |
| --- | --- |
| `src/physics/collision/narrowphase.rs` | `plane_box` 8 角点核心 + `(plane,box)`/`(box,plane)` 窄相分支与模块边界更新 |
| `src/physics/collision/narrowphase/tests.rs` | 冻结上游 + 原生 f64 对照、4/2/1/0 接触数量与退化薄板断言 |
| `src/physics/collision.rs` / `src/physics/mod.rs` | `GEOM_BOX`、`plane_box` 导出 |
| `fixtures/plane-box/model.xml` | plane + 普通盒 + 薄板盒，nq=14、ngeom=3 |

## 3. 参考来源与生成方式

| 项 | 值 |
| --- | --- |
| 模型源 | `fixtures/plane-box/model.xml`（sha256 见 fixture 登记；plane + box_a 0.5/0.4/0.3 + box_b 0.5/0.4/0.0005，margin=0） |
| 用例 | 7 个世界：flat4 / edge2 / corner1 / gap0 / yaw45_flat4 / thin_flat4 / thin_gap0；qpos 由角点最低间隙解析构造 |
| 上游参考 | MJWarp `71da24d956…`（Warp 1.15.0）：`kinematics` + `collision`（NXN 宽相 + `plane_box_wrapper`） |
| 原生参考 | MuJoCo 3.15.0（pinned venv）f64 `mj_kinematics` + `mj_collision` |
| 生成脚本 | `scripts/authoring/author-g09-plane-box.ps1` + `.py`（离线；产品测试不启动 Python） |
| 产物 | `fixtures/plane-box/cases.json`、本目录 `authoring.json` |

## 4. 对照结果（本次实测）

| 判定 | 结果 |
| --- | --- |
| 产品 vs 冻结上游 | 15 个接触逐字段 max abs **5.960e-08** ≤ 1e-6（FMA 末位差）；按 `(worldid, geom)` 分组、组内顺序一致 |
| 产品 vs 原生 f64 一级判定 | `mj_collision` 接触（dist/pos/frame）max abs **1.192e-07** ≤ 1e-5；接触数逐世界一致 |
| 多接触/退化数量 | 每世界接触数 **4 / 2 / 1 / 0 / 4 / 4 / 0**（flat4 / edge2 / corner1 / gap0 / yaw45_flat4 / thin_flat4 / thin_gap0） |
| 上游 vs 原生计数 | 上游 15 与原生 `[4,2,1,0,4,4,0]` 完全一致 |

复跑：`cargo test --locked --lib plane_box -- --nocapture` → **1 过 0 败**；
全量 lib 358 过 0 败；`cargo fmt --check` 与 `cargo clippy --all-targets --locked -- -D warnings` 通过。

## 5. 验收指标映射（NP01–NP04）

- **NP01**：新增路径全 f32（角点、法线、距离、接触位置），无 f64 中间量。
- **NP02**：记录冻结上游提交、`plane_box`/`plane_box_wrapper`、模型 sha256、几何布局与环境。
- **NP03**：原生 f64 `mj_collision` 一级判定（1.192e-07 ≤ 1e-5）+ MJWarp f32 二级对照（5.960e-08 ≤ 1e-6）。
- **NP04**：不适用——不发射内核、不做性能测量（`resources` 为空）。

## 6. 未验收边界

- 不实现 box-box、capsule/cylinder/ellipsoid/cone 等其余凸体对，也不实现 GJK/EPA、
  mesh/convex 支持映射、CCD/multiccd。
- 不实现 hfield/SDF 地形碰撞、flex 接触、显式 `<pair>` 覆盖、collision sensor
  接触类型位、EFC 行地址与约束求解、世界批量参数、GPU 内核与图执行。
- 非 `plane-box` 的候选对仍返回 `UnsupportedPair`；不宣称完整 G09 或 P5 验收。

## 7. 复现

```powershell
scripts/authoring/author-g09-plane-box.ps1
cargo test --locked --lib plane_box -- --nocapture
```
