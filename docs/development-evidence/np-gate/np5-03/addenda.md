# NP5-03 addendum（NP5-REVIEW-LOW 卡，2026-11-11）：`inventory.unimplemented` 计数口径注记

本文件为 append-only：NP5-03 的 `README.md`/`evidence.json`/`receipt.json`/`logs/selfcheck.txt` 历史字节不改；
`manifest.txt` 4→5 条（新增本 addenda）。任务：`2026-11-11-np5-review-low-items-addenda`。

## L3（QA 复核低项）——已加注区分

- 事实：NP02 清单 `docs/development-evidence/np02-evidence/inventory.json` 的
  `counts.unimplemented = 7` 是 **NP02 原始历史快照口径**（7 个未实现子系统）；NP5-03 按证据把其中
  **3 条关闭为 `closed-implemented`**（渲染/BVH、hfield/SDF、GJK/EPA：后续片已实现、回执到位），
  故当前剩余 = **4**（`flex collision` / `constraint` / `solver` / `island`，卡 H `blocked-with-card`、
  `counts_as_coverage=false`），以 receipt `unimplemented_remaining=4` 与
  `evidence.conclusion.unimplemented_remaining=4` 为准。
- 两处数字各自有出处、不构成矛盾宣称：**7 = 历史快照（不删除）**；**4 = NP5-03 处置后剩余（现行口径）**。
- 计数纪律：`unimplemented=7` 不得用作当前未实现数；4 个未实现子系统仍 blocked、不计覆盖、不写成通过。

## 边界与非声明

append-only；不改 inventory 历史计数快照与 NP5-03 历史字节；零 GPU、零窗；
不宣称 NP02 通过、不宣称覆盖已验收；不改判据/参考/容差。
