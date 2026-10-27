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
| G6-2.1 | 先补齐模块与写入范围 | **partial**：G2/G3/G4/G5 均 verified；设备三缺口片已补——multicontact box/cylinder（`9a282074`）与 convex-vs-mesh（`16214a22`）、NVRTC 产品 launcher（`dfcc1bdc`，UNC-2 已关闭）、primitive 组合层 13/13（分支层部分）；仍缺 mesh-mesh 多接触（M3 在途）、primitive 分支层未覆盖 3 类、默认产品管线接线（见下方 uncovered 与 §6/§7） |
| G6-2.2 | 性能测试须独占GPU | **met**：一次获批独占窗口（HEAD `9c0a0480`，wall 5.5 s，峰值 +1171 MiB，守卫未触发，0.5 s 采样，跑前跑后 0 计算进程）；host/设备 kernel-only/设备端到端/上游 f32 CPU 四列已落 `perf/`，覆盖缺口显式 |
| G6-2.3 | 记录真实测试与未测项 | met：G2-G5/G6 各 receipt 的 verification 为实测命令输出、unmet 明确列出未覆盖 |
| G6-2.4 | 先核对 NP01/NP02，再验收 NP03/NP04 | **partial**：NP01/NP02 按 `g10-p5-precheck` 与契约核对；NP03 对上游 f32 侧完成，native 侧接触字段按用户拍板 B 只作诊断（契约 `09896792`）、其 5% 红线不适用，其它字段 5% 红线未全量；NP04 待 G6-2.2 |

### native 侧处置（1 行，随 G6-3 拍板 B 落地）

| requirement_id | 内容 | 状态与依据 |
| --- | --- | --- |
| G6-3.1 | native 侧接触字段：用户 2026-10-25 拍板 B（待验收不放行、只作诊断） | **met（处置已落地，不提供放行信号）**：契约 `docs/numerical-precision-contract.md` `## 接触字段（native 侧）`（提交 `09896792`）+ 证据 `docs/development-evidence/native-contact-tolerance-decision-b/`；E 复算样本量 = `ellipsoid-box` 1 case / 2 接触点；`hfield-box` 仍不可配对、样本不增 |

### 未覆盖六项（显式；UNC-1/2/3 已按设备三缺口片更新为 partial，范围见下）

1. multicontact 设备版（UNC-1，**partial**）：`9a282074` 已实装 box/cylinder multicontact 并与上游接触数组逐字段对照（`cylinder-box` 314/314 位级、`box-box-margin` 391/392 位级）；mesh 一侧经四片串行落地后，**convex-vs-mesh（`mesh-box`/`mesh-cylinder`）已覆盖并位级一致**（`16214a22`，各 4/4 位级、ULP 0、`contact.geom=[1,0]`）；**mesh-mesh 多接触已覆盖**（M3 `f1e5cda1`，nacon 4/4、bit_equal 4/4）；**一般 mesh 已按 S4 收窄**：`nmesh>1` 有 1 例验收覆盖（S4 U6 `mesh-mesh-two-assets`，nacon 3/3），`degree>10` / `polygon>16` 仅「限制存在」（S4 U7/U8），其中 **U8 如实记「设备 count=1 vs 上游 4」= 真实功能差距、不进分母**（backlog `2026-10-27-mesh-polygon-gt16-device-gap`，不 start）。mesh 侧口径拆细见 §7.3。
2. NVRTC 产品 launcher（UNC-2，**closed（2026-10-26，Chief 批准；限定语见 `map.json` 的 `UNC-2.closure.qualifier` 与 `docs/development-evidence/g10-p5-cuda-default-pipeline-stage2/README.md` §7.1）**）：`dfcc1bdc` 已实现并经 `cuda-probe` 特性实际调用，与 G5 探针三 case 逐字节一致（QA `bcd3c48b`）；**默认构建/默认产品管线未接入**。
3. primitive narrowphase 设备版（UNC-3，**partial**）：**组合层 13/13 已实现且各有上游 GPU 对照 case**——既有 `9d72159c`（+ `9a282074` 的 `box_box`）覆盖 sphere-box/capsule-box/box-box；B1 `249a4dbd` 覆盖 plane 6 组合（6/6）；B2 `6ab5cb22` 覆盖 sphere/capsule 4 组合（4/4）。**分支层部分覆盖**（S4 收口后口径见 §6.2：U1/U3/U4/U5/U6/U9 计「本窗验收覆盖」，U2 保持「不可达 / 非上游防御」，U7/U8 仅「限制存在」；mesh 边/顶点分支层仍只静态核对，见 S2 `branch-map.json`）。**未宣称全路径 / 全分支。**
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

## 6. UNC-3 更新（2026-10-26；B1/B2 收口 + B3 分支片收口后治理）

**组合层 13/13**（每个组合都有上游 GPU 参考 case；判据：位级优先 / 非位级 4 ULP 口径 / 结构零容差 / 行序按结构键配对）：

| 组合 | 覆盖片（提交） | case（nacon） | 结果 |
| --- | --- | --- | --- |
| plane-sphere / plane-capsule / plane-ellipsoid / plane-cylinder | B1 `249a4dbd` | 各 1（nacon 1） | 4/4 pass |
| plane-box | B1 `249a4dbd` | 1（nacon 3） | pass |
| plane-mesh | B1 `249a4dbd` | 1（nacon 3） | pass（穷举分支，4 顶点） |
| sphere-sphere / sphere-capsule / sphere-cylinder | B2 `6ab5cb22` | 各 1（nacon 1） | 3/3 pass |
| capsule-capsule | B2 `6ab5cb22` | 1（nacon 1） | pass（GPU 侧融合 det≈2.6e-12 → 非平行分支；平行轴分支未由 device 触发，见债务） |
| sphere-box / capsule-box | 既有 `g10-p5-primitive-narrowphase-cuda`（`9d72159c`） | 3 | pass（80/80 位级） |
| box-box | 既有 `g10-p5-multicontact-cuda`（`9a282074`） | 1（5 接触/world） | pass（391/392 位级 + pos 1 ULP） |

**分支层部分覆盖（B1/B2/B3 后；不宣称全分支）**：

已由 **B3** `e221e6c2`（QA `856b7835` pass；上游参考 `upstream-primitive-branches-gpu.json` sha `a8407a90…`）覆盖并通过：
- `plane_convex` **graph hillclimb** 分支（mesh `vertnum=10`、`graphadr=0`、`graph[0]=10`）；
- `sphere_sphere` **`dist==0` fallback**（两球心 f32 距离 bits=0 → `n=(1,0,0)`）；
- `sphere_capsule` **投影 clamp**（proj 0.15000000596 > half 0.119999997）；
- `sphere_cylinder` **端面 cap** 与 **角点 corner**（`side=false` 两态各自命中）；
- `capsule_capsule` **non-parallel det 分支**（det≈2.073600335e-4，远离 1e-15）；
- **swap 几何顺序**：device 按 pinned `_add_geom_pair` 类型序规范化（`type1 > type2` 交换），`contact.geom` 与上游同序，4/4 swap 变体通过；既有调用方源码级 no-op、13 组合判据内无回退。

仍未覆盖（3 类，不宣称已验证）：
1. `plane_cylinder` degenerate x 轴回退（len_sqr 未触发兜底）；
2. `plane_ellipsoid` 非上游退化保护（零长/非有限未触发）；
3. `plane-box` 8 角点中 5 个未 detected；`plane-capsule`/`plane-cylinder` 其余接触候选未 detected。

**债务**：sphere-box / capsule-box / box-box 的分支级未逐条标注（既有片只到 case 层）；`capsule_capsule` **平行轴分支**在融合后端（det≈2.6e-12 → 非平行）未被 device 触发，仍未覆盖。

**证据指针**：B1 `docs/development-evidence/g10-p5-primitive-plane-cuda/`（§5 窗口结果、§9 未覆盖、§12 QA 观察）；B2 `docs/development-evidence/g10-p5-primitive-sphere-capsule-cuda/`（§6 判据与机制、§9 分支、§10 窗口结果、§11 行序修复、§12 QA 观察收口）；既有片 `g10-p5-primitive-narrowphase-cuda/`、`g10-p5-multicontact-cuda/`；B3 `docs/development-evidence/g10-p5-primitive-branch-coverage/`（§7.4 验证窗、§12 QA 观察收口）；独立复核 `...-plane-cuda-independent-check/`（pass）、`...-sphere-capsule-cuda-independent-check/`（pass）、`...-primitive-branch-coverage-independent-check/`（pass）。

**边界**：UNC-3 仍为 **partial**；只写「组合层 13/13、分支层部分覆盖」，不宣称全路径 / 全分支 / 跨平台；精度与等价性结论仅限上表 case 与已批准判据内。

### 6.2 第三次更新（2026-10-27；S3 捕获 + S4 分支收口验证窗 attempt1 + QA `efccbf63` pass 后）

**基准（全部 verified，数字只引原文、未复算）**：S3 fixtures/脚本 `96112ddf`；S3 捕获窗 `43ec5676`（attempt1 有效轮、raw A/B 逐字节一致、11-11 两次捕获一致；QA `68ca6fa5` pass / 观察 10，收口 `2d4a5f37` + `e4713472`）；S4 实现片 `b3e56a5f`（新 target 341 行 + launcher 扩 hfield-capsule 499 行；unit 994/0/376）；S4 验证窗 `055d0c8a`（10 实体 attempt1；QA `efccbf63` pass / 0 阻断 / 观察 5）。

**分支收口项（本条计入「本窗验收覆盖」的范围）**：

| 项 | 分支/内容 | 本窗结果（判据内） | 证据 |
| --- | --- | --- | --- |
| U1 | `plane_cylinder` 退化 x 轴回退 | `plane-cylinder-degenerate` nacon **3/3**、failures 0（GPU 参考 dist 全 `-0.009999997913837433`、pos z `-0.004999998956918716`） | S3 参考 `upstream-branch-closure-gpu.json` `24d89e92…`；S4 branch_closure |
| U3 | `plane-box` 8 角点 | 5 pose：face-down `{0,1,2,3}` **4/4**、face-up(Rx180) `{4,5,6,7}` **4/4**、tilt-x `{0,1}` 2/2、tilt-y `{1,3}` 2/2、single `{7}` 1/1 → 并集 **8/8** | 同上 |
| U4 | `plane_capsule` 横躺非退化帧基 + 双端点 | `plane-capsule-lying` nacon **2/2**（端点 ±0.12） | 同上 |
| U5 | `plane_cylinder` 4 条候选全写入 | `plane-cylinder-deep-tilt` nacon **4/4** | 同上 |
| U6 | mesh `nmesh>1` 进分母 | `mesh-mesh-two-assets` nacon **3/3**（ContactMulti、`geom_dataid=[0,1]`、nmesh=2 / 12 vert / 10 poly） | 同上 |
| U9 | `hfield-capsule` 设备回归 | launcher 单 case **275/275 bit、ULP 0**（对比冻结 probe `out1/`）；**不外推到 hfield 其他路径** | S4 launcher |
| U2 | `plane_ellipsoid` 非上游退化保护 | **不可达 / 非上游防御**（pinned 无保护；设备 `primitive_plane_shapes.cu` 才加 `!(length>0)||!isfinite`，静态 diff 依据） | S1 `reachability.json` |
| U7 | degree>10 限制 | **仅限制存在**：fixture max_degree=24>10；设备 count=1 写「与 degree>10 限制语义一致」（同时记 dist/pos 防 count 偶合） | S4；S2 M-05 |
| U8 | polygon>16 限制 | **仅限制存在 + 行为差距**：设备 count=1 vs 上游 **4**；上游支持 20 顶点 polygon（动态容量），设备 `FACE_MAX=16` 逐 polygon 回退 → **行为差异 / 限制存在**，不进分母、不改判据 | S4；S2 M-09；backlog `2026-10-27-mesh-polygon-gt16-device-gap`（不 start） |

**两条容量限制单列**：`G5_MC_MESH_DEG_MAX=10`（单顶点相邻 polygon 数，`multicontact_mesh.cu:120/148`）与 `G5_MC_MESH_FACE_MAX=16`（单 polygon 顶点数，`multicontact_mesh.cu:180`）是设备固定容量（`contact.cu` scratch 决定）；上游按模型级 `nmeshdegmax`/`npolygonmax` 动态容量（pinned `collision_convex.py` L1262-1289、`types.py` L1661-1662）。**两条都只写「限制存在」，不写「已验证回退行为与上游一致」。**

**窗口与数字口径（QA S4 观察 O1/O2 已吸收）**：
- **case 级比较合计 33**（multicontact 3 + launcher 4 + primitive 3 + plane 6 + sphere_capsule 4 + branches 6+4 + M2 2 + M3 1）+ stage2 **17 dump**（record-C 只记录）。「13 组合」是 B1/B2/B3 时代**旧口径（已作废，仅作历史）**；launcher 一律写 **4**。
- **回归对象 = M3 窗口冻结二进制**（8 个 `target/m3-freeze` 实体，构建 01:06）；M3 提交 `f1e5cda1` 在冻结**之后**（含 O7 值域校验 + 测试文件拆分），设备接触路径与 M3 同源、O7 由新 target 覆盖。**不写成「当前树重编通过」**（S4 曾重编 m2/m3，sha 变，已换回 M3 已签副本：`2 s4-build + 8 m3-freeze`）。
- 回归面：13 组合（旧口径）/ B3 6 分支 + 4 swap / M2 两 case / M3 mesh-mesh 全 failures 0；stage2 record-C 17 dump 两次一致。S4 窗：10 实体 exit 0、body_wall 238.8 s、871 样本、delta 95/96/109、free_min 12918、util 46（streak 0）、residual 0、零编译。
- runner 改进项（QA O3–O5，**本片不改 runner**）：`facts.txt` 的 `guard_aggregate` 字典串缺陷、guard sampler 写文件 BOM、stage2 日志控制台转码。

**统计与状态**：UNC-3 仍 **partial**（组合层 13/13 不变；分支层新增 6 项「本窗验收覆盖」+ U2/U7/U8 三条边界项）；**不宣称全分支 / 全路径**；不写「窗口通过」。

**边界**：不改判据 / 容差 / 参考 / 期望值；「本窗验收覆盖」只对上述 case 与已批准判据成立；U8 的功能差距是真实债务（backlog，不 start）；S2 的 12 条「可验证」仍需定向 case 才能转覆盖。

## 7. UNC-1 更新（2026-10-27；mesh 四片串行收口后第一次更新）

**基准（全部 verified，数字只引原文、不复算）**：ABI sidecar 片 `097c54b9`（形状修正 `b50d93a6` + 复核 `91a9fdea`）、mesh 捕获片 `35192320`（QA `fe8956f4`）、32 数组 case 生产片 `ac01d86d`（QA `f191fdba` pass / 0 阻断 / 观察 6）、convex-vs-mesh 设备实现片 `16214a22`（测试入口 `85cce21b`；QA `271fe926` pass / 0 阻断 / 观察 8）。

| 维度 | 本次更新前 | 本次更新后 |
| --- | --- | --- |
| 组合层（设备 multicontact） | box/cylinder（`9a282074`） | box/cylinder **+ convex-vs-mesh**（`mesh-box`/`mesh-cylinder` 各 nacon 4/4、bit_equal 4/4、ULP 0、`contact.geom=[1,0]`） |
| 设备输入侧 | 旧 G5 23 数组 + mesh 无输入 | 新增 9 个 mesh polygon slice（sidecar 504 / 72 B 视图）与 **2 case × 32 数组 bins**（同一来源：xml/qpos/geom_xpos/geom_xmat 与 M1 参考逐位一致） |
| mesh 分支覆盖 | 未实现 | **部分**：`_intersect1/_intersect2`、`_mesh_normals`、`_mesh_edge_normals`、`_mesh_face`（逆序写出）按 pinned `collision_gjk.py` rev `71da24d9` 移植；face/边法线路径由两 fixture case 命中；swap 几何顺序沿用 G5 的 pinned 类型序规范化 |
| 未覆盖 | mesh multicontact 全部 | **mesh-mesh 多接触（M3 在途）**；一般 mesh（nmesh>1、degree>10、polygon>16 走固定容量 clamp 回退）；`hfield-capsule` 设备接触回归（ABI 504 后无设备证据，仅 launcher `hfield-sphere` + 静态偏移断言补偿）；mesh 全类型/全分支；Linux/跨平台 |
| status | partial | **partial（不变）** |

**窗口口径**：M2 验证窗 attempt4 = **数值全绿 + guard 因 free floor 环境边界 abort，经 Chief 2026-10-26 裁定接受为有效轮**（不写「窗口通过」）；attempt1–3 作废原因留档于 M2 证据 §8.1。M3 起 guard floor 派生公式 `min(10240, baseline_free − 3072)`，**不追溯改 M2 数据**。

**不得声明**：mesh 参与的多接触路径整体已覆盖、mesh-mesh 已实现、一般 mesh 已支持、mesh 全类型/全分支、跨平台；UNC-1 仍为 partial。

**证据指针**：`docs/development-evidence/mesh-multicontact-m0a/`、`...-m0a-fix-mesh-poly-shapes/`、`...-m1/`、`...-m0b/`（+ `...-m0b-independent-check/`）、`...-m2/`（§7 窗口内容、§7.3 hfield-capsule 未覆盖、§8 attempt 与裁定、§8.5 派生口径）、`...-m2-independent-check/`。第二次更新（mesh-mesh）在 M3 verified + QA 复核后做。

### 7.2 第二次更新（2026-10-27；M3 收口 + QA `ab586682` pass 后）

| 维度 | 第一次更新时 | 第二次更新后 |
| --- | --- | --- |
| 组合层（设备 multicontact） | box/cylinder + convex-vs-mesh | **+ mesh-mesh 多接触**（M3 `f1e5cda1`，QA `ab586682` pass / 0 阻断 / 观察 7）：nacon=4、**bit_equal 4/4**、dist/pos/frame ULP 0/0/0、`contact.geom=[0,1]`、failures 0，对照新参考 `96e7895d…` |
| 设备侧改动 | M2 新增 `multicontact_mesh.cu` 等 | M3 **0 行新设备代码**（M2 分支已覆盖两侧 MESH；`DEVICE_SOURCES` 28→28、入口 6→6、冒烟四数字与 M2 相同） |
| 设备输入侧 | 2 case × 32 bins（mesh-box/cylinder） | **+ mesh-mesh 32 数组 bins**（9 slice `[2,2,36,12,12,48,16,16,48]`，18 条自检 PASS，`cross_check` 带 actual/reference） |
| 输入校验（O7） | `polyvert/polymap` 元素值域未校验 | **已补 4 组值域规则**（vertadr/vertnum 前提与区间、polyvert 界、polymap 界、mesh dataid 界）+ 负例/正例；`run()` 零改动 |
| 窗口 | M2 attempt4（Chief 裁定接受） | **M3 验证窗 attempt1**：9 实体 exit 全 0、new floor `min(10240, baseline_free−3072)=10240`、732 样本、delta 278/283/283、free_min 13557、util 60（streak 0）、residual 0、零编译；回归 13 组合 + B3 6+4 swap + M2 两 case 全 failures=0 |
| 未覆盖 | mesh-mesh 多接触、一般 mesh、hfield-capsule、全类型/全分支、跨平台 | **收窄为**：一般 mesh（nmesh>1 通用 / degree>10 / polygon>16 回退）、mesh 分支层未逐条覆盖、`hfield-capsule` 设备回归、`efc_address` 缺口（设备全 −1）、UNC-6 深穿透未触发、跨平台、性能 |
| status | partial | **partial（不变）** |

**统计口径不变**：12 条 requirement（met 8 / partial 4 / unmet 0）、uncovered 6 条；本表只更新 UNC-1 的范围文字，
不改任何已 verified 数字、判据、容差、参考或期望值。

**不得声明（更新后）**：一般 mesh 已支持、mesh 分支层全覆盖、mesh 全类型、跨平台、性能；`hfield-capsule`
与 `efc_address` 仍为未覆盖项；G10@P5 整体通过/精度目标达成不在本 map 的声明面。

**整片汇总**：`docs/development-evidence/mesh-multicontact-mesh-summary/README.md`（交付链、门禁数字、
未覆盖与「不宣称」清单、债务、后续片候选）。

### 7.3 第三次更新（2026-10-27；S4 分支收口验证窗 + QA `efccbf63` pass 后）

| 维度 | 第二次更新后 | 第三次更新后 |
| --- | --- | --- |
| 组合层（设备 multicontact） | box/cylinder + convex-vs-mesh + mesh-mesh | **不变**（S4 回归复跑：M2 两 case 4/4 bit `[1,0]`、M3 mesh-mesh 4/4 bit `[0,1]`，用 **M3 窗口冻结二进制**） |
| mesh 一般拓扑 | 未覆盖（nmesh>1 通用 / degree>10 / polygon>16） | **收窄**：`nmesh>1` **仅 1 例验收覆盖**（S4 U6 `mesh-mesh-two-assets` nacon 3/3）；**一般拓扑未穷举**；`degree>10` / `polygon>16` 仅**限制存在**（U7 count=1 与限制语义一致；U8 设备 1 vs 上游 4 = **真实功能差距**，进 backlog、不进分母） |
| mesh 分支层 | 部分（M2/M3 由 face/边法线路径 case 命中） | **仍只静态核对**：S2 `branch-map.json` mesh 14 条 = 已 case 覆盖 3 / 仅静态核对 4 / 仅限制存在 2 / 未覆盖 5；edge/vertex 路径无 case |
| `hfield-capsule` 设备回归 | 未覆盖 | **计本窗验收覆盖（launcher 单 case 275/275 bit、ULP 0）**；**不外推到 hfield 其他路径**（hfield-sphere / sdf-box / ellipsoid-box 按既有口径） |
| `efc_address` / UNC-6 / 跨平台 / 性能 | 未覆盖 | **不变** |
| status | partial | **partial（不变）** |

**不得声明（更新后）**：mesh 已覆盖、一般 mesh 已支持、mesh 分支层全覆盖、mesh 全类型、跨平台；U8 差距未解决前**不写「与上游一致」**。

**证据指针（第三次更新新增）**：S3 捕获 `docs/development-evidence/g10-p5-branch-closure-capture/`；S4 实现与窗口 `docs/development-evidence/g10-p5-branch-closure-validation/`（README + `window-receipt.json` + 20 份窗口日志）；S4 复核 `docs/development-evidence/unc3-branch-closure-s4-independent-check/`；S2 静态映射 `docs/development-evidence/unc3-branch-layer-static-map/branch-map.json`；S3 复核与观察收口 `...-s3-independent-check/`、`...-s3-observation-closure/`。
