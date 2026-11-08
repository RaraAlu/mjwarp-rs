# S-3 验证窗材料（perworld L v2 灯 · G-L2）

- 任务：`2026-11-09-render-perworld-l-v2-validate-window-s3-fill`（owner Arch、p80；**材料卡已 verified**，fill 轮由 fill 卡承载；**验证窗未开**）
- 依据：**已签 spec v3（S-1）** `docs/development-evidence/render-bvh/perworld-l-v2-light-criterion/spec.md` sha256 `bb064fe0a85ab38abba8043768666a0d9f73fc53d9f2f3dd72eace060c8a924d` / **39,379 B** / 344 行 / 15 节；逐条对齐 **§10.1（判据实体）**、**§10.2（窗口材料与纪律）**、**§4/§5/§6（判据 A/B/C 与 C1/C2）**、**§8（失败条件）**、**§9.3（自查顺序）**、**§12 G-L2**
- **fill 状态（2026-11-09）**：G-L1 已收口，8 组 `PENDING-G-L1` 中可填的 6 组已填实；剩余 **`PENDING-FREEZE`**（12 个 live exe 的 sha/bytes 与最终 aggregate，冻结时产出）、**`PENDING-QA`**（G-L1 采后 QA 复核锚点）、**`PENDING-REFRESH`**（实体最终 verified 锚点）。
- 本材料 **CPU、零 GPU、零采集**；**不 start 窗口卡**、不碰 `src/` / `fixtures/` / `tests/`；本 fill 轮**新增一个冻结器脚本**（见 §2.6，`unchanged scripts/` 除外该新文件）。

## 1. 前置核对表（缺项禁签 / 禁开窗）

| # | 前置 | 当前状态 | 证据锚点 |
| --- | --- | --- | --- |
| P0 | **S-1 已签且 spec 冻结** | ✅ | spec v3 `bb064fe0a85ab38abba8043768666a0d9f73fc53d9f2f3dd72eace060c8a924d`/39,379 B；S-1 receipt `status=effective`；签名卡 `…-spec-v3-signature`、QA `5e82feed` |
| P1 | **G-L1 采集证据 verified**：两跑逐位一致 + C1/C2 两套两对 ≥576 + 元数据/双相机/legacy/§2.4 计数自核 | ✅ **G-L1 已收口**（QA 采后复核 `PENDING-QA`） | 窗卡 `2026-11-09-render-perworld-l-v2-capture-window-g-l1`；提交 **`6951c3fe`**；证据 `docs/development-evidence/render-bvh/perworld-l-v2-capture-window-g-l1/`（README `d7517c29…`/11,360 B、receipt `da780949…`/10,952 B、manifest **43/43**）；readout `verdict=pass-margin`；`two_captures_bitwise_identical=true`；`legacy_reference_untouched=true` |
| P2 | **判据实体 `render_per_world_light` 实现 verified + QA PASS** | 实体可用（§2.4 字典适配 fix 复核 PASS `be12145e`）；最终锚点 `PENDING-REFRESH` | 最终以 verified 提交为准 |
| P3 | **guards 冻结**：1 新 exe + 10 必需输入（可选 0）→ **107 条** | 清单写死 §2；冻结器已落（CPU 自检 PASS）；**实际冻结 `PENDING-FREEZE`** | `materials/guards-freeze-list.txt`、`artifacts/freezer-selfcheck.txt` |
| P4 | **三点基线 + S-3b signed floor**（用户签） | **suggested 已测**（§3.3）；用户签名值 `PENDING` 开窗前 5 分钟实测 | §3.3、`materials/baseline-suggestion.txt` |
| P5 | **常规 monitor 60/60 util<40**（`compute_filtered=0`、`build=0`） | 窗口时执行 | §3 |
| P6 | **零编译口径**：冻结阶段 `cargo test --no-run` exit 0，body 内零编译 | 冻结时执行并由冻结器留 `cargo-artifacts.jsonl` | §3 |
| P7 | 树 clean、无未提交改动、HEAD 记录 | 窗口时执行 | §3 |
| P8 | **新参考 5 文件实测 sha/bytes，总 11,078,239 B** | ✅ | §2.3 |

> 缺任一项 → **S-3 不得签**、G-L2 不得开；不得为凑条件改阈值/参考/相机/recipe，不得复用旧 suite 冻结件冒充。

## 2. guards 冻结清单（spec §10.2 逐条写死）

### 2.1 条目算术（写死）

既有 perworld suite **96 条** = 11 exe + 83 输入 + 2 条 criterion v2.2 清单（`perworld-impl-s1/window-attempt3/guards-perworld-s1.json`，**只读**）。本窗：

- 新增 **1 个 exe**：`render_per_world_light` → 12 exe
- 新增输入 **10 条必需**（§2.3）→ 83+10 = 93 输入
- **可选输入 = 0 条（Chief 2026-11-09 决定）**，不得用可选名额替换必需行
- 2 条 criterion v2.2 清单保留（`recipe` / `aa2`），不改文本

合计 **107 条**（12 + 93 + 2）。

### 2.2 新增 exe（1 条）

| 项 | 值 |
| --- | --- |
| 实体 | `render_per_world_light` |
| live 可执行文件 | `PENDING-FREEZE`（`cargo test --no-run --message-format=json` 解析，形如 `target/debug/deps/render_per_world_light-<hash>.exe`） |
| stash 副本 | `target/render-bvh/perworld-l2-window/frozen-exes/render_per_world_light.exe` |
| 三重一致 | `live == guard == stash`（path/bytes/sha256；缺一即停手，不运行非冻结 exe） |

### 2.3 新增输入（10 条必需，逐条实测）

| # | 路径 | sha256 | bytes |
| --- | --- | --- | --- |
| 1 | `fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040/rgb.npz` | `602574c2…` | 5,531,026 |
| 2 | `…/depth.npz` | `82eec6da…` | 1,844,650 |
| 3 | `…/seg.npz` | `c3f09b16…` | 3,687,826 |
| 4 | `…/state.npz` | `a1c5a321…` | 2,486 |
| 5 | `…/capture.json` | `8b53e122…` | 12,251 |
| 6 | `fixtures/render-bvh/scene-a-light/scene.xml` | `4b354054…` | 5,134 |
| 7 | `scripts/capture_render_perworld.py` | `f3af09ab…` | 23,637 |
| 8 | `scripts/perworld_capture_meta.py` | `3fdb8595…` | 19,154 |
| 9 | `scripts/perworld_light_prep.py` | `9fd5f8c2…` | 7,206 |
| 10 | `docs/development-evidence/render-bvh/perworld-l-v2-light-criterion/spec.md` | `bb064fe0…` | 39,379 |

- 参考 5 文件合计 **11,078,239 B**（实测；spec §2.3 的 11,076,054 是**估算**，冻结与签名以实测为准）。
- `capture.json.files_sha256` 与上表 4 个 npz 逐位一致（冻结器自检项）。

### 2.4 只读保留（写死）

- 旧 suite `guards-perworld-s1.json`（`ca3ecd5c…`/38,659 B，96 条，aggregate `8a4ddc50…`）与其 stash `target/render-bvh/perworld-s1-window/frozen-exes/`（11 文件）**只读保留**；不重冻、不改写、不复用为本次证据。
- 冻结器在冻结后复查旧 suite sha 不变；**实测旧 stash 有 1 条漂移**（`render_shadow_reference.exe`：记录 101,102,921 B/`ce5139…`，实际 100,859,152 B/`4500bb…`，最后写入 2026-11-03T23:09:54+08:00），属**预存在事实**，只报不修；G-L2 会重建并重新 stash 自己的 12 个 exe，因此旧 stash 仅历史参照。**冻结器在 `-Execute` 遇到该漂移会拒绝，除非 Chief 裁定后加 `-AllowOldStashDrift`。**
- 历史清单 `guards.json` / `guards-r02-skybox.json` / `guards-r08-rgb-hetero.json` / `guards-sdf-validate.json` 同样只读。

### 2.5 aggregate 与 guards JSON

- 算法（写死）：**对全部条目 sha256 十六进制串按 repo-relative 路径排序后无分隔拼接，再取 sha256**；冻结器已用旧 suite 96 条独立复算验证算法一致（`8a4ddc50…`）。
- 新清单路径：**`scripts/validate/render-bvh/guards-perworld-l2-light.json`**（冻结时写入，`PENDING-FREEZE`）；必含 `suite`/`task_id`/`generated_at`/`counts`(107)/`entity_order`(12)/`frozen_exes`/`guard_inputs`(93)/`outline_lists`(2)/`frozen_exe_stash_dir`/`aggregate_algorithm`/`aggregate_sha256`/`old_suite`/`old_stash_drift`/`optional_inputs=0`。
- `aggregate_sha256` 与 12 个 exe 的 sha/bytes = **`PENDING-FREEZE`**。

### 2.6 冻结器（本 fill 轮新增；未真冻结）

- 脚本：`scripts/validate/render-bvh/freeze-guards-perworld-l2-light.ps1`（`14d82190…` / 20,802 B；ASCII-only）
- 默认 **dry-run/结构自检**：不跑 cargo、不构建、不 stash、不写 guards JSON；只写 `target/render-bvh/perworld-l2-window/freeze-plan-dryrun.json`。已跑出 **SELFCHECK PASS 13/13**（`artifacts/freezer-selfcheck.txt`），其中旧 suite 96 条 aggregate 独立复算一致、旧 suite sha 基准一致、10 条必需输入 sha/bytes 逐条一致、参考总量 11,078,239 B、`capture.json.files_sha256` 一致、12+93+2=107、新 exe 唯一且与旧 11 条不重名、新 stash 目录与旧只读 stash 不同。
- `-Execute`（**窗口期**，monitor PASS 后、body 前，Backend 执行）：唯一允许 `cargo test --locked --no-run` 的位置；解析 12 个 exe、stash、计算 aggregate、写 guards JSON、复查旧 suite 未变与 `live==guard==stash`。
- **旧 stash 漂移**：`-Execute` 默认拒绝，需 Chief 裁定 + `-AllowOldStashDrift`。

## 3. 窗口纪律（spec §10.2，写死）

1. **冻结时机**：常规 monitor PASS 之后、窗口 body 之前运行冻结器 `-Execute`；**冻结后零编译**（不跑 `cargo`/`nvcc`；实测只跑冻结 exe；按 §10.2，冻结后仅允许 `--features cuda-probe` 口径的构建/运行）。
2. **三点基线**：开窗前 **5 分钟内**三次采样（~5 s 间隔，`nvidia-smi` 逐字记录）；`baseline_free = min(三点 free)`（**采样取严**，与合并规则不同）。
3. **floor**：现场 `onspot_floor = baseline_free − 3,072 MiB`；用户 S-3b 签的 `signed_floor`；**`floor_effective = max(signed_floor, onspot_floor)`**——验收条件 `free ≥ floor_effective`，**floor 越高越严**（取 `min` 等于放宽）。基线三点、monitor 全部样本、实体运行期间抽检都须 `free ≥ floor_effective`。
4. **monitor**：`monitor-until-clean.ps1 -Required 60 -MaxMinutes 30 -MaxUtil 40 -OutCsv <显式路径>`；要求 **exit 0、60/60、util<40、`compute_filtered=0`、`build=0`**，monitor 期间无其它 CUDA 进程；脚本内部 10,240 MiB 下界**不替代** `floor_effective`。**authoring 例外（util<60）不适用于验证窗**（用户只批 G-L1）。
5. **树与 HEAD**：开窗前 `git status --porcelain` 空、`git rev-parse HEAD` 记录；窗口内不提交、不改仓。
6. **零编译证明**：冻结器 `-Execute` 留 `cargo-artifacts.jsonl`；body 前记录 build 进程数 0；body 后 `git status` 空。
7. **冻结 exe 三重一致**：`live == guard == stash`（12 条，path/bytes/sha256）；任一不等 → 停手。
8. **单次运行、失败即停**：不重试、不改阈值/参考/相机、不换冻结件；失败保留现场（日志/产物原样）。

### 3.3 S-3b 建议值（2026-11-09 零 GPU 三点实测，`suggested`）

- 测量：`2026-11-09T00:42:10/15/20+08:00`（~5 s 间隔），host DARKSEA，`nvidia-smi` only；used **3,034 MiB**、free **13,028 / 13,028 / 13,028 MiB**、util **35 / 38 / 38 %**、28 °C；构建进程 0；HEAD `6951c3fe`、树 clean。
- `baseline_free = min(三点 free) = **13,028 MiB**`（**采样取严**）；`onspot_floor = 13,028 − 3,072 = **9,956 MiB**`。
- **suggested signed floor = 9,956 MiB**（**不复用 S-2 的 13,302/10,230**：不同窗口、不同时刻）。
- 开窗前必须**现场重测**三点并重算；`floor_effective = max(signed, 现场)`（**更高更严**）。
- 记录：`materials/baseline-suggestion.txt`。

## 4. 实体执行预期（spec §4/§5/§6/§8/§10.1，写死）

- **判据 A（逐 world 渲染，两台相机各自）**：每个 world、每台相机 rgb `max_byte_diff = 0`；depth `mismatched_pixels = 0` 且 `beyond_ulp = 0`；seg 逐像素 `(id,type)` 全等；两跑 sha256 相等（rgb/depth/seg 各自、逐 world、逐相机）。
- **判据 B（灯字段数据面）**：`state.npz` 的 `light_xpos`/`light_xdir` 与产品侧逐 world **f32 位型相等，0 例外**。
- **判据 C（world 间判别力，参考侧自算）**：C1 = `cam-persp` 相邻 world 对 `max_channel|Δrgb| ≥ 1/255` 计数；C2 = `cam-hinge` depth 位型变化 + seg pair 变化计数；**两对 world 均 ≥576 为采集前置门槛**（[384,576) 余量不足停手、<384 无判别力拒通过）；判据 A 验收门槛仍 384。
- **G-L1 实测（判据 C 的对账基准）**：C1 magnitude **38,285 / 40,644**、bitwise **43,604 / 41,815**；C2 depth **35,828 / 33,883**、seg **16,041 / 16,754**；诊断 cam-persp depth/seg **0/0**、cam-hinge rgb magnitude **50,363 / 52,329**、bitwise **54,380 / 54,250**。
- **§2.4 计数对账**：`capture.json` 五组字段与实体自算 C1/C2 **两对 world 逐项相等**；实体 `check_capture_c1` 须输出 **Compared**（Skipped ⇒ 证据不完整停手）。
- **world 隔离**：只有该 world 变化时其它 world 输出不变（两台相机都断言）；不替代 §4–§6。
- **诊断项只记录不判**；不得冒充 C1/C2。
- **criterion v2.2 关系**：既有 10 个回归实体继续按 v2.2 两臂清单判（recipe 183 px / `cfd1038a…`；aa2 172 px / `14893fc6…`），本实体**不继承**、不得引用其清单或 `|ULP|≤128` 洗通过。
- **refit 前提**：C2 依赖每 world refit（P0-1）；C2 退化为 0 即 §8 失败，停手。

## 5. 证据清单（`window/*`，详见 `materials/evidence-checklist.txt`）

baseline（三点原文 + `baseline_free`/`onspot_floor`/`signed_floor`/`floor_effective` 算术）、monitor（csv/summary/exit code）、guards 冻结副本（新 guards JSON + `cargo-artifacts.jsonl`）、冻结 exe stash + live/guard/stash 三重一致表、12 个实体日志与退出码、`readout.json`（C1/C2 + 元数据自核）、`capture.json` 副本、参考 5 文件 sha/bytes、两跑 sha 对照、legacy 断言、零编译指纹、`git-before/after`、`post-gpu`、incidents、本窗 manifest。

- 证据目录名（已定稿）：**`docs/development-evidence/render-bvh/perworld-l-v2-validate-window-g-l2/`**。
- 冻结器自检与本轮基线建议随材料：`artifacts/freezer-selfcheck.txt`、`artifacts/freeze-plan-dryrun.json`、`materials/baseline-suggestion.txt`。

## 6. QA 预审重点（完整预审在 fill 后，10 项）

1. 与已签 spec §10.1/§10.2/§12 G-L2 逐条对齐；2. guards 算术 **107**（12+93+2）、10 条必需输入逐条 sha/bytes 与实测一致、可选 0、旧 suite/stash 只读且漂移已如实记录；3. 冻结器行为（dry-run 不构建、`-Execute` 唯一构建点、aggregate 先验算法复算、`live==guard==stash`、旧 suite 冻结后复查、漂移默认拒绝）；4. 纪律：冻结在 monitor 后/body 零编译、三点基线 5 分钟内、`floor_effective=max` 且区分采样 min、monitor 60/60 util<40、失败即停、authoring 例外不适用；5. 实体预期覆盖 A/B/C + 384/576 + G-L1 实测计数对账 + world 隔离 + v2.2 非继承 + refit；6. §2.4 对账要求 `Compared`；7. PENDING 剩余仅 `PENDING-FREEZE`/`PENDING-QA`/`PENDING-REFRESH` 且落点明确；8. 双签摘要 S-3a/S-3b 可分、S-3b 标 suggested、不得复用 S-2 数值；9. 边界（CPU/零 GPU/不改代码，除新增冻结器脚本；四门禁；manifest 卫生）；10. 非声明（不宣称验收通过/采集有效/上游等价/渲染结论）。

## 7. S-3 双签摘要（**草案，QA 完整预审 PASS 后再定稿**）

| 签名项 | 内容 |
| --- | --- |
| **S-3a（验证窗计划）** | G-L2 单次验证窗、执行者 Backend（唯一写者）；guards 冻结 = **107 条**（旧 96 + 1 新 exe `render_per_world_light` + 10 条必需输入，可选 0），旧 suite/stash 只读；参考 5 文件 **11,078,239 B**（sha 见表 §2.3）；冻结在 monitor 后、body 零编译、`live==guard==stash`、单次失败即停 |
| **S-3b（基线 + signed floor）** | `suggested`：`baseline_free = 13,028 MiB`、`signed floor = 9,956 MiB`（2026-11-09 零 GPU 三点实测）；开窗前 5 分钟内现场重测并按 **`floor_effective = max(signed, 现场)`（更高更严）**；**不复用 S-2 的 13,302/10,230** |
| **失败条款** | 任一判据/计数/元数据/legacy/冻结件/编译纪律/monitor 失败 → 停手报用户；不重试、不改阈值/参考/相机、不换冻结件 |
| **证据与 QA** | 证据目录 `…/perworld-l-v2-validate-window-g-l2/`；QA 材料完整预审 + 窗口后 QA 复核；窗口记录同 pilot/S-2 格式 |
| **签名不构成开窗** | 仍需 QA PASS + Chief G-L2 token + 现场 floor 取更高 + monitor 60/60 + 零编译 |

## 8. 剩余 PENDING（本 fill 后）

| # | 待填项 | 落点 |
| --- | --- | --- |
| 1 | **`PENDING-FREEZE`**：12 个 live exe 的 path/bytes/sha、新 guards JSON 的 `aggregate_sha256` 与 `guards_sha256` | §2.2/§2.5、receipt `guards` |
| 2 | **`PENDING-QA`**：G-L1 采后复核卡锚点（QA PASS 后由 fix 填入） | §1 P1、receipt `preconditions[P1]` |
| 3 | **`PENDING-REFRESH`**：实体最终 verified 提交 + QA PASS 锚点 | §1 P2、receipt `preconditions[P2]` |

> **本材料在 G-L2 窗口前必须保持声明**：S-3 未签、guards 未冻结、窗口卡未登记（G-L2 执行窗另卡）；不得宣称采集有效 / 灯验收 / 上游等价 / 任何渲染数值结论。

## 9. 边界与非声明

- 本 fill 轮**零 GPU、零采集、未开窗**；`unchanged src/`, `unchanged fixtures/`, `unchanged tests/`；`scripts/` **仅新增** `freeze-guards-perworld-l2-light.ps1`（不改既有脚本）；四门禁通过；材料 sha256/bytes 记 receipt+manifest。
- 不构成 G-L2 启动：需实体 QA PASS + **用户 S-3 双签** + **Chief G-L2 token** + 窗口条件。
- **不宣称**采集有效、per-world 灯验收、与上游等价、任何渲染数值结论。
