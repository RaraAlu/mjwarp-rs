# 架构树剩余 154 未登记源收口（方案 B） · QA 独立复核

任务卡：`2026-10-30-arch-unregistered-sources-remaining-check`（priority 55，Owner QA）
被核对象：`2026-10-29-arch-unregistered-sources-remaining`（**verified**，Arch）
被核提交：基线 `645befa7`（主卡 revision.before）→ 候选 `18f5177f` → 实施 `1d321d94` / `34b25c9b` → 收口 `84ef186f`
复核时 HEAD：`84ef186f`。**只读**：未改代码/证据、未重跑窗口、未做设备执行；允许的门禁构建与 NVRTC 纯编译门已自跑（`logs/`）。

---

## 1. 结论

**verdict：pass（架构结论与全部计数/归属/API/impact/零代码改动均独立成立）/ 阻断 0 / 需修 1（轻微，证据卫生）/ 观察 9。**

关键独立复算（不采信证据结论，全部由 git 对象与现盘现算）：

| 指标 | 我的复算 | 证据声明 | |
| --- | --- | --- | --- |
| 模块数 | 66 → **68** | 66 → 68 | ✓ |
| 叶子数 | 59 → **61** | 59 → 61 | ✓ |
| 登记条目（src+include+native 内） | 680 → **798** | 680 → 798 | ✓ |
| 登记不同路径（同上） | 668 → **786** | 668 → 786 | ✓ |
| 未登记 | **154 → 36**（36 全部 `include/warp-headers/`） | 同 | ✓ |
| 跟踪文件（src+include+native） | **822**（`645befa7`/`18f5177f`/HEAD 三点一致） | 822（P1 记 823） | ✓ |
| API 键总数 / 重复 | **398 / 0** | 398 / 0 | ✓ |

118 个新登记文件逐条核对：全部存在且被跟踪、归属前缀与模块领域一致、**新增重复 0**、无臆造归属。5 条 collision API 与描述**逐字原文搬迁**（键、zh、en 三重比对全等），`physics.core` 88 → 83；新叶 `runtime.collision-gpu` 的 `run_product_collision` 与源码真实入口一致。
`validate --fingerprints` 我重跑 **错误 0 / 警告 5**，与 `logs/validate-final.txt` 行行相同；`leaf-too-coarse` **34 → 37** 我按模块树独立重建并逐条确认翻转项。
6 处 impact 抽样 6/6 与证据一致（含 vendored 头仍「没有模块对应」）；零代码改动成立；NVRTC 纯编译门复跑显示内核身份三值未变。

**唯一需修**：`logs/module-counts-after.txt` 的 committed blob 带 **UTF-8 BOM**（`EF BB BF`，2 049 B，收口提交 `84ef186f` 引入）。除该文件外其余 14 个证据 blob 均 LF 无 BOM。机械修复：去 BOM 重提交，**不改任何数字、不需重跑**。

---

## 2. 计数与清单（复核点 1/3）

我直接解析两个提交点的 `.normify/modules/**/*.md`（`[[source]]`/`[[apis]]`），并自己枚举叶子（无子模块 id 前缀者）：

| 指标 | `645befa7` | `84ef186f` |
| --- | --- | --- |
| 模块数 | 66 | 68 |
| 叶子数 | 59 | 61 |
| 全部模块条目 / 不同路径 | 1132 / 1115 | 1250 / 1233 |
| 三棵树内条目 / 不同路径 | 680 / 668 | 798 / 786 |
| 跟踪文件（`git ls-tree` src+include+native） | 822 | 822 |
| 未登记 | **154** | **36** |

154 的构成我按路径分类复现，与主卡完全一致：`physics collision 72 / physics 其余 19 / runtime collision_gpu 20 / runtime 其余 7 / include 36 / native 0`；after 的 36 条**逐条**确认都在 `include/warp-headers/`。
**822 vs P1 记 823**：我在 `f86b8575`/`645befa7`/`84ef186f` 三点现算均是 822，且 `git diff --name-status f86b8575..84ef186f -- src include native` 只有 2 个 `M`（`megakernel.rs`、`megakernel_arrays.rs`）、无增删 ⇒ 差异来自 P1 的计数口径（多计 1 条），主卡 README §5 的解释成立。

---

## 3. 118 个自家文件的归属（复核点 1）

逐模块新增数（我按基线/收口两个模块树做集合差）：

| 模块 | 新增 | 我的归属前缀校验 |
| --- | --- | --- |
| `mjwarp.physics.collision`（新叶） | **72** | 全部 `src/physics/collision*` ✓ |
| `mjwarp.runtime.collision-gpu`（新叶） | **20** | 19 × `src/runtime/collision_gpu*` + `src/runtime/collision_upload.rs` ✓ |
| `mjwarp.physics.core` | **17** | callback / derivative / history / integrator / inverse / rne_bias / sensor ✓ |
| `mjwarp.physics.mass-matrix` | **1** | `src/physics/mass_matrix/multiply/borrowed/device_energy/schedule.rs` ✓ |
| `mjwarp.physics.tendon-velocity` | **1** | `src/physics/tendon_velocity/tests/launcher.rs` ✓ |
| `mjwarp.runtime`（容器） | **5** | backend(+tests) / launcher/frozen / launcher/warp / presence ✓ |
| `mjwarp.runtime.external` | **2** | `src/runtime/external/view.rs` + `view/tests.rs` ✓ |
| 合计 | **118** | 无一条落在模块领域外 |

- 118 个路径**全部存在且被跟踪**；新增集合内**无重复**；跨模块重复对与基线**完全一致**（基线已有 12 对历史重复：root `mjwarp` 与子叶、`io/mod.rs`、`transfer.rs`、gravity 族等），**新增重复 0 对**。
- 叶子规模：collision 72 源、collision-gpu 20 源（行数口径见 O2）；`physics.core` 104→121、`mass-matrix` 45→46、`tendon-velocity` 16→17、`runtime` 53→58、`external` 1→3，与 README §2 一致。
- 交叉验证：我把证据 `logs/module-counts-after.txt` 的 67 行按公式反推（col1=三棵树内源数、col2=非 file 协议 API 数、col3=全部源条目+全部 API 条目），**67/67 全部吻合**，col1 合计 798、col2 合计 392；唯一无行的是 `mjwarp.physics` 容器（0 源 / 0 API，见 O4）。

---

## 4. API 迁移（复核点 2）

| 项 | 我的复核 |
| --- | --- |
| `physics.core` API 数 | 88 → **83**，删除的正是 5 条 collision 键，无其他增删 |
| `physics.collision` API | **5 条**，键集合 = 5 条 collision 键 |
| 原文搬迁 | 5/5 条 **protocol + 路径 + zh 描述 + en 描述逐字相同**（与基线 `physics.core` 原文对比） |
| `runtime.collision-gpu` API | 1 条 `rpc:mjwarp_rs::runtime::collision_gpu::run_product_collision` |
| 真实入口 | `product_entry.rs` 有 `pub fn run_product_collision`；`collision_gpu.rs` 有 `mod product_entry;` + `pub use product_entry::{…, run_product_collision, …}` ✓ |
| 全项目 API 键 | **398** 条（392 rpc + 6 file），重复 **0** |
| 依赖失效 | `validate` 终态无 `dep/to-api-invalid`（终态 5 条告警里没有该类） |

---

## 5. validate 与 `leaf-too-coarse` 归因（复核点 4）

我重跑 `normify validate --fingerprints`：**exit 0 / 错误 0 / 警告 5**，警告行与 `logs/validate-final.txt` **逐字相同**：
`api/leaf-empty ×2`（io.render、model.render-model，既有）、`structure/leaf-too-coarse 37`、`layout/order-incomplete ×2`（io/model，既有）。

- **告警净增 0**：基线 5 行 → 终态 5 行（只有 coarse 计数 34→37 变化，其余 4 行逐字不变）。
- **coarse 计数独立重建**：我按「叶子的登记源数 ≥ 2」口径在模块树上数，基线 **34**、收口 **37**；翻转项恰为 **`mjwarp.physics.collision`（新叶 72 源）+ `mjwarp.runtime.collision-gpu`（新叶 20 源）+ `mjwarp.runtime.external`（1→3 源）**，无叶子掉出 ⇒ 主卡「+2 新叶、external 翻 coarse」的归因**逐条成立**。
- 中间态：`logs/validate-intermediate.txt` 记录 10 条告警（5 基础 + 5 `change/api-already-removed`），与「迁移已落、任务未 verified」的解释一致；该文件的捕获不完整（见 O5）。
- 口径提醒（同 P1 O5）：警告文字写「多文件或超220行」，但实测计数的 34/37 只能由「源数 ≥2」复现——单源叶最大 500 行、且 17 个单源叶超 220 行却都不计；即「超220行」分支实际不触发或另有口径，仓库内无该规则文档（O9）。

---

## 6. impact 抽样 6 处（复核点 5）

我自己跑 `normify impact`：

| 路径 | 结果 |
| --- | --- |
| `src/physics/collision/cuda/contact.cu` | exit 0 → `mjwarp.physics.collision` ✓ |
| `src/physics/collision/gjk/epa.rs` | exit 0 → `mjwarp.physics.collision` ✓ |
| `src/runtime/collision_gpu/product_entry.rs` | exit 0 → `mjwarp.runtime.collision-gpu` ✓ |
| `src/physics/sensor.rs` | exit 0 → `mjwarp.physics.core` ✓ |
| `src/runtime/presence.rs` | exit 0 → `mjwarp.runtime` ✓ |
| `include/warp-headers/mat.h` | **exit 1 + 「没有模块对应」** = vendored 政策断言 ✓（预期非缺陷） |

---

## 7. 零代码改动（复核点 6）

- `git diff 645befa7..84ef186f --name-only` = **32 个文件**：`.normify/` **17** + `docs/development-evidence/` **15**，**无第三条路径**。
- `git diff 645befa7..84ef186f -- src tests fixtures include native Cargo.toml Cargo.lock build.rs` = **空**；候选段 `18f5177f..84ef186f` 同样为空。
- 复核期间工作区只有本复核卡（`.normify/changes/…-check.json`）与我自己的证据目录；**未见 `src/`、`.normify/modules|layouts` 的预期外改动**（如有会先报 Chief）。

---

## 8. vendored 头政策（复核点 5/8）

- `mjwarp.runtime.compiler` 描述（zh/en）已写入政策：目录 `include/warp-headers/`、36 文件、上游 `NVIDIA/warp v1.15.0 @ ffa038b1`、W15 manifest 路径、由 `tests/launcher-warp-headers.rs` 断言。
- 我核：`docs/development-evidence/w15-verbatim-headers/headers.json` 存在且 `header_count=36`；`tests/launcher-warp-headers.rs` 含 `copied_headers_match_the_frozen_manifest`；README §4 给出 4 条理由（第二事实源 / 规模噪声 43 321 行 / 图语义无效 / 反例代价）。
- 36 条未登记文件**逐条**落在 `include/warp-headers/`（含 `nanovdb/PNanoVDB.h`），与政策一致。

---

## 9. 门禁（我自跑）与 NVRTC 纯编译门

| 项 | 结果 |
| --- | --- |
| `cargo fmt --check` | exit 0（`logs/fmt.log` 空） |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0 |
| `cargo clippy --all-targets --locked --features cuda -- -D warnings` | exit 0 |
| `cargo test --locked` | exit 0；**136 result 行 / 1106 passed / 0 failed / 392 ignored**（与主卡 `.normify/runs/…/unit.json` 的 1106/0/392 一致；派工期望 1106/0 ✓） |
| `--no-run` 证据日志 | `logs/no-run.txt` 含 `exit=0` 与 `Finished` ✓（中段混入 PowerShell 噪声，见 O8） |
| NVRTC 纯编译门（`--features cuda-probe`，`--ignored`，**无设备执行**） | exit 0：`source_sha256=e68e3412…`、`ptx_sha256=0b947492…`、`ptx_bytes=268275`、`aa_resolve` `0bf8af5a…`/`b28e155c…`/27 536 —— **内核身份三值未变** ✓ |

独立重算脚本：`recompute.py` → `recompute.json`（**24 项 check**，唯一未过项即 §10 的 BOM；`logs/recompute.log`）。

---

## 10. 证据（复核点 7）

- 15 个文件（README + receipt + 13 logs），`receipt.task_id` = `2026-10-29-arch-unregistered-sources-remaining` ✓；`receipt.evidence_files` 与目录**互为全集** ✓。
- **committed blob 逐文件核对**：14 个 LF 无 BOM；**1 个带 BOM** → `logs/module-counts-after.txt`（`EF BB BF`，2 049 B）。这是本卡唯一偏离「证据 LF 无 BOM」之处（**需修**）。
- 工作区另有 10 个文件呈 CRLF：这是 `core.autocrlf=true` 的**检出产物**（committed blob 全 LF、`git diff --quiet` 一致），不是作者输出问题（O1）。
- 主卡证据无 `manifest.txt`（check 卡 acceptance 提到 manifest）：我改用 committed blob 对账（15/15 可核，无自引用问题）→ O7。
- 禁词：主卡 README §9 与 `receipt.not_claimed` 在**否定式非声明**句里字面出现术语 5 处，normify 自身 check（含主卡 `forbid_claim`）通过 → O6（记录，非本轮需修）。

---

## 11. limits

- 本轮为**只读**复核：未改主卡模块/布局/证据，未重跑任何窗口或设备用例；NVRTC 门仅编译、无设备执行。
- 叶子「超220行」规则不在仓库文档内，我对 coarse 计数的独立复现只到「源数 ≥2」口径（数值与翻转项完全吻合）。
- `validate` 的**基线**侧我用模块树重建 34（而不是 checkout 回 `645befa7` 跑一次 validate），避免动工作区；终态侧为实跑。

---

## 12. 观察（需修 1 + 建议/记录 9）

| # | 级别 | 观察 | 建议 |
| --- | --- | --- | --- |
| P1 | **需修（轻微）** | `logs/module-counts-after.txt` committed blob 带 UTF-8 BOM（`84ef186f` 引入）；其余 14 文件 LF 无 BOM | 去 BOM 重提交该文件（内容不变、不需重跑）；若团队接受历史 BOM（`docs/development-evidence` 下历史共 100 个 BOM 文件），可降为观察 |
| O1 | 记录 | 工作区 10 个证据文件 CRLF 系 `core.autocrlf=true` 检出产物，blob 全 LF | 接受；今后核对以 committed blob 为准（README 已写） |
| O2 | 建议 | 行数口径：README 的 17 535 / 4 004 = Σ(每文件 newline+1)；LF 行数为 17 463 / 3 984 | README 加一句口径说明，避免与 `wc -l` 对不上 |
| O3 | 建议 | `module-counts-after.txt` 的 apis 列只计**非 file 协议**（合计 392），漏 tooling 的 6 条 file API，与 receipt `api_keys_total=398` 不同口径且列头未说明 | 列头加「rpc-only」字样或在 README 注明 |
| O4 | 记录 | 该 log 67 行 vs 68 模块：`mjwarp.physics` 容器（0 源 0 API）无行 | 接受；可在 log 头注明「零源容器不列」 |
| O5 | 记录 | `validate-intermediate.txt` 捕获不完整：10 条告警只落 3 条 `change/api-already-removed`（README 称 5 条），且 `exit=` 与值被换行拆开 | 接受（中间态证据，不影响终态）；今后可用重定向而非手抄 |
| O6 | 记录 | 主卡证据在否定式非声明里字面出现术语 5 处（README §9 ×2、`receipt.not_claimed` ×3） | 接受（non-claim，normify check 通过）；若团队要求证据字面 0，改写这几句即可（对照 P1 证据是按 0 写的） |
| O7 | 记录 | 主卡证据无 manifest（check 卡 acceptance 提到 manifest） | 接受（我改用 blob 对账）；如要统一，后续卡补 manifest |
| O8 | 建议 | `no-run.txt` 中段混入 PowerShell 的 stderr 错误记录与 mojibake（cargo 把 `Finished` 写 stderr 被 PS 当 error record） | 用 `cmd /c` 或 `--message-format` 过滤，日志更干净；本卡 `exit=0` + `Finished` 仍可判读 |
| O9 | 建议 | `leaf-too-coarse` 规则仓库内无文档；警告文字「或超220行」与实测计数不符（单源叶最大 500 行、17 个单源叶 >220 却不计） | 补规则文档或修正告警文字（同 P1 O5 遗留） |

---

## 13. 本卡文件

| 文件 | 内容 |
| --- | --- |
| `recompute.py` / `recompute.json` | 24 项独立重算（计数/归属/API/coarse/日志列/validate/impact/零改动/证据/门禁/内核身份） |
| `logs/recompute.log` | 上述脚本 stdout |
| `logs/fmt.log` / `logs/clippy.log` / `logs/clippy-cuda.log` / `logs/unit.log` | QA 自跑四门禁原始输出 |
| `logs/kernel-nvrtc-compile.log` | NVRTC 纯编译门输出（无设备执行） |
| `logs/gates-raw.txt` | 四门禁命令与 exit 码 |
| `manifest.txt` | 本卡文件 sha256/字节/路径（不含自身） |
