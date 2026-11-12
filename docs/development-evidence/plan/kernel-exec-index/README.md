# 内核执行层能力索引（P0-b，85 行）

- 状态：P0-b 产物（L1，0 窗 0 GPU）；未登记片、未排期。
- 用途：把 `docs/implementation-plan.md` §并行计算单元能力定义 的 85 项，逐项落到 A0 契约方法、现状、A1 耦合锚、归属与完成判据，作为 P1 起的排期与验收索引。
- 数据：`capability-index.json`（85 行）；生成脚本 `target/impl-plan-review/gen_capability_index.py`（工作材料，不入仓）。

## 字段

| 字段 | 含义 |
| --- | --- |
| `capability_id` | 能力编号（`G.10` 这类带点编号，与路线图节点 `G10` 不同） |
| `layer` / `domain` / `level` | 设备/语言/库、域名、必/退/射 |
| `contract` | A0 §3 的契约方法/类型 |
| `status` | `done` / `part` / `none` / `n/a`（`n/a` 含 2 项不适用 + 1 项待并入 `D.10`） |
| `status_note` | `n/a` 的状态原因 |
| `evidence_anchor` | 现状证据锚（模块/文件/测试名；来自 A0 评审，按模块级） |
| `a1` | A1 耦合锚：宿主调用点数（`consumer_view`）、内核构造出现数（`classes`，含产品数） |
| `owner` | 归属：`PU-xx（待批）` 或 `backlog（...，未建卡）`；PU 均未登记 |
| `done_criteria` | 按 A0 §10 四要素：契约 / CUDA 实现 / 测试或证据 / PU 映射 |

## 口径

- 行数 85；状态计数 **done 43 / part 25 / none 14 / n/a 3**（`n/a` = 2 不适用 + 1 待并入）。
- 25 part + 14 none = **39 行，全部有 `owner`**（无空行、无空归属）。
- A1 耦合锚只读引用 Backend 的盘点（pin 在下表），本节不重算；A1 数字为 2026-11-12 静态扫描口径。
- 本索引不改变 A0 判断；不声明任何实现状态之外的含义。

## A1 源（pin）

| 文件 | sha256 | bytes |
| --- | --- | --- |
| `target/impl-plan-review/A1-cuda-coupling-inventory.md` | `554267971fe2dc69a0359d6aabba40a7fc18083003132229cd39e9582fd4d3c6` | 37413 |
| `target/impl-plan-review/a1-coupling/a1-host-coupling.json` | `c4e0d0f95aecc030edfb5d43921151f9fb34fccbfa8b019135c2e78345f786aa` | 140411 |
| `target/impl-plan-review/a1-coupling/a1-kernel-coupling.json` | `909f5e756f6d9af58856bb5ff37231048e31621c53b7bc14c8baf59d14668282` | 146631 |
| `target/impl-plan-review/a1-coupling/a1-hotspots.json` | `ed0c9c7c549bfe46a123bd72ffbaee125e673cd556248017c797ba5760bfb6b6` | 8067 |

## 复跑

```
cd D:\Rust\mjwarp-rs
target\mjwarp-parallel\venv\Scripts\python.exe target\impl-plan-review\gen_capability_index.py
target\mjwarp-parallel\venv\Scripts\python.exe target\impl-plan-review\check_capability_index.py
```

## 自检结果（2026-11-12）

- 行数：**85/85**，无空 `capability_id`。
- 归属覆盖：part+none = **39/39** 有 `owner`（PU 或 backlog）。
- 字段完整：`contract` / `evidence_anchor` / `a1` / `done_criteria` 无空值。
- 状态计数：done 43 / part 25 / none 14 / n/a 3（合计 85）。
- 禁词扫描：`0` 命中（扫描词表见卡的 `forbid_claim` 检查；本文件与 JSON 不含禁词）。

## 非声明

- 本索引是工作索引，不是验收证据，不改变任何判据/容差/参考/冻结集。
- `part`/`none` 只表示现状与 A0 契约的差距，不构成对实现难度的判断。
