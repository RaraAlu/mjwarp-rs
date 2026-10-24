# U038-3D 重力传播超差债务 · 有界复跑与对齐判定（debt-2）

任务：`2026-10-23-u038-3d-gravity-propagation-debt-2`（用户 2026-10-24 选择扫 Windows 小项后启动）。
日期：2026-10-24。执行：Backend。复跑 HEAD：`245c461`（本任务开工前）。
范围：首差证据 + 对齐判定；**未改产品源码、测试、容差、断言、fixtures、排除清单、ABI**。

## 1. 结论（先给）

1. **复跑结果不变**：1477 项、**122 位差**、**8 项超差**、max **4.291534423828125e-6**，容差
   `2e-6` 未改；`gravity_operator`（同输入算子对照）77/77 **0 超差**（1 位差、max 1.19e-7），
   `gravity_source_propagation` **8 超差**。与 2026-10-23 的 run5/closure 数字在总量与字段级一致；`gravity_operator` 统一按「0 超差、位差 0~1」表述（run5 位差 0、本次位差 1，均 0 超差，见 §11）。
2. **首差已逐字段/逐 world/逐 dof 定位**（`first-diff.json`）：
   - 首个运动学差异在**父姿态旋转路径**：w2 body3 的 `xpos/xipos` 2 ULP（`xquat/xmat` 在该 world
     逐位一致），`xanchor` 继承该差异；对应表达式为 `rotate(parent_xquat, body_pos)`。
   - 第二类差异在**四元数组合**：仅 w6 出现 `xquat` 7 处（1–3 ULP）与 `xmat` 17 处（1–2 ULP）；
     对应 `multiply(parent_quat, body_quat)` 与由 `xquat` 生成的矩阵。
   - `xaxis` 只在 w4/w6 出现（铰链轴），它不进入重力公式输入。
   - `subtree_com` 的差异大于子体差异（w2 root z 11 ULP、w3 root z 82 ULP，子体仅 1–2 ULP）：
     属**累加顺序**（上游分层原子累加 vs 产品单内核反向顺序累加）。
   - `cdof` 55 位差：自由关节只在**线性分量**（12），球/铰链在角度+线性分量（30+13），
     与 `xmat`、`xipos - subtree_com` 的 offset 一致（上游 `_cdof` 用 xmat 列与 cross）。
3. **因果关联（逐 world 矩阵）**：w0/w1/w5 运动学八字段全一致且 `qfrc` 全一致；w2/w3/w6 的
   重力输入字段（`xipos/subtree_com/cdof`）差异与 `qfrc` 差异同域（19 位差 / 8 超差全部落在
   w2/w3/w6）；w4 只有 `xaxis` 与 `cdof` 少数分量差异，`qfrc` 全部逐位一致。
4. **灵敏度定量**：重力项 `|f| = mass·gravcomp·|g|`，本场景 body3 达 **69.2**（body1/2/4 为
   14.8/12.4/12.4）；`Δqfrc ≈ Σ Δj·f`，1 ULP 的 `Δcdof`（5.96e-8）乘 body3 的力臂即可产生
   ~4e-6 量级差。要 1477 项 0 超差，受影响的 `xipos/subtree_com/cdof` 必须基本**逐位一致**
   （body3 项要求 `Δj ≤ ~2.9e-8`，即小于当前最大 1.79e-7 的 1/6，且其余项需 0 位差），
   **不是"减小一点误差"能达到**。
5. **对齐判定：在不改容差/断言/fixtures 语义、最小改产品源码的约束下不可达**（依据见 §6）。
   触发因素有两类且相互独立：内联上下文决定的**混合 FMA 收缩**（前两轮有界已实测自然写法无效、
   全显式非融合更差）与 **subtree_com 的内核分解/原子累加顺序**；逐调用点显式算序还需要目前
   不存在的「PTX 调用点窗口切片」工具（现有工具只能分离 5 个 `rotate` 调用点中的 3 个）。
6. **未宣称**：U038-3D 未通过、精度未验收、完整三维状态空间未验收；未做 T4 repin（产品源码未改，
   源码身份未漂移）。

## 2. 复跑（当前 HEAD，本次实测）

命令（GPU 独占；本机 RTX 4070 Ti SUPER）：

```powershell
$env:CUDA_PATH = 'D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1'
$env:PATH = "$env:CUDA_PATH\bin;D:\Rust\mjwarp-rs\target\toolchains\mujoco-3.12.0\package\bin;$env:PATH"
$env:MJWARP_CUDA_INCLUDE = "$env:CUDA_PATH\include"
$env:MJWARP_MUJOCO_ROOT = 'D:\Rust\mjwarp-rs\target\toolchains\mujoco-3.12.0\package'
$env:MJWARP_MUJOCO_DLL = "$env:MJWARP_MUJOCO_ROOT\bin\mujoco.dll"
$env:MJWARP_GRAVITY_3D_EVIDENCE_DIR = 'D:\Rust\mjwarp-rs\target\u038-3d\debt2\final'
$env:MJWARP_U038_DUMP_PRODUCT = '1'
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --test t4_gravity_3d_product_chain -- --ignored --test-threads=1
```

- 测试在最后一行断言（`total_outside == 0`）失败于 `left: 8, right: 0`，属**预期**：本任务目标
  是取证与判定，不是宣称通过。`run.json` 在断言前写出，身份校验（package/receipt/raw-bits 哈希、
  `product_sources` 的 physics/resident/gravity 源码摘要）全部通过。
- 运行目录 `target\u038-3d\debt2\final\`（gitignored），证据副本见本目录 `run.json` 等。

| 字段 | 值数 | 位差 | 超差 | max_abs |
| --- | ---: | ---: | ---: | ---: |
| xpos | 105 | 4 | 0 | 5.96e-8 |
| xquat_wxyz | 140 | 7 | 0 | 5.96e-8 |
| xmat | 315 | 17 | 0 | 1.79e-7 |
| xipos | 105 | 4 | 0 | 5.96e-8 |
| xanchor | 84 | 3 | 0 | 5.96e-8 |
| xaxis | 84 | 4 | 0 | 1.19e-7 |
| subtree_com | 105 | 9 | 0 | 8.94e-8 |
| cdof | 462 | 55 | 0 | 1.79e-7 |
| qfrc_gravcomp | 77 | 19 | **8** | **4.29e-6** |
| 合计 | **1477** | **122** | **8** | 4.29e-6 |

`gravity_operator`：77 值、位差 1、0 超差、max 1.19e-7；`gravity_source_propagation`：
77 值、位差 20、8 超差、max 4.29e-6。

## 3. 首差证据（逐字段，`first-diff.json`）

比较口径：产品原始 f32 位（测试 `MJWARP_U038_DUMP_PRODUCT` 导出）vs 冻结上游原始位
（`fixtures/gravity-upstream-abi/three-d-kinematics/raw-bits.json`），逐项按
world/body/joint/dof/component 映射（脚本 `scripts/authoring/compare-u038-first-diff.ps1`）。

| 字段 | 位差 | 首个差异（索引 → 位置） | 产品位 → 上游位 | ULP |
| --- | ---: | --- | --- | ---: |
| xpos | 4 | #39 → w2 body3 x | `3ecdb974` → `3ecdb976` | 2 |
| xquat_wxyz | 7 | #128 → w6 body2 w | `3f3cf802` → `3f3cf803` | 1 |
| xmat | 17 | #288 → w6 body2 (0,0) | `3f35d1d8` → `3f35d1da` | 2 |
| xipos | 4 | #39 → w2 body3 x | `3eda4472` → `3eda4474` | 2 |
| xanchor | 3 | #30 → w2 joint2（hinge, body3）x | `3f038dad` → `3f038dae` | 1 |
| xaxis | 4 | #55 → w4 joint2（hinge, body3）y | `bf3e9e5f` → `bf3e9e60` | 1 |
| subtree_com | 9 | #32 → w2 body0 z | `3d1418c3` → `3d1418b8` | 11 |
| cdof | 55 | #153 → w2 dof3（free body1）linear x | `be1eae9d` → `be1eae9c` | 1 |

**因果次序**（w2 是首个可观测差异 world）：w2 的 `xquat/xmat` **逐位一致**，但
`xpos/xipos`（body3，2 ULP）先差 → 差异来自 `rotate(xquat[parent], body_pos)` 这一父姿态
旋转调用点，不是四元数组合；`xanchor`、`subtree_com`、`cdof` 的 w2 差异都是它的下游
（`xanchor = xpos + rotate(xquat, jnt_pos)`、`subtree_com` 累加 `xipos`、`cdof` 用
`xipos - subtree_com` 的 offset）。

全部 122 位差（8 字段）逐项清单（索引/位置/两侧位型/浮点值/ULP）见 `first-diff.json`
的 `fields[].differences`；重力 19 位差与 8 超差逐 dof 清单见 `gravity.differences_by_dof`。

## 4. 逐 world 关联矩阵（`first-diff.json` 的 `world_diff_matrix`）

| world | xpos | xquat | xmat | xipos | xanchor | xaxis | subtree_com | cdof | qfrc 位差 | qfrc 超差 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| w0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| w1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| w2 | 2 | 0 | 0 | 2 | 2 | 0 | 5 | 14 | 5 | 3 |
| w3 | 1 | 0 | 0 | 1 | 0 | 0 | 4 | 14 | 7 | 4 |
| w4 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 3 | 0 | 0 |
| w5 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| w6 | 1 | 7 | 17 | 1 | 1 | 3 | 0 | 24 | 7 | 1 |

- 重力输入 = `xipos`/`subtree_com`/`cdof`；`qfrc` 差异的 world 集合 {w2,w3,w6} 与这三项
  非零差异的 world 集合一致（w4 的 `xaxis`/`cdof` 少数差异未放大到超差，`xaxis` 不进重力公式）。
- `subtree_com` 差异分布：w2 body0 z 11 ULP（子体仅 1–2 ULP）、w3 body0 z 82 ULP（值为
  1e-3 量级，绝对差仍小），根体差异大于子体 → 分层累加顺序差异的直接证据。

## 5. 重力传播灵敏度（为何需要"逐位一致"）

产品/上游重力项（`src/physics/gravity_compensation/cuda.rs`，与上游 `_gravity_compensation` 同式）：

```text
f_b  = -gravity * mass[b] * gravcomp[b]            # 每体 3 分量，|f_b| = mass·gravcomp·|g|
j_b  = cdof[3d..3d+6] 的线性部分 + cross(cdof[3d..3d+3], xipos[b] - subtree_com[root[b]])
qfrc[d] = Σ_b (j_b · f_b)                          # atomicAdd
```

- 本场景 `|g| = 9.8877`，`|f|`：body1 14.83、body2 12.36、**body3 69.21**、body4 12.36。
- `Δqfrc ≈ Σ_b Δj_b·f_b`；`Δj` 来自 `Δcdof`（最大 1.79e-7 = 3 ULP）与
  `Δ(xipos-subtree_com) × cdof_angular`。1 ULP 的 cdof 差异乘 body3 的力即 ~4.1e-6 量级，
  与实测 8 项 2.44e-6–4.29e-6 同量级（部分抵消后）。
- 要全部 `< 2e-6`：body3 项要求 `Δj ≤ 2.9e-8`（小于 0.5 ULP@1.0），且其余 3 体近 0 位差；
  当前 `Δj` 最大 1.79e-7、且 19 个 qfrc 位差分布在 3 个 world。
  **结论：需要受影响的 xipos/subtree_com/cdof 基本逐位一致，而不是缩小若干 ULP。**

## 6. 对齐判定：最小改产品源码不可达（依据）

1. **根因一：混合 FMA 收缩由内联上下文决定**（`ptx-diff/README.md` §"关键发现"）：
   - 受控同包装下 `rotate/multiply/normalize` 产品与上游生成源码的指令计数与融合数**完全一致**
     （rotate 30 条算术 13 fma、multiply 22 条 6 fma、normalize 10 条 3 fma），不存在表达式写错；
   - 但完整内核里 `rotate` 的 5 个调用点中，产品探针模式只命中 3 个，上游 `_kinematics_branch`
     侧 q-first 模式 0/5；同一上游源码只换包装加载顺序即 **13 fma → 10 fma**，而产品源码与
     "照上游语句重写"的候选转录**不敏感**（恒 13 fma）→ 源级转录无法改变收缩。
   - 前两轮有界尝试已实测：第 1 轮源形态改写（run7）8/77 不变、位差 122→123；第 2 轮全显式
     非融合 PTX（run8）8/77 不变、位差 122→147（更差），因为上游实际是**混合收缩**
     （`_kinematics_branch` 99 fma / 284 条 f32 算术）。
2. **根因二：subtree_com 的内核分解/累加顺序**（同一证据）：上游 `_subtree_com_init`（mul 6）+
   `_subtree_com_acc`（**atom.global.add.f32 6** 按树层累加）+ `_subtree_div`（div 3），产品为
   单内核单线程反向顺序累加（0 原子）；本任务首差数据进一步显示根体差异（11/82 ULP）大于子体
   （1–2 ULP），即纯顺序/结合性差异，无法用 FMA 转录消除。
3. **逐调用点显式算序所需的工具尚不存在**：要按冻结 PTX 逐调用点写显式 `fma.rn`/`mul`/`add`
   序列，必须先切出每个调用点的指令窗口；现工具只能自动分离 5 个 `rotate` 调用点中的 3 个，
   `multiply` 3 个中 1–2 个。补该工具 + 逐内核转录 + subtree_com 分解属于新的系统性任务。
4. **改动不是"最小"**：`KINEMATICS_CUDA` 与 `COM_POSITION_CUDA` 被 resident/attached/质量链等
   多条产品路径共享（`src/physics/resident/plan_kernels.rs`、`src/physics/attached.rs` 等），
   任何算序/内核分解改动都会改变这些路径的产物，必须全量重验（G01 双模式、T4 锚点 repin、
   G26/G03/NP03/NP04 等）且**不保证** 1477/0。
5. **本任务有界纪律**：同类 PTX/数值修复的前两轮已耗尽且结论明确（不可由源级复现）；
   剩余路线需要新工具链与跨路径重验，不适合在本任务再"盲试第三轮"，故按验收第 3 条
   停手、登记选项、不宣称通过。

## 7. 选项（报 Chief/用户裁决）

| 选项 | 内容 | 代价/风险 |
| --- | --- | --- |
| A 系统性对齐 | 另立任务：PTX 调用点窗口切片工具 → 逐调用点显式算序转录 → subtree_com 分层累加对齐 → T4 repin → 全量重验 | 大；跨多个共享内核；不保证 1477/0；回归面广 |
| B 维持已知债务 | 维持用户 2026-10-23 裁决 B：8/77 登记已知债务、不宣称通过；本轮不改进产品数值 | 无代码风险；债务长期存在 |
| C 比较口径裁决 | 把 3D 全链验收拆成「重力算子同输入 0 超差」+「运动学字段各自容器」两段口径（需用户裁决修改验收口径；本任务未改口径） | 需用户裁决；不改产品数值 |

本任务不自行选择 A/C；维持 B 现状并如实登记，等裁决。

## 8. 未满足项与边界

- **未满足**：1477 项 / 0 超差未达成（8 超差、max 4.29e-6），验收目标未满足；不宣称通过。
- 未改产品源码/测试/容差/断言/fixtures/排除清单/ABI；未做 T4 repin（源码身份未漂移）。
- 未跑满额 G01/双模式、release/Linux、性能与完整三维状态空间；NP03/NP04 未宣称。
- 本任务只用到 GPU 做一次定向链复跑（约 1.1 s 测试内执行，含编译/加载），结束后已释放。
- W13、Linux、g03-parity 未触碰。

## 9. 复现

```powershell
# 1) 定向 GPU 链复跑（见 §2 环境变量；测试会因 8 超差按预期失败）
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --test t4_gravity_3d_product_chain -- --ignored --test-threads=1

# 2) 首差离线对照（只读证据文件，不跑 GPU）
target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -ExecutionPolicy Bypass `
  -File scripts\authoring\compare-u038-first-diff.ps1
```

## 10. 文件

| 文件 | 内容 |
| --- | --- |
| `run.json` | 本次复跑的机器可判定结果（字段统计、重力算子/传播、身份哈希） |
| `product-fields-raw.json` | 产品八字段原始 f32 位（测试内导出） |
| `product-gravity-raw.json` / `upstream-gravity-raw.json` | 产品/现算上游重力原始位（77 项） |
| `first-diff.json` | 逐字段/逐 world/逐 dof 首差对照（含 122 位差明细、重力 19 位差与 8 超差、world 矩阵、灵敏度） |
| `../../scripts/authoring/compare-u038-first-diff.ps1` | 首差离线对照脚本（只读） |
| `README.md` | 本文（结论、证据、判定与选项） |

历史证据（不改动）：`../README.md`、`../known-deviation.md`、`../ptx-diff/`、`../first-diff-round1.md`、
`../ptx-order-round2.md`、`../closure.md`。

## 11. 修正记录（2026-10-24，用户裁决 B）

- 用户 2026-10-24 裁决 **B：维持已知债务、不宣称 U038-3D 通过**；本任务判定「最小改动不可对齐」结论保留，复跑数字不变：1477 项 / 122 位差 / 8 超差 / max 4.291534423828125e-6；容差 2e-6（f32 位 0x360637BD）未动。
- 有界两轮 run7（源形态改写）与 run8（全显式非融合）记录在案：均 8/77 超差不变，位差 122→123 / 122→147（更差）。
- `gravity_operator` 统一表述为「**0 超差、位差 0~1**」（run5 位差 0、2026-10-24 复跑位差 1，max 1.19e-7，均 0 超差）；总量/字段级与 run5 一致。
- 本修正只改证据表述，不改产品源码/测试/容差/断言/fixtures/排除清单/ABI；`three-layer-acceptance` 留在 Windows 小项收口里程碑统一更新。
