# R06-S3b D8 复核：cam2 depth 缺口诊断（H1/H1b 分离）——**PASS-with-observations**

- 复核任务：`2026-11-04-render-r06-s3b-flex-d8-review`（owner QA）
- 复核对象：`2026-11-04-render-r06-s3b-flex-cam2-depth-gap`，提交 `a1cc22ae3fd58d0c6cfb9d2bda642f7d734bf7ee`；证据 `docs/development-evidence/render-bvh/r06-s3b-flex-cam2-depth-gap/`（README 120 行、manifest 16 条）
- 本片**只读复核**：`src/`、`fixtures/`、D8 证据文件**零改动**（`unchanged` 检查）；**零 GPU**、不重跑 D8 微窗口；独立复算只用两个已入库输入 + 一个 target-only CPU replay 产物（sha 记录在 §3），全程不 import、不执行 D8 的分析脚本。
- **结论：PASS-with-observations（1 需修-低 + 1 观察-低，均不影响 D8 结论）。**
  D8 的 headline（321/297/259、per-flexid、seg 0/76,800、轮廓距离、绝对差）、记账（微窗口基线 / dump sha+bytes / 两跑 hash / WIP 还原 / guards live≠frozen）与 exclusion 表**全部独立复现**；manifest 16/16 自核通过。
- 判据归类复核：按《机器判据裁定标准 v1》，「4 ULP 在薄胶囊近掠射像素上近似等价位级」归 **B 类（参考-前提型），降级使用**；Chief 2026-11-04 裁定采纳并写死签名级边界（原话见 §7.5）。**本片不据此宣布任何 pass，attempt5 仍按冻结判据跑。**

## 1. 复核面与结果

| # | 复核面 | 方法 | 结果 |
| --- | --- | --- | --- |
| 1 | 微窗口前置（compute=0、build、util、free、温度） | 读 `logs/d8-pre-window-facts.txt`（§2 逐行引） | 一致 |
| 2 | dump sha+bytes、两跑 hash、WIP 还原 | 读日志 + 对 target 实体重算 sha256 | 一致（§2/§3） |
| 3 | guards 口径（frozen attempt=1、16/16、exes=2、inputs=14、aggregate；live≠frozen） | 读日志 + 与 D8 README:47 对齐 | 一致；attempt5 必须重冻成立 |
| 4 | 复现红：cam2 depth `(mismatched,beyond4)=(321,297)` | 读 `logs/d8-run.log` | 一致 |
| 5 | H1/H1b 分离数字（总体、per-flexid、非 flex、seg、轮廓距离、绝对差） | **独立复算**（§3） | 全等 |
| 6 | 参考侧语义（depth==0 47,725 全非 flex；flex 深度全非零） | 独立复算 | 一致 |
| 7 | D8 manifest（16 条 sha256+bytes） | 自核脚本复算 | 16/16 通过 |
| 8 | `artifacts/summary.json` 数字 | 与复算值逐项比对 | 除 ULP 直方图外全等；直方图见 §4 |
| 9 | CPU-warp replay 敏感集重叠 | 独立复算（target-only 产物，sha 记录） | 317/321=98.8%，一致 |
| 10 | exclusion 表 H2–H7 | 抽核（文件:行，§6） | 通过 |
| 11 | 判据归类 A/B/C | 《机器判据裁定标准 v1》 | B 类、降级使用（§7） |
| 12 | z2vec 卡 `04793345` 卡面口径 | 读卡 JSON + 对照口径 | 按口径登记，通过（§8） |
| 13 | 越界/非声明 | `git diff` 范围与 README §9 | `src/`、`fixtures/`、判据/参考零改动 |

## 2. 微窗口记账与 guards（引 D8 原始日志，行号可查）

以下为 `docs/development-evidence/render-bvh/r06-s3b-flex-cam2-depth-gap/logs/d8-pre-window-facts.txt` 的**原行**（§10 的复算日志会重新打印同样的行）：

- `:89` `compute_filtered_count=0`；`:90` `# fresh three-point baseline:`；`:91-93` 三点 `14,314 MiB, 2 %` / `14,314 MiB, 4 %` / `14,314 MiB, 2 %`、temp 29。
- `:96` `run_exit=101 expected_judge_red wall_s=2.02 gpu_before=14318MiB/2%/29C gpu_after=14318MiB/8%/29C` —— 微窗口 **2.02 s**、单次无重试、退出即释放，与 D8 README:48 一致。
- `:98` / `:99` / `:100`：`depth_all.f32le` **787,200 B** `fa4cceaa…`、`seg_all.i32le` **1,574,400 B** `c4be9853…`、`meta.json` `a347b644…`；与 D8 README:50、`logs/dump-sha256.txt:3-5` 三方一致，且**对 target 实体重算 sha256 仍相等**（§3）。
- `:101` `wip_restored=true tree_after_restore_only_card_json=true` —— WIP 已还原、未入库（D8 README:51）。
- `:7-10` guards：`guards_sha256=70d4cbc6…`、`guards_status=frozen attempt=1 counts=frozen_exes:2,inputs:14,total:16 aggregate_inputs=ec6fa222…`、frozen `render_flex_validate` exe `d9686515…` bytes **107,672,581**。
- `:52-53` live：WIP exe `8a38fc6c…` 107,982,020；committed exe `49923e3e…` **107,660,771** → **live ≠ frozen**，与 D8 README:47 的「attempt5 必须重新真冻结」判断一致。
- 两跑 hash：`logs/d8-analyze.log:2` 的 `meta.json` 打印 `two_run_hashes` 6 项成对相等（rgb `a4f29b09…` / depth `fa4cceaa…` / seg `c4be9853…`），本次复算另从 target `meta.json` 直接核 `trh[:3]==trh[3:]` 为 True。

## 3. 独立复算（不复用 D8 脚本；命令见 §10）

输入 sha256（复算日志首段会打印；参考侧与 D8 证据引用的同一 `/reference/base/` 同源）：

| 输入 | bytes | sha256 |
| --- | --- | --- |
| D8 `artifacts/cam2-depth-seg.npz` | 34,405 | `dc3f98e9dc115a40cb72c72f3e0a7fa4812788ea444c6d7b508a3b18ef4d5391` |
| `fixtures/render-bvh/scene-flex/reference/base/depth.npz` | 787,918 | `c3ee4adca6cf9874f40ab87b7f72a16b17f26647e45758342895919b725d8deb` |
| `fixtures/render-bvh/scene-flex/reference/base/seg.npz` | 1,575,106 | `f3b00fa82df1e54d4c89309d820212d9808038e00ae25f25841f2b40a6277bef` |
| CPU replay（**target-only，未入库**）`target/s3b-verify/cpu-replay/cpu_cam2_depth.npz` | 17,580 | `75e5d5dd076a50bacf5ddf05c5506b11f0fb27035baca921d50bcf730025d646` |

复算结果（与 D8 README:59-68 逐项对照）：

| 指标 | 复算值 | D8 声明 | 判定 |
| --- | --- | --- | --- |
| mismatched / beyond4 / max_ulp | **321 / 297 / 259** | 321 / 297 / 259 | 一致 |
| flexid0（403 px） | mism 70 / beyond4 64 / max 259 | 同 | 一致 |
| flexid1（251 px） | mism **251** / beyond4 233 / max 155 | 同 | 一致 |
| 非 flex mismatch / beyond4 | 0 / 0 | 0 | 一致 |
| mismatch 落在 flex 区 | 321 / 321 | 321/321 | 一致 |
| seg vs 参考 | 0 / 76,800 失配（两通道全等） | 0/76,800 | 一致 |
| 绝对差（m） | 中位 2.384186e-6 / p90 1.144409e-5 / max 3.695488e-5 | 2.384e-6 / 1.144e-5 / 3.695e-5 | 一致 |
| 距 flex 轮廓 | 轮廓 196 px；中位 1.41 px；≤2 px 222/321；≤4 px **300/321** | 同 | 一致 |
| 参考 depth==0 | 47,725 px，全非 flex；flex 深度全非零 | 同 | 一致 |
| CPU replay 敏感集重叠 | 317/321 = **98.8%**；replay ULP 在 product-equal 中位 0（max 1049）、在 product-mismatch 中位 19（max 459） | 98.8%、中位 19 vs 0 | 一致 |
| replay beyond4 in flex | 597 / 654 | 597/654（D8 `logs/cpu-replay-sensitivity.log`） | 一致 |
| product beyond4 ∩ replay beyond4 | 256 / 297 | 256/297（D8 `logs/d8-analyze.log`） | 一致 |
| D8 manifest | 16 条 sha256+bytes 全过 | — | 通过 |
| `summary.json` 数字 | 除直方图外逐项 OK | — | 见 §4 |

> 注：`replay ULP at product-equal max=1049` 说明「同算法、不同 codegen」的 CPU 回放本身在 12.5% 像素上超 4 ULP（cam2 9,633 px、flex 区 597/654）——这既是 D8「冻结 ≤4 ULP 对薄胶囊近掠射等价于位级」判断的支撑，也是 §7 把它按 B 类**降级**而非当作位级等价声明的原因。

## 4. 需修-低：ULP 直方图桶边界双端闭区间（共享端点重复计数）

- **现象**：`README.md:60` 中文标签写成互斥桶「0、1、2–4、5–16、17–64、65–1024、>1024」，但值 **76,484 / 7 / 19 / 172 / 109 / 33 / 0** 之和 = **76,824**，比 76,800 多 24。
- **根因（代码级）**：D8 `scripts/analyze_dump.py:62-63` 用 `edges=[0,1,2,4,16,64,1024,1e9]` + `(ulp >= lo) & (ulp <= hi)`——**两端都闭**，共享端点 1/2/4/16/64 被相邻两桶各计一次；`artifacts/summary.json` 的 `ulp_histogram` 键名 `0-1`、`1-2`、`2-4`、`4-16`… 也沿用同一口径。
- **量化**：共享端点上的像素数 `ulp==1:5`、`==2:2`、`==4:15`、`==16:2`、`==64:0`，合计 **24** = 76,824 − 76,800。
- **互斥复算（QA 独立，8 桶）**：`[0,1,2,3-4,5-16,17-64,65-1024,>1024] = [76,479, 5, 2, 17, 157, 107, 33, 0]`，和 = **76,800**；合并成 D8 的 7 桶口径为 `76,479 / 5 / 19 / 157 / 107 / 33 / 0`。
- **影响评估**：只影响分布标签与桶和；`mismatched=321`、`beyond4=297`、`max_ulp=259`、per-flexid、seg、距离、绝对差与 H1/H1b 结论**均不受影响**（§3 逐项吻合）。
- **处置**：Chief 2026-11-04 指示「Backend 下次证据提交顺手改成 `[lo,hi)`/端点去重并补 replay 关键统计或 sha，不阻塞修复卡」。本片登记为**需修-低**，不改 D8 证据字节（改证据属重写历史，非本片范围）。
- 复现：§10 日志 `== ULP histogram: D8 committed inclusive-bounds reproduction vs QA disjoint ==` 段。

## 5. 观察-低：per-pixel CPU replay 产物未入库

D8 只入库了 replay 的**汇总日志**（`logs/cpu-replay.log`、`cpu-replay-sensitivity.log`），逐像素产物 `target/s3b-verify/cpu-replay/cpu_cam2_depth.npz`（17,580 B，sha `75e5d5dd…`）在 target 未入库，因此 98.8%（317/321）**无法仅凭仓内文件复算**。本次复核把该产物的 sha256/bytes 记入本证据（§3 表），并已在复算日志中原样打印；建议后续（Backend 下次证据提交）把 replay 关键统计或该 npz 的 sha 写进 D8 证据，不必入库大文件。

## 6. exclusion 表抽核（H2–H7）

| 假设 | D8 结论 | 本次抽核到的事实 |
| --- | --- | --- |
| H2 D5 残留（相机位姿） | 否 | `logs/d8-run.log` 打印三相机 data 位姿与 model-static 对照（cam0 angle=0.025° max_abs=0.000000；cam1/cam2 的 85.517°/75.562° 是二者姿态定义差，非位姿误差；D5 的位级 provenance 已由 D5 卡 `ab8da091` 验过，本片不重开 D5）；且 cam0/cam1 depth 位级过（`(0,0)`） |
| H3 D7 残留（flex 上传/几何） | 否（几何） | seg 位级全等（复算 0/76,800）+ D7 witness → 命中几何一致 |
| H4 depth 语义/公式 | 否 | `render_megakernel.cu:243-245` 与上游 `render.py:1285-1294` 同式（D8 README §2 表）；cam0/1 位级过 |
| H5 参考/期望口径 | 否 | 复算：参考 `depth==0` 47,725 px 全非 flex、flex 654 px 深度全非零连续 |
| H6 BVH leaf AABB | 否（hit 层） | seg 位级全等 + 非 flex mismatch 0 → 候选/命中面一致 |
| H7 depth 槽/相机映射 | 否 | `meta.json`（D8 `logs/d8-pre-window-facts.txt` 引）`depth_adr=[0,76800,120000]`、`cam_res` 与参考形状一致；cam2 rgb 过 |

## 7. 判据归类：B 类（参考-前提型），降级使用

### 7.1 A/B/C 判定
- **不是 A（本质-推导型）**：无法由深度语义从本质上推出「≤4 ULP ≈ 位级」；该说法是经验性的、绑定本次参考位级/recipe/GPU 的结论。
- **不是 C（完整性-哈希型）**：不涉及完整性/哈希棘轮。
- **是 B（参考-前提型）**：结论成立依赖外部前提（同参考位级口径、差异局限于薄胶囊近掠射子集、误差有界、判据未放宽、范围受限）；前提不成立即降级。

### 7.2 五条前提（必须与结论同时声明）
1. **同一参考位级口径**：同 reference npz（depth `c3ee4adc…` / seg `f3b00fa8…`）、同 recipe、同 GPU/工具链（RTX 4070 Ti SUPER、driver 596.36、CUDA 12.8.1、warp 1.15.0 / mujoco_warp 3.15.0 pinned）。
2. **差异仅限 1D 薄胶囊（r=0.02/0.04）近掠射轮廓子集**：321/321 在 flex 区、300/321 距轮廓 ≤4 px（中位 1.41 px）、非 flex mismatch **0**、seg 位级 **0/76,800** 失配、hit/miss 不变。
3. **误差有界且属同面数值尾差**：|Δ| 中位 2.38e-6 m / p90 1.14e-5 / max **3.70e-5 m**；ULP max **259**、无 >1024；「同算法、不同 codegen」CPU 回放敏感集重叠 **98.8%**（replay ULP 在 product-mismatch 像素中位 19、在 product-equal 中位 0）。
4. **冻结判据未放宽**：`tests/render_flex_validate.rs:344/:347` 仍断言 `(mismatched, beyond)=(0,0)`、`MAX_DEPTH_ULP=4`（`src/render/kernel/output.rs:15`）；D8 未改判据/容差/参考（其 README §9），本复核 `src/`、`fixtures/` 零 diff。
5. **范围仅本场景/本几何/该容差族**：scene-flex、1D flex 薄胶囊、R05/R02 家族 depth ≤4 ULP 口径；不外推到其它场景、几何或 rgb/seg。

### 7.3 降级含义与签名级边界
- 只登记为「**已知 codegen 级 ULP 尾差，在薄胶囊近掠射子集上按 ≤4 ULP 计**」的 B 类前提结论；
- **单独不得作通过依据**、**不得改写成「位级等价」**、**不得据此宣布 attempt5/flex 渲染通过**；
- **写死一条（Chief）**：不得用这条 B 类前提替代冻结的 bit-first 判据；若要用它替代 → 属**放宽判据（签名级）**，必须用户签，不能由 QA/Chief 内部消化；**attempt5 仍按冻结判据跑**。

### 7.4 失效条件（任一即升级阻断重裁）
差异逸出 flex 轮廓子集 / seg 或命中面变化 / 绝对差超出 0.04 mm 量级 / 出现 >1024 ULP 长尾 / 参考或 recipe/GPU 变更 / 换场景或几何。

### 7.5 裁定原话与来源
- 裁定归属：**Chief**（QA 审计 + Chief 裁定，按《机器判据裁定标准 v1》）。**不是**任何 Bot 的建议；Arch 的「A 类保留性质」仅属建议、不构成裁定。
- Chief 2026-11-04 消息原话（D8 复核回复，逐字）：
  > **判据归类：B 类采纳**（五前提 + 降级含义 + 失效条件按你写的落）。**写死一条**：不得用这条 B 类前提替代冻结的 bit-first 判据；若要用它替代 → 属**放宽判据（签名级）**，必须用户签，不能内部消化。attempt5 仍按冻结判据跑。
- 来源：Chief 2026-11-04 对本复核的回复（本会话消息，`part 18` 之后）；本证据只记录、不改写裁定。

## 8. z2vec 卡 `04793345` 卡面口径核对

对象：`.normify/changes/2026-11-04-render-r06-s3b-flex-z2vec-parallel-branch.json`（`status=backlog`、owner Backend、p40、`depends_on=[]`）。核对结论：**按口径登记，通过**。

- **可判口径**：acceptance 第 2 条明确「**可判口径用矩阵、不用字段序**（字段序依赖容器约定，易误判）」，并给出三条矩阵断言：`vec=(0,0,-1)` → **单位阵**（修复后必须相等；修复前产品为退化矩阵）；`vec=(0,0,1)` → `diag(-1,-1,1)`（**不是单位阵**，勿按直觉判红）；`vec=(0,1,0)` → `[[1,0,0],[0,0,1],[0,-1,0]]`。验收不依赖字段序。
- **边界**：不改判据/容差/参考、不动 `fixtures/`、不改 `scene-flex`；零 GPU（纯 host/静态可判）；「本卡与 attempt5 无依赖关系，**不得用它当 attempt5 前置**」，并写明与 cam2 depth 缺口（D8）、attempt4 CUDA 700 无关。
- **优先级**：p40 backlog，符合「潜伏缺陷、当前场景无 z 向 flex 边未触发」的定位；不改动任何在途判据面。
- 备注：intent 用字段序 `(1,0,0,1)` vs 上游 `(1,0,0,0)` 描述**现象**、验收用**矩阵**判——符合口径，无需修改。

## 9. 非声明与边界

- 不宣称 D8 的 H1 修复完成、不宣称 attempt5 通过、不宣称 flex 渲染通过/不通过；不据此发布任何判据结论。
- 不改判据/容差/参考/guards；不重冻；不改 D8 证据字节（直方图缺陷按 §4 交 Backend 下次证据提交修）。
- 本片只读：`src/`、`fixtures/` 零 diff（`unchanged` 检查）；零 GPU、零窗口、零重跑。
- 证据只覆盖「D8 证据的可复核性 + 该判据说法的 B 类归类」；不覆盖 flex 修法（另有 fix 卡）。

## 10. 复现

```powershell
# 仓库根执行；CPU-only，不触 GPU。--out 让脚本自己以 LF 写日志（字节确定、manifest 可复现），
# 同时仍把同样内容打到控制台。
$ev = "docs\development-evidence\render-bvh\r06-s3b-flex-d8-review"
target\mjwarp-parallel\venv\Scripts\python.exe -B "$ev\scripts\qa_recompute.py" --out "$ev\logs\qa_recompute.log"
# 若 CPU replay 的 target 产物不存在，脚本会用 --no-replay 跳过该段（其余复核照常）：
target\mjwarp-parallel\venv\Scripts\python.exe -B "$ev\scripts\qa_recompute.py" --no-replay
```

环境：Windows 11 26200 / python 3.12.13 / numpy 2.5.4（venv `target/mjwarp-parallel/venv`）。

## 11. 产物清单

- `README.md`（本文）、`facts.json`（机器可读，与本文同源）、`manifest.txt`（sha256+bytes）
- `scripts/qa_recompute.py`（独立复算脚本，只用 §3 表列输入）
- `logs/qa_recompute.log`（原样输出，含输入 sha、直方图对比、manifest 自核、summary 比对、dump 实体核对、replay 交叉、D8 记账引文）
