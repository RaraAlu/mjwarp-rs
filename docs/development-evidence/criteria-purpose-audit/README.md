# 机器判据的目的与本质审计（只读）

任务：`2026-10-31-criteria-purpose-audit`（QA；owner QA；无 GPU 资源；只读）
pin：`b908075c`（start 时 HEAD；`revision.before`）
审计标准：`standard.md`（v1；§5 按 Chief 2026-10-31 最终口径：QA 报告 → Chief 定夺；仅放宽数值/改参考/对外发布三类报用户）
产出：`materials/criteria-table.md`（31 条判据逐项表）、`materials/high-risk.md`（A/B/C 分组高危清单）、`materials/checks-inventory.txt`、`materials/gates-policy.txt`、`materials/src-tolerance-constants.txt`、`materials/tests-assertions.txt`、`materials/docs-criteria-statements.txt`、`materials/downgrade-impact.txt`

## 1. 为什么要审计

用户指出：做验收裁定前没有先定义「裁定的目标/服务对象/本质」，导致 U4 把参考产物的编译形态位级当成 SDF 正确性（`MAX_DEPTH_ULP=4` 只有一句「最大允许差」，无推导、未说服务哪个决定）。本审计把仓库里**所有机器判据点**枚举出来，按 `standard.md` 三问 + 界来源 + A/B/C 分类，输出高危清单与影响面；**只报告、不处置、不改任何判据/数值/代码/参考**。

## 2. 范围与覆盖统计（全部为只读扫描；原始输出见 materials）

| 类别 | 覆盖 | 关键数字 |
| --- | --- | --- |
| `.normify` 任务 checks | 784 张卡全部解析 | 524 张有 checks，**260 张无任何 checks**（约 33%）；status：verified 616 / abandoned 100 / backlog 41 / proposed 25 / in_progress 2 |
| check kinds | gate 1322、json 446、unchanged 246、forbid_claim 503 | gate 明细：fmt 403、unit 345、clippy 295、clippy-cuda 276、clippy-nocuda 1、unit-nocuda 1、g26-energy 1；**无 `record` 类** |
| gate 定义/阈值 | `.normify/gates.toml`（9 个定义）、`policy.toml`（11 条规则）、`config.toml`（提交/尺寸规则） | 被 checks 引用 7 个；**g01-full / g01-full-release 无卡引用**；无数值容差阈值（全是 exit code / libtest 汇总） |
| `src/**` 容差常量 | 38 个 `const *ULP/TOL/EPS/LIMIT/MIN_/MAX_` 命中，逐条带 ±3 行上下文 | 判据相关子集：`MAX_DEPTH_ULP=4`、GJK/EPA/hfield 上游常量 10 项、`TOLERANCE=1e-6/2e-6` 等；其余为运行时上限/格式限制 |
| `tests/**` 比较与位级断言 | 299 个测试文件 | ULP 相关 388 处、LSB 86 处、seg/mismatched 109 处、通用容差 384 处、位级表述 44 处、sha/hash 1123 处、一致性 38 处 |
| `docs` 判据口径 | contracts/spec + 证据 README 172 个文件 | 922 条判据/容差表述；判据版本化文档目前只有 `render-bvh/sdf-criterion-v2/spec.md`（v2，已签署） |
| 降级影响面 | 784 张卡面文本匹配（同卡可多命中，数字为上限式粗估） | `MAX_DEPTH_ULP/4 ULP` 80 卡/70 已验收；RGB LSB 37/30；bit_equal 10/10；4·ULP@scale 21/19；完整性族 297/275 |

## 3. 分类结果（31 条判据）

- **A 本质-推导型 9 条**：CR-01（门禁零失败）、CR-02（GPU 全链 gate，从属型）、CR-13（seg 离散 exact）、CR-14（两跑确定性）、CR-15（shadow 一致性）、**CR-17（SDF criterion v2：B=128 ULP 误差模型推导 + 冻结清单）**、CR-19（格式/工具链硬上限）、CR-29（物理结构不变量）、CR-30（性能/资源预算，需求型）。
- **B 参考-前提型 11 条**：CR-10（`MAX_DEPTH_ULP=4`）、CR-11（depth 位级）、CR-12（RGB ≤1 LSB）、CR-16（DepthSeg 跨路径位级）、CR-18（U4 历史用法）、CR-23（物理上游常量镜像）、CR-24（g10 `bit_equal+4 ULP@scale`）、CR-25（witness 采样面）、CR-26（faithful-fma 4 ULP/4.8e-7）、CR-27（物理测试 2e-6）、CR-28（宿主探针 1e-6）。
- **C 完整性-哈希型 11 条**：CR-03（receipt json）、CR-04（unchanged）、CR-05（forbid_claim）、CR-06（policy）、CR-07（config/模块指纹）、CR-08（260 张无 checks 卡，缺省）、CR-20（内核身份 sha）、CR-21（guards/manifest/aggregate）、CR-22（ABI 81 槽）、CR-31（证据 README 判据口径，记录层）、CR-32（判据版本/契约）。

高危标记（按 §4）：**B 组 10 条为高危**（CR-10/11/12/16/18/23/24/25/26/27）；**C 组 6 条高危误用**（CR-04/05/08/20/21/31）；**A 组 5 条需补前提/注明边界**（CR-02/15/17/29/30）。逐条依据与影响面见 `materials/high-risk.md`、`materials/criteria-table.md`。

## 4. U4 案例（审计中必须逐项展开的一条）

`render_sdf_reference` 曾用 `MAX_DEPTH_ULP=4` + depth `mismatched==0`（CR-10/11）判 SDF 正确性，而参考来自 scene-a 普通球 primitive 的 warp GPU 采集（CR-18）。被测物本质是 SDF（`gtype==8` 走底层 mesh `ray_mesh_leaf`），与 scene-a 的 `ray_sphere` 不是同一 kernel 形态；上游 warp 对 `a=dot(vec,vec)` 的三项 FMA 收缩在不同场景生成不同次序（RCA：scene-a `(0,1,2)`、scene-sdf `(1,0,2)`），所以**没有任何单一产品实现能同时位级匹配两份参考**：attempt1/2 同一球轮廓 215/76,800（max 191 ULP）、4/43,200（max 25 ULP）；加 fix 后 attempt3 反使 6 个 scene-a 回归在 183 像素上 max 73 ULP、aa2 臂 max 91 ULP。此行同时命中高危①②③。criterion-v2（CR-17）用误差模型推导 B=128 并冻结受影响像素清单，是仓库里第一个 A 类判据，但前提（冻结时 `√det_min`/`|c|max` 极值、单场景、清单与 dump/guards 同源）须在落地文档中写明；本审计不评价其数值是否最优。

## 5. 边界与不宣称

- 只读：未跑 GPU、未开窗、未冻结、未构建；未改任何判据、数值、代码、参考、fixtures。
- 未逐条枚举 299 个测试文件里的每一行断言，而是按判据族聚合（同一 helper 的断言归一族）；`tests-assertions.txt` 保留 7 组关键字的命中样本与总数，可复查。
- 降级影响面是**卡面文本匹配的上限式粗估**（一卡可命中多族），不是精确归属；精确归属需按卡逐条人工核对。
- 本审计**不写处置建议**（Chief 定夺）；**不宣称**判据全部合规、**不宣称**已可发布；未产生任何签名级结论。
