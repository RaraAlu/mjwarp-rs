# T4 身份非 ignored 活体校验覆盖（QA 复核观察 4）

- 任务：`2026-10-25-t4-identity-live-check-coverage`（Backend）
- HEAD：`7c4012823f9557a28110bd549b3268ecb70736b0`（实现与验证时）；其后 `eac9af71`
  仅新增 `.normify` 治理任务 JSON，`src/`、`tests/`、`fixtures/` 无变化
- 性质：**只新增测试**——不改任何期望值/判据/容差/排除清单/产品源码/`gates.toml`/G01 脚本；
  测试纯 CPU、默认特性可跑、不建 GPU 会话、不调用编译。
- 证据文件：本 README、`receipt.json`、`normal.log`（正常通过）、`tamper-sha.log`
  （篡改期望 sha 的反向判别）、`tamper-path.log`（篡改路径集合的反向判别）、`default-lib.log`
  （默认 `cargo test --locked --lib` 通过记录）。

## 1. 背景

repin-13 r2 的 QA 独立复核（`c4a684c8`）观察 4：`fixtures/precision/t4-current-kernels.json` 的
**20 条 `product_sources`** 中，只有 `src/physics/mod.rs`（physics_core 组合摘要）在非 `#[ignore]`
的活体校验里被覆盖；其余 19 条的 live 校验在 `#[ignore]` 的 recapture 测试内，默认门禁跑不到——
这正是身份漂移潜伏约 2 天的直接原因。本片补默认特性可执行的活体校验。

## 2. 新增内容

### 2.1 `tests/t4_identity_live_check.rs`（64 行，2 个测试）

- `live_product_sources_match_fixture`：
  1. 读取 fixture（路径可由测试专用 seam 覆盖，见 §3），断言 `product_sources` **恰好 20 条**；
  2. **集合完整性**：fixture 路径集合必须等于本测试的规范集合（20 条，`EXPECTED_PATHS`）；
     缺失/多出时给出诊断（防新增/删除源码但未重钉，或 fixture 被改路径集合）；
  3. **逐个活体比对**：每条路径重新读源码 → LF 归一化 → sha256 与 fixture 记录比较；
     `src/physics/mod.rs` 使用 **physics_core 组合摘要**口径（锚点
     `docs/development-evidence/t4-resident-anchors/physics.rs` 的 5 个 `include_str!`：
     `mod.rs` / `com_position.rs` / `device_checks.rs` / `history.rs` / `kinematics.rs`，
     逐文件 LF 归一化后以 `\n` 连接再 sha256）；
  4. 失败时输出聚合诊断：`path: expected=<64hex> actual=<64hex>`（或「无法计算活体摘要（原因）」）。
- `sha256_and_lf_normalization_match_known_vectors`：对自实现 SHA-256 与 LF 归一化做已知向量自检
  （`sha256("")`、`sha256("abc")`、CRLF 归一化），防止支撑代码本身出错导致假绿/假红。

### 2.2 `tests/support/t4_identity_live.rs`（426 行）

无外部依赖（测试自带 FIPS 180-4 SHA-256，含已知向量自检）+ 最小 JSON 提取（只取
`product_sources` 的 `path`/`sha256`）+ LF 归一化 + physics_core 组合摘要 + 规范路径集合。
**没有改 `Cargo.toml`**：`sha2` 目前是 `cuda-probe` 的 optional 依赖，而本任务文件范围不含
Cargo 清单；因此不新增 dev-dependency，测试自带摘要实现（纯 CPU、默认特性）。
两个新文件均 < 500 行（64 / 426）。

## 3. 反向判别（真跑，未改仓库文件）

测试专用 seam：环境变量 `T4_IDENTITY_LIVE_FIXTURE` 可指向 fixture 的**临时副本**；不设置时
读取仓库内 fixture（默认行为不变）。

| 判别 | 做法 | 结果 |
| --- | --- | --- |
| 篡改期望值 | 复制 fixture 到 `target/t4-identity-live-tampered-sha.json`，把
`fixtures/mjwarp-mass-solve/manifest.json` 的期望 sha 首字符 `0`→`1` | **exit 101**，
`T4 身份活体校验失败（1 条差异，共 20 条）`，诊断列出该路径 expected/actual |
| 篡改路径集合 | 临时副本把 `src/physics/sleep.rs` 改成 `src/physics/sleep_renamed.rs` | **exit 101**，
`T4 身份路径集合不完整（新增/删除源码或未重钉？）：缺少 ["src/physics/sleep.rs"]，多出 ["src/physics/sleep_renamed.rs"]` |
| 正常 | 仓库内 fixture，不设 seam | **exit 0，2 passed** |

命令原文：

```powershell
$env:T4_IDENTITY_LIVE_FIXTURE = (Resolve-Path 'target\t4-identity-live-tampered-sha.json').Path
cargo test --locked --test t4_identity_live_check -- --nocapture
Remove-Item Env:T4_IDENTITY_LIVE_FIXTURE -ErrorAction SilentlyContinue
cargo test --locked --test t4_identity_live_check -- --nocapture
```

原始输出见 `tamper-sha.log` / `tamper-path.log` / `normal.log`。判别结束后
`git status --porcelain -- fixtures/precision/t4-current-kernels.json` 为空（仓库 fixture 未被改动）。

## 4. 验证

- 默认 `cargo test --locked --lib`：**447 passed / 0 failed / 5 ignored**（lib 测试数不变；
  新增测试在独立集成 target 里，见 `default-lib.log`）。
- 新 target：`cargo test --locked --test t4_identity_live_check` → **2 passed / 0 failed**
  （默认特性、无 GPU、无编译调用）；`cargo test --locked`（gate.unit 等价命令）会一并执行该 target。
- `cargo fmt --check`、`cargo clippy --all-targets --locked -- -D warnings`、
  `cargo clippy --all-targets --locked --features cuda-probe -- -D warnings` 均 exit 0（预检）。
- `normify gate 2026-10-25-t4-identity-live-check-coverage` → fmt/clippy/clippy-cuda/unit 全绿、
  `normify check` 全部通过（结果见 `.normify/runs/<task>/`）。

## 5. 边界

- 未改任何期望值/判据/容差/排除清单/产品源码/`gates.toml`/G01 脚本；测试只读仓库文件。
- 不跑 GPU/Linux；不建 NVRTC 会话；不调用编译。
- 活体校验是「源码内容 vs fixture 记录」的**检测器**，不自动重钉：漂移时必须走 repin 流程更新
  fixture 期望（本片未改任何期望值）。
- `T4_IDENTITY_LIVE_FIXTURE` 是测试专用 seam，默认未设置时行为与仓库 fixture 完全一致；
  记录在测试注释中，不影响产品/门禁环境。
- 集合完整性以测试内规范路径集合为基准：源码新增/删除时应同步更新该集合与 fixture（走 repin），
  否则测试会失败提示复核。

## 6. 复现

```powershell
cargo test --locked --lib                                  # 447 passed / 5 ignored
cargo test --locked --test t4_identity_live_check -- --nocapture   # 2 passed
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --all-targets --locked --features cuda-probe -- -D warnings
normify gate 2026-10-25-t4-identity-live-check-coverage
normify check 2026-10-25-t4-identity-live-check-coverage
```
