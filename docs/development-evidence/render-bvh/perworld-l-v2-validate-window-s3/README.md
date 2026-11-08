# S-3 验证窗材料（perworld L v2 灯 · G-L2；**proposed，未 start**）

- 任务：`2026-11-08-render-perworld-l-v2-validate-window-s3-material`（owner Arch、p80；**本卡只承载材料，不是验证窗卡**）
- 卡状态说明：按 Chief 派工本卡应保持 `proposed`，但 normify 硬规则禁止 `proposed` 卡提交工作（commit-msg 钩子拒绝），因此材料落地时 start 了本卡；**验证窗本身未开**（仍被 G-L1 收口 + 用户 S-3 双签 + Chief token 挡住），G-L1 收口后填 PENDING、跑门禁再收口
- 依据：**已签 spec v3（S-1）** `docs/development-evidence/render-bvh/perworld-l-v2-light-criterion/spec.md` sha256 `bb064fe0a85ab38abba8043768666a0d9f73fc53d9f2f3dd72eace060c8a924d` / **39,379 B** / 344 行 / 15 节；逐条对齐 **§10.1（判据实体）**、**§10.2（窗口材料与纪律）**、**§4/§5/§6（判据 A/B/C 与 C1/C2）**、**§8（失败条件）**、**§9.3（自查顺序）**、**§12 G-L2**
- 本材料 **CPU、零 GPU、零采集**；**不 start 窗口卡**、不碰 `src/` / `fixtures/` / `scripts/` / `tests/`
- **G-L1 参考尚未产出**：凡需新参考 sha/bytes/份数/证据锚点的位置一律写 `PENDING-G-L1`（完整清单见 §8），**G-L1 收口后填入、冻结前定稿**；**签名卡字段在 G-L1 完成前不定稿**

## 1. 前置核对表（缺项禁签 / 禁开窗）

| # | 前置 | 当前状态 | 证据锚点 |
| --- | --- | --- | --- |
| P0 | **S-1 已签且 spec 冻结** | ✅ | spec v3 `bb064fe0a85ab38abba8043768666a0d9f73fc53d9f2f3dd72eace060c8a924d`/39,379 B；S-1 receipt `status=effective`；签名卡 `…-spec-v3-signature`、QA `5e82feed` |
| P1 | **G-L1 采集证据 verified**：两次采集逐位一致 + C1 与 C2 两套两对 world 均 ≥576 + 元数据/双相机/legacy 自核 | **PENDING-G-L1** | `PENDING-G-L1: g-l1-evidence`（G-L1 窗卡 id/提交、reference 5 文件 sha/bytes、capture.json 判别力计数） |
| P2 | **判据实体 `render_per_world_light` 实现 verified + QA 复核 PASS** | 实体已可用（§2.4 字典适配 fix 复核 PASS `be12145e`；`check_capture_c1` 显式 Compared/Skipped） | 最终以 **G-L1 收口时的 verified 提交**为准：`PENDING-REFRESH: entity-anchors` |
| P3 | **guards 冻结**：1 新 exe + ≥10 必需输入（≤12）+ 旧 suite 只读保留 | 清单写死于 §2；**实际冻结 PENDING-G-L1** | `PENDING-G-L1: l2-guards-json`（suite 名/条目数/aggregate/stash） |
| P4 | **三点基线 + S-3b signed floor**（用户签） | **PENDING**：开窗前 5 分钟内实测，签名前标 `suggested` | 方法见 §3；数值 `PENDING-G-L1: signed-floor` |
| P5 | **常规 monitor 60/60 util<40**（`compute_filtered=0`、`build=0`） | 窗口时执行 | `materials/window-discipline.txt` |
| P6 | **零编译口径**：冻结阶段 `cargo test --no-run` exit 0，body 内零编译 | 冻结阶段执行并记录 | §3.4 |
| P7 | 树 clean、无未提交改动、HEAD 记录 | 窗口时执行 | §3.5 |
| P8 | **新参考体积 ≈11.08 MB / 5 文件**（实测量入清单） | **PENDING-G-L1** | `PENDING-G-L1: reference-files` |

> 缺任一项 → **S-3 不得签**、G-L2 不得开；不得为凑条件改阈值/参考/相机/recipe，不得复用旧 suite 冻结件冒充。

## 2. guards 冻结清单（spec §10.2 逐条写死）

### 2.1 条目算术（写死）

既有 perworld suite **96 条** = 11 exe + 83 输入 + 2 条 criterion v2.2 清单（`perworld-impl-s1/window-attempt3/guards-perworld-s1.json`）。本窗：

- 新增 **1 个 exe**：`render_per_world_light` → 12 exe
- 新增输入 **10 条必需**（下表）至多 **12 条** → 输入 83→93..95
- 2 条 criterion v2.2 清单保留（`recipe` / `aa2` 两臂），不改文本

合计 **107 条（最少）/ 109 条（最多）**，最终以冻结清单实际条目为准，**不得少冻结下列 10 条必需输入**。

### 2.2 新增 exe（1 条）

| 项 | 值 |
| --- | --- |
| 实体 | `render_per_world_light` |
| 可执行文件 | `target/debug/deps/render_per_world_light-<hash>.exe`（**PENDING-G-L1**：hash 由冻结时 `cargo test --no-run --message-format=json` 解析） |
| stash 副本 | `target/render-bvh/perworld-l2-window/frozen-exes/render_per_world_light.exe`（**PENDING-G-L1**） |
| 三重一致 | `live == guard == stash`（path/bytes/sha256 逐位；缺一即停手） |

### 2.3 新增输入（10 条必需，逐条写死）

| # | 路径 | 说明 |
| --- | --- | --- |
| 1 | `fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040/rgb.npz` | 新 tag 5 文件之一（双相机内容、5 个文件不变） |
| 2 | `…/depth.npz` | 同上 |
| 3 | `…/seg.npz` | 同上 |
| 4 | `…/state.npz` | 同上（判据 B 的 `light_xpos`/`light_xdir` 来源） |
| 5 | `…/capture.json` | 同上（含 §2.4 判别力计数、双相机快照、元数据四项） |
| 6 | `fixtures/render-bvh/scene-a-light/scene.xml` | 场景变体（sha `4b354054…`/5,134 B，已验证） |
| 7 | `scripts/capture_render_perworld.py` | 采集脚本（`f3af09ab…`/23,637 B/490 行） |
| 8 | `scripts/perworld_capture_meta.py` | 采集元数据模块 + refit helper（`3fdb8595…`/19,154 B/379 行） |
| 9 | `scripts/perworld_light_prep.py` | 备选模块（`9fd5f8c2…`/7,206 B/160 行） |
| 10 | `docs/development-evidence/render-bvh/perworld-l-v2-light-criterion/spec.md` | 本规格（`bb064fe0a85ab38abba8043768666a0d9f73fc53d9f2f3dd72eace060c8a924d`/39,379 B） |

**其余至多 2 条**（冻结器/证据输入，可选，`PENDING-DECIDE: optional-inputs`）：候选 = 冻结器脚本本身、L2 guard 清单副本/证据输入；**不得**用可选名额替代上面任何必需条目。

### 2.4 只读保留（写死）

- `docs/development-evidence/render-bvh/perworld-impl-s1/window-attempt3/guards-perworld-s1.json`（96 条）与其 `target/render-bvh/perworld-s1-window/frozen-exes/` stash **只读保留**，不重冻、不改写、不复用为本次证据。
- 既有 `guards.json` / `guards-r02-skybox.json` / `guards-r08-rgb-hetero.json` / `guards-sdf-validate.json` 等历史清单同样只读。

### 2.5 aggregate 与清单字段（写死）

- 沿用 96 条算法：**对所有条目 sha256 十六进制串按 repo-relative 路径排序后无分隔拼接再取 sha256**；新 suite 的 `aggregate_sha256` 写入新 guards JSON（**PENDING-G-L1**）。
- 新 guards JSON 必含：`suite`、`task_id`、`generated_at`、`counts`（frozen_exes/guard_inputs/outline_lists/total）、`entity_order`（12）、`frozen_exes`（每条 entity/source/path/bytes/sha256/stash_path/stash_bytes/stash_sha256）、`guard_inputs`（每条 path/bytes/sha256）、`outline_lists`（2 条）、`frozen_exe_stash_dir`、`aggregate_algorithm`、`aggregate_sha256`。
- `PENDING-G-L1: l2-guards-json` —— 路径建议 `scripts/validate/render-bvh/guards-perworld-l2-light.json`，冻结后填实际值。

## 3. 窗口纪律（spec §10.2，写死）

1. **冻结时机**：常规 monitor PASS 之后、窗口 body 之前完成 guards 冻结；**冻结后零编译**（不跑 `cargo`/`nvcc`；实测只跑冻结 exe；按 §10.2，冻结后仅允许 `--features cuda-probe` 口径的构建/运行）。
2. **三点基线**：开窗前 **5 分钟内**三次采样（~5 s 间隔，`nvidia-smi` 逐字记录）；`baseline_free = min(三点 free)`（**采样取严**，与合并规则不同）。
3. **floor**：现场 `onspot_floor = baseline_free − 3,072 MiB`；用户 S-3b 签的 `signed_floor`；**`floor_effective = max(signed_floor, onspot_floor)`**——验收条件 `free ≥ floor_effective`，**floor 越高越严**（取 `min` 等于放宽）。基线三点、monitor 全部样本、实体运行期间抽检都须 `free ≥ floor_effective`。
4. **monitor**：`scripts/validate/render-bvh/monitor-until-clean.ps1 -Required 60 -MaxMinutes 30 -MaxUtil 40 -OutCsv <显式路径>`；要求 **exit 0、60/60、util<40、`compute_filtered=0`、`build=0`**，monitor 期间无其它 CUDA 进程；脚本内部 10,240 MiB 下界**不替代** `floor_effective`。**authoring 例外（util<60）不适用于验证窗**（用户只批 G-L1）。
5. **树与 HEAD**：开窗前 `git status --porcelain` 空、`git rev-parse HEAD` 记录；窗口内不提交、不改仓。
6. **零编译证明**：冻结阶段 `cargo test --no-run` **exit 0** 并留 `cargo-artifacts.jsonl`；body 前记录 build 进程数 0；body 结束后再证 `git status` 空。
7. **冻结 exe 三重一致**：`live == guard == stash`（每个条目 path/bytes/sha256）；任一不等 → 停手。
8. **单次运行、失败即停**：不重试、不改阈值/参考/相机、不换冻结件；失败保留现场（日志/产物原样）。

## 4. 实体执行预期（spec §4/§5/§6/§8/§10.1，写死）

- **判据 A（逐 world 渲染，两台相机各自）**：每个 world、每台相机 rgb `max_byte_diff = 0`；depth `mismatched_pixels = 0` 且 `beyond_ulp = 0`；seg 逐像素 `(id,type)` 全等；两跑 sha256 相等（rgb/depth/seg 各自、逐 world、逐相机）。
- **判据 B（灯字段数据面）**：`state.npz` 的 `light_xpos`/`light_xdir` 与产品侧逐 world **f32 位型相等，0 例外**。
- **判据 C（world 间判别力，参考侧自算，不用产品侧数据）**：C1 = `cam-persp` 相邻 world 对 `max_channel|Δrgb| ≥ 1/255` 计数；C2 = `cam-hinge` depth 位型变化 + seg pair 变化计数；**两对 world 均 ≥576 为采集前置门槛**（[384,576) 余量不足停手、<384 无判别力拒通过）；判据 A 验收门槛仍 384，两者都须满足。
- **§2.4 计数对账**：`capture.json` 的 `c1_rgb_changed_px`（bitwise+magnitude）/`c2_depth_changed_px`/`c2_seg_changed_px`/两个诊断项，与实体自算的 C1/C2 计数**两对 world 逐项相等**；实体 `check_capture_c1` 应输出 **Compared**（旧数组/缺字段时才会 Skipped → 视作证据不完整停手）。
- **world 隔离**：只有该 world 变化时其它 world 输出不变（两台相机都断言）；该断言**不替代** §4–§6。
- **诊断项只记录不判**：`cam-persp` depth/seg 预期 0/0、`cam-hinge` rgb 预期大；不得冒充 C1/C2。
- **criterion v2.2 关系**：既有 10 个回归实体继续按 v2.2 两臂清单判（recipe 183 px / `cfd1038a…`；aa2 172 px / `14893fc6…`），本实体**不继承**、不得引用其清单或 `|ULP|≤128` 洗通过。
- **refit 前提**：C2 依赖每 world refit（P0-1）；若 C2 退化为 0，按 §8 失败停手，不得现场改采集/相机。

## 5. 证据清单（`window/*`，详见 `materials/evidence-checklist.txt`）

baseline（三点原文 + `baseline_free`/`onspot_floor`/`signed_floor`/`floor_effective` 算术）、monitor（csv/summary/exit code）、guards 冻结副本（新 guards JSON + `cargo-artifacts.jsonl`）、冻结 exe stash + live/guard/stash 三重一致表、12 个实体日志与退出码、`readout.json`（C1/C2 + 元数据自核）、`capture.json` 副本、参考 5 文件 sha256+bytes、两跑 sha 对照、legacy 断言、零编译指纹、`git-before/after`、`post-gpu`、incidents、本窗 manifest（**PENDING-G-L1**：证据目录名建议 `docs/development-evidence/render-bvh/perworld-l-v2-validate-window-g-l2/`）。

## 6. QA 预审重点（材料读前，9 项）

1. 与已签 spec §10.1/§10.2/§12 G-L2 逐条对齐（不是转述）；2. guards 算术 96→107..109 且 10 条必需输入逐条在位、旧 suite 只读；3. 新 exe 1 条 + stash/三重一致口径；4. 纪律：冻结在 monitor 后/body 零编译、三点基线 5 分钟内、`floor_effective=max` 且区分采样 min、monitor 60/60 util<40、失败即停；5. 实体预期覆盖 A/B/C + C1/C2 384/576 + §2.4 对账 + world 隔离 + v2.2 非继承；6. PENDING-G-L1 清单完整、无提前定稿；7. 双签摘要 S-3a/S-3b 可分且 S-3b 标 suggested；8. 边界（CPU/零 GPU/不改代码/四门禁/manifest 卫生）；9. 非声明（不宣称验收通过/采集有效/上游等价/渲染结论）。

## 7. S-3 双签摘要（**草案，floor 数值待填；G-L1 完成前不定稿**）

| 签名项 | 内容（可粘贴，PENDING 处待填） |
| --- | --- |
| **S-3a（验证窗计划）** | G-L2 单次验证窗、执行者 Backend（唯一写者）；guards 冻结 = 96 + 1 新 exe（`render_per_world_light`）+ 10..12 新输入（含 spec §10.2 的 10 条必需）；新参考 5 文件 ≈11.08 MB（sha `PENDING-G-L1`）；冻结在 monitor 后、body 零编译、单次失败即停 |
| **S-3b（基线 + signed floor）** | `suggested`：开窗前 5 分钟内三点实测（`PENDING-G-L1: signed-floor`）；`baseline_free = min(三点)`；`signed floor = baseline_free − 3,072 MiB`；开窗现场重算并按 **`floor_effective = max(signed, 现场)`（更高更严）** |
| **失败条款** | 任一判据/计数/元数据/legacy/冻结件/编译纪律失败 → 停手报用户；不重试、不改阈值/参考/相机、不换冻结件 |
| **证据与 QA** | 证据目录 `…-validate-window-g-l2/`（PENDING-G-L1）；QA 材料预审 + 窗口后 QA 复核；窗口记录同 pilot/S-2 格式 |
| **签名不构成开窗** | 仍需 QA PASS + Chief G-L2 token + 现场 floor 取更高 + monitor 60/60 + 零编译 |

## 8. PENDING-G-L1 清单（G-L1 收口后逐项填入）

| # | 待填项 | 位置 |
| --- | --- | --- |
| 1 | G-L1 窗卡 id / 提交 / 证据路径 / QA 复核锚点 | §§1 P1、receipt `preconditions[P1]` |
| 2 | 新参考 5 文件 sha256 + bytes + 总字节（≈11.08 MB 实测） | §2.3 #1–5、receipt `guards.inputs` |
| 3 | `capture.json` 的 §2.4 判别力计数实测值 | §4、receipt `entity_expectations` |
| 4 | 新 guards JSON 路径 / suite 名 / 条目数 / aggregate / stash 路径 | §2.2/§2.5、receipt `guards` |
| 5 | `render_per_world_light` exe 路径/bytes/sha + 三重一致 | §2.2 |
| 6 | S-3b `suggested` baseline 三点实测 + signed floor 建议值 | §3.2/§3.3、`materials/signature-summary.txt` |
| 7 | 实体最终 verified 提交与 QA PASS 锚点（`PENDING-REFRESH`） | §1 P2 |
| 8 | 可选输入 0–2 条的最终取舍（`PENDING-DECIDE`） | §2.3 |

> **G-L1 未收口前：本材料不得定稿、S-3 签名卡不得发、guards 不得冻结、窗口卡不得 start。**

## 9. 边界与非声明

- 本卡**零 GPU、零采集、不改代码**；`unchanged src/`, `unchanged fixtures/`, `unchanged scripts/`, `unchanged tests/`；四门禁通过；材料 sha256/bytes 记 receipt+manifest。
- 不构成 G-L2 启动：需 G-L1 收口 + 实体 QA PASS + **用户 S-3 双签** + **Chief G-L2 token** + 窗口条件。
- **不宣称**采集有效、per-world 灯验收、与上游等价、任何渲染数值结论；本材料只是计划。
