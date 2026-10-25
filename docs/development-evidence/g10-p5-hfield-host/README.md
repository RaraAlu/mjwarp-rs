# G4 hfield 宿主窄相（对齐 pinned `mujoco_warp/_src/collision_convex.py`）

任务：`2026-10-25-g10-p5-hfield-host`。上游基线 `71da24d956378a87a703b6e1442b13aec0c4ac29`。
本片无 GPU、不实装 GPU 内核；产品测试不跑 Python。

## 1. 结论

- 已实现上游 `_hfield_filter` + `ccd_hfield_kernel` 的宿主版：
  - `_hfield_filter`：水平/垂直 box-sphere 粗筛 + 用支撑函数取对侧 geom 在 hfield 帧内的紧 AABB，
    再做 box-box 测试；返回 `xmin/xmax/ymin/ymax/zmin/zmax` 子网格边界。
  - prism 六顶点构造：按上游顺序（底三角形 z=-size[3] 与顶三角形高程链式前移）沿行/列扫描，
    `hfield_data[adr+(r+dr)*ncol+c]*size[2]+margin`，含 prism 高度早退测试。
  - 每个 prism 调 G2 的 `ccd()`（geom1 = hfield 棱柱，geom2 = 对侧 geom，全走 CONVEX 表）；
    接触 `pos=0.5*(w1+w2)`、`frame=make_frame(w1-w2)`、世界系转换 `hf_mat@local+hf_pos`。
  - 0..3 四接触选择：最小距离、最远点、离 `min_pos→pos1` 直线最远、离另一条三角边最远；
    `MIN_DIST_TO_NEXT_CONTACT=1e-3` 提前结束。
  - `MJ_MAXCONPAIR=50` 溢出：计数达上限后停止扫描并置 `overflow`（对应上游 `OverflowType.HFIELD`）。
- **判据**：三个 case 对同配方上游 f32（warp CPU 后端，两次捕获逐位相同）**全字段位级一致**：
  | case | count | dist | pos | frame |
  | --- | --- | --- | --- | --- |
  | `hfield-box` | 8/8 | 位等 8/8，0 ULP | 0.00 ULP@尺度 | 72/72 位等，0 |
  | `hfield-sphere` | 8/8 | 位等 8/8，0 ULP | 0.00 ULP@尺度 | 72/72 位等，0 |
  | `hfield-capsule` | 7/7 | 位等 7/7，0 ULP | 0.00 ULP@尺度 | 63/63 位等，0 |
  优于「≤4 ULP、位级优先」口径；无超差项。
- **重复接触点按全字段多重集比较**：参考含重复点（G0b 记录 hfield-sphere 8 行 6 个不同点、
  hfield-capsule 7 行 3 个不同点），本片对 `(worldid, dist, pos, frame)` 排序后逐字段比位型，
  原始顺序不作判据；三例重复点结构也全等（capsule 7 行/两 world）。
- 数据取 G1 接通的 `hfield_size/nrow/ncol/adr/data` 字段（本片参考 3×3、size=[1,1,0.1,0.1]）。

## 2. 改动文件

| 文件 | 内容 |
| --- | --- |
| `src/physics/collision/hfield/mod.rs` | `HfieldData`（G1 字段校验）与模块导出 |
| `src/physics/collision/hfield/filter.rs` | 上游 `_hfield_filter`（含紧 AABB 支撑查询） |
| `src/physics/collision/hfield/driver.rs` | 子网格/prism 扫描、`ccd()` 调用、0..3 接触选择、`MJ_MAXCONPAIR` 溢出 |
| `src/physics/collision/hfield/tests.rs` | 三 case 位级判据 + filter/溢出单测 |
| `src/physics/collision.rs` | 注册 `pub mod hfield` |
| `src/physics/collision/gjk/mod.rs` | 导出 `pub(crate) use support::support`（复用 G2 支撑函数） |
| `scripts/authoring/g10-p5/capture_hfield_f32.py` | 上游 f32 CPU 参考补采（authoring only） |
| `fixtures/g10-p5/hfield-f32/` | 三个 case 的参考 JSON（含模型数组、f32 运动学、接触字段） |
| `docs/development-evidence/g10-p5-hfield-host/` | 本目录 |

新文件均 ≤500 行（`driver.rs` 约 330 行、`tests.rs` 约 340 行）。

## 3. 方法

- 参考补采：pinned `mujoco_warp 3.15.0` / `warp 1.15.0` / `mujoco 3.15.0`，**device=cpu**；
  输入沿用 G0b/d04 参考模型与 qpos（f32 位型原样）；每 case 两次捕获逐位相同（3/3）；
  dump `geom_xpos/geom_xmat`（f32）、接触 17 字段、hfield 与几何静态数组（`hfield_size/nrow/ncol/adr/data`、
  `geom_type/size/dataid/margin/gap/rbound`）、模型 `opt.ccd_tolerance/ccd_iterations`（本片 1e-6/35）。
- 判据输入：`geom_xpos/geom_xmat` 直接取上游 f32 运动学；geom2 变换到 hfield 帧
  （`pos=mat1ᵀ(pos2-pos1)`、`rot=mat1ᵀ·mat2`，mat3 约定沿用 G2/G3 已验证的列存储）；
  `margin=geom_margin[g1]+geom_margin[g2]`、`gap` 同式；`gjk_iterations=epa_iterations=35`。
- 比较：`worldid` 分组后按 `(worldid,dist,pos,frame)` 排序逐字段比位型，并校验 geom 对 `(hfield,对侧)`、
  每 world 接触数与四接触上限。日志：`capture-cpu.log`（authoring）。

## 4. 未覆盖 / unmet

- **GPU 内核与 `nacon` 截断/原子计数未实装**（G5）；本片只有宿主数学与接触选择。
- **数值面只有三个 case**：`hfield-box`（box 对侧）、`hfield-sphere`、`hfield-capsule`；
  ELLIPSOID/CYLINDER/MESH 对侧只经同一 `ccd()/support()` 通用路径实现，未补 CPU 参考（G4 采样面）。
- **hfield 与 mesh/cylinder 的 CPU 参考未采**（同上；后续如需要可在 G5/G6 补）。
- **flex/TRIANGLE 支撑**不在本片；`nworld>1` 按参考自带 2 world 验证，未扩更多世界。
- **Linux/T4 未验证**（仅 Windows + warp CPU 后端）。
- **公开 API 未暴露**：`physics::collision` 仍私有，数值判据在 lib 内 `hfield/tests.rs`；
  crate 外入口留待后续 API 小片。
- 性能未测（G6）。

## 5. 验证

- 默认工具链 `cargo test --locked`：全过（本片新增 hfield 单测 3 项；总计数以门禁记录为准）。
- `cargo test --locked --lib physics::collision::hfield -- --nocapture`：3 passed
  （三 case 判据 + filter 单测 + `MJ_MAXCONPAIR` 溢出单测）。
- 四门禁 fmt/clippy/clippy-cuda/unit 由 `normify gate 2026-10-25-g10-p5-hfield-host` 记录，
  绑定本片范围（`src/physics/`、`tests/g10_p5_sdf_host.rs`、`scripts/authoring/g10-p5/`、`fixtures/g10-p5/`、
  本证据目录）。
- 无 GPU、产品测试不跑 Python、未改 S2–S8 证据与 `fixtures/mjwarp-complex-physics/*.xml`、未改 `Cargo.*`。
