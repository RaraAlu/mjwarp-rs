# render split 步 2（53 源归位）· QA 独立复核

任务卡：`2026-10-30-render-module-split-step2-check`（priority 56，Owner QA）
被核对象：`2026-10-30-render-module-split-step2`（Arch，**verified**）
被核提交：基线 `996188fb`（主卡 revision.before）→ 落地 `e98595ec` → 收口 `141abad9`（复核时 HEAD）
**只读**：未改 `.normify`/证据、未跑 GPU；门禁构建自跑（`logs/`）。

---

## 1. 结论

**verdict：pass / 阻断 0 / 需修 0 / 观察 6。**

| 复核点 | 我的独立结果 | |
| --- | --- | --- |
| 53 源归位 | kernel **22** / bvh **11** / context **19** / 容器 **1**（`src/render/mod.rs`）；四者并集 **= 基线 53 条**，missing 0 / extra 0 / distinct 53 | ✓ |
| 归属规则 | 我按路径规则独立重建期望集合：kernel = `src/render/kernel/**` 19 + `camera*` 3（`camera.rs`、`rays.rs`、`camera/cuda/build_rays.cu`）；bvh = `src/render/bvh/**` 11；context = `src/render/context*` 16 + `texture*` 3；容器 = `mod.rs` ⇒ **四者与证据清单逐条相同** | ✓ |
| 纯搬迁 | `git diff 996188fb..141abad9 -- src tests fixtures Cargo.toml Cargo.lock` **空**；`render.toml` 不在 diff；22 个变更文件全部落在 `.normify/{modules,changes,runs}` + `step2/` 证据 | ✓ |
| API/deps | 容器 0/0、kernel 1 API+4 deps、bvh 2 APIs、context 8 APIs+2 deps；全仓 **398 键 / 重复 0**；render 的 **6 条 `from_api` 依赖**；**API/deps 逐字前后完全相同** | ✓ |
| 计数/coarse | 模块 71 / 叶子 63 不变；`leaf-too-coarse` **36→39**，新增恰为三叶，无掉出 | ✓ |
| 指纹 | 三叶由 `pending` 刷成 **3 个唯一 64-hex 真值**；`validate --fingerprints` **0 错误 / 5 警告**，无漂移 | ✓ |
| 行数口径 | LF：kernel 6,945 / bvh 3,662 / context 5,086 / 容器 385 = **16,078**；Σ(LF+1)=**16,131**（差 53=文件数）——我逐文件重算完全一致 | ✓ |
| 步 1 证据 | 合法修正提交 `a95062b3` 只改 3 文件（卡/README/receipt）且数字与实测一致（749 B / 421 字符 / sha `f680bfbe…` / 两 body 逐字节相同）；`a95062b3..141abad9` 对步 1 README/receipt/logs **零改动** | ✓ |
| impact | 6/6 落位正确：kernel/bvh/context 三叶 + `camera.rs`→kernel + `texture.rs`→context + `mod.rs`→容器 | ✓ |
| 门禁 | 我自跑 fmt/clippy/clippy-cuda **exit 0**；`cargo test --locked` **1106 passed / 0 failed / 392 ignored**（136 result 行），与主卡 `.normify/runs` 的 unit 结果**逐项相同** | ✓ |

---

## 2. 53 源归位（复核点 1）

我直接解析 `996188fb` 与 `141abad9` 两个提交点的 module md（`[[source]]`）：

- 基线容器 `mjwarp.render` 53 源；步 1 后三叶各 0 源（源码仍在容器）。
- 收口：`kernel=22`、`bvh=11`、`context=19`、`container=1`；**assigned=53、distinct=53**；`missing=[]`、`extra=[]` ⇒ **并集 == 基线 53 条**（不重不漏）。
- 归属正确性我按「路径规则」独立重建（不是照抄清单）：
  - kernel = `src/render/kernel/**`（19）+ `src/render/camera.rs` + `src/render/rays.rs` + `src/render/camera/cuda/build_rays.cu`；
  - bvh = `src/render/bvh/**`（11）；
  - context = `src/render/context*`（16）+ `texture.rs` + `texture/cuda.rs` + `texture/sample.cu`；
  - container = `src/render/mod.rs`（仅 1 条）。
  四组**逐条相等**；`rays.rs`/camera 三件与 texture 三件的归位与主卡理由（`include_str!` 依赖方向）自洽。
- 其它模块源集合**零变化**（我逐模块对比 `996188fb` vs `141abad9`，render 四模块之外无任何变化）。
- 行数：我逐文件按 LF 重算 = 6,945 / 3,662 / 5,086 / 385，合计 16,078；Σ(LF+1)=16,131（差 = 53）。材料：`materials/verify-pathset-recompute.txt`。

## 3. 纯搬迁（复核点 2）

- `git diff 996188fb..141abad9` = **22 个文件**：modules 4、changes 2、runs 4、`step2/` 证据 12；`src/tests/fixtures/Cargo.*` **零改动**；`.normify/layouts/mjwarp/render.toml` **不在 diff**。
- 主卡 `unchanged(paths=[src/,tests/,fixtures/])` 检查我现在重跑 `normify check` 主卡仍**通过**。
- 步 1 证据：`a95062b3..141abad9` 对 `render-module-split/{README.md,receipt.json,logs}` **空 diff**；`a95062b3` 自身只改 3 个文件，且修正后的数字与我从 git blob 实测一致（749 B / 421 字符 / `f680bfbe…` / 两 body **逐字节相同**）。

## 4. API / deps 零变动（复核点 3）

| 模块 | 步 1 | 步 2（我复算） |
| --- | --- | --- |
| `mjwarp.render`（容器） | 0 API / 0 deps | **0 / 0** |
| `mjwarp.render.kernel` | 1 / 4 | **1 / 4** |
| `mjwarp.render.bvh` | 2 / 0 | **2 / 0** |
| `mjwarp.render.context` | 8 / 2 | **8 / 2** |

- 四个模块的 API 键+protocol+zh+en 与 deps 六元组在 `996188fb`→`141abad9` **逐字完全相同**。
- 全仓 398 API 键、重复 0；render 的 6 条 `from_api` 依赖（kernel 4、context 2）不变。
- 口径提示（O2）：主卡「deps 6 不变」指 **render 的 6 条 `from_api` 依赖**；全仓 deps 226 / `from_api` 141，前后亦不变。

## 5. validate 与 coarse（复核点 4）

我重跑 `normify validate --fingerprints`：**exit 0 / 错误 0 / 警告 5**，与 `step2/logs/validate-final.txt` **逐行相同**：
`api/leaf-empty ×2`（io.render、model.render-model）、`structure/leaf-too-coarse **39**`、`layout/order-incomplete ×2`（io/model）。
- coarse 我按「叶子源数 ≥2」口径独立重建：`996188fb` = **36**、`141abad9` = **39**；新增恰为 `render.kernel/bvh/context`，无掉出、无其它变化。
- 模块 71 / 叶子 63 不变。
- 中间态：`step2/logs/validate-intermediate.txt` 比终态多 **1 条** `evidence/fingerprint-drift（4 个模块）`，与「先写占位指纹再 `module source`」的解释一致，`done` 后消失（我现跑已无该告警）。

## 6. 指纹（复核点 5）

三叶指纹在 `996188fb` 均为 `pending`；`141abad9` 为 **64 位 hex 真值**且三者互不相同：
`kernel b8aeeafe…`、`bvh 7c4c0fea…`、`context abca50c5…`；`validate --fingerprints` 0 错误、无 drift。

## 7. impact 6 处（复核点 5）

| 路径 | 我实测 |
| --- | --- |
| `src/render/kernel/megakernel.rs` | `mjwarp.render.kernel`（22 源/1 API） |
| `src/render/bvh/refit.rs` | `mjwarp.render.bvh`（11 源/2 API） |
| `src/render/context/lifecycle.rs` | `mjwarp.render.context`（19 源/8 API） |
| `src/render/camera.rs` | `mjwarp.render.kernel` |
| `src/render/texture.rs` | `mjwarp.render.context` |
| `src/render/mod.rs` | `mjwarp.render`（容器，1 源/0 API） |

与 `step2/logs/impact-*.txt` 六处一致。

## 8. 门禁（我自跑）

fmt / clippy / clippy-cuda 全 **exit 0**；`cargo test --locked` **exit 0**：**136 result 行 / 1106 passed / 0 failed / 392 ignored**；主卡 `.normify/runs/2026-10-30-render-module-split-step2/unit.json` 的 `tests` 与我逐项相同（1106/0/392），四个 run 记录均 `passed=true / exit=0`。
独立重算脚本：`recompute.py` → `recompute.json`（**17 项 check，problems=0**）。

## 9. 证据（复核点 6）

- `step2/` 共 **12 文件**（README + receipt + 6 impact logs + no-run + validate-final + validate-intermediate + `materials/verify-pathset.txt`），committed blob **全部 LF 无 BOM**；`receipt.task_id` 正确。
- 与步 1 证据**零文件重叠**（步 1 文件只做数字修正，未新增/覆盖）；「容器保留 1 个门面源 `src/render/mod.rs`」的口径更新句存在于 step2 README §2。
- 行数口径（LF 16,078 / Σ(LF+1) 16,131，差 53）在 README §1 写明，我复算一致。
- 禁词：step2 README §6 与 `receipt.not_claimed` 在否定式非声明里字面出现术语 5 处；normify 自身 check 通过（**记录，非需修**）。

## 10. 附：B 的 TASK_ID 修复只读核对（`996188fb`，Chief 指定）

- 脚本 `scripts/capture_render_perworld.py`：**23 531 B / sha256 `971cb8b13db82a9076c81b15936b2c97ffe2df8d3c0e47d1d9d0b10c78552775` / 497 行 LF / 非 ASCII 0**；
- `TASK_ID` 已 = `2026-10-30-render-perworld-capture-authoring`（与主卡 id 一致）；
- receipt `script`（sha/bytes/lines）与 README 已同步；`materials/dryrun-plan.json` 的 `task_id` 与新脚本 sha/bytes 已刷新；
- 我用 pinned python **离线重跑 `--dry-run`**：exit 0，输出 JSON 与刷新后的 `dryrun-plan.json` **逐字段相等**；`--out-dir` 仍不存在。
材料：`materials/b-task-id-fix-check.txt`。

## 11. limits

- 只读复核：未改模块/布局/证据、未跑 GPU；步 2 是**注册搬迁**，磁盘文件未移动，因此不需要设备侧验证。
- 未复核后续口径（`mod.rs` 最终归哪叶、三个 API 的「归属≠源码位置」是否消除）——主卡声明留在后续决定。
- 「coarse=39」的规则我按「叶子源数 ≥2」复现（与 36/39 及新增项吻合）；超 220 行分支仍无仓库文档（既有 O5 类）。

## 12. 观察（6）

| # | 级别 | 观察 | 建议 |
| --- | --- | --- | --- |
| O1 | 建议 | `step2/README.md §5` 引用的 **`logs/gate-*.txt` 并不存在**；门禁结果实际在 `.normify/runs/2026-10-30-render-module-split-step2/{fmt,clippy,clippy-cuda,unit}.json`（我已核 passed/exit/unit 数） | 把 README 的指向改成 `.normify/runs/`（或把 gate 日志落 `logs/`）；不影响结论 |
| O2 | 记录 | 「deps 6 不变」的口径是 **render 的 6 条 `from_api` 依赖**；全仓 deps 226 / `from_api` 141（前后亦不变） | 证据里注明「render 6 条 from_api」，避免被误读成全仓只有 6 条 |
| O3 | 记录 | 三叶占位指纹在中间态触发 1 条 `evidence/fingerprint-drift（4 模块）`；`done` 后消失 | 接受（主卡已在中间态留档） |
| O4 | 记录 | 容器保留 `mod.rs` 门面，`OutputChannels`/`ImageOutput`/`ImageBatchLayout` 仍「声明在 context 叶、实现在容器源」 | 接受（本片点名保留的已知偏差） |
| O5 | 记录 | 先例核对：根容器 `mjwarp` 确有 **8 个 `src/` 源**（含 `src/lib.rs`）、`mjwarp.runtime` 容器含 `src/runtime/mod.rs` | 与证据一致，记录 |
| O6 | 记录 | 步 2 的 22 文件 diff 精确落在 modules/changes/runs/step2 四类，无第三类；步 1 证据自修正后未被再改 | 记录（口径干净） |

---

## 13. 本卡文件

| 文件 | 内容 |
| --- | --- |
| `recompute.py` / `recompute.json` | **17 项**独立重算（路径并集/行数/纯搬迁/API-deps/coarse/指纹/validate/impact/步 1 修正/证据/门禁/主卡 check/B 修复/卡状态），problems=0 |
| `logs/recompute.log` | 上述脚本 stdout |
| `logs/fmt.log`、`logs/clippy.log`、`logs/clippy-cuda.log`、`logs/unit.log` | QA 自跑四门禁原始输出 |
| `logs/gates-raw.txt` | 四门禁 exit 码 |
| `materials/verify-pathset-recompute.txt` | 我的路径并集/归属/行数重算明细 |
| `materials/b-task-id-fix-check.txt` | B 的 TASK_ID 修复只读核对（脚本 sha/bytes/lines、receipt/README/材料自洽、dry-run 复跑） |
| `manifest.txt` | 本卡文件 sha256/字节/路径（不含自身） |
