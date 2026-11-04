# G10@P5 家族收口（地形与 SDF 碰撞）

任务：`2026-11-04-g10-p5-family-closeout`（owner Arch）。日期：2026-11-04（空闲段）。
**边界：证据索引与确缺口登记，零 GPU、零 `src/`/`tests/`/`fixtures/`、不改判据/容差/参考、不改已 verified 证据（含 `g10-p5-acceptance-map/`）。**

## 1. 一句话结论

G10@P5（地形、体积 SDF 与用户距离梯度钩子；验收「地形接触、梯度与越界」）**54 张子卡全部 verified**，
依赖 `g07-p5`/`g08-p5` 均已 verified，渲染侧地形/SDF 资产管线已由 r01 + R05 hfield + SDF 线交付 →
2026-10-23 顺延条所等的恢复条件**已满足**；本轮只做**家族级收口索引**。
**计数如实（不美化）：12 条 requirement = met 8 / partial 4 / unmet 0；另有 6 项 uncovered。**
**本卡不是「整族通过」的声明**——partial 与 uncovered 的边界见 §2/§3。

## 2. 12 条 requirement 映射（依据 `docs/development-evidence/g10-p5-acceptance-map/`，verified）

### 2.1 实质七行（「地形接触、梯度与越界」）

| id | 内容 | 实现/证据 | 状态 |
| --- | --- | --- | --- |
| G6-1.1 | 地形接触（host）：hfield 三 case | `2026-10-25-g10-p5-hfield-host`（+其独立复核） | **met**：count 8/8、8/8、7/7；dist 全位级 0 ULP；pos 0.00 ULP@尺度；frame 72/72、72/72、63/63 位等 |
| G6-1.2 | 地形接触（设备）：hfield prism CCD + `MAXCONPAIR=50→HFIELD(32)` | `2026-10-25-g10-p5-gpu-kernels`；设备数值只认 `g10-p5-gpu-kernels/gpu-window/cases/hfield-capsule`、`hfield-sphere` | **partial**：主判据（hfield-sphere 8/8、hfield-capsule 7/7 全字段位级）来自 G5 交付片；但 `HFIELD(32)` 的 overflow=32 实测**只来自 QA 独立复核**，交付片 README 只写实现映射 → 按同一层级标准记 partial |
| G6-1.3 | 梯度（host）：SDF 梯度/下降 + `sdf-box` | `2026-10-25-g10-p5-sdf-host-r2` + `-sdf-relmat-fix` | **met**：80/80；dist 80/80 位等；pos 0.00 ULP@尺度；frame 720/720 位等 |
| G6-1.4 | 梯度（设备）：SDF narrowphase | `2026-10-25-g10-p5-gpu-kernels`；`gpu-window/cases/sdf-box` | **met**：dist 80/80 位等；pos 203/240 位等（1.0 ULP@scale 0.09834）；frame 675/720（max_abs 2.384e-7） |
| G6-1.5 | 越界（设备）：OverflowType 位语义 | `2026-10-25-g10-p5-ccd-overflow-bit-fix`（`99c91469`） | **met**：强制 horizon 3 例 5/6 world 修前 272 → 修后 **256**，与上游位级相等 |
| G6-1.6 | 越界（宿主）：同上 + 真实容量门 | `2026-10-25-g10-p5-host-ccd-overflow-bit-fix`（`5a176fee`） | **met**：`ccd()`/`ccd_pair` 272→256；容量截断仍置 CCD(16)；G2–G5 回归不变 |
| G6-1.7 | 越界（容量截断）：`naconmax`/`naccdmax` | 设备 `contact.cu` + 宿主 `CcdCapacity`；case 见 QA 复核 | **partial**：截断行为由 **QA 独立复核**以自造 case（`sdf-box-naconmax40`、`ellipsoid-box-naccdmax1`）覆盖并通过，G5 交付片自身未含截断 case |

### 2.2 流程四行

| id | 内容 | 状态与依据 |
| --- | --- | --- |
| G6-2.1 | 先补齐模块与写入范围 | **partial**：G2–G5 均 verified；设备三缺口片已补（multicontact box/cylinder `9a282074`、convex-vs-mesh `16214a22`、NVRTC 产品 launcher `dfcc1bdc`），primitive 组合层 13/13；mesh 多接触经 M0a–M3 补齐（M3 `f1e5cda1`：nacon 4/4、bit_equal 4/4），mesh 容量 20 面由 `2026-10-31-g10-p5-mesh-cap20` 补齐；**仍余**：默认产品管线接入（UNC-2 限定语）、primitive 分支层部分、mesh 边/顶点分支仅静态核对、mesh polygon>16 设备计数差 |
| G6-2.2 | 性能测试须独占 GPU | **met**：一次获批独占窗口（HEAD `9c0a0480`；wall 5.5 s；峰值 **4834 MiB（+1171）**；守卫 3072 MiB/50% 未触发；0.5 s 采样 7 点；跑前跑后本任务计算进程 0） |
| G6-2.3 | 记录真实测试与未测项 | **met**：各 receipt 的 verification 为实测命令输出、unmet 明确列出 |
| G6-2.4 | 先核对 NP01/NP02，再验收 NP03/NP04 | **partial**：NP01/NP02 已核对；NP03 上游 f32 侧完成、native 侧接触字段按用户 2026-10-25 拍板 B **只作诊断**（5% 红线不适用）、其它字段 5% 红线未全量；NP04 随 G6-2.2 性能窗适用项满足（相对基准） |

### 2.3 native 处置一行

| id | 内容 | 状态 |
| --- | --- | --- |
| G6-3.1 | native 侧接触字段按用户拍板 B：待验收不放行、只作诊断 | **met（处置已落地，不提供放行信号）**：契约 `docs/numerical-precision-contract.md` 接触字段节（`09896792`）+ 证据 `native-contact-tolerance-decision-b/`；E 复算样本量 = `ellipsoid-box` 1 case / 2 接触点；`hfield-box` 仍不可配对、样本不增 |

## 3. 未覆盖六项（uncovered，逐条在案）

1. **UNC-1 multicontact = partial**：box/cylinder 已覆盖（`cylinder-box` 314/314 位级、`box-box-margin` 391/392）；convex-vs-mesh 覆盖并位级一致（`16214a22`）；mesh-mesh 多接触覆盖（M3 `f1e5cda1`）；`nmesh>1` 有 1 例（S4 U6）；**`degree>10`/`polygon>16` 仅「限制存在」**，其中 U8 如实记「设备 count=1 vs 上游 4」= 真实功能差距，**不进分母**，backlog `2026-10-27-mesh-polygon-gt16-device-gap`（不 start）。
2. **UNC-2 NVRTC 产品 launcher = closed（2026-10-26，Chief 批准）**，限定语：**默认构建/默认产品管线未接入**（`cuda-default-pipeline-stage2` §7.1）。
3. **UNC-3 primitive narrowphase = partial**：组合层 13/13 各有上游 GPU 对照；分支层部分覆盖（U1/U3/U4/U5/U6/U9 计本窗覆盖，U2 不可达，U7/U8 仅限制存在）；mesh 边/顶点分支层仅静态核对。**未宣称全路径/全分支。**
4. **UNC-4 Linux/T4 未覆盖**（用户 2026-10-24 裁决集中回测）。
5. **UNC-5 G5 窗口逐秒峰值未取到**（脚本缺陷已修，G6 性能窗按 0.5 s 采样另列）。
6. **UNC-6 深穿透额外采样面**：`cylinder-box` 4/24 单字段超 4·ULP@scale（x1 1/24 max 8.926e-7=29.95×、x2 3/24 max 1.527e-7=10.25×）；**用户 2026-10-25 拍板 A：不修、列未覆盖**；归因为派生法线差（机制未定，非系统性缺陷，n=24 无显著线性相关）。

## 4. 设备 / host 证据分界（写死）

- **设备数值**只引 `docs/development-evidence/g10-p5-gpu-kernels/gpu-window/`（cases：`hfield-capsule`、`hfield-sphere`、`sdf-box`、`mesh-mesh` 等）与其独立复核；
- **host 数值**只引 `g10-p5-hfield-host`、`g10-p5-sdf-host-r2`、`g10-p5-sdf-relmat-fix`、`g10-p5-host-ccd-overflow-bit-fix` 的 CPU 位级结果；
- **没有窗口记录的行为**（`HFIELD(32)` overflow=32、容量截断 case）明确标注「仅 QA 独立复核记录」，**不冒充交付片实测、不冒充设备位级**。

## 5. NP 核对

| 项 | 结论 |
| --- | --- |
| NP01 | **已核对；E 指标口径 1 项待用户签（A2）** —— 本卡不代签、不宣称满足 |
| NP02 | 满足（frozen inventory / evidence gate / receipt rehash / new kernel evidence / ptx entity restore 全 verified） |
| NP03 | 不适用、不宣称（用户 2026-10-24 Linux 终止） |
| NP04 | 适用项满足（相对基准；性能窗 G6-2.2 已独占采集） |

## 6. 已知限制

1. 12 条里 **4 条 partial**（G6-1.2、G6-1.7、G6-2.1、G6-2.4），**不得按 met 计**。
2. **默认产品管线未接入**（UNC-2 限定语）：launcher 经 `cuda-probe` 特性可调用，默认构建未接。
3. **未宣称全路径/全分支覆盖**（UNC-1/UNC-3）。
4. mesh polygon>16 设备计数差（backlog 不 start）；深穿透 4/24 超界（用户拍板 A 不修）。
5. Linux/T4 未覆盖；性能只到相对基准（无绝对承诺）。
6. native 侧接触字段只作诊断（拍板 B），不提供放行信号。

## 7. 非声明

- 不声明 G10 整族通过；
- 不声明设备全路径/全分支覆盖；
- 不与上游完全一致、不作对外发布依据；
- 不替渲染线（R01–R10）或其它物理阶段背书。

## 8. 记账说明

- **G10@P5 不挂 `docs/development-evidence/render-bvh-precheck/plan.json`**（那是渲染/BVH 线；G10 属碰撞线），仓库内也没有第二份 `plan.json`；验收映射以已 verified 的 `g10-p5-acceptance-map/map.json` 为唯一依据，本卡不改它。
- `roadmap-g10-p5` 节点：**本卡不动**；等你确认后按 P8 同款方式（加一条 `json` 检查 → `start→done` 纯状态）处理。
- 本卡提交只含本目录与本卡/QA 卡记录。
