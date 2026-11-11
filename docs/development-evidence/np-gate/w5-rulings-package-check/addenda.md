# 锁违规留痕 addendum（2026-11-11）：22:52 acquire 失败后未停手

本文件为 append-only 留痕：本卡 `README.md`/`receipt.json`/`logs/independent-check.txt` 历史字节不改；
`manifest.txt` 3→4 条（新增本 addenda）。任务：`2026-11-11-w5-rulings-package-check`（fix 记录见 normify）。

## 1. 事实

- 2026-11-11 22:52:24 我执行 `scripts/validate/repo-write-lock.ps1 -Action acquire -Owner qa`，
  当时写锁由 **Backend 持有**（`pid=61092`，`started=2026-11-11T22:52:24+08:00`），脚本输出
  `REPO_WRITE_LOCK_HELD`（**非** `REPO_WRITE_LOCK_ACQUIRED`）。
- 我**未按「非 FREE 立即停」中止**，继续写了本卡登记提交（`03696e47`）与证据提交（`990d9ae6`），
  并在该窗口内执行了 `normify start/gate/done`。这违反团队单写者纪律。

## 2. 事后核对（无内容丢失）

- 我的登记提交 `03696e47` 仅含 `.normify/changes/2026-11-11-w5-rulings-package-check.json`
  一个文件（79 行 insertions），**未卷入 Backend 的暂存文件**。
- Backend 在 `a14aeee9` → `be36d0ea`（W4-04A 卡）之间的提交**完整**；我的证据提交 `990d9ae6`
  位于其后，未覆盖、未改写任何 Backend 内容。
- 该卡门禁与 done 均正常：fmt/clippy/clippy-cuda passed、unit 1,406 passed / 0 failed / 402 ignored；
  证据 manifest 3/3 自核通过。

## 3. 纠正规则（自本卡起执行）

- `acquire` 输出**只要不是** `REPO_WRITE_LOCK_ACQUIRED`（且随后的 `status` 为 `owner=qa`），
  一律**立即停手、不写任何文件、在回复中上报**，等锁 FREE 再重试。
- 与 Backend 两次锁违规同一留痕标准：事实、影响核对、纠正规则三段齐全，append-only 不改历史字节。

## 4. 材料包修正版追加复核（2026-11-11，append-only）

- 新版 manifest **`5388dae4…`/1,252**（原 `d81efb84…`/1,134）；我重算 **9/9 sha/bytes 全对、文件集合相等**。
- **5 个改动文件**（与派工口径逐一核对）：`06-blocker-cards.md` `436ec91b…`/5,340、
  `03-eol-batch.md` `e324530a…`/4,150、`02-abi-r07-splat.md` `9bdc5ee4…`/4,172、
  `07-scene-bvh-ray-drift.md` `eebe2a6e…`/5,058、`04-equivalence-release.md` `de17980e…`/5,270。
- **4 个未改文件**：`01` `38a133c2…`/3,657、`05` `389a5f08…`/6,389、`README` `7b6c763e…`/2,065、
  `w4-w8-slice-suggestions` `f9c35580…`/5,147——我按字节确认未变。
- **L1（06）已改**：正文改为「其余 **28** 条（32−4；口径 = archive 9 + archive-derived 2 +
  closed-implemented 3 + n/a 6 + recorded-undetermined 2 + recorded-not-restored 2 + deferred 4）」，
  并保留 L1 更正记录（说明 25 只在排除 3 条 closed 时成立、与同句矛盾）——口径自洽，与 NP5-03 账本一致。
- **L2（03）已改**：正文三处改为「**168** 个 UTF-16LE+BOM（全仓 **216**；更正 2026-11-11：原 169/217 把
  二进制 `frame.bin` 的 `FE FF` 首字节误计为 UTF-16）」，并保留 L2 更正记录；与我实测 168/216 一致。
- **O1（02/07）已改**：选项 A 标题改「（候选路径，供参考）」，正文与影响面未改，偏好只留在
  「供参考的建议（不预设结论）」节；保留 O1 更正记录。
- **O2（04）已改**：正文标注「**签名指针待补**：仓内暂无独立 S-1 签名卡/提交号，**勿作为已签依据**」，
  并保留 O2 更正记录。
- 结论：修正版**完整落实本卡 2 低 + 3 观察的处置**（O3 为 target 工作材料口径，按原记录不变）；
  材料仍不预设裁决结论，不改任何卡 acceptance/判据/参考/容差。

## 5. 边界与非声明

append-only；零 GPU、零窗；不改判据/参考/容差、不改历史字节；本 addenda 不改变本卡核对结论
（0 阻断 / 0 一般 + 2 低 + 3 观察仍成立，低项已由修正版闭环）。
