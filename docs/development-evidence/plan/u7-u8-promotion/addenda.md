# Addenda（append-only 修正记录）—— U7/U8 判据卡 QA L1/L2 与观察记录

适用：`docs/development-evidence/plan/u7-u8-promotion/`（卡 `2026-11-12-u7-u8-promotion-criteria`，内容提交 `1da6bbad`）。
性质：QA 复核（PASS；0 阻断 / 0 一般 / 2 低 / 3 观察）后按 Chief 指令的 **append-only 修正**——历史提交字节不改（原句只在下方豁免引文块内逐字引用），本文件 + 两个被修正 JSON 的新版本 + manifest 重算构成修正面。
边界：不改判据/容差/参考/fixtures；不宣称 U7/U8 通过；零 GPU、零窗。

## 1. L1（低）：`u7-u8-criteria.json` 的 `boundary.not_claimed` 含无否定语境的「位级等价声明」

- 原句（历史版本 `1da6bbad`，逐字引文；**该表述已撤回/取代**，现行版本见下）：
<!-- policy-scan:quote:start -->
撤回（5）：以下为历史版本逐字引文，本 addenda §1 已取代该表述；supersedes:
reviewed_by: arch-2026-11-12
"not_claimed": ["与上游一致（over-range）", "mesh 容量缺口已清", "U8 已通过（A2 窗之前）", "位级等价声明"]
<!-- policy-scan:quote:end -->
- `policy_scan.py` 原命中：1 命中（`u7-u8-criteria.json`；命中项即上方引文中已撤回的那条表述）。
- **修正（现行版本）**：`boundary.not_claimed` 最后一项改为「**不宣称位级等价（bit-level equivalence not claimed）；不提供位级等价保证**」。
- 语义不变：仍表示「禁止把任何结论写成位级等价」；只是把该条写成合规否定式，使 scanner 在否定窗口内识别为否定、不再命中。
- 重扫结果：本目录 6 个证据文件 `policy_scan` **0 命中**（见 §4）。

## 2. L2（低）：`receipt.json` 的 `kind` 写「no card, no commit」与事实矛盾

- 原句：`"kind": "0-window docs evidence (CPU-only, no GPU window, no card, no commit)"`
- 事实：本证据已建卡并提交——卡 `2026-11-12-u7-u8-promotion-criteria`（`56ba6bb0` 登记），内容提交 `1da6bbad`，本条修正为 `normify fix` 提交。
- **修正**：`kind` 改为「0-window docs evidence (CPU-only; landed via card 2026-11-12-u7-u8-promotion-criteria; commits 56ba6bb0 registration + 1da6bbad content + this normify fix commit)」；并新增 `card`、`commits`、`corrections` 字段；`tree.head` 更新为修正时 HEAD（原 `caeeb99b…` 为撰稿时 HEAD，保留在 git 历史的旧版本中）。
- 边界不变：仍为 0 窗 CPU-only 证据；卡片/提交事实以本节与 `receipt.json` 为准。

## 3. QA 观察（只记录，不改内容）

- **O1（观察）**：`g10-p5-partial-uncovered.json` 中 UNC-1/UNC-3 的术语与主台账存在混用（UNC-1 行同时出现「partial」「未覆盖」口径）。**处理**：不改 JSON 内容（台账计数口径以 g10-p5-acceptance-map 为准）；在此登记：UNC-1/UNC-3 的 `group=uncovered` 行仍属「uncovered 6」列表，其中状态字段可为 partial/closed/unmet；引用时应以 `id` + `status` + `group` 三者合读。
- **O2（观察）**：`a2-slices.json` L55 的 `"见 criteria §1.3"` 指代有歧义。**处理**：不改 JSON 内容；在此更正——该处应指 **`README.md` §1.3（「限制语义 vs 实际触发」边界口径）**，不是 `u7-u8-criteria.json` 的字段结构。

## 4. 复验（本修正提交前执行）

- `python scripts/np-gate/policy_scan.py <本目录 6 个文件>` ⇒ **files=6 hits=0 RESULT: PASS**（执行日志作为工作材料留在 `target/u7u8-fix/policy_scan.txt`，非证据；证据只记命令与结果计数）。
- 四门禁 `normify gate 2026-11-12-u7-u8-promotion-criteria --retry "QA L1/L2 append-only 修正"`：fmt / clippy / clippy-cuda / unit。
- `manifest.txt` 重算：6 行（README / 三份 JSON / receipt / 本 addenda），逐条 sha256+bytes 自核、0 mismatch。

## 5. 非声明

本 addenda 只修正 L1/L2 的措辞与事实字段并记录两条观察；不改判据/容差/参考/fixtures、不改任何已 verified 数字；不宣称 U7/U8 已通过、不宣称 mesh 容量缺口已清、不宣称 G10@P5 整族通过。
