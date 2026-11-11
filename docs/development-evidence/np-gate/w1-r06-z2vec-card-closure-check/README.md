# QA 复核 W1-CLOSE-Z2VEC 治理关闭片（关闭 R06 z2vec backlog 卡）（`2026-11-11-w1-r06-z2vec-card-closure`，提交 `db95f7b3`）

- 复核对象：卡 `2026-11-11-w1-r06-z2vec-card-closure`（`docs/development-evidence/plan/w1-r06-z2vec-card-closure/`）；内容提交 **`db95f7b3`**。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 0 低 + 2 观察）**。

## 1. 独立核验

- **acceptance 5/5 且原文未动 ✅**：我在 `da74c9fe^` 与 `db95f7b3` 两个 blob 上核——acceptance 均为 5 条且**逐字相同**；状态 `backlog(checks=5)` → **`verified`**、`closed_at=2026-11-11T15:33:02Z`。
- **零 `.cu` 改动 ✅**：`19a68b49` 是**唯一且最后**一个改 `src/render/kernel/cuda/render_flex.cu` 的提交（`git log -1 --` 实测）；当前文件 sha `97d5fb2c…`/7,426 B，`:47–56` 为近平行反向分支四行赋值 `quat[0..3]=1,0,0,0`；关闭链 `da74c9fe`/`5009c1c1`/`40ed8b20`/`c21a421b`/`79361ae1`/`db95f7b3` **无 src/tests/fixtures/scripts 改动**。
- **测试 9 passed ✅（我重跑）**：`cargo test --locked --features cuda-probe --test render_flex_z2vec -- --test-threads=1` → **9 passed / 0 failed / 0 ignored / 0 filtered**；`tests/render_flex_z2vec.rs` 实测 9 个 `#[test]`、sha `334100b7…`/8,956，三条矩阵断言（单位阵 / `DIAG_NEG_NEG_POS` / y 轴目标）均在位。
- **内核身份未变 ✅**：`render_megakernel` 回执 `sourceSha256=cf356b72…`、PTX `b63a51dd…`（我核 receipt 原件），与关闭卡口径一致——修复后身份未被本链改动。
- **证据指针/门禁 ✅**：`r06-s3b-flex-z2vec/` README `e28cbeb9…`/9,605、receipt `5bc16d57…`/7,103、manifest `491d9089…`/733；`perworld-l-v2-flex-z2vec-pin/` README `d5262719…`/7,436、receipt `50388381…`/2,798、manifest `9e848448…`/1,251——全部可解析；关闭证据 manifest **`9e9e1c1c…`/649（5 行）全对**；`logs/gate-z2vec.txt` 记录 unit **1406 过 / 0 败**。

## 2. 发现

### 观察
- **O1**：关闭链只动卡面与证据目录（`c21a421b` 新增 6 个证据文件、其余仅卡 JSON），**未重做修复、未改 `.cu`**；卡面 acceptance 5 条原文逐字未动，与「治理关闭、不改卡面」口径一致。
- **O2**：「当前 megakernel 身份即修复后值」的证明方式是内核身份（source `cf356b72…`/PTX `b63a51dd…`）+ 离线 PTX/CPU 镜像，**不是设备端重跑**；README 已强制标注「CPU 镜像断言不等价于设备端验收」，记录正确。

## 3. 非声明

本复核只对关闭/评估的事实、指针与边界负责；**不宣称**任何门禁或阶段准入通过；不宣称 NP 通过；
不宣称设备端渲染变化生效；不宣称上游等价；不宣称位级等价；不宣称双平台或发布；零 GPU、零窗。
