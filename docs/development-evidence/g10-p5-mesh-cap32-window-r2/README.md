# r2 窗口材料目录（历史安置）与 attempt 2 PASS 收据镜像

- 任务：`2026-11-12-w2-u7u8-full-condition-assert`（卡保持 in_progress；主卡注册时的窗口证据目录就是本目录）。
- 目录历史：
  1. **窗口前材料**：`window-readiness.md`、`refreeze-10-exe-r2.json`/`.tsv`、`guards-r2.tsv`、
     `logs/nvrtc-identity-r2.txt`（r2 冻结轮、最小窗就绪）。
  2. **attempt 1（r2 冻结轮）FAILED**：U7/U8 comparator 21 条不一致（对序/行序），
     append-only 记录在 `docs/development-evidence/g10-p5-mesh-cap32-window-r2-fail/`（verdict=FAILED、attempt=1、exit_code_capture_bug）。
  3. **修正**：U7/U8 用例 pair 按上游 `_add_geom_pair` 归一化（`g1=1(box), g2=0(mesh), type1=6, type2=7`），
     只重建 branch exe → **r3 冻结**（`ac823b58…`/41,782,272 B；9 旧 exe sha 不变）。
  4. **attempt 2（r3 冻结轮）PASS**：完整证据（README/receipt/logs/facts/guards/manifest）在
     `docs/development-evidence/g10-p5-mesh-cap32-window-r3/`。
- 本目录仅新增 `receipt.json` **收据镜像**（`kind=window-receipt-mirror`）：因主卡运行中检查项冻结、指向本目录
  `receipt.json` 的字段路径，故按实际 attempt 2 PASS 字段镜像并提供 attempt 1 FAILED 指针与完整证据路径；
  **它不是对 r2 exe（attempt 1）的 PASS 声明**。
- 非声明：不宣称 r2 exe 通过、mesh 容量缺口已清、与上游等价（over-32）、G10@P5 整族通过、NP01/NP02 门禁通过；
  不作位级等价声明、不提供位级等价保证。

## 文件

`window-readiness.md`、`refreeze-10-exe-r2.json`/`.tsv`、`guards-r2.tsv`、`logs/nvrtc-identity-r2.txt`、
`receipt.json`（attempt 2 PASS 镜像）、`README.md`（本文件）。
