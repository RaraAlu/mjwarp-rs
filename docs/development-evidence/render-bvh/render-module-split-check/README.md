# mjwarp.render 结构反转 · 步 1 · QA 独立复核

任务卡：`2026-10-30-render-module-split-check`（priority 55，Owner QA）
被核对象：`2026-10-30-render-module-split`（**步 1**，Arch，verified）
被核提交：基线 `9e1567a2`（主卡 revision.before）→ 落地 `7de14d30`；复核时 HEAD `4cdeffec`（仅 backlog 卡）
**只读**：未改 `.normify`/证据、未跑 GPU；门禁构建已自跑（`logs/`）。

---

## 1. 结论

**verdict：pass（结构/迁移/布局/validate/零代码改动全部独立成立）/ 阻断 0 / 需修 1（证据数字，非结构）/ 观察 6。**

| 项 | 我的复算 | 证据声明 | |
| --- | --- | --- | --- |
| 模块数 | 68 → **71** | 68 → 71 | ✓ |
| 叶子数 | 61 → **63** | 61 → 63 | ✓ |
| API 键（全项目）/ 重复 | **398 / 0**（迁移前后同集） | 398 / 0 | ✓ |
| `mjwarp.render` | 叶子(53 源/11 API/6 dep) → **容器(53 源/0 API/0 dep)** | 同 | ✓ |
| 三新叶 | kernel 1 API+4 dep、bvh 2 API、context 8 API+2 dep（均 0 源，步 1） | 同 | ✓ |
| `leaf-too-coarse` | **37 → 36**（掉出者恰为 `mjwarp.render`） | 37 → 36 | ✓ |
| validate | **错误 0 / 警告 5**（与证据终态日志逐行相同） | 同 | ✓ |

**唯一需修**：证据里登记的 body 数字（`README §4`、`receipt.measured_lines`、偏差 3）与 committed 文件不符——见 §5 与 `materials/body-number-audit.txt`。**实质结论更强**：正文**逐字节相同**（登记说「末尾 +1 LF」并不存在）。修正只需改数字，不需重跑、不影响本卡任何结构结论。

---

## 2. 结构与归属（复核点 1）

- 新增模块恰为 `mjwarp.render.kernel` / `.bvh` / `.context`，无模块删除；叶子 61 → 63（`mjwarp.render` 退出叶子集，+3 新叶）。
- **容器化**：`mjwarp.render` 迁移前后 **53 个源路径集合完全相同**（不重不漏）；API 11 → **0**；依赖 6 → **0**；6 条依赖**全部**带 `from_api`（印证「容器不能有 `from_api`」的迁移必要性）。
- **三叶归属**：API 数 kernel=1 / bvh=2 / context=8，与主卡分配表一致；`from_api` 逐条属于**所在叶自己的** API（kernel 4 条均 `rpc:mjwarp_rs::render::render`；context 2 条均 `rpc:mjwarp_rs::render::ImageBatchLayout`）⇒ 满足 `dep/from-api-non-leaf` 的规则要求；validate 终态无该告警。
- uid 全模块唯一（71/71），三新叶 `fingerprint = "pending"`（无源阶段合法，`validate --fingerprints` 0 错误无漂移）。

## 3. 11 API 与 6 依赖逐字（复核点 1）

- **11/11 API 逐字**：与 `9e1567a2:.normify/modules/mjwarp/render.md` 比对 **protocol + 路径 + zh 描述 + en 描述** 四元组全等（不是近似）。
- **6/6 依赖逐字**：`(kind, to, from_api, to_api, [deps.label].zh, [deps.label].en)` 六元组全等；分布 4 → kernel、2 → context；`[deps.label]` 双语原文随迁（例：`只读物理派生缓冲 / Derived-state view`、`基础辅助的真实依赖 / Foundation helper`）。
- **其它模块零影响**：全仓 inbound 依赖 = 0（无 `dep.to = mjwarp.render`、无 `to_api` 指向旧 11 API）；除 render 三叶外所有模块的依赖集合逐条不变。
- API 总键 398 → 398、重复 0（纯搬家，无新增/删除语义）。

## 4. 布局（复核点 1）

新建 `.normify/layouts/mjwarp/render.toml`：`order = [mjwarp.render.bvh, mjwarp.render.context, mjwarp.render.kernel]`（3 个直接子模块、无重复、顺序与主卡声明一致）、`mode = "layers"`、`max_columns = 3`、双语 `[reading]` 均在。validate 无 `layout/missing`，`layout/order-incomplete` 仍只有既有 io/model 两条。

## 5. 容器 body 与行数（复核点 1）

| 项 | 旧 `render.md`（`9e1567a2`） | 新 `render/index.md`（`7de14d30`） |
| --- | --- | --- |
| 文件字节 | 9 119 | 3 923 |
| 总行数 | **393** | **201** |
| 最后一个 `+++` | **第 369 行** | **第 177 行** |
| body 行数 | 24 | 24 |
| body 字节 / 字符 | **749 B / 421 字符** | **749 B / 421 字符** |
| body sha256 | `f680bfbe2a589b81a68ca3368b45c07d268aa3fcf78f4f90985e45a32d29b730` | 同左（**逐字节相同**） |

⇒「正文 24 行、共 393/201 行、fence 369/177」与登记一致；**正文逐字保留成立（且是逐字节相同）**。
**需修（P1）**：登记写「旧 body 421 字节、新 422 字节、新文件末尾补 1 个 LF」、两个 body sha256 为 `4f54ed56…`/`483d91ae…`——实测 421 是**字符数**（真实 749 B），两个 sha256 都无法从 committed body 复现（实际 `f680bfbe…`），且两文件 body 逐字节相同（旧文件末尾本来就有 LF）。详见 `materials/body-number-audit.txt`（含我试过的候选口径）。建议改数字即可（不用重跑）。

## 6. validate 与 coarse 归因（复核点 4）

我重跑 `normify validate --fingerprints`：**exit 0 / 错误 0 / 警告 5**，与 `logs/validate-final.txt` **逐行相同**：
`api/leaf-empty ×2`（io.render、model.render-model，既有）、`structure/leaf-too-coarse 36`、`layout/order-incomplete ×2`（io/model，既有）。
- **coarse 37 → 36 独立重建**：按「叶子源数 ≥ 2」口径数叶子，基线 37、收口 36；**掉出者恰为 `mjwarp.render`**（53 源，升容器后不计），3 个新叶 0 源不计入，无新增叶子 ⇒ 归因成立。
- 终态 0 条 `change/api-already-removed`；无 `dep/from-api-non-leaf`、无 `layout/missing`。
- 中间态：`logs/validate-intermediate.txt` 记 16 条告警（5 基础 + 11 条迁移态），但日志只捕获了 3 条 `change/api-already-removed`（O1）。

## 7. impact（步 1 中间态，符合预期）

我自跑 `normify impact`：

| 路径 | 结果 |
| --- | --- |
| `src/render/kernel/megakernel.rs` | 容器 `mjwarp.render`（53 源 / 0 API）✓ 步 2 应解析到 `.kernel` |
| `src/render/bvh/refit.rs` | 容器 `mjwarp.render` ✓ 步 2 应解析到 `.bvh` |
| `src/render/context/lifecycle.rs` | 容器 `mjwarp.render` ✓ 步 2 应解析到 `.context` |

与证据 `logs/impact-*.txt` 三处一致；这是步 1 的**声明内中间态**，不是缺陷（O3）。

## 8. 零代码改动（复核点 5）

- `git diff 9e1567a2..7de14d30 --name-only` = **21 文件**：`.normify/` **8** + `docs/development-evidence/` **13**，无第三条路径。
- `src/`、`tests/`、`fixtures/`、`Cargo.toml`、`Cargo.lock` diff = **空**。
- `7de14d30..4cdeffec`（HEAD）只多一个 backlog 卡文件，**不触**模块树/布局 ⇒ 在 HEAD 跑的 validate/impact 与 `7de14d30` 同口径（无模块差异）。

## 9. 门禁（自跑）

| 门禁 | 自跑结果 | 证据 |
| --- | --- | --- |
| `cargo fmt --check` | exit 0（`logs/fmt.log` 空） | 证据 gate-fmt pass |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0 | 证据 gate-clippy pass |
| `cargo clippy … --features cuda -- -D warnings` | exit 0 | 证据 gate-clippy-cuda pass |
| `cargo test --locked` | exit 0；**136 result 行 / 1106 passed / 0 failed / 392 ignored** | 证据 gate-unit **逐项相同** |
| `--no-run` 证据日志 | 含 `exit=0` + `Finished` | ✓ |

独立重算脚本：`recompute.py` → `recompute.json`（**17 项 check**，唯一未过项即 §5 的证据数字）。

## 10. 证据与卡状态（复核点 5）

- 主卡证据 **13 文件**（README + receipt + 11 logs）**全部 committed blob LF 无 BOM**；`receipt.task_id` 正确。
- 主卡 `status=verified`、`revision.before=9e1567a2`、有 `closed_at`；本复核卡 `in_progress` 且 start 时 pin 到 `4cdeffec`。
- 主卡证据在**否定式非声明**里字面出现三个禁词（README §8、`receipt.not_claimed`）——normify 自身 check 通过；本卡证据已用码点构造词表规避（O2）。

## 11. limits

- 只读复核步 1：未搬源、未改模块/布局/证据，未跑 GPU。
- **步 2 未复核**：46 源搬迁、impact→三叶、coarse 36→39、新叶 fingerprint 刷新都不在本卡范围（O3/O4）。
- 证据 body 数字的「应为多少」我只给出实测值，不代改主卡证据（需 Arch 重提交）。

## 12. 观察（需修 1 + 观察 6）

| # | 级别 | 观察 | 建议 |
| --- | --- | --- | --- |
| P1 | **需修（轻微）** | 证据 body 数字不实：421/422「字节」实为字符数（真实 749 B）；两个 body sha256 无法复现（实际 `f680bfbe…`）；「新文件末尾补 1 LF」不成立（两 body 逐字节相同） | 按实测改正 `README §4`/偏差 3 与 `receipt.measured_lines`；无需重跑，正文保留结论更强（byte-identical） |
| O1 | 记录 | `validate-intermediate.txt` 截断：16 条告警只落 3 条 `change/api-already-removed`（README 称 11） | 接受（中间态）；下次用重定向整段落盘 |
| O2 | 记录 | 主卡证据否定式非声明里字面出现三个禁词 5 处（README §8 ×2、`receipt.not_claimed` ×3） | 接受（normify check 通过）；若团队要求字面 0，改写几句 |
| O3 | 记录 | 容器持 53 源、三叶 0 源、impact 落容器：步 1 声明内偏差 | 步 2 复核必须重核「源集合不重不漏 / impact→三叶 / coarse 36→39」 |
| O4 | 记录 | 三新叶 `fingerprint = "pending"`（validate 无漂移） | 步 2 搬源后必须刷新指纹，否则会报漂移 |
| O5 | 记录 | 正文保留旧绝对路径 `C:/Rust/mjwarp-rs/…`（历史正文逐字） | 接受（逐字保留优先于路径现代化） |
| O6 | 建议 | `no-run.txt`/中间态日志沿用 PowerShell 捕获，含 stderr 噪声/截断（同前卡） | 用 `cmd /c` 重定向，避免逐条手抄 |

---

## 13. 本卡文件

| 文件 | 内容 |
| --- | --- |
| `recompute.py` / `recompute.json` | 17 项独立重算（结构/容器/归属/API/依赖/body/布局/validate/coarse/impact/零改动/证据/门禁/uid） |
| `logs/recompute.log` | 上述脚本 stdout |
| `logs/fmt.log`、`logs/clippy.log`、`logs/clippy-cuda.log`、`logs/unit.log` | QA 自跑四门禁原始输出 |
| `logs/gates-raw.txt` | 四门禁命令 exit 码 |
| `materials/body-number-audit.txt` | body 字节/字符/sha 实测与登记值逐项对照（需修依据） |
| `manifest.txt` | 本卡文件 sha256/字节/路径（不含自身） |
