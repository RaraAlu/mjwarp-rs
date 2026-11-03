# 六、修后变异回归自证（QA 观察 1 的 closed 证据）

来源：QA 复核 `2026-11-04-precision-np01-b2d-b2e-b3-check`（提交 `c7841522`）观察 1——三条变异（重复同 `row_id` 副本、文件名↔`row_id` 互换、字段表重复副本）在**修前**全部 PASS。本片在 `5b5ced53` 落地硬化（`file_stem == row_id`、field 侧 `covered` 去重后比表数、host 侧表数断言），现补**修后**回归自证：三条变异必须全部 FAIL，且恢复后基线必须全绿。

执行日期：2026-11-04；基线 HEAD `ff681521`（含本片硬化）。原始输出逐字节保留在 `logs/`。

## 1. 基线（未变异，必须 PASS）

| 命令 | 结果 | 日志 |
| --- | --- | --- |
| `cargo test --locked --test precision_host_gate` | **1 passed / 0 failed** | `logs/baseline-host.txt` |
| `cargo test --locked --test precision_type_gate` | **4 passed / 0 failed** | `logs/baseline-type.txt` |

## 2. 三条变异（修后必须 FAIL）

| # | 变异（对应 QA 实验） | 操作 | 修后观测 | 断言位置 | 日志 |
| --- | --- | --- | --- | --- | --- |
| 1 | **重复同 `row_id` 副本（host 侧）** | `host-tables/gjk_host.json` 复制为 `gjk_host_duplicate.json` | **FAIL**：`gjk_host_duplicate.json 文件名≠row_id` | `tests/precision_host_gate.rs:25` | `logs/mut-1-dup-copy-host.txt` |
| 2 | **文件名↔`row_id` 互换（host 侧）** | 用 `ccd_host.json` 内容覆盖 `gjk_host.json`（等价于两表 `row_id` 互换） | **FAIL**：`gjk_host.json 文件名≠row_id` | `tests/precision_host_gate.rs:25` | `logs/mut-2-swap-host.txt` |
| 3 | **字段表重复副本（field 侧）** | `field-tables/kinetic_energy.json` 复制为 `kinetic_energy_duplicate.json` | **FAIL**：`kinetic_energy_duplicate.json 文件名≠row_id`（3 passed / 1 failed） | `tests/precision_type_gate.rs:311` | `logs/mut-3-dup-copy-type.txt` |

说明：三条变异都在**主防线** `file_stem == row_id` 上被拦下；对应 QA 修前「PASS（未拦截）」的三条全部翻红。计数断言（`covered` 去重后 `len == tables.len()`、host 侧 `covered.len() == host_sites.len()`）是第二道网，用于大小写不敏感文件系统/符号链接副本这类文件名合法但条目重复的情形——本次变异实验走的是主防线，计数断言未被触发，**不声称它被独立验证过**。

## 3. 恢复（必须回到全绿）

| 命令 | 结果 | 日志 |
| --- | --- | --- |
| 删除两个 `*_duplicate.json`、`git checkout -- host-tables/gjk_host.json` | `git status --porcelain` = **0 行** | — |
| `cargo test --locked --test precision_host_gate` | **1 passed / 0 failed** | `logs/restored-host.txt` |
| `cargo test --locked --test precision_type_gate` | **4 passed / 0 failed** | `logs/restored-type.txt` |

## 4. 边界与非声明

- 只跑变异实验与恢复，**不改**判据/容差/表内容/产品代码；`src/`、`include/warp-headers/`、`fixtures/` 零改动。
- 零 GPU。
- 本证据不改变 NP01 状态：NP01 **仍未验收**（E 指标未定稿、host/device 逐表达式位级对照未补）。
