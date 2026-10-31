# 高危清单（按 A/B/C 分组；只报告，不处置）

判定标准见 `standard.md`。高危标记 = 命中 §4 任一条：①位级判据但参考来自不同 kernel 形态/场景；②容差常量无推导注释/无来源；③同一判据跨场景/跨平台复用未复核前提；④用证据完整性代替数值正确性；⑤用「诊断已定位」代替通过。

## 0. 命中总览（按影响面排序）

| 判据 | 分类 | 命中规则 | 影响面（粗估，文本扫描） | 一句话 |
| --- | --- | --- | --- | --- |
| CR-10 `MAX_DEPTH_ULP=4` | B | ①②③ | 80 卡 / 70 已验收 | 无推导的 4 ULP 被当渲染（含 SDF）发布界，跨场景复用 |
| CR-11 depth 位级 `mismatched==0` | B | ①③ | 同 CR-10 家族；U4 attempt1/2/3 | 位级要求 + 参考跨 kernel 形态，直接造成 U4 三次 INVALID_ROUND |
| CR-12 RGB ≤1 LSB | B | ②③ | 37 卡 / 30 已验收 | 界可由 8-bit 量化推出但无书面推导/前提 |
| CR-16 DepthSeg 跨路径位级 | B | ①③ | S7b1/R09/SDF 家族 | 两条 kernel 形态要求位级相等，形态差异可致位差，前提未写 |
| CR-23 物理上游常量（INTERSECT_TOL/FACE_TOL/EDGE_TOL/…） | B | ①② | 碰撞 G2/G3/G4/G5、branch closure、mesh 系列 | 上游常量镜像，无本地推导；GNU/MSVC FMA 收缩差异会变结果 |
| CR-24 g10 `bit_equal + 4 ULP@scale` | B | ①③ | 10 卡 bit_equal + 19 卡 ULP@scale | 参考直接当界（裁决文档只解释 scale 语义，无误差模型）；跨 case/场景复用；witness 面 1/24 超界 |
| CR-25 witness 采样面判据 | B | ①③ | mesh multicontact / witness 卡 | 采样面未覆盖不可推断等价（1/24 超界已记录未决） |
| CR-26 faithful-fma 4 ULP / 4.8e-7 | B | ③ | faithful-fma / case-evidence 系列 | 参考是 CPU 侧，产品是 GPU/编译器收缩形态，前提跨形态 |
| CR-27 物理测试 TOLERANCE=2e-6 | B | ② | fluid/gravity 冻结设备对照 | 「沿用 2e-6」，无推导 |
| CR-28 宿主探针 1e-6 容差 | B | ② | math/history/samples | 探针型容差，与发布判据混用易误读 |
| CR-18 U4 历史用法（B 类判据跨形态用于 SDF） | B | ①②③ | U4 attempt1/2/3 + 回滚 | 事故记录：四重命中，触发 criterion-v2 修订 |
| CR-31 证据 README 判据口径 | C（记录） | ④ | 172 文件/922 条表述 | U4 事故的文档层根因：把「同配方参考位级」写成 SDF 正确性、未写前提 |
| CR-04 `unchanged` 检查 | C | ④ | 279 已验收卡 | 「判据/参考零改」被当作「结论可信」 |
| CR-05 `forbid_claim` 词表 | C | ④ | 458 已验收卡 | 黑名单≠声明为真，词表外表述不受约束 |
| CR-21 guards/manifest/aggregate | C | ④ | 275+ 卡文本命中 | 冻结完整≠数值正确（U4 attempt3 冻结完整、body 仍 INVALID_ROUND） |
| CR-20 内核身份 sha | C | ④ | 窗口/冻结类卡 | 身份一致只说明旧 guards 可用性，不证明输出正确 |
| CR-08 260/784 卡无任何 checks | C（缺省） | — | 33% 任务 | 约三分之一任务没有机器判据，验收仅靠文本 |

## A 组（本质-推导型；可作发布依据）

| ID | 判据 | 高危标记 | 需要补写/注意的前提 |
| --- | --- | --- | --- |
| CR-01 | 门禁（fmt/clippy/unit…） | — | 工具链/feature 组合 |
| CR-02 | GPU 全链/完整回归 gate | 中 | 布尔通过可能掩盖内部数值判据；g01-full/g01-full-release 当前**无卡引用** |
| CR-13 | seg 逐像素 exact | — | 同场景/同 id 语义/同相机顺序 |
| CR-14 | 两跑 sha 确定性 | — | 同 context/同配置 |
| CR-15 | shadow 开/关一致性 | 中 | 单调方向（off≥on）的推导未写 |
| CR-17 | **SDF criterion v2（B=128 ULP）** | 中 | 误差模型 §3 + 冻结时极值（√det_min/|c|max）+ 单场景；清单必须与 dump/guards 同源；临时放宽=违规 |
| CR-19 | 格式/工具链硬上限 | — | 同格式/工具链版本 |
| CR-29 | 物理结构不变量（nacon/efc/overflow/geom 排序） | 中 | 部分不变量来源应标「上游契约」 |
| CR-30 | 性能/资源预算 | 中 | 同硬件/独占窗口；跨机器需重采基线 |

## B 组（参考-前提型；不得单独作发布依据）

| ID | 判据 | 高危标记 | 缺失前提/界来源 |
| --- | --- | --- | --- |
| CR-10 | `MAX_DEPTH_ULP=4` | **高（3 条全中）** | 无推导；需同 kernel 形态/同 recipe/同 GPU/同 warp/同场景 |
| CR-11 | depth `mismatched==0` 位级 | **高** | 同 CR-10；U4 事故直接命中 |
| CR-12 | RGB ≤1 LSB | 高 | 可补 8-bit 量化推导；需同量化口径 |
| CR-16 | DepthSeg 跨路径位级 | 高 | 需同 f32 求值顺序/同编译形态 |
| CR-18 | （历史）位级 4 ULP 用于 SDF | **高（事故）** | SDF（mesh 底层求交）vs scene-a 球 primitive 参考形态不同 |
| CR-23 | 物理上游常量镜像 | 高 | 需标上游版本 + 编译器收缩前提 |
| CR-24 | `bit_equal + 4 ULP@scale` | 高 | 无误差模型；跨 case/场景复用；witness 1/24 超界 |
| CR-25 | witness 采样面 | 高 | 采样面覆盖不足，不可推断等价 |
| CR-26 | faithful-fma 4 ULP / 4.8e-7 | 高 | 参考是 CPU 侧；GPU 收缩形态不同 |
| CR-27 | 物理测试 2e-6 | 高 | 无推导，历史沿用 |
| CR-28 | 宿主探针 1e-6 | 低-中 | 探针不替代设备执行 |

## C 组（完整性-哈希型；不动，但需防误用）

| ID | 判据 | 高危标记 | 误用风险 |
| --- | --- | --- | --- |
| CR-03 | receipt json `/task_id` | — | — |
| CR-04 | `unchanged` 路径不变 | 高 | 完整性≠数值正确（「零改动」≠「判据有效」） |
| CR-05 | `forbid_claim` 词表 | 高 | 黑名单≠声明真实；词表外表述不受约束 |
| CR-06 | policy（架构/命名） | — | 架构零违规≠功能正确 |
| CR-07 | config/模块指纹 | — | 指纹一致≠代码正确 |
| CR-08 | 无 checks 的 260 张卡 | 高（治理缺口） | 约 33% 任务无机器判据 |
| CR-20 | 内核 source/PTX sha | 高 | 身份≠正确性 |
| CR-21 | guards 92 / manifest / aggregate | 高 | 冻结完整≠body 有效（attempt3 例证） |
| CR-22 | ABI 81 槽/签名/JSON 字段 | — | 兼容性≠正确性 |
| CR-31 | 证据 README 判据口径 | 高 | 文档把 B 类写成 A 类（U4 根因之一） |
| CR-32 | 判据版本/契约 | 中 | 仅 SDF 判据做到版本化签署，其余无版本 |

## 附：U4 案例展开（CR-18）

- 判据：`render_sdf_reference` 用 `MAX_DEPTH_ULP=4` + depth `mismatched==0`，参考是 scene-sdf 的 warp GPU 采集。
- 被测物本质：SDF（`gtype==8`）在设备侧走底层 mesh 求交（`ray_mesh_leaf`），与 scene-a 的普通球 `ray_sphere` 不是同一 kernel 形态；上游 warp 对 `a=dot(vec,vec)` 在三项 FMA 收缩上按场景生成不同次序（RCA：scene-a `(0,1,2)`、scene-sdf `(1,0,2)`）。
- 结果：attempt1/2 同一球轮廓 215/76,800（max 191 ULP）与 4/43,200（max 25 ULP）；加 fix 后 attempt3 反使 6 个 scene-a 回归在 183 像素上 max 73 ULP、aa2 臂 max 91 ULP——**无单一产品实现能同时位级匹配两份参考**。
- 审计结论：该判据在 U4 上的使用同时命中 ①②③：位级 + 参考跨 kernel 形态、常量无推导、跨场景复用。criterion-v2（CR-17）用误差模型推导 B=128 + 冻结清单做了首次 A 类修正，但前提（冻结时极值、单场景、清单同源）必须在落地时写明。
