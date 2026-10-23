# G09@P5 凸体与网格切片：plane-box 多接触与退化几何

任务：`2026-10-11-roadmap-g09-p5`。日期：2026-10-23。执行：Backend（批次 4）。
矩阵：G09 凸体与网格（`GJK、EPA、CCD等上游路径` / `退化几何与多接触` / P5）。
复核修正：`2026-10-23-g09-validation-gap-fix`（退化尺寸校验落空与回执 sha256 登记修正）。

## 1. 阶段语义与细化口径

旧计划 G09@P5 覆盖凸体/网格（GJK/EPA/CCD），当前无 BVH、网格/凸体资产管线与
GJK/EPA 实现。按 Chief 边界，本片只交**不依赖 BVH 的凸体原语部分**：

- 对齐上游 `collision_primitive_core.plane_box`：8 个盒角点（`i&1/2/4` 位序取 ±半尺寸）
  到平面的有符号距离与接触位置；`plane_box_wrapper` 对 8 个候选**逐个** `write_contact`。
  上游**没有去重步骤**：8 个候选全部参与检测，写出数量由 margin/gap 过滤决定
  （面接触 4、棱 2、角 1、间隙外 0；`≤4` 只是几何结论，不是去重上限）。
- 接入 G08 窄相框架：`primitive_narrowphase` 新增 `(plane, box)` 与 `(box, plane)`
  两个方向分支，复用既有 `write_contact`/材质混合/接触字段语义。
- 退化几何：薄板（半尺寸 z=5e-4）平放退化为 4 接触、薄板在 margin+gap 外为 0 接触；
  边/角姿态下 8 个角点中仅有 2/1 个通过 margin/gap 检测，故为 2/1 接触。
- 退化输入显式拒绝：受支持几何（plane/sphere/box）的 `geom_size` 在窄相入口校验，
  非正/非有限尺寸返回 `NarrowphaseError::InvalidGeomSize{geom, axis}`。

## 2. 实现

| 文件 | 内容 |
| --- | --- |
| `src/physics/collision/narrowphase.rs` | `plane_box` 8 角点核心 + `(plane,box)`/`(box,plane)` 窄相分支；`InvalidGeomSize` 错误 |`n| `src/physics/collision/narrowphase/validate.rs` | `validate`/`validate_geom_sizes`：数组长度、世界批量与 plane/sphere/box 尺寸校验（退化输入显式拒绝） |
| `src/physics/collision/narrowphase/tests/plane_box.rs` | 冻结上游 + 原生 f64 对照、4/2/1/0 接触数量、退化薄板断言、非正/NaN/Inf 拒绝测试、box-box 候选拒绝 |
| `src/physics/collision/narrowphase/primitive.rs` | `plane_box`/`plane_sphere`/`sphere_sphere` 与向量辅助 |
| `src/physics/collision.rs` / `src/physics/mod.rs` | `GEOM_BOX`、`plane_box` 导出 |
| `fixtures/plane-box/model.xml` | plane + 普通盒 + 薄板盒，nq=14、ngeom=3 |

## 3. 参考来源与生成方式

| 项 | 值 |
| --- | --- |
| 模型源 | `fixtures/plane-box/model.xml`（sha256 `a78b15472ffe8d0e3a8f6e61b8c4512dfc94c889ceca8ea344b76dab3cdf1586`；plane + box_a 0.5/0.4/0.3 + box_b 0.5/0.4/0.0005，margin=0） |
| 用例 | 7 个世界：flat4 / edge2 / corner1 / gap0 / yaw45_flat4 / thin_flat4 / thin_gap0；qpos 由角点最低间隙解析构造；候选只含 plane-box 对（未引入 box-box） |
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
| 退化输入拒绝（复核补强） | plane/sphere/box 的非正/NaN/±Inf 尺寸逐项返回 `InvalidGeomSize{geom,axis}`；合法 fixture 仍通过；box-box 候选返回 `UnsupportedPair` |

复跑：`cargo test --locked --lib plane_box -- --nocapture` → **3 过 0 败**
（原 plane-box 对照 1 + 退化尺寸拒绝 1 + box-box 拒绝 1）；
全量 lib 360 过 0 败；`cargo fmt --check` 与
`cargo clippy --all-targets --locked -- -D warnings` 通过。

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

## 7. 复核修正（已修）

1. **退化尺寸校验落空**（复核发现）：原 `validate` 只校验数组长度，`plane_box`/
   `primitive_narrowphase` 对 `geom_size` 无非正/有限性校验。已补 `validate_geom_sizes`
   与 `InvalidGeomSize`，覆盖 plane/sphere/box 的 0/负/NaN/±Inf 拒绝测试，落实验收条款。
2. **回执 sha256 登记错误**：原回执 `results.references.model_sha256` 误填 git blob
   hash `63743a8a…`，已改为实际文件 sha256 `a78b1547…`（见 §3）。
3. **措辞修正**（QA 建议）：`primitive_narrowphase` 注释改为「三类基础几何对」；
   本 README 直列 model sha256；「去重后最多 4 接触」按上游实际改正（无去重，
   数量由 margin/gap 过滤决定）；补 box-box 候选拒绝用例（fixture 候选只含 plane-box）。

## 8. 复现

```powershell
scripts/authoring/author-g09-plane-box.ps1
cargo test --locked --lib plane_box -- --nocapture
```
