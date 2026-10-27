# UNC-3 S2 · mesh 分支层静态映射表 + 老片（sphere-box / capsule-box / box-box）分支级清单

- 任务：`2026-10-27-unc3-branch-layer-static-map`（方向③ brief 第 4 条；**无 GPU、无编译**）
- upstream revision：`71da24d956378a87a703b6e1442b13aec0c4ac29`
- 产物：`branch-map.json`（30 条分支条目，机器可读）+ 本 README
- 结论一句话：**分支层只有少数几条有数值 case；其余是「可验证」（分支可达、可加 case）或「仅静态核对/未覆盖」，本片不宣称任何分支「已覆盖」。**

## 1. 方法与范围

读了设备侧源码（行号按当前 HEAD）：

- `src/physics/collision/cuda/multicontact.cu`（234 行，全读）
- `src/physics/collision/cuda/multicontact_mesh.cu`（191 行，全读）
- `src/physics/collision/cuda/primitive.cu`（L40-189）、`primitive_wrapper.cu`、`boxbox.cu`、`boxbox_wrapper.cu`、`multicontact_geom.cu`（分支/容量定位）

对应 pinned：

- `collision_gjk.py`：`_intersect1/2` L1591-1616、`_mesh_normals` L1620-1686、`_mesh_edge_normals` L1690-1734、`_mesh_face` L1926-1948、`multicontact` L2216-2451
- `collision_primitive_core.py`：`box_box` L595-1049、`sphere_box` L1050-1101、`capsule_box` L1105-1260
- `collision_primitive.py`：`sphere_box_wrapper` L1087、`capsule_box_wrapper` L1161、`box_box_wrapper` L1250

分类定义（与 S1/S5 台账一致，四类外加两档）：**已 case 覆盖**（有数值 case 且验证窗判据内通过）/ **可验证**（可达、有参考、可加 case，当前未逐分支标注）/ **仅静态核对**（无 case，不宣称覆盖）/ **仅限制存在**（只证模型级超界 + 上游行为）/ **不可达或防御**（静态推理不可达）/ **未覆盖**（无 case 且未定论）。

## 2. 汇总

| 家族 | 条目 | 已 case 覆盖 | 可验证 | 仅静态核对 | 仅限制存在 | 不可达/防御 | 未覆盖 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| mesh-multicontact | 14 | 3 | 0 | 4 | 2 | 0 | 5 |
| sphere-box | 3 | 0 | 2 | 0 | 0 | 1 | 0 |
| capsule-box | 5 | 0 | 4 | 1 | 0 | 0 | 0 |
| box-box | 8 | 0 | 6 | 0 | 0 | 0 | 2 |
| **合计** | **30** | **3** | **12** | **5** | **2** | **1** | **7** |

（与 `branch-map.json` 的 `counts.by_status` 实算一致：已 case 覆盖 3 / 可验证 12 / 仅静态核对 5 / 仅限制存在 2 / 不可达-防御 1 / 未覆盖 7。）

## 3. mesh 分支层（14 条）

| ID | 分支 | 设备位置 | 状态 | 证据/说明 |
| --- | --- | --- | --- | --- |
| M-01 | sidecar 缺失（无 poly 视图 / `mesh_polyadr<0`）→ 单接触回退 | `multicontact.cu:59-65` | 已 case 覆盖 | M0a 空 sidecar bins + validate_mesh_poly 空 sidecar 单测 |
| M-02 | 面法线恢复 dim3（face-face 对齐） | `multicontact_mesh.cu:68-94` | 已 case 覆盖 | M2 mesh-box/mesh-cylinder 4/4 bit、M3 mesh-mesh 4/4 bit |
| M-03 | 法线恢复 dim2（边→最多 2 面） | `multicontact_mesh.cu:96-115` | 未覆盖 | 无 mesh 边-面 case |
| M-04 | 法线恢复 dim1（顶点→degree 面） | `multicontact_mesh.cu:117-129` | 未覆盖 | S3 U7 只有上游参考 count=1 |
| M-05 | **degree>10 guard** → return 0 | `multicontact_mesh.cu:120-122,148-150` | 仅限制存在 | S3 U7 fixture（24>10）上游 count=1；设备侧未跑 |
| M-06 | polymap 交集为空 → 0 | `multicontact_mesh.cu:81-83,86-88,107-109` | 仅静态核对 | 无空交集 case |
| M-07 | 边法线 dim2 | `multicontact_mesh.cu:139-143` | 未覆盖 | 无 edge-face case |
| M-08 | 边法线 dim1（顶点→前驱构边） | `multicontact_mesh.cu:145-170` | 未覆盖 | 含 degree guard（同 M-05） |
| M-09 | **polygon 顶点>16 guard** → return 0 | `multicontact_mesh.cu:180-182` | 仅限制存在 | S3 U8（20>16）上游 count=4、接触面含 20-gon 顶帽（几何佐证，polygon 号未打印）；设备侧未跑 |
| M-10 | 面恢复返回 0 → 单接触回退 | `multicontact.cu:195-197` | 未覆盖 | 仅容量越界组合触发；M-05/M-09 的后果路径 |
| M-11 | 面/边法线均不匹配 → 单接触回退 | `multicontact.cu:128-131,151-158` | 仅静态核对 | 现有 case 均走 face-face 对齐 |
| M-12 | edge-contact witness 交换/回翻 | `multicontact.cu:207-211,229-230` | 仅静态核对 | 无 mesh edge-contact case |
| M-13 | polygon_clip 空 → 单接触回退 | `multicontact.cu:223-225` | 仅静态核对 | 无裁剪为空 case |
| M-14 | mesh-mesh 两侧视图二选一 | `multicontact.cu:59-61` | 已 case 覆盖 | M3 两侧同类型 mesh；不同类型组合未覆盖 |

**给 S4 的直接输入**：M-05/M-09 是设备 guard 与上游容量语义**不同**的两处（设备逐 polygon / 整 vertex degree；上游按模型级 `npolygonmax`/`nmeshdegmax` 动态容量），S4 对 U7/U8 必须描述性对照并同时记 dist/pos，若设备 ≠ 上游（U8 设备 1 vs 参考 4）写「与上游行为不同 / 限制存在」，不动 fixture、不改判据。

## 4. 老片三家族（16 条）

| ID | 分支 | 设备位置 | 状态 | 说明 |
| --- | --- | --- | --- | --- |
| SB-01 | 球心在盒内 → 6 面最近面 | `primitive.cu:59-81` | 可验证 | primitive sphere-box-shallow/-deep 80/80，未逐分支标注归属 |
| SB-02 | 球心在盒外 → clamp + 最深点中点 | `primitive.cu:82-89` | 可验证 | 同上 |
| SB-03 | normalize 除零防御 | `primitive.cu:56` | 不可达/防御 | 静态推理：盒内已先拦截、盒外 dist>MINVAL |
| CB-01 | 面最近配置（`n_out>1` 跳过） | `primitive.cu:115-141` | 可验证 | capsule-box 80/80；子分支未单列 case |
| CB-02 | 边最近配置（`det<MINVAL` 退化跳过） | `primitive.cu:143-165` | 可验证 | 轴平行退化可构造，尚未单列 |
| CB-03 | 线段参数 clamp（端点/中点 3×3） | `primitive.cu:172-189` | 可验证 | 端点接触在 capsule-box case 内 |
| CB-04 | 双端点 → 2 槽 | `primitive_wrapper.cu:23-29` | 可验证 | capsule-box case |
| CB-05 | `axisdir` 计算后未再使用（静态注记） | `primitive.cu:101-102` | 仅静态核对 | 与上游逐行一致，无行为差异 |
| BB-01 | 3 面轴分离早退（`c<-margin`） | `boxbox.cu:127` | 可验证 | G5 box-box-margin 391/392（max_pos_ulp=1） |
| BB-02 | 9 边叉积轴，`cross_length<MINVAL` 跳过 | `boxbox.cu:154` | 可验证 | 轴对齐盒含平行边 |
| BB-03 | 边轴分离早退（`c3<-margin`） | `boxbox.cu:165` | 可验证 | 同上 |
| BB-04 | 无候选轴（`axis_code==-1`）早退 | `boxbox.cu:190` | 未覆盖 | 可达性未判定 |
| BB-05 | 面路径（rotmore + 角点恢复 8 槽） | `boxbox.cu:203-231` | 可验证 | G5 box-box-margin |
| BB-06 | 边路径 | `boxbox.cu:304` 起 | 可验证 | 同上 |
| BB-07 | 边路径退化（`|rnorm.z|<MINVAL`）早退 | `boxbox.cu:346` | 未覆盖 | 防御性早退 |
| BB-08 | 未填充槽语义（`dist=MJ_MAXVAL`、pos/normal 全零） | `boxbox.cu:96` | 可验证 | box-box-margin 部分槽未填充 |

**给 S5 的直接输入**：三家族的现有 case 覆盖的是「有/无接触 + 位级数值」层，**不是分支层**；分支级清单里 12 条「可验证」需要在后续片加定向 case 才能转「已覆盖」，本片不写「已覆盖」。CB-05 与上游一致（无差异）；SB-03 属防御不可达，不必补 case。

## 5. 边界（不宣称项）

- 本片是**静态映射**：不跑设备、不开 GPU 窗口、不编译、不重跑任何判据；「可验证」≠「已覆盖」。
- 不改判据、容差、参考、fixtures、产品代码、设备源。
- 行号按当前 HEAD 记录；后续改动需重核。
- 不写「G10已实现 / 完整物理通过 / 双平台通过 / 上游完全等价 / 原生放行 / 精度达标」；不写「窗口通过」。

## 6. 证据文件

| 文件 | 说明 |
| --- | --- |
| `branch-map.json` | 30 条分支条目（id/家族/分支/设备位置/pinned 位置/状态/证据/注记 + 分类定义 + counts） |
| `README.md` | 本文件 |
| `receipt.json` | 机器可读摘要（任务、方法、分类计数、给 S4/S5 的输入、manifest sha、边界） |
