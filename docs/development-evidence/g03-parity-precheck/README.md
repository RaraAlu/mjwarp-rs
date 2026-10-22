# G03 质量求解准入缺口可行性预检

任务：`2026-10-22-g03-parity-precheck`
日期：2026-10-22
基线：`93c5cb3`（登记）/ `10913ce`（W-D4 与阶段证据之后）
关联债务：`2026-10-22-g01-g03-np03-admission-debt`（proposed，只跟踪）
原任务：`2026-10-16-next-g03-parity`（proposed，**本预检不 start**）

## 结论摘要

1. **缺口可本机观测、可本机尝试修复**，测量不需要外部参考、nvJitLink 或 MathDx：
   冻结上游参考（`fixtures/mjwarp-mass-solve/reference.json`）、原生参考
   （`fixtures/mass-solve/*.json`）、产品 GPU 探针与本机 MuJoCo DLL 都在仓内/本机。
2. **根因是产品解算与上游 MathDx cuSOLVER 的剩余 f32 算序差异**，不是缺外部参考：
   - `case4/world4` 的 `solution`（rhs1/rhs2）本库对原生参考的归一化误差为
     **1.318 / 1.167**，冻结上游对照分数 **0.925 / 0.440**，5% 红线
     **0.972 / 0.462**；失败元素为 rhs1 的 idx0 与 rhs2 的 idx3。
   - 同一块的因子（`ld`、`diagonal_inverse`）仍被准入，缺口集中在**求解阶段**。
   - 上游该块（mixed-joints，单个 11 维 `tile_cholesky` 块）的 `factor_m`/`solve_m`
     走 **MathDx cuSOLVER**（生成 CUDA 调用 `potrf_11_11_1_89_32_1_1_5_x_x_0`
     与 `potrs_11_11_1_89_64_1_1_5_x_x_0`，authoring 缓存含对应 LTO/fatbin），
     产品侧是自研列式 Cholesky + 前代/回代，并已仿真 MathDx 的
     `rsqrt.approx` 与乘减融合约束（`src/physics/mass_solve/cuda.rs`）。
3. **历史对照**：`2026-10-12` 首差专项（`docs/windows-g03-first-difference.md`、
   `fixtures/mjwarp-mass-solve/first-difference/acceptance.json`）当时三个探针
   NP03 域为 0 失败（240/240/152 块，分别 4/4/12 块对照放行）；当前为
   2/240、2/240、6/152 失败。
4. **逐提交对照**（本预检实测，见 `bisect.json`）：
   - `c74aede`（`8a5bac0` 前一个提交）：`mass_solve_probe` **6 过 0 败**；
   - `8a5bac0`（「对齐非零位形旋转链」，把 rotate 改为冻结 Warp 的 dot/cross 分组）：
     同测试 **3 过 3 败**，NP03 缺口在该提交出现。
   - 该提交本身是正确的上游对齐（提交记录 T4 非零链 1134 项逐位一致）；它使
     kinematics 更接近上游，同时让 mass_solve 解算路径的剩余差异暴露出来。
5. **不建议直接 start `2026-10-16-next-g03-parity`**：该任务验收含完整双模式平台与
   NP01–NP04 全验收，范围登记于模块拆分前，外部条件未冻结。应先由用户/Chief
   裁决修复路线，再登记聚焦实施任务。

## 可修性路线与条件

### 路线 A1（推荐先评估）：本机产品算序对齐，无需外部运行时组件

继续现有「首差」方法，把 legacy 质量求解的 `tile_cholesky` 分块与多右端项求解
对齐上游 MathDx cuSOLVER 的实际指令序：

- 材料（本机已有，非产品依赖）：pinned authoring venv
  `target/mjwarp-parallel/venv`（warp-lang 1.15.0、mujoco 3.15.0、numpy 2.5.4）；
  authoring 缓存 `target/g03-block-reference/warp-cache/1.15.0/` 中的生成 CUDA、
  LTO 与 fatbin；CUDA 工具链自带 `nvdisasm`；历史 `target/native-mathdx7*/`
  产物的 potrf LTO/cubin。此前 `4dc8893` 已用同类材料对齐因子首差
  （`rsqrt.approx` + 乘减融合约束）。
- 改动面：`src/physics/mass_solve/`（legacy 求解内核与必要元数据），不新增外部
  依赖、不改公开 ABI、不改容差与 fixtures。
- 前置/影响：产品源码变化会漂移 T4 源码身份 → 需登记并完成源码身份重钉；
  需重跑 `mass_solve_probe` 定向集、双模式门禁与既有 GPU 回归。
- 风险：cuSOLVER potrs 内部算法闭源，反推工作量大且不能保证逐位一致；
  历史 state7 尾部仍有 1 ulp 级差异（`docs/windows-g03-first-difference.md`）。
- 验收目标：3 项 `mass_solve_probe` 在 all-features 配置下 NP03 准入通过，
  且不放宽任何容差/断言语义。

### 路线 A2（需精度契约解释）：只提高解算精度、不对齐上游逐位

若只把解相对原生参考的误差压到红线内（例如改善回代累加顺序），会改变产品数值口径、
离开「对齐冻结上游」的约定。需用户/Chief 对精度契约解释作裁决，**不作为默认方案**。

### 路线 B（需外部组件或用户裁决，不满足「不引入」前提）

| 路线 | 内容 | 阻塞 |
| --- | --- | --- |
| B1 | 运行时经 nvJitLink 链接 MathDx LTO | M7/W13 仅评审未实现；许可、部署、版本待决策；属外部组件 |
| B2 | 重制冻结参考（如禁用 MathDx，改用 Warp 标量 `tile_cholesky` 分支） | 需 pinned Python authoring 环境（本机 venv 存在，但产品测试不得启动 Python）与用户批准的基线口径变更；属改基线 |
| B3 | 调整 NP03 5% 余量或准入策略 | 5% 余量由用户 2026-10-11 冻结，需用户裁决 |

参考：`docs/development-evidence/layers-w13.md`（nvJitLink 仅评审）、
`docs/development-evidence/layers-w14.md`（MathDx 仅离线参考）、
`docs/unified-launcher.md` L5（MathDx 链接保留仓外）。

## 复现

### 1) 离线诊断（读冻结 fixtures + 产品 GPU 探针，不参与构建）

```powershell
# 把 dump_probe.rs 复制进临时 cargo 包，依赖 path 指向仓库根：
#   [dependencies] mjwarp-rs = { path = "../..", features = ["cuda-probe"] } ; serde_json = "1"
# 包内 src/main.rs = docs/development-evidence/g03-parity-precheck/dump_probe.rs
$env:CUDA_PATH = '<repo>\target\toolchains\cuda-12.8.1'
$env:PATH = "$env:CUDA_PATH\bin;$env:PATH"
$env:MJWARP_MUJOCO_ROOT = '<repo>\target\toolchains\mujoco-3.12.0\package'
$env:MJWARP_MUJOCO_DLL = "$env:MJWARP_MUJOCO_ROOT\bin\mujoco.dll"
cargo +1.99.0-x86_64-pc-windows-msvc run --offline --manifest-path <temp-pkg>\Cargo.toml -- `
  '<repo>' '<repo>\target\g03-precheck-dump\report.json'
```

报告 `report.json` 已收入本目录（19 KB）：本库/上游解算、逐元素误差、与
10/12 trace 的对比。关键值与 `all-probes` 的 NP03 detail 一致
（1.3181156248560328 / 1.1674567649562604）。

### 2) 历史提交对照（预检实测）

```powershell
git worktree add <repo>\target\wt-before '8a5bac0~1'
git worktree add <repo>\target\wt-after '8a5bac0'
$env:CARGO_TARGET_DIR = '<repo>\target\bisect-rotate'   # 两个 worktree 共用，节省依赖编译
$env:CUDA_PATH = '<repo>\target\toolchains\cuda-12.8.1'
$env:PATH = "$env:CUDA_PATH\bin;$env:PATH"
$env:MJWARP_MUJOCO_ROOT = '<repo>\target\toolchains\mujoco-3.12.0\package'
$env:MJWARP_MUJOCO_DLL = "$env:MJWARP_MUJOCO_ROOT\bin\mujoco.dll"
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --test mass_solve_probe `
  -- --ignored --test-threads=1 --nocapture
```

结果：`c74aede` 6 过 0 败；`8a5bac0` 3 过 3 败。原始日志（本机、gitignored）：
`target/bisect-before-rotate.log`、`target/bisect-after-rotate.log`。

### 3) 当前 master 的失败明细

`target/g01-acceptance/9bd3489f5b4542409f104f5feac1c4fb/all-probes.log`：
NP03 失败块数 factors-and-rhs 2/240、native-snapshots 2/240、rhs-counts 6/152。

## 边界

- 只做预检：未改产品代码、测试、容差、fixtures、脚本；未实现任何修复。
- 不宣称 G03、NP01–NP04、release 或 Linux 验收通过。
- 未运行完整双平台矩阵；未重制任何冻结参考；未启用 MathDx 或 nvJitLink。
- 历史 `10/12` 的 0 失败结论取自既有 acceptance 证据，本次只做对照，不重新宣称其验收范围。
