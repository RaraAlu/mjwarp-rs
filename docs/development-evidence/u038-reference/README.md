# U038 独立流体与原子对照 · 证据

任务：`2026-10-16-next-u038-reference`（Backend，2026-10-23）。
验收（任务逐字）：①补真实冻结流体设备输出，覆盖非精确多体原子归约；②不使用候选中间量伪装独立参考，保留原始位型与失败。

结论摘要：

- 已生成并入库两组**真实冻结上游设备输出**（Warp 冻结模块设备读回，离线 authoring 生成，产品测试不启动 Python）：
  流体 `_fluid_force_83bf3546…` 与多体原子 `_gravity_force_7c63ac28…`。
- 产品侧只新增两条 `#[cfg(test)] #[ignore]` 定向 GPU 对照测试，容差沿用 2e-6，未改内核/PTX/ABI/断言/fixtures 语义。
- 对照结果：流体 270 槽 0 失败（16 槽位差，max_abs `2.98e-08`，首差 1 ULP）；原子 28 槽 0 失败
  （位差 0，max_abs 0；场景为**非精确**归约：独立 f64 复算偏差 `5.91e-08`≠0、单 dof 最多 3 体贡献）。
- 本任务不改变 U038-3D 已知债务（8/77、max `4.29e-06`）的登记状态；不宣称 U038、完整上游等价或 NP01–NP04 通过。

## 1. 参考来源（冻结上游）

| 项 | 值 |
| --- | --- |
| 上游提交 | `71da24d956378a87a703b6e1442b13aec0c4ac29` |
| Warp | 1.15.0（pinned wheel，`warp.dll` sha256 `7c006eb3…`） |
| MuJoCo / mujoco-warp | 3.15.0 / 3.15.0 |
| 设备 | RTX 4070 Ti SUPER，驱动 596.36（本机 `cuda:0`） |
| 流体入口 | `_fluid_force_83bf3546_cuda_kernel_forward`（passive 冻结模块） |
| 原子入口 | `_gravity_force_7c63ac28_cuda_kernel_forward`（passive 冻结模块） |
| 缓存模式 | `frozen_artifacts_seeded_then_loaded_cached`：把仓内冻结 `.cu/.meta/.ptx` 种子到新缓存目录，Warp 以 `(cached)` 加载，加载前后 sha256 校验 |

冻结产物哈希（与仓内 `fixtures/gravity-upstream-abi/generated`、`independent-kinematics` 一致，authoring 每步复核）：

| 模块 | cu | meta | ptx |
| --- | --- | --- | --- |
| smooth | `844dca14…` | `2bb1bcdb…` | `4179acd0…` |
| passive | `dd9338e9…` | `4a80327b…` | `d06154da…` |

离线 authoring 入口：`scripts/authoring/author-u038-reference.ps1`（pinned venv 审计 + `author-u038-reference.py`）。
脚本只读冻结上游，产品构建/测试/门禁均不引用。

## 2. 夹具

`fixtures/gravity-upstream-abi/u038-reference/{fluid,atomic}/`：

| 文件 | 内容 |
| --- | --- |
| `scenario.json` | 场景输入（拓扑、周期参数、qpos/qvel、periods） |
| `intermediate.json` | 冻结 smooth 链设备读回的中间量原始位（xpos/xquat/xmat/xipos/ximat/xanchor/xaxis/subtree_com/cdof/geom_xpos/geom_xmat/cvel） |
| `output.json` | 冻结 passive 内核设备输出原始位（`values` + `bits` + `run_variants` + `deterministic_runs`） |
| `natural-validation.json` | 天然校验（put_model 拓扑、独立 f64 复算、冻结哈希、缓存模式） |
| `authoring.json` | 生成器/脚本/环境/产物 sha256 |
| `receipt.json` | 上面 5 个文件的 sha256 回执 + 冻结入口/上游提交 |
| 5 个 `.py` 副本 | 生成脚本快照（common/chain/scenarios/validate/author） |

场景规模：

- 流体：9 worlds × 5 bodies × 5 geoms；椭球 + 惯性盒共享常驻链；多周期 wind(2)/density(3)/viscosity(5)/
  body_mass(1)/body_inertia(1)/geom_size(4)/geom_pos(3)/geom_quat(2)；含 0 系数跳过与多 geom 同体。
- 原子：7 worlds × 5 bodies × 4 hinge；两棵树（body1→2→3 链 + body4 独立根），单 dof 最多 3 体贡献；
  body_mass(1)/gravity(2)/body_gravcomp(5) 周期；**质量为非 2 幂值**（`0.3/0.7/1.1/0.5`）、
  gravcomp 与 gravity 取不可精确表示的十进制值，使多体贡献求和为**非精确 f32 归约**。

## 3. 天然校验（离线）

1. **固定环境**：pinned venv（Python 3.12.13）版本逐项核对、python prefix 为 venv、上游 checkout clean 且 HEAD 为冻结提交；
   venv 内 75 个 `mujoco_warp` 源文件与上游逐一 sha256 相同；`warp.dll` 与 pinned wheel RECORD 一致。
2. **冻结产物**：种子缓存副本与仓内冻结 `.cu/.meta/.ptx` sha256 逐一相同；Warp 日志显示 `(cached)` 加载。
3. **确定性**：smooth 链跑 2 次逐位相同；流体设备输出跑 2 次逐位相同；原子设备输出跑 8 次
   `run_variants=1`（本设备/调度下 `atom.add` 归约顺序稳定，见 §4 说明）。
4. **独立 f64 复算**：用上游公式 + 设备中间量在 f64 重算，与冻结设备输出比对：
   流体 max_abs `6.06e-08`；原子 max_abs `5.91e-08`（非零即证明归约含 f32 舍入，不是可精确表示场景）。
5. **边界**：流体零风/零密度/零粘度为逐位精确零。
6. **拓扑天然校验**：由等价 MJCF 经 `mujoco_warp.put_model` 生成 `body_branches/body_branch_start/
   body_tree_levels/body_isdofancestor/dof_parentid` 并与夹具拓扑逐项比对，`exact_match=true`。

### 3.1 生成期修正记录（如实保留）

首轮 authoring 的 `body_subtreemass` 派生实现有误（把「子体自身质量」加到父体，而不是「子树质量」），
导致冻结 smooth 链 `_subtree_div` 的分母错误，参考输出不是上游等价参考。已修正为按 MJCF 顺序
（parent index < child index）反向累加子树质量，并记录来源：

- 流体：`[15, 15, 6, 2, 3]`（world/body1=15）；修正前错误值 `[6, 13, 6, 2, 3]`。
- 原子：`[2.6, 2.1, 1.8, 1.1, 0.5]`；修正前（旧精确场景）`[9, 3, 6, 4, 8]`。

修正后夹具重新生成，`intermediate.json`/`output.json` 位型随之变化；本目录报告与提交均为修正后产物。
旧的错误产物未入库、未用于任何产品对照。

## 4. 产品侧对照

产品测试（只读夹具，不启动 Python，不改产品数值路径）：

- 流体：`src/physics/fluid_force/tests/frozen_device.rs`（`src/physics/fluid_force.rs` 仅加测试模块注册）。
  走 shared resident 路径：`KinematicsPlan`（含最小 flex 边以启用 G01 qvel 写入）→
  `ResidentComVelocityPlan` → `FluidForcePlan::from_resident` → `update_resident`；
  按 `[force(3), torque(3)]`/body 与上游 `fluid_applied` 逐槽比较。
- 原子：`src/physics/gravity_compensation/tests/frozen_device.rs`（`src/physics/gravity_compensation/tests.rs` 注册子模块）。
  `KinematicsPlan` + `GravityCompensationPlan`，与冻结 `qfrc_gravcomp` 逐槽比较；对全部
  `run_variants` 位型取最近距离判定容差。原证书测试断言与容差未动。

结果（报告 JSON 由测试写出，可复跑）：

| 场景 | 槽数 | 容差 | 位数差 | max_abs | 首差 | 上游 run variants |
| --- | --- | --- | --- | --- | --- | --- |
| 流体 | 270 | 2e-6 | 16 | `2.98e-08` | world=1 body=2 comp=3，1 ULP（`39b0be08`→`39b0be07`） | 2 次逐位一致 |
| 原子 | 28 | 2e-6 | 0 | `0.0` | 无 | 8 次单一位型 |

原子场景说明：`nonexact_reduction_order_verified=true` 依据是「非精确归约已覆盖」——
独立 f64 复算偏差 `5.91e-08`≠0、单 dof 最多 3 体贡献、容差内逐槽比对通过；
**未观测到顺序位差**（`order_bit_differences_recorded=0`，上游 8 次运行仅 1 种位型），
如实记录，不伪造逐位差异。§6 说明两者关系。

## 5. 复现

```powershell
# 离线生成夹具（pinned venv + 冻结缓存；会重写 fixtures/.../u038-reference）
scripts\authoring\author-u038-reference.ps1

# 定向 GPU 对照（NVRTC 12.8 工具链）
$env:CUDA_PATH = 'target\toolchains\cuda-12.8.1'
$env:PATH = (Join-Path $env:CUDA_PATH 'bin') + ';' + $env:PATH
$env:MJWARP_U038_FLUID_REPORT  = 'docs\development-evidence\u038-reference\fluid-report.json'
$env:MJWARP_U038_ATOMIC_REPORT = 'docs\development-evidence\u038-reference\atomic-report.json'
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe --lib `
  physics::fluid_force::tests_frozen_device::gpu_shared_fluid_matches_frozen_upstream_device `
  -- --ignored --exact --nocapture --test-threads=1
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe --lib `
  physics::gravity_compensation::tests::frozen_device::gpu_matches_frozen_upstream_atomic_device_output `
  -- --ignored --exact --nocapture --test-threads=1
```

产物报告：`fluid-report.json`、`atomic-report.json`（本目录）。

## 6. 未验收边界

- 只覆盖冻结上游路径可表达的流体（椭球 + 惯性盒、共享常驻链）与多体原子归约子集；不覆盖完整流体物理、完整 G04/U038。
- 原子场景确为**非精确归约**，但本设备/调度下上游 `atom.add` 顺序在 8 次运行中稳定且产品按上游顺序合并，
  因此**未观测到顺序位差**；这不等价于「任意调度下顺序无关」，也不等价于跨平台结论。
- 未跑 Linux、release、性能；不覆盖 NP01–NP04 四类精度验收，不宣称精度达标/完整验收。
- U038-3D 全链 8/77 越差（max `4.29e-06`）为**已登记已知债务**，本任务不修复、不改其验收状态。
- W13（nvJitLink）、Linux、g03-parity 不动。

## 7. 文件

- 夹具：`fixtures/gravity-upstream-abi/u038-reference/fluid/`、`.../atomic/`
- 生成脚本：`scripts/authoring/author-u038-reference.ps1`、`author-u038-reference.py`、
  `u038_reference_common.py`、`u038_reference_chain.py`、`u038_reference_scenarios.py`、`u038_reference_validate.py`
- 产品测试：`src/physics/fluid_force/tests/frozen_device.rs`、
  `src/physics/gravity_compensation/tests/frozen_device.rs`
  （注册：`src/physics/fluid_force.rs`、`src/physics/gravity_compensation/tests.rs`）
- 报告：`docs/development-evidence/u038-reference/fluid-report.json`、`atomic-report.json`
- 设计预检：`docs/development-evidence/u038-reference/design-precheck.md`
