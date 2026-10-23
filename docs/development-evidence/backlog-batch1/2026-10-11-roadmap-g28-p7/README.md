# G28@P7 查询阶段切片：原语射线查询 ray/rays

任务：`2026-10-11-roadmap-g28-p7`。日期：2026-10-23。执行：Backend（批次 1）。
矩阵：G28 查询（`jac、ray、rays、contact_force` / `单体、批量与无命中` / P7/P8）。

## 1. 阶段语义与细化口径

原图只有 G28@P7 粗切片，未给具体函数。按当前代码布局，本片取
架构计划 API `mjwarp_rs::physics::query::{ray, rays}`（U039/U040；G28；P7/P8）
的**原语射线查询**宿主侧实现：对冻结的六个原语 geom（plane/sphere/capsule/
ellipsoid/cylinder/box）求交，返回最近命中距离、geom 编号与法线；
覆盖**单体** `ray`、**批量** `rays` 与**无命中**（距离 -1、geomid -1、法线零）。

P7 阶段只覆盖「原语射线查询」侧：不构建 BVH、不支持 mesh/hfield/flex/SDF、
不做 geomgroup/flg_static/bodyexclude 过滤；`jac`（U072）依赖运动学链、
`contact_force`（U070）依赖碰撞与约束子系统（G07–G18 未实现），二者如实列为
未验收边界（§6）。验收口径未放宽：矩阵核心验收「单体、批量与无命中」逐项交付
并有上游/原生双参考；完整 G28 如实登记未验收。

## 2. 实现（`src/physics/query.rs`）

- `GeomPrimitive`（geom_type/pos/mat/size，世界坐标系行主序）与 `Ray`（pnt/vec）。
- `ray_geom`：逐分支对齐上游 `ray_geom`/`_ray_map`/`_ray_quad`：
  plane、sphere、capsule（圆柱侧 + 上下半球帽）、ellipsoid、cylinder（侧 + 上下盖）、
  box（逐轴面 + 矩形内判据）。
- `ray`：遍历 geom 取**最近正距离**（严格小于，首个最小者胜）；无命中返回
  dist=-1、geomid=-1、normal 零。
- `rays`：逐射线独立求交，顺序与输入一致（批量 ≡ 逐条单体，测试强制校验）。
- 拒绝面：非原语 `geom_type`（如 mesh=7）返回 `UnsupportedGeom`；必要尺寸为 0
  返回 `DegenerateSize`；不静默回退为“无命中”。

## 3. 参考来源与生成方式

| 项 | 值 |
| --- | --- |
| 模型源 | `fixtures/ray-query/model.xml`（sha256 `e866f105…`，ngeom=6，含平面/球/胶囊/椭球/圆柱/旋转盒） |
| 原生参考 | MuJoCo 3.15.0（pinned venv）`mj_ray` + `mj_kinematics`（NP03 一级） |
| 上游参考 | MJWarp 冻结提交 `71da24d956378a87a703b6e1442b13aec0c4ac29`，Warp 1.15.0，`mujoco_warp.ray`（NP03 二级） |
| 生成脚本 | `scripts/authoring/author-g28-ray-query.ps1` + `.py`（离线，产品测试不启动 Python） |
| 产物 | `fixtures/ray-query/ray-cases.json`、本目录 `authoring.json` |
| 射线 | 9 条：6 种原语各 1 次命中、2 次无命中、1 次多命中取最近（胶囊优先于其后球体） |

## 4. 对照结果（本次实测）

| 判定 | 结果 |
| --- | --- |
| 产品 vs 冻结上游（9 条射线） | geomid **9/9 一致**；距离与法线逐分量在冻结容差 **1e-6** 内 |
| 产品 vs 原生 f64 一级判定 | max_abs_dist **1.68e-07**、max_abs_normal **1.83e-07**、geomid 不符 **0** |
| 单体/批量等价 | `rays[i]` 与 `ray(ray[i])` 逐条相等（测试强制） |
| 无命中 | 2 条射线输出 dist=-1、geomid=-1、法线零 |
| 最近命中 | 多命中射线命中胶囊（geom 2，dist≈0.8）而非更远的球体 |
| 原语覆盖 | 六种原语 geom 编号 {0,1,2,3,4,5} 全部被命中 |

## 5. 验收指标映射（NP01–NP04）

- **NP01**：新增路径全 f32（geom 位姿/尺寸、射线、距离、法线），无 f64 中间量。
- **NP02**：记录冻结上游提交、`mujoco_warp.ray` 函数名、模型源 sha256、环境与射线清单。
- **NP03**：两级判定——原生 `mj_ray`（f64）先行（1.68e-07/1.83e-07，geomid 0 不符），
  再与 MJWarp f32 逐位参考比较（geomid 全一致，距离/法线在 1e-6 内；差异属 FMA
  收缩顺序，已在容差中说明）。
- **NP04**：**不适用/未测**——本片不发射内核、无批量对照与性能记录。

## 6. 未验收边界

- 未实现 BVH 与 mesh/hfield/flex/SDF 求交；未做 geomgroup/flg_static/bodyexclude 过滤；
  未实现多世界广播（本片单世界冻结输入）。
- `jac`（U072）未实现：依赖运动学链（cdof/世界位姿）集成。
- `contact_force`（U070）未实现：依赖碰撞（G07–G11）与约束/求解（G12–G18）子系统。
- 无设备内核与上传路径；不宣称完整 G28 或精度验收通过；NP04 未测。

## 7. 复现

```powershell
# 离线重新生成参考（不进入产品链，需要 pinned venv 与 GPU）
scripts/authoring/author-g28-ray-query.ps1

# 产品侧定向测试（只读冻结 fixture，不启动 Python）
cargo test --locked --lib query
```
