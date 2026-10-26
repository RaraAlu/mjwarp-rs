# G10@P5 收口（G6）：验收映射 / 性能 / native 容差落点

- 任务：`2026-10-25-g10-p5-acceptance-perf-native`（P5 G6，Arch 起草、Chief 认可；依赖 G5 设备内核、G5 溢出位修正、宿主溢出位修正、`.bin` 卫生独立复核均 verified）
- 状态：**进行中**——G6-1 映射与 G6-3 native 框架已落；G6-2 性能待独占 GPU 窗口批准后采集（本 README 与 `receipt.json` 在窗口后回填数字）
- 上游 pinned：`71da24d956378a87a703b6e1442b13aec0c4ac29`
- 证据纪律：**只读既有 verified 证据与 receipt 的原文/数字，不复算、不改数字**；本目录所有数字均可点回 `map.json` 的 `evidence_paths`

## 1. G6-1 验收映射（`map.json`）

roadmap 任务 `2026-10-11-roadmap-g10-p5` 的 6 条 acceptance 里，实质只有「地形接触、梯度与越界」，其余为流程口径。本片映射为：

### 实质三条（拆分后 7 行）

| requirement_id | 内容 | 实现位置 | 状态 |
| --- | --- | --- | --- |
| G6-1.1 | 地形接触（host）：G4 hfield 三 case | `2026-10-25-g10-p5-hfield-host` | met：count 8/8、8/8、7/7，dist 全位级 0 ULP、pos 0.00 ULP@尺度、frame 72/72、72/72、63/63 位等 |
| G6-1.2 | 地形接触（设备）：G5 hfield prism CCD + `MAXCONPAIR=50→HFIELD(32)` | `2026-10-25-g10-p5-gpu-kernels` | **partial**：地形接触主判据（hfield-sphere 8/8、hfield-capsule 7/7 全字段位级）来自 G5 交付片；但 `HFIELD(32)` 的 overflow=32 实测只来自 QA 独立复核（G5 交付片 README §9 只写实现映射，未含 overflow=32 实测），与 G6-1.7 同一证据层级标准 |
| G6-1.3 | 梯度（host）：G3 SDF 梯度/下降 + `sdf-box` | `2026-10-25-g10-p5-sdf-host-r2` + `-sdf-relmat-fix` | met：80/80；dist 80/80 位等、pos 0.00 ULP@尺度、frame 720/720 位等 |
| G6-1.4 | 梯度（设备）：G5 SDF narrowphase | `2026-10-25-g10-p5-gpu-kernels` | met：sdf-box dist 80/80 位等；pos 203/240 位等（1.0 ULP@scale 0.09834，见 `result.json` + receipt `regression_after_fix`；README §11.7.2 全表为修前 202/240 旧值）；frame 675/720 位等（max_abs 2.384e-7） |
| G6-1.5 | 越界（设备）：OverflowType 位语义 | `2026-10-25-g10-p5-ccd-overflow-bit-fix`（`99c91469`） | met：强制 horizon 3 例 5/6 world 修前 272 → 修后 256，与上游位级相等 |
| G6-1.6 | 越界（宿主）：同上 + 真实容量门 | `2026-10-25-g10-p5-host-ccd-overflow-bit-fix`（`5a176fee`） | met：ccd()/ccd_pair 272→256；容量截断仍置 CCD(16)；G2/G3/G4/G5 回归不变 |
| G6-1.7 | 越界（容量截断）：`naconmax`/`naccdmax` | 设备 `contact.cu` + 宿主 `CcdCapacity` | **partial**：截断行为由 QA 独立复核（verified）以自造 case 覆盖并通过（`sdf-box-naconmax40`、`ellipsoid-box-naccdmax1`），G5 交付片自身未含截断 case |

### 流程四条（拆分后 4 行）

| requirement_id | 内容 | 状态与依据 |
| --- | --- | --- |
| G6-2.1 | 先补齐模块与写入范围 | **partial**：G2/G3/G4/G5 均 verified；但 multicontact 设备版、NVRTC launcher、primitive narrowphase 未实装（UNC-1/2/3） |
| G6-2.2 | 性能测试须独占GPU | **met**：一次获批独占窗口（HEAD `9c0a0480`，wall 5.5 s，峰值 +1171 MiB，守卫未触发，0.5 s 采样，跑前跑后 0 计算进程）；host/设备 kernel-only/设备端到端/上游 f32 CPU 四列已落 `perf/`，覆盖缺口显式 |
| G6-2.3 | 记录真实测试与未测项 | met：G2-G5/G6 各 receipt 的 verification 为实测命令输出、unmet 明确列出未覆盖 |
| G6-2.4 | 先核对 NP01/NP02，再验收 NP03/NP04 | **partial**：NP01/NP02 按 `g10-p5-precheck` 与契约核对；NP03 对上游 f32 侧完成，native 侧接触字段按用户拍板 B 只作诊断（契约 `09896792`）、其 5% 红线不适用，其它字段 5% 红线未全量；NP04 待 G6-2.2 |

### native 侧处置（1 行，随 G6-3 拍板 B 落地）

| requirement_id | 内容 | 状态与依据 |
| --- | --- | --- |
| G6-3.1 | native 侧接触字段：用户 2026-10-25 拍板 B（待验收不放行、只作诊断） | **met（处置已落地，不提供放行信号）**：契约 `docs/numerical-precision-contract.md` `## 接触字段（native 侧）`（提交 `09896792`）+ 证据 `docs/development-evidence/native-contact-tolerance-decision-b/`；E 复算样本量 = `ellipsoid-box` 1 case / 2 接触点；`hfield-box` 仍不可配对、样本不增 |

### 未覆盖六项（显式；UNC-1/2/3 已按设备三缺口片更新为 partial，范围见下）

1. multicontact 设备版（UNC-1，**partial**）：`9a282074` 已实装 box/cylinder multicontact 并与上游接触数组逐字段对照（`cylinder-box` 314/314 位级、`box-box-margin` 391/392 位级）；**mesh multicontact 未实现**（需 mesh polygon ABI）。
2. NVRTC 产品 launcher（UNC-2，**closed（2026-10-26，Chief 批准；限定语见 `map.json` 的 `UNC-2.closure.qualifier` 与 `docs/development-evidence/g10-p5-cuda-default-pipeline-stage2/README.md` §7.1）**）：`dfcc1bdc` 已实现并经 `cuda-probe` 特性实际调用，与 G5 探针三 case 逐字节一致（QA `bcd3c48b`）；**默认构建/默认产品管线未接入**。
3. primitive narrowphase 设备版（UNC-3，**partial**）：**组合层 13/13 已实现且各有上游 GPU 对照 case**——既有 `9d72159c`（+ `9a282074` 的 `box_box`）覆盖 sphere-box/capsule-box/box-box；B1 `249a4dbd` 覆盖 plane 6 组合（6/6）；B2 `6ab5cb22` 覆盖 sphere/capsule 4 组合（4/4）。**分支层部分覆盖**（8 类分支未覆盖，台账见 §6）。**未宣称全路径 / 全分支。**
4. Linux/T4 未覆盖（UNC-4；用户 2026-10-24 裁决 Linux 集中回测）
5. G5 窗口逐秒峰值未取到（UNC-5；Start-Job 相对路径缺陷，脚本已修，G6 性能窗口按 0.5 s 采样另列）
6. `cylinder-box` 深穿透额外采样面 4/24 单字段超 4·ULP@scale（UNC-6；x1 1/24 max 8.926e-7、x2 3/24 max 1.527e-7）——**用户 2026-10-25 拍板 A：不修、列未覆盖**。定性归因见 `deep-epa-witness-sampling`（`c44578be` + 措辞修正 `ed88495d`，QA 复核 `2c052637`）：非系统性缺陷，根因收窄为「派生法线差（机制未定）」，1/24 明显超 + 2/24 边界级，与深度无显著线性相关（n=24）。**声明面判据不调整**（witness 判据保持原样，x1/x2 不降级为信息量）；若未来要覆盖该面，需补 `face_idx`/三顶点/权重数据并另立片。

每行的判据原文、实测数字、证据路径、原因与影响见 `map.json`。

**状态统计（12 条 requirement）**：met 8（含 G6-2.2 性能与 G6-3.1 native 侧处置）/ partial 4（G6-1.2、G6-1.7、G6-2.1、G6-2.4）/ unmet 0；另 uncovered 6 条均为 unmet。`map.json.summary` 与 `receipt.json.verification.map_summary` 同此口径。

## 2. G6-2 性能（已采集，独占 GPU 窗口）

**窗口事实**：HEAD 绑定 `9c0a0480`（相对 `6510317f` 仅 docs/治理提交，`git diff 6510317f..HEAD -- src tests Cargo.toml Cargo.lock` 为空）；基线 used 3663 / 16376 MiB、util 0%、29°C；0.5 s 采样 7 点，峰值 **4834 MiB（+1171）**，守卫 3072 MiB/50% 未触发；跑前跑后本任务相关计算进程 0；window wall **5.5 s**；桌面负载（395 进程、`vmware-vmx` 约 17.5 GiB 等）与硬件/驱动/nvcc 版本已入 `perf/README.md` §1。

| 列 | 覆盖 | 关键中位（ms） |
| --- | --- | --- |
| host 纯 CPU 单线程 | 9 case（G2 实际 harness 五 + `sdf-box` + hfield 三） | 0.010（mesh-mesh）～2.697（sdf-box） |
| 设备 kernel-only（CUDA events） | 7 case（冻结 G5 case 目录） | 0.021（hfield-sphere）～4.774（sdf-box） |
| 设备端到端（含 H2D/D2H） | 同上 7 case | 0.538（cylinder-box）～5.060（sdf-box） |
| 上游 f32 CPU 参考 | 9 case（含 `box-box-margin`，缺 `ellipsoid-box`） | 0.645（box-box-margin）～11.722（sdf-box） |

- 四列覆盖缺口（host 缺 `box-box-margin`、设备缺 `convex-mesh`/`degenerate-mesh`/`hfield-box`、上游缺 `ellipsoid-box`）在 `perf/README.md` §4 显式记录，**不补齐、不跨列外推**。
- 设备端到端列对 20–50 µs 的内核是传输主导（输出按产品形状固定 `naconmax=8192` 行 D2H）；设备计算量看 kernel-only 列。host 计时无 CPU 隔离；窗口 5.5 s，峰值采样 7 点。
- 编译/JIT/加载时间不计入任何列；**性能不进精度判据、不跨硬件外推、不用于 NP03/NP04 放行声明**（NP04 仅表示「已记录耗时与显存」）。完整数字、解释与复现命令见 `perf/README.md` 与 `perf/perf-summary.json`。

## 3. G6-3 native 容差落点（用户 2026-10-25 拍板 B）

见 `native-framework.md`：**用户 2026-10-25 拍板选项 B（经 Chief 转达）**，契约已落地 `docs/numerical-precision-contract.md` `## 接触字段（native 侧）`（提交 `09896792`）——`dist`/`pos`/`frame`/`normal` 的 native 侧**只作诊断、不参与通过或失败**，**不写任何生效 atol/rtol**；接触精度仍由对上游 f32 判据承担（不表示接触字段没有精度要求）；红线 `E_candidate <= 1.05*E_mjwarp` 只在其它字段生效。E 复算照做并标注样本量（`ellipsoid-box` 1 case / 2 接触点，诊断性质、不可外推）；`hfield-box` 的 native 每 world 接触数不可配对（native 桥只暴露模型字段拷贝，无 contact/narrowphase 输出）→ 如实记录「不可配对、样本不增」。A（粗门禁）留待真 native 样本扩充后再议。

## 4. 文件

| 文件 | 内容 |
| --- | --- |
| `map.json` | 12 条 requirement 行 + 6 条 uncovered 行（requirement_id/实现位置/判据原文/实测数字/证据路径/状态/原因/影响） |
| `native-framework.md` | G6-3 native 落点：用户拍板 B 收口（诊断不参与判定、不落生效容差）+ `hfield-box` 不可配对 |
| `perf/README.md` | G6-2 性能证据：窗口事实、四列口径、数字、覆盖缺口、解释与限制、复现命令 |
| `README.md` | 本文件（方法、摘要、未覆盖、证据纪律） |
| `receipt.json` | 任务回执（task_id、映射摘要、perf/native 状态、gate、unmet） |

## 5. 边界与不得声明

- 不改已批准判据（对上游 f32 的向量尺度 ULP / frame scale=1.0 / 结构零容差）、不改已 verified 证据数字、不改 S2–S8 证据、`Cargo.*`、`fixtures/mjwarp-complex-physics/*.xml`。
- NP03 其它字段 5% 红线全量完成前，**不得声明 G10@P5 整体通过 / 精度达标 / 上游完全等价 / 双平台通过 / 原生放行 / 接触字段精度已达标 / 性能达标**；native 侧只写「待验收、诊断读数见证据」，性能只写「已记录耗时与显存（独占窗口）」。
- 本 map 只做证据映射；G6 独立复核（QA）将独立复算 met/partial，并核对本轮契约 B 措辞与 `09896792` 一致、没有把诊断写成放行。

## 6. UNC-3 更新（2026-10-26；B1/B2 收口后治理）

**组合层 13/13**（每个组合都有上游 GPU 参考 case；判据：位级优先 / 非位级 4 ULP 口径 / 结构零容差 / 行序按结构键配对）：

| 组合 | 覆盖片（提交） | case（nacon） | 结果 |
| --- | --- | --- | --- |
| plane-sphere / plane-capsule / plane-ellipsoid / plane-cylinder | B1 `249a4dbd` | 各 1（nacon 1） | 4/4 pass |
| plane-box | B1 `249a4dbd` | 1（nacon 3） | pass |
| plane-mesh | B1 `249a4dbd` | 1（nacon 3） | pass（穷举分支，4 顶点） |
| sphere-sphere / sphere-capsule / sphere-cylinder | B2 `6ab5cb22` | 各 1（nacon 1） | 3/3 pass |
| capsule-capsule | B2 `6ab5cb22` | 1（nacon 1） | pass（平行分支；GPU 侧 det≈2.6e-12，见 B2 README §6） |
| sphere-box / capsule-box | 既有 `g10-p5-primitive-narrowphase-cuda`（`9d72159c`） | 3 | pass（80/80 位级） |
| box-box | 既有 `g10-p5-multicontact-cuda`（`9a282074`） | 1（5 接触/world） | pass（391/392 位级 + pos 1 ULP） |

**分支层部分覆盖（8 类分支未由 fixture 触发；不宣称已验证）**：
1. `plane_convex` graph 分支（mesh `vertnum≥10`；B1 fixture 为 4 顶点走穷举分支）；
2. `sphere_sphere` 零距离 fallback（`dist==0`）；
3. `sphere_capsule` 投影 clamp 路径（B2 case proj=0，未 clamp）；
4. `sphere_cylinder` 端面 / 角点分支（B2 case 只触发侧面分支）；
5. `capsule_capsule` non-parallel 分支（B2 fixture 在未融合 f32 下 det=0 → 平行 4 端点；GPU 侧因 FMA 收缩走非平行但只写 1 接触）；
6. `plane_cylinder` degenerate x 轴回退（len_sqr 未触发兜底）；
7. `plane_ellipsoid` 非上游退化保护（零长/非有限未触发）；
8. `plane-box` 8 角点中 5 个未 detected；`plane-capsule` / `plane-cylinder` 其余接触候选未 detected。

**其它未覆盖路径**：交换几何顺序（调用方按 `(capsule,sphere)` 等传入）的分派已实现但无 case 覆盖；device `contact.geom` 用调用方顺序、pinned 按类型序规范化，仅同序时一致（QA B2 观察 O5，记后续覆盖清单）。

**债务**：sphere-box / capsule-box / box-box 的分支级未逐条标注（既有片只到 case 层）。

**证据指针**：B1 `docs/development-evidence/g10-p5-primitive-plane-cuda/`（§5 窗口结果、§9 未覆盖、§12 QA 观察）；B2 `docs/development-evidence/g10-p5-primitive-sphere-capsule-cuda/`（§6 判据与机制、§9 分支、§10 窗口结果、§11 行序修复、§12 QA 观察收口）；既有片 `g10-p5-primitive-narrowphase-cuda/`、`g10-p5-multicontact-cuda/`；独立复核 `...-plane-cuda-independent-check/`（pass）、`...-sphere-capsule-cuda-independent-check/`（pass）。

**边界**：UNC-3 仍为 **partial**；只写「组合层 13/13、分支层部分覆盖」，不宣称全路径 / 全分支 / 跨平台；精度与等价性结论仅限上表 case 与已批准判据内。
