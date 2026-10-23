# NP01 逐字段精度映射证据（Windows）

任务：`2026-10-24-np01-field-precision-map`。日期：2026-10-24。执行：QA。
性质：**只读映射清单 + 偏离/缺口清单 + 本轮可证结论**；不构成 NP01 通过或完整验收。

## 1. 本轮可证结论（不写「通过」）

- 在产品**当前已实现的物理路径**范围内，逐字段映射到冻结上游 MJWarp 的 `Model`/`Data` 类型声明后：
  - **147/167** 个产品字段行能在上游 `types.py` 的 `Model`/`Data` 中找到同名（或登记别名）字段，类型语义一致：**无 f32→f64 升精度、无 f64→f32 降精度、无混合类型偏离**（全部为 f32/i32/bool 及其展平 vec/mat 布局）。
  - 其余 **20** 个为产品内部布局/中间量名（如 CSR 值数组、块表、mulm 布局），上游无同名 Data/Model 字段，**语义映射待下一轮**（`diff.json` 中 `internal_layout_field` 单列，不冒充覆盖）。
- `f64` 扫描（`src/physics` 非测试源码）：命中 **20** 处，**全部已分类**且均不在物理内核/状态路径（2 处测试断言、17 处诊断/读回校验、1 处辅助校验 API）；**无未分类 f64**。
- 19 个产品 CUDA 内核模板的标量扫描：**全部 f32、0 处 `double`**。
- 编译选项：fmad=true、fast_math=false、C++17 与上游 Warp 1.15.0 一致；**1 项差异观察**——产品固定 `compute_70`（PTX 向前兼容），上游按设备架构（本机 SM89 → compute_89），非类型偏离但指令选择不同，需在 NP01/next-g03-parity 记录；产品侧另有 `F32Physics` 编译门禁（约束更强）。
- **缺口 10 项**（7 项未实现、3 项部分覆盖），见 §5；**不冒充覆盖**。

## 2. 输入与复现

| 项 | 值 |
| --- | --- |
| 产品源码 | 仓库根（生成时 `product_commit = 687754c`，见 `mapping.json.inputs`；Backend「内核来源路线标准化」提交后重跑，统计与锚点仍一致） |
| 上游 MJWarp | `target/mjwarp-parallel/upstream`，commit `71da24d956378a87a703b6e1442b13aec0c4ac29` |
| 上游 Warp | `target/research/warp`，commit `ffa038b1adf3927a8e893d4536f1a7562f17d749`（v1.15.0） |
| 映射规格 | `scripts/np01-field-map/spec.json`（产品模块/文件、别名、累加量、f64 分类、编译选项锚点、缺口） |
| 工具 | `scripts/np01-field-map/np01-field-map.ps1`（PowerShell 7，只读；**不启动 Python**） |

复现（在仓库根执行；pin 不符或目录缺失会显式报错并停手，不猜测）：

```powershell
target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -ExecutionPolicy Bypass `
  -File scripts\np01-field-map\np01-field-map.ps1 `
  -ProductRoot D:\Rust\mjwarp-rs `
  -OutDir docs\development-evidence\np01-field-mapping
```

输出：`mapping.json`（191 行 = 167 字段 + 19 内核 + 5 编译选项；含 `task_id`）、`diff.json`（偏离/未覆盖/缺口）。工具对两处上游 pin、`types.py`、编译选项锚点做强校验；本次实跑通过。

## 3. 映射口径

- 上游类型：解析上游 `mujoco_warp/_src/types.py` 中 `Model`/`Data` dataclass 的 typed 字段；Warp 标量 `float`→f32、`int`→i32、`bool`→bool、`vec3`/`quat`/`mat33`/`spatial_vector`/`vecN`→f32[N]（Warp 1.15.0 语义；`float64`/`vec3d` 才映射 f64）。
- 产品类型：解析 `src/physics/**` 中 `&'a [T]` 与 `&'a [[T; N]]` 字段声明；切片展平布局与上游 vec/mat 逐元素对照（结论中标注「展平为产品 [f32] 布局」）。
- 列含义（`mapping.json.rows[]`）：`module`（产品模块）、`object`（结构体.字段/内核/编译选项）、`upstream_type`、`upstream_accum_type`、`product_type`、`product_accum_type`、`f64_mixed_flag`、`conclusion`、`evidence_path`（产品 `file#L` + 上游引用 `types.py#L`/上游文件行）。
- 累加类型：对已知累加量（`subtree_com`/`cinert`/`crb`/`M`/`qLD`/`qfrc_*`/`sensordata`/`ten_J` 等）按 `spec.accumulations` 登记；其余按「同字段（逐元素 f32）」。
- 字段别名（示例）：`matrix→M`、`ld→qLD`、`diagonal_inverse→qLDiagInv`、`ten_jacobian→ten_J`、`flexedge_jacobian→flexedge_J`、`subtree_mass→body_subtreemass`、`size→geom_size`、`mat→geom_xmat`、执行器字段 `dyntype→actuator_dyntype` 等（完整见 `spec.json.aliases`）。
- 扫描排除：产品 `*/tests.rs`、`*/tests/*` 测试代码；`f64` 扫描覆盖 `src/physics` 非测试源码。规格中 `referenced_extra` 登记的类型/工具文件（`types.py` 等）不计入「未引用上游模块」。

## 4. 结果数字（本次实跑）

| 指标 | 数值 |
| --- | --- |
| 映射行合计 | **191**（字段 167 + 内核 19 + 编译选项 5） |
| 字段行中上游类型命中 | **147/167** |
| 类型偏离（升/降精度/不一致） | **0** |
| 内核 `double` 偏离 | **0**（19 个内核模板均 f32 主导） |
| 产品内部布局字段（语义映射待下一轮） | **20** |
| `f64` 命中 / 已分类 / 未分类 | **20 / 20 / 0** |
| 编译选项差异 | **1**（`gpu_arch`：compute_70 vs 上游设备架构 89） |
| 缺口 | **10**（未实现 7、部分覆盖 3） |
| 未被本轮引用的上游模块 | **0**（`referenced_extra` 已单列 7 个类型/工具文件） |

字段结论分布（147 条一致项）：i32 42、f32 41、f32[3] 展平 23、f32[9] 展平 10、f32[6] 展平 9、bool 5、f32[2] 展平 5、f32[10] 展平 5、i32[2] 展平 3、f32[4] 展平 2、f32[5] 展平 1。

`f64` 分类（20 处）：测试断言 2（`constants.rs`，仅 `#[cfg(test)]`）、诊断校验 17（`device_checks.rs` 2、`kinematics.rs` 2、`mass_solve/probe.rs` 4、`mass_solve/sparse/**` 7、`resident/checks.rs` 1、`resident/output.rs` 1）、混合辅助 API 1（`history.rs::validate_times` 的 f64 时间参数；`HistoryBuffer` 状态仍 f32）。

## 5. 缺口清单（未覆盖，不冒充）

| # | 上游路线 | 状态 | 依据 |
| --- | --- | --- | --- |
| 1 | 渲染/BVH（`render.py`、`render_util.py`、`bvh.py`） | **未实现** | 产品无 render/BVH 子系统；r01-p8 与 R02–R10 顺延 |
| 2 | 地形/SDF（`collision_sdf.py`、hfield） | **未实现** | g10-p5 顺延（产品无 hfield/SDF 资产管线） |
| 3 | 凸体/网格 GJK/EPA/convex（`collision_gjk.py`、`collision_convex.py`） | **未实现** | 产品仅原语窄相（G08/G09 边界） |
| 4 | 柔体碰撞（`collision_flex.py`） | **未实现** | 产品 flex 仅 passive/edge/face/hessian 切片 |
| 5 | 约束求解（`constraint.py`：efc 行地址/约束一致性） | **未实现** | 产品未实现约束装配/求解 |
| 6 | 求解器管线（`solver.py`：CG/Newton、收敛判定） | **未实现** | 产品无约束求解器管线 |
| 7 | 岛屿/休眠分组（`island.py`） | **未实现** | 产品仅 `sleep.rs` 树级休眠切片 |
| 8 | IO/CLI（`io.py`、`cli.py`：MJCF 解析、回放、命令行） | 部分实现 | 产品仅有状态导出与入口；MJCF/回放/CLI 未实现 |
| 9 | block_cholesky 完整块分解 | 部分覆盖 | 产品仅 dense/small 因子切片 |
| 10 | smooth/passive/forward/inverse/sensor/history 完整语义 | 部分覆盖 | 产品为逐切片最小交付；本轮只做字段级映射 |

## 6. 编译精度选项对照

| 选项 | 产品 | 上游（Warp 1.15.0） | 结论 |
| --- | --- | --- | --- |
| fmad | `true`（`WARP_1_15_PHYSICS`，`recipe.rs`） | `fuse_fp=True`（`warp/_src/build.py`） | 一致 |
| fast_math | `false` | 模块默认 `"fast_math": False`（`warp/_src/context.py`） | 一致 |
| C++ 标准 | `--std=c++17`（`CppStd::Cpp17`） | `--std=c++17`（`warp/native/warp.cu`） | 一致 |
| GPU 架构 | `compute_70`（固定，PTX 向前兼容） | 按设备架构（本机 SM89 → `compute_89`） | **差异观察**：指令选择不同，非类型偏离；需在 NP01/next-g03-parity 记录 |
| 精度门禁 | `F32Physics`（编译产物 f32 门禁，拒绝 f64） | 无编译期 f32 门禁（按声明类型编译） | 产品更严格，不构成偏离 |

## 7. 边界与使用方式

- 本表是**字段/契约级**映射：不验证运行时数值等价、不替代 NP03 证据、不构成 NP01 通过；运行时数值等价仍以 NP03/NP04 与 G01 证据为准。
- 映射范围限于**产品当前已实现的物理路径**（`src/physics` 已实现模块的公开字段/内核/累加量 + `src/runtime` 精度相关编译选项）；未实现路线按 §5 列为缺口。
- 上游 pin 固定：`71da24d9…`（MJWarp）与 `ffa038b1…`（Warp 1.15.0）；pin 不符或目录缺失时工具显式报错停手（本次实跑通过）。
- 用途：作为 `next-g03-parity` 重划 Windows-only 的 NP01 输入表；后续若实现 §5 缺口或新增模块，需重跑本工具并更新 `spec.json`（模块/别名/缺口）。
- 未改产品源码/测试/容差/fixtures；工具只读源码并写 `mapping.json`/`diff.json`。
