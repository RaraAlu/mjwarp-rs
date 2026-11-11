# Addenda（append-only，2026-11-12）：Arch L1/L2 修正追加复核

本文件为 append-only：本目录 `README.md`/`receipt.json`/`logs/independent-check.txt` 历史字节不改；
`manifest.txt` **3→4** 条（新增本 addenda）。对象：提交 **`51702a6b`**（被复核卡
`2026-11-12-u7-u8-promotion-criteria`，`normify fix` 留痕；该卡 `addenda.md` 新文件）。

## 1. 新 sha 逐项核（vs 派工口径）

- `u7-u8-criteria.json` **`4290569e…`/4,979**、`receipt.json` **`d80178b0…`/5,113**、
  `addenda.md`（新）**`a9ddb713…`/4,133**、`manifest.txt` **`7fd8a2e1…`/745**——我逐字节重算，**4/4 全部匹配**。
- **未变文件**：`README.md` 20,850、`g10-p5-partial-uncovered.json` 5,681、`a2-slices.json` 8,667
  与修正前 `e997ea1e` 对应 blob **逐字节相同**。

## 2. L1（`not_claimed` 否定式）核

- 现 `boundary.not_claimed` 末项为「**不宣称位级等价（bit-level equivalence not claimed），不提供位级等价保证**」
  ——带否定语境，满足 scanner 语义。
- 历史原句只出现在 `<!-- policy-scan:quote:start -->` … `<!-- policy-scan:quote:end -->` **引文块内**
  （块内含 `supersedes:` 与 `reviewed_by: arch-2026-11-12`），符合「标记引文块 + 撤回/替代声明」豁免规则；
  块外无未否定原文。
- 我重跑 `policy_scan.py`（本目录 6 个文件）→ **files=6 hits=0 RESULT: PASS**。

## 3. L2（`receipt.kind`）核

- `kind` 已去掉「no card, no commit」，改为「0-window docs evidence (CPU-only; landed via card
  2026-11-12-u7-u8-promotion-criteria; commits 56ba6bb0 registration + 1da6bbad content + this normify fix commit)」。
- 新增 `card` 与 `commits{registration:56ba6bb0, content:1da6bbad, fix:this commit}` 字段，与 `kind` 事实一致；
  `tree.head` 注明为撰写时 HEAD（原 `caeeb99b…`），并说明历史旧版本保留。

## 4. Arch `addenda.md` 与修正留痕核

- Arch addenda 结构完整：§1 L1 修正（原句引文块 + supersedes、新版本否定式、policy 复扫 0）、
  §2 L2 修正、§3 **O1/O2 观察记录**（不改 JSON 内容，只登记处理口径）、§4 复验（policy 0 / 四门禁 / manifest 6 行自核）、
  §5 非声明。
- 卡 JSON 的 `addenda` 修正记录（`at=2026-11-11T17:38:30Z`、`reason`、`prev`、`hash`）在位，卡仍 `verified`。
- **gate 绑定新 HEAD 窗口**：修正后 gate runs 时间 `01:39:25–01:39:43 +08`，修正提交时间 `01:39:52 +08`
  ——门禁在修正提交前、基于新 HEAD 树执行，unit `1409/0/402`。
- `manifest.txt` 6 行（README / 三 JSON / receipt / addenda）我重算 **6/6 全对**、0 mismatch。

## 5. 结论

Arch 的 L1/L2 修正**闭环**：2 低均已按 append-only 处置、历史字节未改、policy 0、门禁绑定新 HEAD；
O1/O2 两条观察已按「记录即可、不改内容」登记。**本 addenda 不改变我的原复核结论**（PASS，0 阻断/0 一般/
2 低/3 观察），仅记录修正复核结果。

## 6. 非声明

append-only、零 GPU、零窗；不改判据/参考/容差/历史字节；**不宣称** U7/U8 通过、mesh 容量缺口已清、
G10@P5 整族通过、准入/发布、上游等价；不宣称位级等价。

## 7. 扫描范围说明（本卡自身证据）

- 本卡历史日志 logs/independent-check.txt（e997ea1e 时落地）保留了当时 policy_scan 的**原始输出引用**，
  其中含被引用的禁词短语；按 append-only 纪律不改历史字节。
- 本轮复扫：README.md / 
eceipt.json / ddenda.md **0 命中**，日志文件因历史引用命中 1 次——
  属历史日志引用、非新声明；后续对本卡目录扫描时建议把 logs/ 排除或按历史引用处理。
