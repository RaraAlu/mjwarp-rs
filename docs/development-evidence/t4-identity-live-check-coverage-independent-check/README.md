# T4 身份活体校验覆盖 · QA 独立复核（20/20 + 反向判别 + 默认门禁纳入）

- 复核任务：`2026-10-25-t4-identity-live-check-coverage-independent-check`（QA）
- 被复核任务：`2026-10-25-t4-identity-live-check-coverage`，实现提交 **`00831281`**
- 复核 HEAD：`c3acc6b6`；性质：CPU-only、无 GPU 窗口；不改产品树/判据/容差/门禁
- **结论：pass（阻断 0 / 发现 0 / 观察 4）**。独立复算 **20/20** 条 `product_sources` 全部一致
  （含 `src/physics/mod.rs` 的 physics_core 组合摘要，按锚点 `include_str!` 列表独立解析重算）；
  自建临时 fixture 副本经 seam 的反向判别全部按预期红/绿；该测试确实被默认 `cargo test --locked`
  与 gate unit 纳入（当前 604 passed / 7 ignored），且纯 CPU、无 GPU/NVRTC/编译调用；实现提交边界干净。

## 1. 独立复算（自写脚本，不复用被测支持模块）

- 脚本：`qa_recompute_identity.py`（Python `hashlib`，不 import/不运行 `tests/support/t4_identity_live.rs`）。
- 口径：
  - 单文件：`sha256(LF 归一化字节)`（`b"\r\n" → b"\n"`，我独立实现并核对）；
  - `src/physics/mod.rs`：从锚点 `docs/development-evidence/t4-resident-anchors/physics.rs` 用正则
    解析 `include_str!` 列表（**不硬编码**，实际解析到 5 个：mod / com_position / device_checks /
    history / kinematics），逐个按锚点目录解析、LF 归一化、以 `b"\n"` 连接后 sha256。
- 结果（`qa-identity-recompute.json`）：fixture `product_sources` **20 条，逐条 expected == actual，
  0 差异**；重复路径 0、缺失文件 0。
- physics_core 组合摘要 recompute = `33a6e55f8f758528db9c21d82ff639ce794ce80408c9271f4394f3667036e546`，
  与 fixture 中 `src/physics/mod.rs` 记录一致（该组合摘要由 5 文件、合并后 42,705 bytes 计算，明细见 JSON）。
- 集合完整性交叉核对：fixture 路径集合 20 条 = 交付 `EXPECTED_PATHS` 常量 20 条（我从支持文件文本解析，
  仅作交叉记录，不作为独立 oracle）；「哪些路径应在集合内」本质是交付判据，我复核的是它被一致执行且可判别。

## 2. 反向判别（自建临时副本，仓库 fixture 未被改动）

脚本：`qa_make_tampered_fixtures.py` 生成 `target/qa-t4-check/` 下 4 份副本；`run_discrimination.ps1`
经 seam `T4_IDENTITY_LIVE_FIXTURE` 驱动被测测试（`cargo test --locked --test t4_identity_live_check`）：

| case | 副本改动 | 期望 | 实测 exit | 关键诊断 |
| --- | --- | --- | --- | --- |
| normal-repo-fixture | 不设 seam，读仓库 fixture | 2 passed | **0** | `test result: ok. 2 passed` |
| tamper-sha | 1 条期望 sha 首字符 `0→1`（`fixtures/mjwarp-mass-solve/manifest.json`） | 失败 | **101** | `T4 身份活体校验失败（1 条差异，共 20 条）` + `expected=18cc1fd3… actual=08cc1fd3…` |
| tamper-path-rename | 1 条 path 改名（`src/physics/sleep.rs` → `src/physics/sleep_renamed.rs`） | 失败 | **101** | `路径集合不完整：缺少 ["src/physics/sleep.rs"]，多出 ["src/physics/sleep_renamed.rs"]` |
| tamper-path-remove | 删除 1 条 path（19 条） | 失败 | **101** | `product_sources 条数应为 20，实际 19`（断言于测试 L24） |
| normal-copy | 不改内容的副本（只走 seam） | 2 passed | **0** | 证明红/绿差异只来自 fixture 内容，不是 seam 本身 |
| normal-restored | 清除 seam 后重跑仓库 fixture | 2 passed | **0** | 恢复后绿 |
| list | `--list` | 列出 2 个测试 | **0** | `live_product_sources_match_fixture` / `sha256_and_lf_normalization_match_known_vectors` |

- 汇总 `qa-discrimination.json`（含逐 case 的诊断 token 检查）；原始日志在 `logs/`。
- `git status --porcelain -- fixtures/precision/t4-current-kernels.json` = 空 → **仓库 fixture 未被改动**。
- 说明：path 类篡改在「集合完整性/条数」检查即失败，因此其诊断含 path/集合信息但不含
  expected/actual sha 对（sha 类篡改的诊断才含该对）；这与交付行为一致，见观察 3。

## 3. 默认门禁纳入与纯 CPU 独立核（含 unit 计数变化）

- **机制核**：`.normify/gates.toml` 的 `[gate.unit]` 命令是 `cargo test --locked`（**无 `--lib`**）→
  自动发现并执行 `tests/t4_identity_live_check.rs`。
- **我实测**：`cargo test --locked -- --list`（默认命令的 list 模式，exit 0）列出 **611** 条测试条目
  （= 604 passed + 7 ignored），其中包含：
  `live_product_sources_match_fixture: test`、`sha256_and_lf_normalization_match_known_vectors: test`
  （`logs/default-cargo-test-list-extract.txt`；完整 UTF-16 列表在 `target/`，不入库）；
  `cargo test --locked --test t4_identity_live_check` 实测 **2 passed**。
- **gate unit 计数**：本轮在主树 HEAD `c3acc6b6` 跑 `normify gate` → **unit 604 passed / 0 failed / 7 ignored**、
  EXIT 0。交付声称「595→597」的独立核（committed run records）：
  - `00831281:.normify/runs/2026-10-25-case-evidence-frame-diagnosis/unit.json` → 595 passed，at 17:29:39Z；
  - `00831281:.normify/runs/2026-10-25-g10-p5-cuda-default-pipeline/unit.json` → 595 passed，at 16:25:16Z；
  - `00831281:.normify/runs/2026-10-25-t4-identity-live-check-coverage/unit.json` → **597 passed**，at 17:35:24Z。
  → 两项基线记录均早于实现后的 597，且两个新测试非 ignored，**595→597 成立**。
- **纯 CPU**：被测两文件无真实 `#[ignore]` 属性（仅文档注释提及 `#[ignore]`）；
  无 `std::process`/`Command`/`cuda`/`nvrtc`/`tokio`/`thread`/`net` 代码引用（仅支持文件第 1 行文档写「无 GPU/NVRTC/外部依赖」）；
  依赖只有 `std::collections::BTreeSet` 与 `std::path`；测试只读仓库文件 + 自实现 SHA-256。全程未开 GPU 窗口。
- **seam 边界**：`T4_IDENTITY_LIVE_FIXTURE` 在 `src/` 中 0 命中；产品二进制（上一轮 3b7db351 构建，
  产品 `src/` 此后未变）扫描 **不含** 该字符串（`qa-static-checks.json`）。

## 4. 实现提交边界（00831281 逐文件）

`git diff --name-status 00831281~1..00831281` = **13 个文件**：
- `tests/t4_identity_live_check.rs`（72 行）、`tests/support/t4_identity_live.rs`（444 行）；
- `docs/development-evidence/t4-identity-live-check/`（README/receipt/4 份日志）；
- `.normify`（任务 JSON + 4 个 runs JSON）。

**未改**：`src/`、`Cargo.toml`、`Cargo.lock`、`.normify/gates.toml`、`scripts/`、`fixtures/`、G01 脚本、
任何既有期望值/判据/容差。`fixtures/precision/t4-current-kernels.json` 在该提交与当前工作树均无改动。
（旁证：实现片自身不 repin、不改 fixture，活体校验只做检测器——与交付声明一致。）

## 5. 与交付证据对照（不一致并列）

| 交付声明 | 我的独立结果 |
| --- | --- |
| normal 2 passed | 一致（repo fixture 与 normal-copy 各 2 passed） |
| tamper sha：exit 101，诊断含 expected/actual | 一致（并复现同一切点/同一对 sha） |
| tamper path：exit 101，诊断含 missing/unexpected | 一致（同样改 `sleep.rs`，诊断相同语义） |
| 默认 `cargo test --locked` 会执行该集成 target | 一致（`--list` 611 条含 2 条；gates.toml 无 `--lib`） |
| unit 595→597 | 一致（committed run records 595@17:29Z → 597@17:35Z） |
| 支撑文件 426 行 / 测试 64 行 | **不一致**：实际 444 行 / 72 行（提交 stat 亦为 +444/+72）；文档计数过时，非行为问题 |

## 6. 观察（非阻断）

1. **Arch 消息中的边界范围与提交不符**：消息写 `git diff c3acc6b6~2..c3acc6b6` 并称含 `00831281`，
   但该范围是 `3b7db351..c3acc6b6`（93 文件，fma 修复 + QA 证据 + 计数更正），**不含** live-check 实现提交；
   `00831281` 是其祖先（约 17 个提交前）。我按任务 JSON 的 `git diff 00831281~1..00831281` 逐文件核。
2. 交付 README/receipt 的行数计数过时（写 64/426，实际 72/444；均 <500 行，无功能影响）。
3. path 类篡改的反向判别诊断只含 path/集合信息，不含 expected/actual sha 对（集合检查先失败）；
   与交付行为一致。若验收要求「path 篡改也必须出现 expected/actual」，则需测试调整判据顺序——按当前
   任务字面（「失败且诊断含 path/expected/actual」）我判为各有针对性诊断，记录供裁定。
4. `live-check` 只是检测器，不自动 repin（漂移时失败提示走 repin 流程）——交付已如实声明，非缺陷。

## 7. 口径与边界

- 证据仅写本目录；未改产品代码/判据/容差/fixture/gates/G01；不改仓库 fixture（篡改只用 `target/` 副本）。
- 未跑 GPU/Linux、未宣称跨环境全绿；未宣称精度或物理通过；只对「身份漂移可被默认测试发现」这一覆盖主张给结论。
- 门禁：`normify gate …` 在主树 HEAD `c3acc6b6` 执行 → fmt/clippy/clippy-cuda 通过、unit **604/0/7 ignored**、
  EXIT 0；`normify check …` 全部检查通过。

## 8. 复现与显式路径

```powershell
# 独立复算（20/20 + physics_core 组合摘要）
python docs\development-evidence\t4-identity-live-check-coverage-independent-check\qa_recompute_identity.py
# 生成临时篡改副本 + 反向判别（仓库 fixture 不动）
python docs\development-evidence\t4-identity-live-check-coverage-independent-check\qa_make_tampered_fixtures.py
& docs\development-evidence\t4-identity-live-check-coverage-independent-check\run_discrimination.ps1
# 静态/边界/门禁纳入核对
python docs\development-evidence\t4-identity-live-check-coverage-independent-check\qa_static_checks.py
cargo test --locked -- --list   # 611 条，含两条 live-check
# 门禁/check（证据冻结后；stdout 写 target/，避免 gate scope 自失效）
& docs\development-evidence\t4-identity-live-check-coverage-independent-check\run_gate_check.ps1
```

证据目录 `docs/development-evidence/t4-identity-live-check-coverage-independent-check/`：
`README.md`、`receipt.json`、`artifacts-sha256.json`、`qa-identity-recompute.json`、
`qa-discrimination.json`、`qa-fixture-tamper.json`、`qa-default-test-list.json`、`qa-static-checks.json`、
`logs/`（8 份：6 个判别 case 日志 + list + default cargo test --list 抽取）、脚本
（`qa_recompute_identity.py`、`qa_make_tampered_fixtures.py`、`run_discrimination.ps1`、
`qa_static_checks.py`、`qa_extract_test_list.py`、`run_gate_check.ps1`、`qa_artifacts_sha256.py`）。
未入库：`target/qa-t4-check/**`（临时 fixture 副本与 gate/check stdout）。
