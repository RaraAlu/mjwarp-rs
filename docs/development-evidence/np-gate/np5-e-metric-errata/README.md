# NP5-ERRATA-E：E 判据状态 append-only 更正（零 GPU、零窗）

- 任务：`2026-11-11-np5-e-metric-errata`（owner Backend）；登记提交 `40b7f16a`；基线 HEAD `7dc47b38cf3c48925e48fba3e357845b8cd63826`。
- 派工：Chief 2026-11-11 22:2x（用户已确认材料包 05 的 E 口径冲突判断正确）。
- 依据：A2 卡 `docs/development-evidence/np01-e-criterion-a2/README.md`（`2026-11-05-precision-np01-e-criterion-a2`、提交 `b9cb2993`）；
  契约 `docs/numerical-precision-contract.md` L267-L272 与「### E指标定稿（NP01，用户2026-11-05拍板选项①）」。
- 结论：**只更正表述与理由，不改分类/结论/站点数据**；NP01 与 NP02 **均仍未验收**、`any_gate_passed=false`；
  「NP01 精度偏离 0 项未完成」仍成立、**不宣言 NP01 通过**。

## 1. 现行 E 口径（更正后）

- **已签 A2（2026-11-05，选项①「同码跨后端核对」，签名级）**；范围 CONVEX/SDF × dist/pos/frame
  （normal 并入 frame），**97 标量**；判据写法「**实测 worst ≤ 已签 atol**」；不用 E 比值；**不宣称 native**
  （native 侧「仅诊断」保留）。
- 「NP01 精度偏离 0 项未完成」理由：全产品 30 路径 E/atol 核对未完成 / 跨 recipe 位级判据前提不成立 /
  device 级仅可映射子集 / 7 未实现子系统 + Linux 双平台阻塞。
- 「E 指标未定稿（2026-10-27）」只作为**已撤回的历史表述**保留在历史文件字节中，不再作为现行口径。

## 2. 受影响文件清单与逐处更正

### 2.1 仓库证据（append-only addenda；历史字节不改）

| 目录 | addenda（sha256 / bytes） | 逐处更正 | manifest |
| --- | --- | --- | --- |
| `np-gate/np5-02/` | `5c5280db910cc40d51710e16f2d6969ff8b9d580b8a542d264e5f16280747bd8` / 5057 B | 9 条 | 4 → 5 条（历史 4 文件 sha/bytes 不变） |
| `np-gate/np5-04/` | `94b41c8fde50f028c14be4d17f50bc4e4524855ee38777b2720e93a53a361405` / 4589 B | 7 条 | 4 → 5 条（同上） |

16 条逐处明细见 `corrections.json` 的 `repo_evidence[]`（file/line/field/历史快照/现行口径），
与两 addenda 的表格一一对应。历史文件 pre-change sha256：

| 目录 | 文件 | pre-change sha256 / bytes |
| --- | --- | --- |
| np5-02 | README.md | `15a247c70291613a3b5842e2aa290967f9fb9477a1d956f40d06bf072157c119` / 5483 |
| np5-02 | evidence.json | `6526ca1331842e230e1886defdcb7c9dbc51cf21daaec83577cf6b11d788debe` / 32689 |
| np5-02 | receipt.json | `98b05ef7aaea51bf8723995a207792b016c1f9f13540892e65071f08f21aebbb` / 10211 |
| np5-02 | logs/selfcheck.txt | `0a71e3776b1b0370c74423debdfecc81eac0cf02a9e01a8cb119799fa86c461c` / 1356 |
| np5-04 | README.md | `20d5e976743046d1aee3ec9dad34f803e8ffb223c559414e993705f837559d0d` / 3788 |
| np5-04 | evidence.json | `4a9f42cb3a5c8cb7b050f438c4e3f70e92964655516f6fb0b7d34211b08d0477` / 6177 |
| np5-04 | receipt.json | `fd4ef266968d54a09e4993fbf7c118cccf717d9ee27f39c9f5a2d1133f35d3aa` / 2206 |
| np5-04 | logs/selfcheck.txt | `bb5274336c647e6aad774e87b70e258ac4c844e3ca951f77c9ac9fa23b006b7b` / 1044 |

### 2.2 分类/结论/数据不变（对照）

`precision_deviation_zero_items_verified=false`、`any_gate_passed=false`、608/598、33 张类型表、
engine-semantic source 20 / RE-3 204 站点、recipe 差 5 case、已知偏离 1 项、NP02 inventory 102/98、
32 缺口处置 9/2/3/4/6/2/2/4、4 未实现子系统、7 项残余阻塞、7 个裁决点 id 集合——全部不改。

### 2.3 W5 working docs（target，工作材料，不入仓）

| 文件 | 更正 | 新 sha256 / bytes |
| --- | --- | --- |
| `w5-np01-02-plan.md` | append §8 更正（E 已签 A2；NP5-02 未完成理由更新；原文不改） | `99dce362df2871ac29ba42bcffe270fa58ea064799f56a164d330111ac021268` / 8983 |
| `np5-rulings-memo.md` | append §7 更正注记（§5 第 3 条为当时口径；原文不改） | `7a1f21a7e90592fd59983fbc7069d8836fbca2fc89d8470118761eb2c54c758a` / 10307 |
| `w5-rulings-package/05-e-metric.md` | append 确认与执行记录（Chief 确认 A2 已签；append-only 已执行；§2 选项不选边） | `389a5f0821231560b406553581c98382ec4cab4f7c8df2f5bfa4bd477d993c75` / 6389 |
| `w5-rulings-package/manifest.txt` | 重算（05 sha/bytes 更新） | `d81efb84228261849573280abd2a8cb98a8b6c3d9e1c4789183fd1618d2f0f15` / 1134 |

## 3. 验证

- selfcheck（本目录 `logs/selfcheck.txt`）：历史字节不变、两 addenda 与 manifest 自核、16+4 条更正在位、
  不变量对照、working docs 新 sha、policy_scan 0 命中 + `--self-test` 34/34、编码/边界 PASS。
- 四门禁：fmt / clippy / clippy-cuda / unit（unit 1,406 过 / 0 败 / 402 ignored）。
- 本目录 `manifest.txt` 4 文件（README/receipt/corrections.json/logs/selfcheck.txt；不列自身）。

## 4. 非声明

不宣称 NP01/NP02 通过、不宣称 E 完成、不宣称上游等价/双平台、不提供位级保证；
不把未实现子系统写成通过；不改判据/参考/容差；不新增/改任何分类或站点数据。

## 5. 边界

写域仅 `docs/development-evidence/np-gate/np5-02/`、`np5-04/`、`np5-e-metric-errata/` + `.normify/`
（target 工作材料为工作区文件、不入仓）；`src/`/`tests/`/`fixtures/`/`scripts/` 零改动；零 GPU、零窗。
