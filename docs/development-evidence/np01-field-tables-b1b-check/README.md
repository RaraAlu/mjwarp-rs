# NP01 逐字段类型表 B1b 批 —— QA 独立复核（只读）

- 被复核实现卡：`2026-11-04-precision-np01-field-tables-b1b`（实现提交 **`abbe89ba`**，12 文件）
- 复核卡：`2026-11-04-precision-np01-field-tables-b1b-check`（QA，p60）
- 复核时点：2026-11-04（本地空闲时段）。复核基线：`abbe89ba`；复核执行时工作区 HEAD 已前进到后续批次
  （B2a/B2b/B2c），待批计数随之收紧，属**预期**，本复核对 B1b 的断言一律按 `abbe89ba` 的提交内容核对。
- 边界：只读复核；未改产品代码/判据/容差/参考/fixtures/清单；零 GPU（未运行 nvidia-smi，未开窗）。

## 1. 结论

**PASS**（无阻断、无表内事实错误、无门禁放宽）。

- 四张表（`kinematics` / `crb` / `mass_multiply` / `mass_solve`）结构、逐字段事实、fma/atomicAdd 口径、
  `PENDING_ROWS` 收紧、清单指针与 README 批次段均成立；边界零改动与零 GPU 成立。
- 观察 2 条（不阻断，见 §6）。

## 2. 范围与边界（对 `abbe89ba`）

`git show --stat abbe89ba` 共 12 文件：

- `docs/development-evidence/np01-type-mapping/field-tables/{kinematics,crb,mass_multiply,mass_solve}.json`（新增）
- `docs/development-evidence/np01-type-mapping/field-tables/README.md`、`inventory.json`
- `tests/precision_type_gate.rs`（`PENDING_ROWS` 25→21）
- 卡 JSON + 4 个门禁回执（`.normify/runs/…`）

对 `src/`、`fixtures/`、`include/` 的 diff **为空**（`git show --name-only` 无这些路径）；零 GPU。

## 3. 表结构与逐字段事实

四张表均 `schema=1`、`batch=B1b`、`row_id` 与 inventory 行 id 一致（`kinematics` / `crb` / `mass_multiply` /
`mass_solve`）。逐表核过：

- 顶层 `batch/layer/audit_level/status/source/upstream/evidence` 非空；`kernels[]` 非空，每个 kernel 的
  `params`/`outputs` 非空、`intermediates`/`accumulations` 为数组且非空。
- `evidence[]` 中**路径型条目**全部存在（命令型条目为可复跑指令，按原样保留）：`kinematics` 3、`crb` 4、
  `mass_multiply` 4、`mass_solve` 9，缺失 0。
- `kinematics`：`outputs_12` 共 12 条 = **本表 7 条**（xpos/xquat/xmat/xipos/ximat/xanchor/xaxis）
  + **3 条指向 `com_position` 表**（subtree_com/cinert/cdof，`rigid_com_position` 内核）
  + **2 条指向 `crb` 表**（crb/matrix_dense_block，`rigid_mass_matrix` 内核）；`extra_sources` 登记了这两个交叉内核，
  与 `outputs_12` 的指向一致。
- `kinematics` 的 `matrix_inertial` 如实登记为**仅定义、当前 `rigid_kinematics` 路径未调用**；`rounding_notes`
  写明 fma 7 = `ufma` 定义 1 + `matrix_inertial` 内 `ufma` 调用 6，与 `raw-kernels.json`（`src/physics/mod.rs`
  `fma=7`）一致。

## 4. 口径核对（fma / atomicAdd）

按 `raw-kernels.json`（文本扫描口径：`fmaf|__fmaf|fma(`、`atomicAdd` 文本）逐行对照：

| 表 | 行字节 | raw-kernels | 表内登记 | 结论 |
| --- | --- | --- | --- | --- |
| `kinematics` | `src/physics/mod.rs` | fma 7 / atomicAdd 0 | fma 7（定义 1 + matrix_inertial 6）；执行路径无显式 fma | 一致 |
| `crb` | `src/physics/mass_matrix.rs` | fma 19 / atomicAdd 0 | fma 19（`mfma` 定义 1 + 调用 18） | 一致 |
| `crb`（常驻段） | `src/physics/mass_matrix/resident.rs` | fma 3 / atomicAdd 2 | resident 2 处 `atomicAdd`（accumulate 10 槽循环、matrix cell 累加） | 一致 |
| `mass_multiply` | `src/physics/mass_matrix/multiply/cuda.rs` | fma 0 / atomicAdd 0 | fma 0、atomicAdd 0，纯 f32 顺序乘加 | 一致 |
| `mass_solve` | `src/physics/mass_solve/cuda.rs` | fma 4 / atomicAdd 0 | fma 4（2 处 `fmaf` + 2 处 `__fmaf_rn`） | 一致 |
| `mass_solve`（原子段） | 同文件 | atomicAdd 文本 0 | 表内另登记 `atom.global.add.f32` 内联原子站点（文本扫描不计入），并说明 MathDx tile/scalar 两条舍入路径 | 一致（区分如实） |

`mass_multiply` 的上游仍为**文件级**（`upstream.confidence` 文件级，未升格为函数级），与表内绑定一致。

## 5. 门禁收紧与清单

- `abbe89ba`: `PENDING_ROWS` 声明长度 **21**、实际 21 个 id；被移除的正是本批四行
  （`kinematics` / `crb` / `mass_multiply` / `mass_solve` 均不在待批清单）。
- inventory `rows` = **30**；`field-tables/*.json` = **9**（B1 5 + B1b 4）⇒ **30 = 9 + 21** 自洽。
- 门禁四类断言（`tests/precision_type_gate.rs::field_tables_cover_registered_rows_without_silent_gaps`）：
  ①每张表 `row_id` 必须在 inventory 行里；②7 个顶层段 + 每 kernel `params/outputs/intermediates/accumulations` 非空；
  ③`PENDING_ROWS` 行必须在 inventory 且**不得已有表**；④「既无表又不在待批」的行必须为空（不允许静默缺表）。
  四类断言均真跑（非空跑）。
- 独立执行门禁：`normify gate <check> fmt clippy clippy-cuda unit` 全过；`unit` = `cargo test --locked` **1268 过 / 0 败**。
- inventory 四行已更新：`status` = 「已出逐字段表（B1b）；剩余：…」，`field_table` 指针指向本批四张表（逐行核过）。

## 6. 观察（不阻断）

1. **门禁对 `evidence` 的校验强度 < 验收措辞**：验收写「`evidence[]` 指向的文件存在」，而门禁用例实际只断言
   `evidence` 段非空，不校验路径存在性。本次 QA 已独立逐条校验（四表路径型条目 **0 缺失**），事实成立；
   建议后续把「路径型条目存在」硬化成机判（或在验收措辞里改为「非空 + QA 抽核」），避免口径漂移。
2. **已知历史漂移（B-final 项，非本片）**：`inventory.summary` 仍是旧状态分布（`updatedAt=2026-10-23`），根目录
   `np01-type-mapping/README.md` 的逐行状态未随各批同步。本片不处理，按 B-final 收口时统一刷新。
3. 复核时 HEAD 已到 B2c 批（`PENDING_ROWS` 已由 21→17→12→7），属后续批次正常推进；B1b 的 21/30=9+21 断言按
   `abbe89ba` 核对成立。

## 7. 不声明

本复核只证明「B1b 四张表的逐字段事实与门禁口径与提交一致」。**不声明 NP01 已达验收、不声明类型映射率达标、
不声明与上游一致**；NP01 仍需 30+3 条全部有表且门禁全绿（当前 18/30，余 B2d/B2e 等批次）。
