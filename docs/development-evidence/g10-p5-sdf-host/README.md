# G3 体积 SDF 宿主数学（对齐 pinned `mujoco_warp/_src/collision_sdf.py`）

任务：`2026-10-25-g10-p5-sdf-host-r2`（二次登记承接；首次登记因范围不含父文件 `src/physics/collision.rs` 无法提交，产物由 r2 承接）。上游基线 `71da24d956378a87a703b6e1442b13aec0c4ac29`。
本片无 GPU、不实装 GPU 内核；产品测试不跑 Python。

## 1. 结论

- 已实现体积 SDF 宿主数学：`sdf()`/`sdf_grad()` 分发（PLANE/SPHERE/CAPSULE/CYLINDER/BOX/ELLIPSOID +
  MESH/SDF 的 octree 体积分支）、`box_project`、`find_oct`、`sample_volume_sdf`/`sample_volume_grad`
  （盒内解析权重梯度、盒外有限差分 `h=1e-4`）、`clearance`/`compute_grad` 两式、Armijo `gradient_step`
  （`amin=1e-4`、`rho=0.5`、`c=0.1`）、`gradient_descent`（`sdf_iterations` 次普通步 + 1 次
  intersection 步、Halton(2,3,5) `sdf_initpoints` 个初始点、每初始点一个接触、`pos3 = pos - n*dist/2`）。
- **判据**：`sdf-box`（上游 mujoco_warp f32、warp CPU 后端、两次捕获逐位相同）接触
  **80/80 全字段位级一致**：`dist` 80/80 位等（0 ULP）、`pos` max 0.00 ULP@尺度、`frame` 720/720 位等。
  优于「≤4 ULP、位级优先」契约；无超差项。
- 结构字段全等：接触数 80 = 2 world × 40 initpoints；geom 对 `(0,1)`、worldid 序列与 G0b 原生参考一致；
  octree 二进制参考数组按 JSON 声明长度/shape/sha256 校验。
- `sdf_iterations`/`sdf_initpoints` 取自模型 `opt`（G1 已接通字段；本片参考记为 10/40）。

## 2. 改动文件

| 文件 | 内容 |
| --- | --- |
| `src/physics/collision/sdf/mod.rs` | 类型（`SdfType`/`SdfShape`/`VolumeData` 入口）、常量、导出 |
| `src/physics/collision/sdf/primitives.rs` | 原语 SDF 与梯度（sphere/box/ellipsoid/capsule/cylinder/radial_field） |
| `src/physics/collision/sdf/octree.rs` | `box_project`/`find_oct`/`sample_volume_sdf`/`sample_volume_grad`/`halton` |
| `src/physics/collision/sdf/descent.rs` | `sdf`/`sdf_grad` 分发、`clearance`、`compute_grad`、Armijo 步与下降 |
| `src/physics/collision/sdf/narrowphase.rs` | 单 pair：AABB 交集 + Halton 初始点 + 每初始点接触候选 |
| `src/physics/collision/sdf/tests.rs` | 位级判据（lib 内单测，见 §3）与 `halton`/`box_project` 单测 |
| `src/physics/collision.rs` | 注册 `pub mod sdf`；更新「未验收边界」说明 |
| `src/physics/collision/gjk/mod.rs` | `mod math` → `pub(crate) mod math`（复用已验证 mat3/vec 原语，无逻辑改动） |
| `scripts/authoring/g10-p5/capture_sdf_f32.py` | 上游 f32 CPU 参考补采（authoring only） |
| `fixtures/g10-p5/sdf-f32/` | `sdf-box-cpu.json` + `oct_aabb.f32`/`oct_child.i32`/`oct_coeff.f32`（小端） |
| `tests/g10_p5_sdf_host.rs` | 参考夹具结构校验（二进制 sha256、count/worlds/initpoints、与 G0b 结构一致） |
| `docs/development-evidence/g10-p5-sdf-host/` | 本目录 |

新文件均 ≤500 行（模块最大 `octree.rs` 217 行）。\n\n## 3. 位级判据表（`sdf-box`）

方法：判据输入直接用上游 f32 运动学（`geom_xpos`/`geom_xmat`）与上游 `put_model` 的 f32 模型数组
（octree 三个数组 + `geom_size`/`geom_aabb`/`geom_type`/`geom_dataid`/`mesh_octadr`）；
`worldid` 分组后按 `(dist, pos, frame)` 稳定排序逐分量比较位型。产品测试不依赖 Python。

| 字段 | count | 位等 | max 差值 |
| --- | --- | --- | --- |
| `dist` | 80/80 | **80/80** | 0 ULP |
| `pos` | 80/80 | 80/80 | 0.00 ULP@尺度 |
| `frame` | 720/720 分量 | **720/720** | 0 |
| 结构（count/geom 对/worldid） | 80/80 | 全等（与 G0b 原生参考同结构） | — |

差异项：无。运行输出：`[SDF判据] sdf-box count 80/80 | dist 位等 80/80 max 0 ULP | pos max 0.00 ULP@尺度
| frame 位等分量 720/720 max abs 0e0`。

## 4. 参考与 provenance

`fixtures/g10-p5/sdf-f32/sdf-box-cpu.json`：mujoco_warp 3.15.0 / warp 1.15.0 / mujoco 3.15.0、
device `cpu`、脚本 `scripts/authoring/g10-p5/capture_sdf_f32.py`（sha256 在 JSON）、XML sha256、
f32 qpos 位型、两次捕获逐位相同（`two_captures_bitwise_identical=true`）。
输入 qpos 复用 G0b `fixtures/g10-p5/references/sdf-box.json`（2 worlds）。
octree 数组为上游 `put_model` 的 f32 上传数组（`oct_aabb` 1,523,928 B、`oct_child` 2,031,904 B、
`oct_coeff` 2,031,904 B），JSON 记 dtype/count/shape/sha256。
补采日志：`capture-cpu.log`（authoring，不入产品路径）。

## 5. 与 G1 字段链的关系

- 参考里的 octree 数组就是 mjwarp `put_model` 的 f32 上传数组；产品侧由 G1 交付的字段链提供：
  native 读取（`oct_depth/oct_child/oct_aabb/oct_coeff/mesh_octadr/octnum/noct`）→ `model::volume`
  唯一 f64→f32 转换点（`oct_aabb`/`oct_coeff`/`hfield_size` 一次就近舍入；整数不转换）→ 本片宿主数学。
  G1 已对 3.12 mjb（含 `sdf-cylinder` noct=63,497）做位级 parity，本片不重复该对照。
- `sdf-box` 用 3.15 编译（`mjGEOM_SDF` 需要 3.12 之后的几何类型语义），因此本片数值测试直接读
  authoring 夹具的 f32 数组，而不是从 3.12 mjb 复读 octree。

## 6. 未覆盖 / unmet

- **GPU 内核与 `nacon` 截断/原子计数未实装**（G5）。
- **plugin `user_sdf`/`user_sdf_grad` 与 MESH+mesh 的 `ray_mesh` 分支未实现**：本片只覆盖
  octree（volume）分支；这两类是上游错误/插件路径，无本片参考。
- **hfield 四接触**（G4）、**flex/TRIANGLE 支撑** 不在本片。
- **仅 1 个数值 case**（`sdf-box`）；`sdf-cylinder` 只有 G1 的 octree 位级 parity，无 SDF 接触参考。
- **Linux/T4 未验证**；本片只在 Windows + warp CPU 后端执行。
- **公开 API 未暴露**：`physics::collision` 仍为私有模块（`src/physics/mod.rs` 不在本片范围），
  数值判据放在 lib 内 `sdf/tests.rs`；`tests/g10_p5_sdf_host.rs` 只做夹具结构校验。
  若需要 crate 外入口，应由后续 API 片加一行 re-export。
- 性能未测（G6）。

## 7. 验证

- 默认工具链 `cargo test --locked`：**569 过 0 败 1 忽略**（G2/G1 基线 565 + 本片 sdf 单测 3 + 集成夹具校验 1）。
- `cargo test --locked --lib physics::collision::sdf -- --nocapture`：3 passed（含 §3 位级判据）。
- `cargo test --locked --test g10_p5_sdf_host`：1 passed。
- 四门禁 fmt/clippy/clippy-cuda/unit 由 `normify gate 2026-10-25-g10-p5-sdf-host` 记录，绑定本提交范围。
- 无 GPU、无 Python 产品依赖、未改 S2–S8 证据与 `fixtures/mjwarp-complex-physics/*.xml`、未改 `Cargo.*`。
