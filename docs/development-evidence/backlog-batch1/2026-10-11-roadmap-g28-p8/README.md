# G28@P8 查询阶段切片：多世界批量射线查询 rays_batch

任务：`2026-10-11-roadmap-g28-p8`。日期：2026-10-23。执行：Backend（批次 1 后继）。
矩阵：G28 查询（`jac、ray、rays、contact_force` / `单体、批量与无命中` / P7/P8）。

## 1. 阶段语义与细化口径

P7 切片（`8764ced`）交付单世界原语 `ray`/`rays`。本 P8 切片交付计划 API `rays`
（U040；G28；P7/P8）的**多世界批量**宿主侧实现：`rays_batch` 接收每世界不同的
冻结 geom 位姿（`nworld × ngeom` 的 pos/mat 展开）与共享射线列表，输出
`nworld × nray` 的最近命中距离/geom 编号/法线；复用 P7 的逐原语求交与
最近命中/无命中语义。

原任务模块范围指向 `mjwarp.render`（BVH）；按当前代码布局本片不引入 render/BVH，
只做批量；映射为 `mjwarp.physics.core` 并在 §6 登记 BVH/过滤/设备视图边界。

## 2. 实现（`src/physics/query/batch.rs`）

- `WorldGeoms`（nworld/ngeom/geom_type/size/pos/mat）与 `rays_batch(geoms, rays)`。
- 逐世界组装冻结 `GeomPrimitive` 后调用 P7 `ray`，保证单世界/批量最近命中与
  无命中语义一致。
- 校验世界/geom 维度与 pos/mat/size 长度；`QueryError` 新增 `InvalidLength` 变体。
- 非原语 geom 仍由 P7 显式拒绝（`UnsupportedGeom`）。

## 3. 参考来源与生成方式

| 项 | 值 |
| --- | --- |
| 模型源 | `fixtures/ray-query/batch-model.xml`（sha256 `e03a503b…`；世界平面 + 铰链身体携带球/盒 2 原语，ngeom=3） |
| 世界/射线 | nworld=2、nray=5（qpos=0.0 与 0.6，身体位姿逐世界不同）；每世界 2 次无命中 |
| 原生参考 | MuJoCo 3.15.0（pinned venv）逐世界 `mj_kinematics`+`mj_ray`（NP03 一级） |
| 上游参考 | MJWarp `71da24d956…`，Warp 1.15.0，逐世界 `kinematics`+`ray`（二级；与 `rays` 共用 `_ray` 内核） |
| 生成脚本 | `scripts/authoring/author-g28-batch-query.ps1` + `.py`（离线，产品测试不启动 Python） |
| 产物 | `fixtures/ray-query/batch-cases.json`、本目录 `authoring.json` |

## 4. 对照结果（本次实测）

| 判定 | 结果 |
| --- | --- |
| 产品 vs 冻结上游（2 世界 × 5 射线） | geomid 全部一致；距离/法线逐分量在冻结容差 1e-6 内 |
| 产品 vs 原生 f64 一级判定 | max abs **1.19e-08** ≤ 冻结 1e-05；geomid 0 不符 |
| 无命中 | 每世界 2 条射线输出 dist=-1、geomid=-1、法线零 |
| 逐世界差异 | 世界 1（qpos=0.6）身体旋转后命中结果与世界 0 不同，均与上游一致 |

## 5. 验收指标映射（NP01–NP04）

- **NP01**：新增路径全 f32（位姿、射线、结果），无 f64 中间量。
- **NP02**：记录冻结上游提交、`kinematics`/`ray` 函数名、模型 sha256、环境与批量形状。
- **NP03**：两级判定——原生 f64 逐世界 `mj_ray` 先行（1.19e-08），再与 MJWarp f32 参考比较。
- **NP04**：**不适用/未测**——不发射内核、无批量对照与性能记录。

## 6. 未验收边界

- 未实现 BVH/RenderContext（`rc`）、mesh/hfield/flex/SDF 求交与设备视图。
- 未实现 geomgroup/flg_static/bodyexclude 过滤（批量接口不接受过滤参数）。
- 无设备内核/图执行；不宣称完整 G28 或 P8 验收通过；NP04 未测。

## 7. 复现

```powershell
scripts/authoring/author-g28-batch-query.ps1
cargo test --locked --lib batch
```
