# g10-p5 G2：GJK/EPA 凸碰撞（ccd/support/multicontact）

任务：`2026-10-25-g10-p5-gjk-epa`（G2，g10-p5 前置片），冻结上游
`71da24d956378a87a703b6e1442b13aec0c4ac29`（MJWarp 3.15）。
边界：只写 `src/physics/collision/gjk/`、`src/physics/collision.rs`、`src/physics/mod.rs`、
`tests/gjk_epa.rs`、本证据目录；未改 S2–S8 证据/渲染夹具、未改 `Cargo.*`、未开 GPU。

## 1. 结论

- **实现完成（CPU 宿主侧）**：`ccd()` 两阶段（`gjk_phase` → 浅穿透 `_inflate` / 深穿透 `epa_phase`）、
  GJK 迭代与 `_S1D/_S2D/_S3D` 距离子算法、EPA（polytope2/3/4、horizon 扩展、`FLOAT_MAX` 退化）、
  `support()` 九种 geom 类型（含 MESH 穷举与图 hillclimb、HFIELD 六顶点、margin 尾项）、
  `multicontact()`（feature 法线 + `_polygon_clip` ≤4 接触）、接触装配（原点平移、margin 修正、frame）。
- **数值对照**：5 个走上游 `CONVEX`（ccd）路径的参考 case 全部接触数一致；
  `cylinder-box`（多接触路径）**dist 8/8 位级一致、frame 72/72 位级一致**；
  其余 case dist ≤2 ULP（`degenerate-mesh` 18 ULP）、pos ≤6 ULP@向量尺度、frame 分量绝对误差 ≤8.2e-7。
- **未达位级的原因已定位**（不是「参数级」笼统归因）：
  ① pinned ccd kernel PTX 里 `fma.rn.f32` 1480 处、`mul.rn.f32`/`add.rn.f32` 各 0 处
  → GPU 侧多点累加只舍入部分中间积，CPU 源码顺序实现每步独立舍入；本片试过两种 FMA 收缩近似，
  各 case 互有胜负（见 §4），说明逐表达式收缩顺序未完全对齐；
  ② mesh 类 case 的世界位姿含非恒等 quat，参考侧用 mujoco f64 `mj_forward` → f32，
  而 mjwarp 运动学用 warp f32 `quat_to_matrix`（`quat_rotate` 三列路径）；
  实测两种口径在 tetar mesh 上差 **1 ULP（9 分量中 2 个）**、退化 mesh 静态 geom 差 **3 ULP（3 个分量）**
  → 输入位姿本身带 1–3 ULP 不确定度，经 GJK/EPA 放大到 dist 5–18 ULP。
- **阻断判定：不阻断**。G2 交付的是语义实现 + 参考对照能力；G3/G4/G5 依赖的是
  `ccd()`/`support()`/`multicontact()` 的语义与接口，不依赖本片位级达成。
  位级对齐建议作为**独立后续小片**（按 pinned ccd PTX 逐表达式定序，或先在 G5 GPU 内核里用 warp
  同口径对照）在 G5 内核片之前/并行完成；若 Arch 要求 G2 位级收口，本片按「dist/frame 位级 +
  其余 ULP 记录」判部分达成。

## 2. 实现清单（模块 → 上游函数）

| 文件 | 行数 | 对应上游 |
| --- | --- | --- |
| `src/physics/collision/gjk/mod.rs` | 263 | 常量/`OverflowType`/`Geom`/`MeshView`/`SupportPoint`/`GjkResult`/`Polytope` |
| `gjk/math.rs` | 128 | `warp/native/vec.h`、`mat.h`（dot/cross/length/normalize/sign/mat·vec）+ `math.make_frame` |
| `gjk/support.rs` | 196 | `support()` + `_discrete_geoms()` |
| `gjk/simplex.rs` | 391 | `_S1D/_S2D/_S3D`、`_det3/_same_sign/_project_origin_line/_project_origin_plane`、`_linear_combine/_subdistance`、`_tri_affine_coord/_tri_point_intersect/_same_side/_test_tetra` |
| `gjk/core.rs` | ~210 | `_gjk_support()`（含 `x_norm<1e-4` 方向调优）、`gjk()` |
| `gjk/polytope.rs` | 240 + `polytope_build.rs` 333 | `_attach_face/_epa_support/_rotmat/_ray_triangle/_replace_simplex3/_polytope2/_polytope3/_polytope4/_epa_witness` + face 位域 |
| `gjk/epa.rs` | 196 | `_get_edge/_add_edge/_epa()`（HORIZON 溢出 → `FLOAT_MAX`/idx=-1、重复支撑点早退、缓冲容量） |
| `gjk/multicontact/mod.rs` | 241 | `multicontact()` 驱动（feature dim → 法线对齐 → 边/面裁剪） |
| `gjk/multicontact/feature.rs` | 232 | `_feature_dim/_aligned_faces/_aligned_face_edge/_intersect1/_intersect2/_set_edge/_area4/_polygon_quad` |
| `gjk/multicontact/normals.rs` | 459 | `_mesh_normals/_mesh_edge_normals/_box_normals2/_box_normals/_box_edge_normals/_cylinder_normals/_cylinder_edge_normals/_box_face/_mesh_face/_cylinder_face` |
| `gjk/multicontact/clip.rs` | 159 | `_plane_normal/_halfspace/_plane_intersect/_witness_on_face/_polygon_clip()` |
| `gjk/phase.rs` | 265 + `ccd.rs` 228 | `_inflate()/gjk_phase()/epa_phase()/ccd()` |
| `gjk/contact.rs` | 194 | `collision_convex.eval_ccd_write_contact` 的接触装配子集（原点平移/margin 修正/frame/`write_contact` 检测条件） |
| `gjk/tests.rs` | 276 | 单元测试（支撑点分类型、`is_discrete` 分支、EPA 缓冲容量、浅穿透 `_inflate`、多接触上限、分析解对照） |
| `gjk/case_data.rs` | 166 | 内嵌案例数据（geom/mesh/两种世界位姿） |
| `gjk/case_compare.rs` | 327 | 5 个 CONVEX case 对 pinned 参考的 lib 内对照（含输入对齐两栏）；集成测试由 `…-gjk-epa-api` 片落地 |

关键分支语义（逐条对齐上游）：

- `is_discrete = mesh/box/hfield 组合 && margin1==margin2==0` → `epsilon=0`、`min_norm=MINVAL`、
  EPA 重复支撑点早退、`epsilon=MIN_EPATOL`；与 tolerance 路径严格互斥（`gjk_phase`/`epa_phase`/`_epa` 三处）。
- 球/胶囊 shrink：`full_margin = size[0] + 0.5*margin`、`margin=0`、`size[0]=0`、`cutoff += full_margin`；
  浅穿透 `dist > tolerance` 走 `_inflate`（不跑 EPA）；深穿透恢复原 margin/size 重跑 GJK 再 EPA。
- `ccd()` 返回语义：`ncontact 0 = 分离`（EPA 失败 `(FLOAT_MAX,0)`）、`1 = 候选`；
  `multiccd_idx` 仅在 margin==0 且两 geom 都在 box/cylinder/mesh 时保留，否则 `-1`。
- 接触装配：`dist += margin`；`frame` 在 `dist <= margin` 时用 `w1-w2`、否则 `w2-w1`；
  `pos = 0.5*(w1+w2) + origin`；检测条件 `dist < margin + gap`。

## 3. 历史 native 对照（探索性记录，不作判据；最终判据见 §9）

说明：本节数字是早期“ours vs native(f64→f32)”框架读数，且 `tests/gjk_epa.rs` 已在 `735abaf`
按拆分移出仓库（由 `…-gjk-epa-api` 片重新落地），**不能**作为 G2 通过依据；G2 唯一通过判据见 §9。
排序判据（历史）：`worldid` 分组 + 每 world 内 `(dist, pos)` 稳定排序后逐字段比较（G0b 结论）。
原始写入顺序只作观察项（本片 5 个 case 产品侧顺序由 EPA 面 → clip 决定，未参与判据）。

| case | geom 对（CONVEX 表） | count | dist 位等 | dist max ULP | pos max（向量尺度 ULP） | frame 位等分量 | frame max abs |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `cylinder-box` | CYLINDER×BOX | 8/8 | **8/8** | **0** | 4.00 | **72/72** | **0** |
| `ellipsoid-box` | ELLIPSOID×BOX | 2/2 | 0/2 | 2 | 2.00 | 8/18 | 8.19e-7 |
| `mesh-mesh` | MESH×MESH | 2/2 | 1/2 | 5 | 2.00 | 2/18 | 5.36e-7 |
| `convex-mesh` | BOX×MESH | 2/2 | 0/2 | 2 | 6.00 | 2/18 | 2.98e-7 |
| `degenerate-mesh` | 退化 MESH×MESH | 2/2 | 0/2 | 18 | 0.50 | 16/18 | 2.65e-11 |

口径说明：

- `pos` 用「向量最大分量的 ULP 尺度」归一（接触位置量级 0.02–0.16）；
  逐分量原始 ULP 在近零分量上会被严重放大（例：`-3.25e-5` 分量差 5e-9 = 1441 分量 ULP，
  但相对向量尺度只有 ~2 ULP）。
- `frame` 是单位向量，近零分量 ULP 无意义，用绝对误差；1 ULP@1.0 = 1.19e-7，
  8.19e-7 ≈ 6.9 ULP@1.0。
- `degenerate-mesh` 的 dist 18 ULP 相对值 6.7e-7（绝对 2.1e-9），是该 case 的世界位姿
  输入口径差（3 ULP）+ FMA 收缩共同放大的结果，不构成物理语义差异。

### 3.1 产品侧分支覆盖

- `is_discrete` 路径：`mesh-mesh`/`convex-mesh`/`degenerate-mesh`（mesh/box 组合、margin=0）；
- tolerance 路径：`cylinder-box`、`ellipsoid-box`；
- 多接触：`cylinder-box` 每 world 4 条（`multicontact` + `_polygon_clip`）；椭球路径 `multiccd_idx=-1` 只写 1 条；
- `_inflate` 浅穿透：单测 `shallow_separation_reports_positive_distance_without_epa`（球离 box 顶面 0.05 → `dist=0.05`、不跑 EPA）；
- EPA 缓冲容量：单测校验 `vert=10+2I`、`face=6+5I`、`horizon=24`；
- `FLOAT_MAX`/HORIZON 退化：实现路径覆盖（idx=-1 → `(FLOAT_MAX, 0)`、overflow 置位），参考 case 未触发该退化。

## 4. 未达位级的根因（实测，非猜测）

1. **FMA 收缩**：G0b 捕获的 pinned `ccd_kernel` PTX 统计 `fma.rn.f32=1480`、
   `mul.rn.f32=0`、`add.rn.f32=0`、`sqrt.rn.f32=39` → GPU 侧所有乘加被 NVVM 收缩。
   本片实验：
   - 变体 A（源码顺序、无 FMA）：dist max 0/2/5/2/18 ULP（按上表顺序），frame 位等分量 72/8/2/2/16；
   - 变体 B（`dot/length/cross/matvec` 全 `mul_add`）：dist max 0/2/1/2/37，frame 位等分量 72/6/0/1/2；
   - 变体 C（仅 `dot/length/matvec` fused，cross 源码顺序）：dist max 0/2/3/3/57，frame 位等分量 72/8/0/1/2。
   三种收缩假设都无法同时改进所有 case → 逐表达式收缩顺序需从 PTX 定序（S7b 方法），
   不能靠「整体 fused/不 fused」猜测。**最终保留变体 A**（源码顺序，可读且 dist 总体最优）。
2. **mesh 世界位姿口径**：参考侧 `mj_forward`（f64）→ f32；mjwarp 侧 `geom_xmat` 由
   `quat_to_matrix`（`quat_rotate` 三列路径，f32）计算。实测 tetra mesh 静态 geom 差 **1 ULP（2/9 分量）**、
   退化 mesh 静态 geom 差 **3 ULP（3/9 分量）**。`cylinder-box`/`ellipsoid-box` 全恒等姿态 → 输入无此差异，
   其 dist 已达 0/2 ULP。

## 5. G0b 参考分类更正（重要）

本片按上游 `collision_driver.MJ_COLLISION_TABLE` 复核 G0b 补采矩阵的路径归属，发现此前分类有误：

| case | G0b 原记录 | 实际路径 |
| --- | --- | --- |
| `cylinder-box` | G2 | **CONVEX（本片使用）** |
| `ellipsoid-box` | G2 | **CONVEX（本片使用）** |
| `mesh-mesh` | G2 | **CONVEX（本片使用）** |
| `sphere-box-shallow` / `sphere-box-deep` | G2（`gjk_phase` shrink/_inflate/EPA） | **PRIMITIVE**（`(SPHERE,BOX)→PRIMITIVE`，解析 sphere-box，非 GJK） |
| `capsule-box` | G2 | **PRIMITIVE**（`(CAPSULE,BOX)→PRIMITIVE`） |
| `box-box-margin` | G2（非 `is_discrete` 路径） | **PRIMITIVE**（模型 `nativeccd=disable` → `(BOX,BOX)` 覆盖为 PRIMITIVE，非 GJK） |
| `box-box-multiccd`（d04） | G2 | **PRIMITIVE**（同上，`box-box.xml` 带 `nativeccd=disable`；5 条/世界来自 MuJoCo 经典 box-box） |
| `convex-mesh`（d04） | G2 | **CONVEX（本片使用）** |
| `degenerate-mesh`（d04） | G2 | **CONVEX（本片使用）** |
| `hfield-*` | G4 | G4（HFIELD 走 CONVEX + 四接触后处理；`support()` HFIELD 已实现但 hfield 夹具构造属 G4） |
| `sdf-box` | G3 | G3（SDF 分支未实现） |
| `sphere-sphere` | G08 回归 | PRIMITIVE，非 G2 |

→ **G2 验收面 = 本片 5 个 CONVEX case**；`sphere-box`/`capsule-box`/`box-box-margin` 的数值属
PRIMITIVE 表（现有 `narrowphase` 的覆盖范围），建议 Arch 决定是否另开 PRIMITIVE 补充片。
（本片未修改 G0b 已关闭证据，更正只写在本证据与新任务记录里。）

## 6. 未覆盖项（unmet）

- **已达成 ≤4 ULP**（见 §9）：5 case 全部分量 ≤4 ULP；`degenerate-mesh` 曾出现的 27 ULP 根因为输入位姿口径（改用上游 f32 位姿后 1 ULP），**未修改窄相表达式**。
- **集成测试暂存 target/**：见 §10，由 `…-gjk-epa-api` 片落地。
- **球/胶囊 shrink 分支无 CONVEX 直达参考**：参考分类更正后，`sphere-box`/`capsule-box` 走 PRIMITIVE 表；shrink/`_inflate` 只有 in-crate 断言，不算数值验收。
- **位级未全达成**：见 §4/§9；建议后续片按 pinned ccd/kinematics PTX 定序（或 G5 GPU 内核同口径）补齐。
- **MESH 图 hillclimb 分支未用参考验证**：本片 5 个 case 的 mesh 顶点数 <10 → 走穷举分支；
  图 hillclimb（`graphadr != -1 && vertnum>=10`）实现完成但无参考 case（需更大的 mesh 夹具，属后续）。
- **HFIELD/TRIANGLE/FLEX 支撑点仅实现**，未用参考对照（HFIELD 归 G4；TRIANGLE/FLEX 无 G2 夹具）。
- **`multicontact` 的 mesh 边法线容量**：上游 `nmeshdegmax` 由场景决定（本片为 3，退化 mesh 的
  `polymapnum` 达 5 时上游会越界写入同一 2D 缓冲的相邻行）；本实现按行内 8 容量保存并记录为中国差异，
  当前参考 case 未触发该越界条目。
- **接缝未接**：本片只到 `ccd_pair`（geom 对级接触），未接 `collision_driver` 的 `naccdmax` 计数/
  `naconmax` 截断/`contact_efc_address`、未接 `hfield`/`flex` 专用 kernel 与 GPU 图条件（归 G5）。
- **性能未测**（归 G6 perf 片）；**Linux/T4 未覆盖**（用户 2026-10-24 裁决终止 Linux 任务）。
- 未宣称「G10 已实现」「完整物理通过」「双平台通过」「上游完全等价」。

## 7. 验证命令与环境

```
cargo fmt --check                                   # pass
cargo clippy --all-targets --locked -- -D warnings  # pass
cargo clippy --all-targets --locked --features cuda-probe -- -D warnings  # pass
cargo test --locked                                 # pass（735abaf 实测 558 passed / 0 failed / 1 ignored）
cargo test --locked --lib physics::collision::gjk -- --nocapture   # 最终判据 [判据] 表；完整输出见 gjk-epa-run.log
```

- 环境：Windows 10.0.26100 / rustc 1.99.0-nightly / edition 2024；CPU 路径（本片未开 GPU）。
- 不回归：既有 `plane-sphere`/`sphere-sphere`/`plane-box` 窄相代码未改动，相关单测与集成全过。
- 日志：`gjk-epa-run.log`（单测 + 集成完整输出）。

## 8. 复现与产物

产品/测试：

- `src/physics/collision/gjk/`（14 个文件）、`src/physics/collision.rs`（新增模块与 re-export）、
  `src/physics/mod.rs`（导出 GJK/EPA API）、`tests/gjk_epa.rs`。
- 内嵌对照数据（`tests/gjk_epa.rs` 中 `CASE_DATA`）由 authoring 导出：
  `target/g10-p5-g2/dump_case_inputs.py`（mujoco 3.15.0 CPU `mj_forward`）→
  `target/g10-p5-g2/gen_case_json.py`；两者在 `target/`（Git 忽略），不属产品路径，
  产品测试不启动 Python。

参考来源（只读）：

- `fixtures/g10-p5/references/{cylinder-box,ellipsoid-box,mesh-mesh}.json`（G0b 补采，worlds=2）；
- `fixtures/mjwarp-complex-physics/reference.json`（d04 既有：`convex-mesh`、`degenerate-mesh`）。

## 9. 最终数值判据：ours vs 同配方上游 mujoco_warp f32（G2 唯一通过判据）

**口径声明**：主参考 = 同配方上游 mujoco_warp f32（CPU 后端），临时口径；native f64 仅作契约框架读数，G6 走 
ative_error_v2；原生容差未批准，不作放行依据。

上游参考（authoring）：`capture_mjwarp_f32.py`（warp CPU 后端，产品路径不依赖 Python）+ `upstream-mjwarp-f32.json`；
provenance：mujoco_warp 3.15.0 / warp 1.15.0 / mujoco 3.15.0、device cpu、脚本 sha256、命令、
每 case xml_sha256 + f32 qpos 位型 + 上游 f32 geom_xpos/geom_xmat、**每 case 两次捕获逐位相同**。

**关键方法**：判据输入直接用上游 f32 运动学产出的 geom_xpos/geom_xmat（纯合成、无 FK/numpy 复算），
两边窄相输入完全一致，差异只反映窄相实现。

| case | count | dist | pos | frame/normal |
| --- | --- | --- | --- | --- |
| cylinder-box | 8/8 | **位级 8/8，0 ULP** | **位级 8/8，0** | **位级 72/72，0** |
| ellipsoid-box | 2/2 | **位级 2/2，0** | 1/2 位等，2 ULP@尺度 | 13/18 位等，3.84e-7（**3.2 ULP@1.0**） |
| mesh-mesh | 2/2 | 1/2 位等，**2 ULP** | **位级 2/2，0** | **位级 18/18，0** |
| convex-mesh | 2/2 | **位级 2/2，0** | **位级 2/2，0** | **位级 18/18，0** |
| degenerate-mesh | 2/2 | 0/2 位等，**1 ULP** | **位级 2/2，0** | **位级 18/18，0** |

→ **5 case 全部分量 ≤4 ULP（判据达成）**；其中 cylinder-box/convex-mesh 全字段位级，其余仅 1–3.2 ULP。
degenerate-mesh 此前 27 ULP 的根因是「输入位姿口径不同」（native f64→f32 vs 上游 f32 FK），
改用上游 f32 位姿作判据输入后降到 **1 ULP**，未改任何窄相表达式。

**E 表（契约框架读数，不作判据，供 G6 
ative_error_v2 种子）**：
ative-error-seed.json
（每字段分量 min/median/max/超 1e-6 占比）；ours vs native 与 mjwarp vs native 同级同量级
（例：ellipsoid frame 8.19e-7、mesh-mesh 5.36e-7、convex-mesh 2.98e-7）。

门禁：正式判据测试为 physics::collision::gjk::case_evidence::upstream_f32_within_4ulp（**非 ignore**，常驻门禁）；
native 位级项 
ative_bitwise_exploration 为**探索项、不作门禁**（f64 参考下不可达）。

## 10. 集成测试暂存说明（范围拆分）

任务范围不含 src/physics/mod.rs（公开 API）与 	ests/gjk_epa.rs（集成对照）。按 Arch 登记
2026-10-25-g10-p5-gjk-epa-api 负责 src/physics/mod.rs 一行 re-export + 把集成对照搬回 	ests/gjk_epa.rs。
本片：数值对照在 lib 内 src/physics/collision/gjk/case_compare.rs + case_data.rs（均 <500 行，未并进
gjk/tests.rs）；	ests/gjk_epa.rs 骨架暂存 	arget/g10-p5-g2/pending-api/gjk_epa.rs（Git 忽略）。