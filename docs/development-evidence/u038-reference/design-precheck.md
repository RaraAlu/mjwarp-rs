# U038 独立流体与原子对照 · 设计预检

任务：`2026-10-16-next-u038-reference`。日期：2026-10-23。
验收（原任务逐字）：①补真实冻结流体设备输出，覆盖非精确多体原子归约；②不使用候选中间量伪装独立参考，保留原始位型与失败。
冻结上游：MJWarp `71da24d956378a87a703b6e1442b13aec0c4ac29`，Warp 1.15.0。
容差：2e-6 冻结容差不变；不改产品内核/PTX/断言/fixtures 语义/ABI。

## 1. 现状（基线事实）

- `fixtures/gravity-upstream-abi/generated/` 已存 passive 模块冻结产物（`module.cu` sha256 `dd9338e9…`、
  `module.ptx` sha256 `d06154da…`），其中含 `_gravity_force_7c63ac28…` 与 `_fluid_force_83bf3546…` 两个入口。
- `fixtures/gravity-upstream-abi/cache-quarantine/…/smooth.ptx`（sha256 `4179acd0…`）已存 smooth 模块冻结产物，
  含运动学/质心/comvel/cdof 内核，可用于生成上游中间量。
- 现有测试标记的真实缺口：`gpu_matches_independent_ellipsoid_reference` 报告 `frozen_upstream_device_output: false`；
  `gpu_matches_independent_multi_body_reduction_certificate` 报告 `frozen_upstream_device_output: false`、
  `nonexact_reduction_order_verified: false`（证书 28 槽中 2 槽 4 ULP，未越容差；原严格位断言失败保留）。
- 离线 authoring 框架已就绪：`scripts/authoring/author-u038-3d.py` 直接 `wp.launch` 上游内核并写夹具；
  pinned venv（Python 3.12.13 / Warp 1.15.0 / MuJoCo 3.15.0）与 upstream clone 均在位且 clean。

## 2. 本任务设计

### 2.1 离线 authoring（一次性生成夹具，产品测试不启动 Python）

新增 `scripts/authoring/author-u038-reference.py`（+ `.ps1` pinned-env 包装），两个场景：

1. **流体场景**（共享流体/椭球分支）：固定配置（多 body、多 geom，含 `body_fluid_ellipsoid` 真/假、
   含 0/负 `geom_fluid` 系数跳过、多周期 wind/density/viscosity/body_mass/body_inertia/geom_size），
   - 用 smooth 冻结内核生成上游中间量：`xpos/xquat/xmat/xipos/ximat/subtree_com/cdof/geom_xpos/geom_xmat/cvel`；
   - 用 passive 冻结内核 `_fluid_force_83bf3546…` 执行真实设备，读出 `fluid_applied` 原始位模式；
   - 天然校验：本机缓存重生成产物 sha256 与仓内冻结产物一致；两次运行逐位一致；独立 f64 复算同一上游公式
     与设备输出在 f32 容差内吻合；零速度/零密度边界为精确零。
2. **多体原子归约场景**（非精确归约）：5 body / 4 铰链、两棵树，多个 body 贡献同一 dof；
   - smooth 冻结内核生成 `xipos/subtree_com/cdof`，`body_isdofancestor` 由 `mujoco_warp.put_model` 天然校验；
   - passive 冻结内核 `_gravity_force_7c63ac28…` 执行真实设备，读出 `qfrc_gravcomp` 原始位；
   - 天然校验：重生成产物哈希一致；多次运行记录原位型（若原子顺序导致位差，如实记录，不伪造逐位一致）；
     独立 f64 复算上游公式与设备输出在容差内吻合。

夹具写入 `fixtures/gravity-upstream-abi/u038-reference/{fluid,atomic}/`：`model.json`/`input.json`
（f32 十六进制位）、`output.json`、`raw-bits.json`、`natural-validation.json`、`authoring.json`、`receipt.json`、
生成脚本副本与日志。`authoring_only=true`、`product_python_dependency=false`。

### 2.2 产品侧定向 GPU 对照（测试代码，不改产品逻辑）

1. **流体**：新增 `src/physics/fluid_force/tests/frozen_device.rs`（`src/physics/fluid_force.rs` 仅加
   `#[cfg(test)]` 模块注册）：按夹具 `model.json` 构造产品模型/参数，走共享常驻链
   （`KinematicsPlan` + `ResidentComVelocityPlan` + `FluidForcePlan::from_resident`），
   与冻结上游 `_fluid_force` 设备输出逐体比较 body 力/力矩；容差 2e-6 不变；
   输出首差、最大误差、位差、原始位；报告 `frozen_upstream_device_output: true`。
2. **原子归约**：`src/physics/gravity_compensation/tests.rs` 追加一条定向 GPU 测试：
   同一多体模型/参数，读夹具冻结上游 `_gravity_force` 设备输出，与产品 `GravityCompensationPlan`
   输出比较；记录位差/ULP/首差，容差 2e-6 不变；保留证书严格位断言失败记录；
   报告 `frozen_upstream_device_output: true`、`nonexact_reduction_order_verified` 按实测填写。
   原 `gpu_matches_independent_multi_body_reduction_certificate` 不改断言与容差。

### 2.3 证据与回执

`docs/development-evidence/u038-reference/`：本预检、README（参考来源/天然校验/对照结果/首差/未验收边界）、
fluid/atomic 报告 JSON、生成回执副本；回执 `docs/development-evidence/changes/2026-10-16-next-u038-reference.json`。

## 3. 约束与不改项

- 不改产品内核字符串/PTX、容差、断言、既有 fixtures 语义、ABI、调度；不新增依赖；产品测试不启动 Python。
- 新增/改动的产品源码仅 `#[cfg(test)]` 测试模块（新测试文件 + 两行注册）；内核/plan/ABI 源文件列入 unchanged 检查。
- 新增文件 ≤500 行；超长文件按模块拆分；离线 authoring 只用 pinned venv 与冻结提交。

## 4. GPU 与资源

- 离线 authoring：pinned Python 跑冻结上游内核，单次秒级、显存 <2GB，跑完立即释放。
- 产品定向测试：`--lib` 定向 ignored 测试，RTX 4070 Ti SUPER，单次单测级；不并发大任务。
- 与 QA 报备窗口（约 09:20–10:10），不并行其它 GPU 满载任务。

## 5. 验证计划

1. authoring：pinned env 自检、冻结产物哈希复核、双跑逐位、天然校验（含 `put_model` 矩阵/独立 f64 复算）。
2. 产品：流体/原子各 1 条定向 GPU 测试 0 失败；相关模块定向回归；`cargo test --locked --all-features --no-fail-fast` 不回归。
3. 门禁：gate fmt/clippy/clippy-cuda/unit；unchanged 检查；json 回执；forbid_claim。
4. 证据与回执如实记录未验收边界；不宣称完整 U038/精度/NP 验收。

## 6. 未验收边界（预检期）

- 仅覆盖冻结上游路径可表达的流体（椭球+惯性盒）与多体原子归约子集；不覆盖完整流体物理、完整 U038/G04。
- 不改产品数值路径，不触发 T4 源码身份重钉（除非测试侧源码登记需刷新）。
- 不跑 Linux/release/性能；NP01–NP04 与完整上游等价保持未验收。

## 7. 实施修订（生成后补记 2026-10-23）

- 实际夹具文件名为 `scenario.json`、`intermediate.json`、`output.json`、`natural-validation.json`、
  `authoring.json`、`receipt.json` 加 5 个生成脚本副本；预检 §2.1 中的 `model.json`/`input.json`/`raw-bits.json`
  合并为上述形式（输入在 scenario、原始位在 output/intermediate、回执单列）。
- 原子场景按“非精确归约”要求取非 2 幂质量与不可精确表示的 gravcomp/gravity；独立 f64 复算偏差 `5.91e-08`≠0。
- 修正 `body_subtreemass` 派生 bug（应为子树质量反向累加，原先按子体自身质量累加），
  修正前生成的夹具未入库；详见 README §3.1。
- 天然校验按实际实现收敛为 README §3（拓扑由 `put_model`、设备输出双跑/double-run、独立 f64 复算、零边界）。

