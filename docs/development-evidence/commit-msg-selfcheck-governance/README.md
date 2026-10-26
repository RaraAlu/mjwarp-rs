# commit message 禁词自查脚本（与 scope-selfcheck 同级的提交前机械自查）

**任务**：`2026-10-27-commit-msg-forbid-selfcheck`（owner Arch，Chief 2026-10-27 建议）。
**触发事实**：M3 提交 `f1e5cda1` 的第一次提交尝试被 commit-msg hook 拒绝——提交信息在否定句里写了字面词
「精度达标」，命中 `forbid_claim` 规则。手写提交信息容易漏，故做成脚本。

## 1. 用法

```text
node scripts/commit-msg-selfcheck.mjs <task-id> <提交信息文件> [--repo <path>]
node scripts/commit-msg-selfcheck.mjs --self-test          # 4 条内置断言，可复跑
```

- **词表来源**：任务 JSON 的 `checks[kind=forbid_claim].terms`（与 commit-msg hook 同源）；任务文件不存在或
  无该检查时回退内置默认表（`G10已实现`/`完整物理通过`/`双平台通过`/`上游完全等价`/`原生放行`/`精度达标`），
  并在输出里注明「默认词表」。
- **扫描范围**：提交信息全文（标题 + 正文 + trailer），逐行逐词匹配，命中打印「行号 + 命中词 + 该行原文」。
- **退出码**：`0` 零命中；`1` 命中禁词（hook 会拒绝该提交）；`2` 用法/文件读取错误。
- UTF-8 BOM 与 CRLF 均容错。

## 2. 自测与对照（本次实测）

| 用例 | 输入 | 结果 |
| --- | --- | --- |
| 内置 self-test | 干净 / 含「精度达标」/ CRLF 含「上游完全等价」/ BOM 干净 | **4/4 PASS** |
| 正例对照 | `target/commit-msg-m3.txt`（35 行，已用词表改写） | exit 0，零命中，词表来源=任务 JSON 6 条 |
| 负例对照 | 带「精度达标」「上游完全等价」的临时消息 | exit 1，命中 2 处并定位行 3 |
| 回退对照 | 任务文件不存在（`no-such-task-xyz`） | 回退默认 6 条词表 → exit 1 |

## 3. 流程接入

`docs/development-evidence/scheduling-registration-checklist.md` 新增 **§7.1**：提交前与 `scope-selfcheck` 并列、
**必须**跑本脚本，exit 0 才能 `git commit`。改写口径示例：用「不宣称精度目标已达成」代替字面禁词。

## 4. 边界

- 只做提交信息文本检查；不改任何判据、容差、证据数字与窗口数据。
- 不自造词表口径：以任务 JSON 的 `forbid_claim` 为准；默认表只是兜底。
- 与 hook 的关系是「提前自查」而非替代：hook 仍是最终闸门。
