# g10-p5 G0a 预检：地形/SDF 碰撞上游语义与实现规格

任务：`2026-10-25-g10-p5-precheck`（G0a，只读预检；未写产品代码、未占 GPU）
冻结上游：`71da24d956378a87a703b6e1442b13aec0c4ac29`（MJWarp 3.15）
输入材料：`target/g10-p5-g2-notes.md`（GJK/EPA 语义笔记，G2 实现清单入口）、
`target/g10-p5-prep.md`（预检 v3，含拍板 1A/2A/3A/5A）
拍板映射：1A 只做物理（hfield 接触 + SDF 接触/距离/梯度/越界），不含 hfield 渲染；
2A 按 `worlds=2` 严格验收；3A MJB 由本批次 authoring 生成并冻结（3.12 产品 / 3.15 参考）；
5A GJK/EPA 先独立交付（G2），再 G1/G3/G4/G5/G6。

## 1. 结论

- hfield 碰撞不是独立 primitive：配对表 `(HFIELD, SPHERE/CAPSULE/ELLIPSOID/CYLINDER/BOX/MESH)
  → CollisionType.CONVEX`，走 **CCD（GJK+EPA）**；`_hfield_filter` 做 hfield 局部系盒-球粗筛，
  `ccd_hfield_kernel` 按子网格逐格构造 prism 调 `ccd()`，最后按「最小距离 / 最远 / 边」选 **0..3 四个接触**。
- SDF 碰撞：`sdf_narrowphase` 线程维度 `(sdf_initpoints, naconmax)`，AABB 相交盒内 **Halton(2,3,5)**
  初始化，**每个 initpoint 各写一个接触**（参考 `count=80 = 2 world × 40`）；`gradient_descent`
  = `sdf_iterations` 次非 intersection 步 + 1 次 intersection 步（Armijo 线搜索）。
- **越界/梯度口径更正（必须按此实现/验收）**：`probe-volume-gradient-outside.json` 的
  `distance=[-1.0, -0.7322314, +0.9999001]` 中 **`-1.0` 是内部距离**（cube 半边长 1、点在内部），
  `+0.9999` 才是盒外点且其梯度走 **有限差分 h=1e-4**。G4 映射写「内部距离（负）+ 盒外点
  （正，有限差分梯度）」，**不能**把 `-1.0` 当越界哨兵。
- `sdf_iterations`/`sdf_initpoints` 在 `m.opt`（XML `<option>`），不是 geom 字段；
  `gjk_iterations`/`epa_iterations` **由 `m.opt.ccd_iterations` 派生**（不是独立 opt 字段）。
- 产品侧缺口：`src/physics/collision/` 仅 plane-sphere/sphere-sphere/plane-box（G08@P5）+ G07 宽相；
  **没有 GJK/EPA**（上游 `collision_gjk.py` 2729 行），而 hfield 的 CONVEX 路径与 SDF 的 MESH 分支
  都依赖它；也没有 `hfield_*`、`oct_*`/`mesh_octadr`、`opt.sdf_*` 字段与上传链路。
- 规模重估：约 30+ 人日（GJK/EPA 共享大件 5–7d）。**本片只产出预检证据，不声明任何实现完成。**

## 2. 语义手册（可实现规格）

### 2.1 hfield 语义

| 项 | 规格 |
| --- | --- |
| 配对表 | `collision_driver.py:54-59`：`(HFIELD, SPHERE/CAPSULE/ELLIPSOID/CYLINDER/BOX/MESH) → CollisionType.CONVEX`；hfield 走 `ccd()` |
| 粗筛 | `_hfield_filter`（`collision_convex.py:62`，对应 MuJoCo `mjc_ConvexHField`）：hfield 局部系盒-球测试，水平 `size[0]/size[1]`、垂直 `size[2]`（上）/`size[3]`（下 base）；`hfield_size` 是 **vec4** |
| 子网格 | 由 geom2 的 AABB 反算 `cmin/cmax/rmin/rmax`；`dx = 2*size[0]/(ncol-1)`、`dy = 2*size[1]/(nrow-1)` |
| prism | 6 顶点：底 3 点 `z = -size[3]`，顶 3 点 `z = hfield_data[adr + row*ncol + col]*size[2] + margin`；`geom1.polyvert = prism`、`geomtype1 = HFIELD` |
| CCD | 每 prism 调 `ccd()`（`gjk_iterations = m.opt.ccd_iterations`、`epa_iterations = 16 if 全 box-box 对 else m.opt.ccd_iterations`、`tolerance = m.opt.ccd_tolerance`）；`ncontact == 0 || dist >= margin + gap` 跳过 |
| 接触 | `pos = hf_mat @ (0.5*(w1+w2)) + hf_pos`；`normal = hf_mat @ first_row(make_frame(w1-w2))` |
| 四接触 | contact 0 = min dist；contact 1 = 距 min 最远；contact 2 = 距 min–pos1 连线最远；contact 3 = 距其他三角边最远；门槛 `MIN_DIST_TO_NEXT_CONTACT = 1e-3` |
| 溢出 | `MJ_MAXCONPAIR = 50` 上限；溢出置 `OverflowType.HFIELD` 并只写已有接触 |

### 2.2 SDF 语义

| 项 | 规格 |
| --- | --- |
| 入口 | `has_sdf_geom → sdf_narrowphase`（`collision_driver.py:922`）；线程维度 `(sdf_initpoints, naconmax)`；只处理 `geom2.type == SDF`；AABB 相交盒为空直接返回 |
| 初始化 | AABB 相交盒内 **Halton(2,3,5)** 采样第 `i` 个点（`halton(i,2/3)` × 盒尺寸 + min），再变换到 geom2 局部系 |
| 接触数 | **每个 initpoint 写一个接触**（`write_contact` 每线程一次）→ 参考 `count = nworld × sdf_initpoints`（80 = 2 × 40） |
| `gradient_step` | Armijo：`alpha=2.0`、`rho=0.5`、`c=0.1`、`amin=1e-4`；每轮 `dist0=clearance`，`grad·grad < 1e-12` 提前返回；内层 while 收缩 alpha 直到 `alpha<=amin` 或 `(dist-dist0) <= wolfe`；`dist > dist0` 回退 |
| `gradient_descent` | `sdf_iterations` 次非 intersection `gradient_step` + **1 次 intersection**；法线 `normalize(normalize(rel_matᵀ·grad1) − normalize(grad2))` 再 `rot2` 回世界；`pos = pos − n·dist/2` |
| `clearance` | intersection 时 `max(sdf1, sdf2)`；否则 `sdf1 + sdf2 + abs(max(sdf1, sdf2))` |
| `compute_grad` | intersection 时按 A>B 选 `rel_matᵀ grad1` 或 `grad2`；否则 `grad2 + rel_matᵀ grad1 + max_grad*sign(max_val)` |
| `sdf()` 类型 | PLANE/SPHERE/CAPSULE/CYLINDER/BOX/ELLIPSOID 解析；MESH 用 `ray_mesh` ±法线（`dist > |p|` 时取反向得负值）；SDF：`sdf_type==-1` 走 octree volume，否则 plugin `user_sdf` |
| volume | `box_project`（盒外正距离 + 投影点）+ `find_oct` 八叉树叶插值（`oct_child/oct_aabb/oct_coeff`）；**盒外（dist0>0）梯度用有限差分 h=1e-4**（3 次额外采样），盒内用 octree 权重解析 |
| 参数 | `sdf_iterations`/`sdf_initpoints` 在 `m.opt`；参考默认 10/40（`sdf-cylinder.xml` 显式同值） |

### 2.3 越界/梯度口径（关键，G4 验收映射）

`probe-volume-gradient-outside.json`（`ad6774b5…`）输入 3 点、stages
`sample_volume_sdf` + `sample_volume_grad`：

| 点 | distance | 语义 |
| --- | --- | --- |
| `[0, 0, 0]` | **-1.0** | **内部距离**（cube 半边长 1，点在体积内部），不是越界哨兵 |
| `[0.25, -0.1, 0.15]` | -0.7322314 | 内部距离 |
| `[2.0, 0, 0]` | **+0.9999001** | **盒外/越界点**：`box_project` 正距离 + 投影插值；梯度走**有限差分 h=1e-4** |

G4 映射必须写「内部距离（负）+ 盒外点（正，有限差分梯度）」；写成「-1.0 = 越界」会导致
实现与验收错位。梯度维度 3×3（3 点 × 3 分量）。

### 2.4 参考夹具实测

| 夹具 | 尺寸 | 关键数字 |
| --- | --- | --- |
| `hfield-box.xml` | 440 B，sha `e0fe921c…` | hfield `nrow=ncol=3`、`size="1 1 0.1 0.1"`、elevation 3×3、box freejoint |
| `sdf-cylinder.xml` | 478 B，sha `178bc914…` | `<option sdf_iterations="10" sdf_initpoints="40">`、cube mesh（8 顶点）、`geom type=sdf mesh=cube` |
| `collision-hfield-box.json` | 11,748 B，sha `001d8426…` | `count=8`（每 world 4）、`worlds=2`、`geom` 对恒 `(0,1)`、`dist` 全负、fields 17 项 |
| `collision-sdf-cylinder.json` | 72,785 B，sha `1e05ffb7…` | `count=80`（每 world 40=`sdf_initpoints`）、`dist` 全负、fields 17 项 |
| `probe-volume-gradient-outside.json` | 1,565 B，sha `ad6774b5…` | distance 3 值 + gradient 3×3（见 §2.3） |

参考生成脚本 `fixtures/mjwarp-complex-physics/reference.py`（22,194 B，sha `99151d2e…`，
pinned 3.15 authoring）；`scripts/verify-static-gpu-references.ps1` 只校验清单完整性。

### 2.5 与 G2 笔记的对应关系

本手册 §2.1/§2.2 对应 `target/g10-p5-g2-notes.md` §1（`ccd()` 两阶段）、§2（GJK 迭代）、
§3（`support()`）、§4（EPA）、§5（多接触与 hfield 四接触）、§9（G0a 待确认清单）。
G2 实现时以笔记的逐函数行号与常量为准，本文件只给规格与验收口径。

## 3. 常量与默认参数（带来源）

| 常量 | 值 | 来源（pinned venv 源码） |
| --- | --- | --- |
| `FLOAT_MIN` / `FLOAT_MAX` | `-1e30` / `1e30` | `collision_gjk.py:30-31` |
| `MINVAL` / `MINVAL2` | `1e-15` / `1e-30` | `collision_gjk.py:33-34` |
| `MAXVAL` / `MAXVAL2` | `1e15` / `1e30` | `collision_gjk.py:35-36` |
| `MIN_DIST2/3/4` | `1e-10 / 1e-10 / 1e-17` | `collision_gjk.py:39-41` |
| `MIN_EPATOL` | `1e-7` | `collision_gjk.py:44` |
| `FACE_TOL` / `EDGE_TOL` | `cos(0.0889)` / `sin(0.0889)` | `collision_gjk.py:46-47` |
| `INTERSECT_TOL` | `3e-7` | `collision_gjk.py:50` |
| `MJ_MAXCONPAIR` | `50`（`mujoco.mjMAXCONPAIR`） | `types.py:29` |
| `MJ_MAX_EPAFACES` | `5` | `types.py:36` |
| `OverflowType` | `CCD=1<<4`、`HFIELD=1<<5`、`EPA_HORIZON=1<<8`、`ALL` | `types.py:177-190` |

| 默认参数 | 值 | 来源/备注 |
| --- | --- | --- |
| `ccd_tolerance` | `1e-6` | mujoco 3.15 `m.opt`（纯 CPU 读；参考 XML 未覆盖） |
| `ccd_iterations` | `35` | 同上；**`gjk_iterations = m.opt.ccd_iterations`**（`collision_convex.py:1334`） |
| `epa_iterations` | `16 if 全 box-box 对 else m.opt.ccd_iterations`（本夹具 35） | `collision_convex.py:1272` |
| `sdf_iterations` | `10` | mujoco 3.15 `m.opt`；`sdf-cylinder.xml` 显式同值 |
| `sdf_initpoints` | `40` | 同上 |
| 主求解 `tolerance` | `1e-8` | mujoco 3.15 `m.opt`（非 CCD 容差） |
| `warn_overflow` | `int(OverflowType.ALL)` | `io.py:438`；`warn_overflow=False` 可关 |
| EPA 缓冲容量 | `epa_vert/vert_index = 10 + 2*epa_iterations`；`epa_face/face_pr = 6 + 5*epa_iterations` | `collision_convex.py:1295-1303` |

`gjk_iterations`/`epa_iterations` **不是独立 opt 字段**，G2/G5 一律按上表派生。

## 4. MJB 方案（3A）

### 4.1 生成方式

| 侧 | 工具 | 步骤 |
| --- | --- | --- |
| 3.12 产品 | MuJoCo 3.12.0 C API（`target/toolchains/mujoco-3.12.0/package`） | 离线 C++ 工具（参照 `scripts/authoring/render-bvh/compile_scene_312.cpp` 模式）加载 XML → `mj_saveModel` 写 `.mjb`；用 3.12 重新加载校验 |
| 3.15 参考 | pinned venv mujoco 3.15.0 | `mujoco.MjModel.from_xml_path(xml)` → `mj.save_binary(mjb)`；用 3.15 重新加载校验 |

两者均为 **offline authoring**（不进产品构建/测试/门禁、不占 GPU），产物冻结到
`fixtures/mjwarp-complex-physics/`（或 G0b 新目录，按登记时的 files 决定）。

### 4.2 provenance 字段（每 MJB 一条）

`source_xml.path` + `source_xml.sha256`、`mujoco_version`、`generator`（脚本路径 + sha256）、
`command`、`output.bytes` + `output.sha256`、`loaded_back: true`（同版本重载校验）、
`loader_check`（3.12 产物用 3.12 校验、3.15 产物用 3.15 校验）、`authoring_only: true`、
`reads_product_tree: false`。

### 4.3 `nworld=2` 取舍建议（2A）

- 参考 `worlds=2`，按 2A 严格双世界验收；产品 G07/G08 状态已有 `nworld` 维度
  （`PrimitiveState.nworld`），但 hfield/SDF 的打包链路（hfield_data、octree、`opt.sdf_*`）尚未接线。
- 建议 G0b 先做一次 **nworld=2 打包可行性 spike**（约 0.5d）：确认接触写入顺序、`naconmax`
  维度、world 与 geom 对索引。若不可用：**先报 Arch**，再决定截断单世界验收，并在 G6 receipt
  `unmet` 写「参考 worlds=2 未覆盖、单世界口径」，**不放宽数值判据**。

## 5. 精度映射草案（依据 `docs/numerical-precision-contract.md`）

契约依据：§279 必检范围（上游 f32 时全链 f32、不得 f16/TF32、规则不要求跨设备逐位相等）；
§306 硬性验收指标（逐字段冻结原生容差、既有原生容差不自动放宽、上游 f32 而内部 f64 即失败）；
§142 数值参考优先级（先检查既有原生容差、不借用通用容差、结构错误不进容差回退）；
§362 测试族指标表（位差/容差形式先例）。

| 字段 | 类型 | 建议判据（G2/G3/G4 host，G5 设备） | 来源/理由 |
| --- | --- | --- | --- |
| 接触 `dist` | f32 | **位型全等**目标；不可达时 ≤4 ULP（同配方、同迭代次数、同顺序） | 迭代算法，同初值/同顺序可逐位复现（S7b1 depth 先例：≤4 ULP 且实测 0） |
| 接触 `pos` | f32 vec3 | **位型全等**（或逐分量 ≤1 ULP） | witness 线性组合/中点，纯算术 |
| 接触 `frame`/`normal` | f32 mat33 | ≤4 ULP（normal 含归一化 sqrt/div）；frame 由 normal 派生同判据 | 归一化舍入敏感（S7b1 球面法线先例 1 ULP 放大） |
| 接触数/顺序 | i32 | **全等**（`count` 与逐接触 `geom/worldid/dim/type`） | 结构字段，不放容差 |
| SDF `distance` | f32 | 位型全等目标；≤4 ULP | 梯度下降迭代 + octree 插值/有限差分 |
| SDF `gradient` | f32 vec3 | ≤4 ULP（有限差分含 h=1e-4 除法/减法） | 数值路径长 |
| hfield elevation / `hfield_size` | f32 | 位型全等 | 模型输入，一次转换 |
| octree 系数 `oct_coeff` | f32 | 位型全等 | 模型输入 |

**顺序对齐方法（迭代类必写）**：沿用 S7b1/S7b2 方法——从 pinned 3.15 的 PTX 缓存读取
`gjk`/`epa`/`support`/`sdf` kernel 的指令顺序（`div.rn`/`sqrt.rn`/`fma.rn` 分布），产品设备代码
按同序书写；对 JIT 收缩点显式用 `__fmul_rn/__fmaf_rn/__fsub_rn/__fadd_rn` 固定；
host 参考（G2/G3/G4）与设备内核（G5）用同一表达式顺序，避免双实现漂移。
**不按设备噪声放宽容差**；若同顺序仍超限，回查表达式/常量顺序而不是改容差。

## 6. 拆分方案（5A）

### 6.1 G2（GJK/EPA，独立前置片）能力边界

- 范围：`ccd()`（gjk_phase + epa_phase）、`support()`（sphere/box/capsule/ellipsoid/cylinder/
  mesh/hfield-polyvert/triangle/flex）、`multicontact()`（box/cylinder/mesh）、EPA polytope 与
  缓冲布局、`OverflowType` 溢出语义、`margin/gap/tolerance/cutoff` 语义、`is_discrete` 分支。
- 依赖：无（纯 host 数学，可独立交付；G3/G4/G5 都复用）。
- 验收面：需要 **GJK/EPA 直达参考**（box-box、box-mesh、sphere-box 等）。现有 d04 参考里
  `collision-hfield-box`（hfield prism-CCD 间接覆盖）与 `collision-sdf-cylinder`（SDF 路径，
  不用 GJK）不足以覆盖 G2 全部支撑函数；**G0b 需补采 GJK/EPA 直达 case**（参照上游
  `collision_gjk_test.py` 的用例构造，pinned 3.15 authoring）。
- 估期 5–7d；先立 `is_discrete` 分支/定序/EPA 退化断言，再写数值（S7b 工作方式）。

### 6.2 剩余片顺序与估期

| 片 | 内容 | GPU | 估期 |
| --- | --- | --- | --- |
| G2 | GJK/EPA（独立前置片） | 否（host） | 5–7d |
| G1 | 模型字段：`hfield_*`/`oct_*`/`mesh_octadr`/`opt.sdf_*` 解析与 parity | 否 | 3d |
| G3 | SDF host 数学：`sdf()`/`sdf_grad()`、clearance/compute_grad、gradient_step/descent、volume box_project+find_oct+有限差分、Halton initpoints | 否 | 5d |
| G4 | hfield host：`_hfield_filter` 粗筛 + prism 构造 + CCD 组合 + 四接触选择 | 否 | 3–5d |
| G5 | GPU 内核：GJK/EPA、SDF narrowphase、hfield prism CCD、接触写入 | 是（独占） | 5d |
| G6 | 验收收口：地形接触 + 梯度 + 越界三条映射、精度映射、负向控制、证据 + perf 片 | 是（独占） | 3d + 1d |

依赖：G2 → G1 → G3/G4 → G5 → G6；G0b（夹具/MJB）在 G1 前完成。总估期约 30+ 人日。
每片新文件 ≤500 行；QA 独立复核；GPU 单一独占窗口（沿用 r01-p8/perf 规则）。

## 7. 未满足项与风险

- 本片是**只读预检**：没有实现、没有跑 GPU、没有改产品代码/测试/夹具/`Cargo.*`。
- G2 的验收参考尚缺（需 G0b 补采 GJK/EPA 直达 case）；`_S3D/_S2D/_S1D`、`_polytope2/3/4`、
  `_epa_witness` 的复刻工作量未细化（笔记 §9）。
- 产品 `nworld≥2` 打包对 hfield/SDF 的可用性未验证（G0b spike）。
- hfield/SDF 的 GPU 内核性能未测（归 G6 perf 片，独立窗口）。
- Linux/T4 未覆盖（用户 2026-10-24 裁决终止 Linux 任务）。
- 完整 R01 聚合收口不在本片范围；本片只提供 G0a 预检证据。

## 8. 边界声明与复现

- 本片仅写 `docs/development-evidence/g10-p5-precheck/`；未改产品代码/测试/夹具/`Cargo.*`；未占 GPU。
- 语义核对来源（pinned venv，只读）：`mujoco_warp/_src/collision_{gjk,convex,sdf,driver}.py`、
  `types.py`、`io.py`；模型默认值用纯 CPU `mujoco.MjModel` 读取（不初始化 warp/CUDA）。
- 复现：`target/mjwarp-parallel/venv/Scripts/python.exe` 读取上述源码/默认值；本片无 GPU 命令。
