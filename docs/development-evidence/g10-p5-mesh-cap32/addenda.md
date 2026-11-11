# A2-1 证据勘误与 QA 观察记录（append-only；不改历史字节，仅本条 §L1 的最小修正已同步 README）

- 任务：A2-2 第一节附带修正（Chief 2026-11-12 指示）；被修正证据：`docs/development-evidence/g10-p5-mesh-cap32/`（A2-1，提交 `2ee0540d`）。
- QA 复核结论：**PASS**（0 阻断、0 一般 + 1 低 + 2 观察）。
- 规则：历史字节不改；本文件为生效说明与勘误记录。唯一例外是低 L1 指示的 README §6 措辞修正（见 §L1，逐字记录改前后）。

## L1（QA 低项）：README §6 非声明列表的「位级＋等价」连写短语无否定语境 → policy_scan 1 命中

- 原文（README §6 末项，改前逐字；该旧表述已作废，以下闭标记引用块仅留档，不构成正面声明）：

<!-- policy-scan:quote:start -->
已作废旧表述：…、与上游一致（over-32）、位级等价。
<!-- policy-scan:quote:end -->
- 改为：`…、与上游一致（over-32）、不作位级等价声明、不提供位级等价保证。`
- 理由：policy_scan 的否定豁免窗口只覆盖紧邻 18 字符；原列表式写法距否定词过远 → 1 命中。改后否定词紧邻核心词。
- 复扫：`policy_scan.py` 对 A2-1 证据目录（README/receipt/logs 等）**0 命中**；`--self-test` **34/34 PASS**。
- manifest：10 → **11** 条（新增本 addenda），逐文件 sha256+bytes 重算，历史文件除 README 上述一句外字节未变（README 新 sha 见 manifest）。
- 边界：本次修正只改非声明措辞，不触碰任何测试/源码/判据/结论/数字。

## O1（QA 观察）：边界单测口径写实——「8 条」实为 7 组 9 条

A2-1 README §3 写的「A2-1 边界单测（8 条）」按测试函数**组**计；实际执行条目为 **9 条 = clip 3 + 宿主容量门 4 + A1 保留 2**：

| 组 | 条目 | 数 |
| --- | --- | --- |
| clip（`gjk/multicontact/mod.rs`） | 32 顶点 face1 / 32 顶点 face2 / 33 顶点拒绝 | 3 |
| 宿主容量门（`inputs_from_snapshot_tests.rs`） | 32/32 边界接受 / face=33 拒绝 / degree=33 拒绝 / 无 sidecar 接受 | 4 |
| A1 保留 | 20 顶点 face1 / 20 顶点 face2 | 2 |

另 `host_limits_match_device_macros`（宏同步）为独立守护测试，不在上述 9 条内。定向测试实测 11 passed = clip 5（含 A1 2）+ 宿主 6（含宏同步 1 + 无 sidecar 1）。

## O2（QA 观察）：旧测试改写未在 README 点明

A2-1 把 `inputs_from_snapshot.rs` 原测试 `product_entry_rejects_fan_cone_24_on_both_dimensions`（24/24 超 20/10 → 两维拒绝）**改写为** `product_entry_accepts_fan_cone_24_within_capacity_32`（24/24 在 32/32 内 → 接受）；这是容量提升的预期行为变化，不是放宽判据。README §3 未点名该改写，此处补记；改写的判据依据见 A2-1 receipt 与 `u7-u8-criteria.json` §boundary。

## 边界

- 本文件只记录勘误与观察，不改变 A2-1 任何结论、数字或验收状态（A2-1 仍 verified）。
- 不宣称 U7/U8 通过、mesh 容量缺口已清、与上游等价（over-32）。
