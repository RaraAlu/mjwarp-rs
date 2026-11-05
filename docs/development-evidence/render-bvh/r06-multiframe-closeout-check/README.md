# R06 多帧专题收口治理 —— QA 一致性只读复核

- 任务：`2026-11-05-render-r06-multiframe-closeout-check`（owner QA；**QA 自登记**，Chief 2026-11-05 夜指派；登记提交 `4458a2e9`）
- 复核对象：收口治理卡 `2026-11-05-render-r06-multiframe-closeout`（**verified**；登记 `8f376ae0`、start `73f33bd6`、实现 `6af1d83d`）；**复核目标 HEAD `6af1d83d`**，基线 = 收口前 `d8fc8eeb`
- 边界：**只读复核 + 只写本目录**；CPU-only、零 GPU；不改两卡/backlog 卡/收口证据/既有证据/代码/fixtures
- 方法：独立读卡 JSON 与 `git diff d8fc8eeb..6af1d83d` 比对字段、校验 note 必含要素与证据指针提交存在性、核对 O1 backlog 与收口证据自核/编码/边界；QA 自检 **17/17 PASS**（1 条低观察，与 G-3 O2 同型）

## 1. 结论

**PASS-with-observations（0 阻断）。** 收口治理一致性成立：两卡仅 `note` + `updated_at` 两行变更、`status/title/acceptance/depends_on` 等其余字段逐字未动，状态保持 **backlog / proposed**；note 含 G-2/G-3 覆盖 + 覆盖边界（单场景 scene-flex、3 帧 0/+0.10/+0.20 m、帧内位级 + 跨帧可见性、动画通用性/多场景/长序列未覆盖）+ spec `1d9fa5ce…`/10,506 B 未变 + 四个证据指针提交；roadmap note 明写「保持 proposed、不宣称整族通过」。O1 backlog 卡（`status=backlog`/p15）登记正确且收口范围**未改 `.gitattributes`**；O2/O3 已入收口证据 `known_observations`；变更范围恰 7 文件、`unchanged src/tests/fixtures/scripts` 与既有证据目录；无越界声明。

## 2. 逐项核对

### 2.1 两卡变更面与状态（PASS）
- `git diff d8fc8eeb..6af1d83d` 对两卡 = **4 removed / 4 added 行**（各 1 行 note + 1 行 updated_at）。
- JSON 逐字段比对（排除 note/updated_at）后：`status/title/acceptance/depends_on/checks/revision/...` **完全一致**；tracking `status=backlog`、roadmap `status=proposed` 未变。

### 2.2 note 内容与证据指针（PASS）
- tracking note 含：G-2 采集有效 + G-3 验证 PASS；覆盖边界 = 单场景 `scene-flex`、3 帧（0/+0.10/+0.20 m）、帧内位级 + 跨帧可见性；动画通用性/多场景/长序列**未覆盖**；spec `1d9fa5ce…` 未变；证据指针 G-2 `5f2bd0bb`/`38a7767c` + G-3 `b3a05f2b` + 复核 `d8fc8eeb`。
- roadmap note 含：多帧子项已由 G-2/G-3 覆盖（3 帧集、单场景）；因「动画与几何一致性」广度未覆盖**保持 proposed**、不宣称整族通过；同四证据指针。
- 四个指针提交 `git rev-parse` 全部存在；spec 实测 sha `1d9fa5ce…`/10,506 B 未变。

### 2.3 O1 backlog / O2 / O3（PASS）
- `2026-11-05-render-r06-multiframe-guards-eol-backlog`：`status=backlog`、priority=15；note 记 raw `64bfecdf…`/11,899 B vs HEAD blob `bf4b9302…`/11,696 B 与「今晚不改 `.gitattributes`」。
- 收口范围 `.gitattributes` **零 diff**；收口 receipt `known_observations` 含 O1/O2/O3；README §4 逐条记录。

### 2.4 收口证据 / 边界 / 非声明（PASS）
- 证据目录 manifest 首行 `#`、**2/2** 自核；README/receipt 均 UTF-8/LF/无 BOM、裸 CR/TAB 0、无 U+FFFD。
- 变更范围恰 **7 文件**（tracking/roadmap/closeout/backlog 四卡 + 收口 README/manifest/receipt）；`src/`、`tests/`、`fixtures/`、`scripts/` 与既有证据目录（criterion/capture-g2/verify-g3/entity/entity-fix）**零 diff**。
- 无越界声明：两卡 note、收口 README/receipt 均无「多帧验收通过 / 动画一致性已覆盖 / 上游等价」正向声明；精确禁词仅出现在 closeout 卡自身的 `forbid_claim` 词表与否定句（「不写…」），与 G-3 复核 O2 同型（记已知）。

## 3. 观察清单（0 阻断）

| # | 级别 | 内容 | 说明 |
| --- | --- | --- | --- |
| O1 | 低 | 精确禁词「多帧验收通过 / 动画一致性已覆盖 / 上游等价」只出现在 closeout 卡自身 `forbid_claim` 词表与否定句（不写…）；两卡 note 与收口证据无正向声明 | 工具词表/否定语境，与 G-3 复核 O2 同型；已记已知，无需修 |

## 4. 复现与证据

- `scripts/verify_closeout_review.py`：QA 一致性复核脚本（A 卡 diff/状态、B note 要素与指针、C O1 backlog、D 收口证据、E 边界/禁词）；只读、CPU-only、`sys.dont_write_bytecode`。
- `logs/verify-closeout.log` / `logs/verify-closeout.json`：**RESULT: PASS (17/17，1 observation)**。
- `manifest.txt`：本目录 sha256 + bytes + 证据目录内相对路径（首行 `#`，不自列）。

## 5. 非声明

本复核只确认收口治理的一致性（note/状态/证据指针/边界/非声明）；不重开任何已 verified 判定，不写 R06 整族通过 / 动画与几何一致性全广度覆盖 / 上游等价 / 发布依据。
