# T4 源码身份重钉（第 13 轮 r2）· 证据

- 任务：`2026-10-25-t4-source-hash-repin-13-r2`（Backend）
- HEAD：`4ba11df55a65949f4db9f61bd2d20f9a42a9d19f`（验证主体）；其后 Arch 治理提交
  `da12cc96`、`b47dcd8a`、`c0b4ab64` 仅新增/修正 `.normify` 任务 JSON，不涉及代码/构建
- 性质：**只改身份期望字面量**（fixtures / docs evidence map / gravity ABI 包 / tests 内联常量）；
  不改断言逻辑、容差、排除清单、产品源码、内核 PTX、G01 脚本。未跑 release/Linux。
- 证据文件：`README.md`（本文件）、`drift.json`、`all-features.log`（重钉前）、
  `all-features-after-repin.log`、`all-features-final.log`（本机重钉后两次）、
  `rerun-after-repin.log`（首轮定向复跑，中间产物）、`rerun2-after-repin.log`（定向复跑，最终）、
  `case-evidence-clean-build.log`（环境差异证据）、`diff-stat.txt`。

## 1. 身份重钉正确性

### 1.1 `fixtures/precision/t4-current-kernels.json` product_sources：9/20 漂移（旧→新，完整哈希）

| path | 旧值 | 新值 |
| --- | --- | --- |
| `src/physics/attached.rs` | `9073a9675298ffe734fbe48de51169a440d816ef7cbb433e41fe1d04f1de9197` | `65902db9700a34f5ac8a95b0ed8465b928c5b8b107b2848c7a1ec5b53c7d68fb` |
| `src/physics/com_velocity.rs` | `907158eac24f45657675301e3169b79fca262615e7cedc5318f5dd44effb0988` | `387a8e1a26c95181421340cc9c03b8821cfbd9262bcbfccefe4db1913b59f5eb` |
| `src/physics/mass_matrix.rs` | `6d721178ecdcf27c2985a25443ba336a7c6b501cb6a1dd4f994d64f8fc6b80da` | `0cc96256d59113b8d603771e84e8b74032771fbade0cb658fc599756cbf9c87b` |
| `src/physics/mass_solve.rs` | `dc1090a7b0cfc63b6ee727c394ca2ac0392a31e1c671896a0561307eb2f541ad` | `79c5dda3e23ed3dc2b6b5648fb95ec74d9c8772bab3c992bfbf3ac53b35ae6c6` |
| `src/physics/mod.rs`（**physics_core 组合摘要**） | `d7f6c7d19a17307d281f6e43c9866d0da67bfcbe9af4af3bad305d77ee3a187c` | `33a6e55f8f758528db9c21d82ff639ce794ce80408c9271f4394f3667036e546` |
| `src/physics/resident.rs` | `b2d63abc3763c38435ab8bb76c866fb307f7e144f1db712ae0abc46d1c7fd9cf` | `236a6813054a4d406a14fdfa2daf5d4805cffb2b1bb337d16be586753713368a` |
| `src/physics/rne_bias.rs` | `5b5b13caf0e4e5c3891ecf39e7356806db85a3a3867535418f31ac6ceadb1b45` | `47d362db17fa930e43201485b1e0fd58f519982d5ddceb0927473db377962c0f` |
| `src/runtime/transfer.rs` | `268c351cb4a7048cef7ceb870c8e4a7f1a12633dc474d75065b356f8988e190f` | `3996312041dcf9412203dde3bdcb9531f4f1839c616a6691c95a72b0038796e2` |
| `src/runtime/transfer/kernel.rs` | `ae5ea7092fa32642486e1799e950935da874b9f913334d0d59837a357428af11` | `86f7803ab3c45af15ec59eae9dee95ea9520b2ec3aba2495c419f1b316d7b6b0` |

其余 11 条未变。算法：单文件 = `sha256(LF 归一化文件字节)`；`src/physics/mod.rs` 条目是
**physics_core 组合摘要**（见 1.2），不是单文件摘要。

### 1.2 `physics_core` 三对象辨析（避免误读）

| 对象 | 值 | 说明 |
| --- | --- | --- |
| fixture 旧记录 | `d7f6c7d1…` | repin-12 时的 physics_core 组合摘要（已过期） |
| 锚点文件自身 | `b4bb7ed9…` | `docs/development-evidence/t4-resident-anchors/physics.rs` 这个**文件**的 sha256（不是组合值） |
| live `src/physics/mod.rs` 单文件 | `f840ddfe…` | 单文件 LF 摘要（fixture 的该条目**不用**它） |
| **新组合摘要** | **`33a6e55f8f758528db9c21d82ff639ce794ce80408c9271f4394f3667036e546`** | `sha256(LF(concat(mod.rs,\n,com_position.rs,\n,device_checks.rs,\n,history.rs,\n,kinematics.rs)))`，与锚点 `concat!` 口径、`tests/support/t4_rigid_capture.rs` 的 `digest(normalized(PHYSICS))` 一致 |

组合摘要传播点（全部已重钉到 `33a6e55f…`）：`fixtures/precision/g01-{field-internal,rigid-internal,state-packing,topology-frames}.json`、
`docs/evidence/t4-g03-retest/{camlight-internal,tendon-wake-internal}/map.json`、
`fixtures/gravity-upstream-abi/{three-d,nonzero-product-chain}-package.json`、
`tests/t4_gravity_nonzero_product_chain.rs` 内联期望。

### 1.3 其他组合/单文件身份

| 对象 | 旧值 | 新值 | 口径 |
| --- | --- | --- | --- |
| resident 组合（`resident-source-anchor.rs` 41 文件） | `3c4de13b…` | **不变** `3c4de13b…` | 锚点 `concat!` 组合摘要；本次未新增 resident 子模块 |
| transfer 组合（`transfer.rs` + `buffer.rs` + `session.rs`） | `f6980bd1f248ed178d9044dae9313b1401a4b55e7380822a14a449f9fbee734d` | `3e5c23b79995857fcc3c7ff3d0bb67aaaf05f187a7a26bd37226c8e0a44f81ea` | g01-rigid/state、flex-internal/flex-matrices map |
| model flex 组合（model/flex.rs + edge/face/input/position） | `b3603225…` | **不变** `b3603225…` | flex-internal/flex-matrices map |
| flex_face **historical** | `47a17fc2f053e93afaaab408b2c185f0bbef0cce6687deb4120abc94d478c1e6` | **不变**（`face_lineage::historical` 口径） | flex-matrices map |
| `include/mjwarp_native_probe.h` | `094d1ac1…`（旧） | `03151bf5340c535a5c8139cea6d44ab06b1138ed731c61aa0b87ee48f813a5de` | g01-inputs（LF 归一化） |
| `native/g01_fields.inc` | `d4f1fdfe…` | `5568a39ab7bbec40680d0344740397eb9fc22b3aac13ef3ed33fc46abdd8657c` | g01-inputs |
| `native/model_probe.cpp` | `8e0fab9b…` | `6a01eb01785171d2d03bf243b2b453bbdc001c433c6e77e9621499b363dd57bc` | g01-inputs |
| `docs/evidence/t4-g03-retest/rigid-rotate-recapture/previous-kernels.json` | 9 条旧 product_sources | 与 `t4-current-kernels.json` 对齐（同 1.1） | lineage 期望基线；**未重生成历史捕获产物**，只同步身份记录 |

### 1.4 `inventory_sha256` 重算口径

`fixtures/precision/t4-current-capture-status.json` 的 `inventory_sha256` = **sha256(原始文件字节)**
（已用重钉前状态反验：旧记录 `8a4b6e1f…` == 旧 fixture 的原始 sha256，算法确认）：

- `inventory_sha256`：`8a4b6e1f5c4403e8230c5bc105cf42126822b5b05154f0bc98d9d1457c6b2655`
  → **`61bddcd023bdbc733102c256edd3c206e13ea8c683eef6c821e215804c1b98e3`**
- `previous_inventory_sha256`：旧 `eb16a66b…` → 旧 inventory 值 `8a4b6e1f…`（新增一层历史）

### 1.5 gravity ABI 两包（scope 已扩到 `fixtures/`）

`fixtures/gravity-upstream-abi/{three-d,nonzero-product-chain}-package.json` 的 `product_sources`：
`physics_core` = `33a6e55f…`（组合摘要，测试用 `digest(PHYSICS.replace("\r\n","\n"))`）；
`resident` = `3c4de13b…`（resident 锚点组合）；`gravity_compensation` = `6135a4f2…`（文件原始字节，未变）。
包文件变更后 `PACKAGE_SHA256` 常量同步：
`tests/t4_gravity_3d_product_chain.rs` = `708ced7c46624b86f10a46dff45a66d8ffdfe289cb989b63920d55f3116dc795`；
`tests/t4_gravity_nonzero_product_chain.rs` = `96bd92afe05b482630cff7ffcf3eda9626a29e8b9bff01bcf70f17bc2f0af79b`。

### 1.6 “只改字面量”的 diff 边界自证

`git diff --shortstat`：**18 files changed, 47 insertions(+), 36 deletions(-)**，= **17 个内容文件**
+ `.normify/changes/2026-10-25-t4-source-hash-repin-13-r2.json`（Arch 的 start 记录，非内容改动）
（`git diff` 逐行核对，见 `diff-stat.txt`）：

- `fixtures/precision/` **7 个**：`t4-current-kernels.json`、`t4-current-capture-status.json`、
  `g01-{field-internal,rigid-internal,state-packing,topology-frames}.json`、`g01-inputs.json`
- `docs/evidence/t4-g03-retest/` **5 个**：4 个 map（camlight/tendon-wake/flex-internal/flex-matrices）+
  `rigid-rotate-recapture/previous-kernels.json`（仅身份记录）
- `fixtures/gravity-upstream-abi/` **2 个**包
- `tests/` **3 个**：`t4_gravity_3d_product_chain.rs` / `t4_gravity_nonzero_product_chain.rs` /
  `tests/support/t4_rigid_internal/tests.rs`（旋转谱系期望 `a9d1b4d6…`→`3fa0046f…`，均为**常量字面量**）

注：gravity 两包与 `previous-kernels.json` 在 `.gitattributes` 标了 `-text -diff`，`git diff --stat`
显示为 `Bin 23354 -> 23354 bytes` 属**字节级哈希字面量替换**（长度不变），无格式/内容重排。

**未改**：断言表达式/容差/排除清单、`src/**` 产品源码、内核 PTX、G01 脚本与 unchanged 保护路径、
历史捕获产物（仅 previous-kernels 的身份记录同步，不重生成）。

## 2. 仓库默认环境验收（r2 验收依据，Arch 复核）

Arch 在**仓库默认环境**下的独立复现：干净 target `target/arch-verify-clean`、HEAD `da12cc96`、
**不带 G01 环境变量**。以下三条作为 r2 的验收依据：

| # | 命令 | 结果 |
| --- | --- | --- |
| 1 | `cargo test --locked --no-fail-fast`（默认全量，gate.unit 等价命令） | **exit 0，全 target 0 失败** |
| 2 | `cargo test --locked --features cuda-probe --lib --no-fail-fast`（干净 target） | **678 passed / 0 failed / 137 ignored，exit 0** |
| 3 | `cargo test --locked --lib physics::collision::gjk::case_evidence::upstream_f32_within_4ulp -- --exact` | **1 passed / 0 failed** |

门禁（Arch 侧，强制非缓存）：`normify gate 2026-10-25-t4-source-hash-repin-13-r2 --retry` →
fmt / clippy / clippy-cuda 通过、**unit 595 passed / 0 failed（22.5 s 真跑）**、**exit 0**。

Backend 侧用**同一默认环境**（新建 `target/verify-clean-002`，删空重建）复核第 3 条：
`exit 0，1 passed / 0 failed`（详见 §3.1）。→ 仓库默认环境下 r2 验收通过。

### 2.1 默认门禁为何没抓到身份漂移（记录）

`normify gate` 的 unit 只跑**默认特性** `cargo test`；T4 recapture/identity 测试位于
`#[cfg(feature = "cuda-probe")]` 的模块（默认构建不编译，`--features cuda-probe --lib` 为 815 tests），
因此身份漂移在默认门禁下不可见，潜伏约 2 天（上次重钉 `e4f74816` 之后）。

## 3. 环境差异（构建工具链/target；数值机制归诊断片）

同一棵树、两套**全新 target**、同一条命令，结果相反；差异来源已定位到**构建工具链/target**：

| 环境 | 工具链 / rustc 关键差异 | 该测试结果 |
| --- | --- | --- |
| **仓库默认**（`rust-toolchain.toml` → active GNU；Arch `target/arch-verify-clean`、Backend `target/verify-clean-002`） | `1.99.0-x86_64-pc-windows-gnu` + `-C link-dead-code=yes`（`.cargo/config.toml` 的 gnu rustflags） | **1 passed / 0 failed**；cuda-probe lib 678/0/137；默认全量 exit 0（§2） |
| Backend 显式 MSVC（`target/verify-clean-002-msvc`，全新目录） | `+1.99.0-x86_64-pc-windows-msvc`，无 `link-dead-code` | **FAILED — frame `6.351978e-7`**；cuda-probe lib 677/1/137；all-features lib 693/1/137；默认 lib 446/1/5 |

### 3.1 最小复现协议结果（Arch 指定，当前树 HEAD `b47dcd8a`，`git status --porcelain -- src` 为空）

- `CARGO_TARGET_DIR=target/verify-clean-002`（删空重建）+
  `cargo test --locked --lib physics::collision::gjk::case_evidence::upstream_f32_within_4ulp -- --exact --nocapture`
  → **exit 0，`1 passed / 0 failed`**（从零构建 13.35s）。
- 环境：`rustc -vV` host `x86_64-pc-windows-gnu`（LLVM 23.1.1）；`cargo -V` 1.99.0；除协议显式设置的
  `CARGO_TARGET_DIR` 外**无任何 `CARGO*/RUST*` 环境变量**；`.rustc_info.json` host = gnu。
- 同树、同命令、全新目录 `verify-clean-002-msvc` + 显式 `+1.99.0-x86_64-pc-windows-msvc` →
  **exit 101，FAILED，panic `ellipsoid-box frame 6.351978e-7`** → 红不是 `verify-clean-001` 的旧产物。
- 两条 rustc 命令行（`--crate-name mjwarp_rs`）差异：rustc 路径（gnu vs msvc 工具链）+
  **GNU 独有 `-C link-dead-code=yes`**；其余相同（edition 2024、`-C embed-bitcode=no`、`-C debuginfo=2`、
  `--test`、`--cfg "feature=\"default\""`、无 `--target`、无显式 `-C opt-level`）。
- 二进制：GNU `54,607,221 B` vs MSVC `10,636,288 B`（对应 `link-dead-code`），与 Arch 观察的
  54.6 MB vs 10.6 MB 一致；debug 构建**不保证位级可复现**（同 metadata/同体积、sha 不同）→
  sha 不能当确定性判据。

### 3.2 Arch 侧已查证项（引述）

1. 该路径唯一环境变量是 `G2_DUMP_NATIVE_ERROR`（dump 开关，不参与断言）；干净二进制 + G01 env
   重跑仍 pass → **环境变量不是原因**；
2. 两次全新 debug 构建：同名同体积 `54,607,221 B`、sha 不同（`fe6512ab…` / `7372395d…`）
   → debug 构建不可位级复现，**不能拿 sha 当确定性判据**；
3. `--release` 同测试 → **1 passed**，排除优化导致的 frame 超限；
4. `verify-clean-001/debug/deps` 有 1557 个文件（全新目录 295）→ 是**多配置叠加产物**
   （Backend 会话中交替使用 GNU 与 MSVC 工具链写同一 `target`），来源不可靠、不作证据源。

### 3.3 本机 MSVC 构建的红日志（对照，数字保留）

| 命令（MSVC 构建） | exit | 结果 |
| --- | --- | --- |
| `--all-features --no-fail-fast`（`all-features-final.log`） | 101 | lib **693 passed / 1 failed / 137 ignored**；其余 target 0 失败 |
| `--features cuda-probe --lib` | 101 | **677 passed / 1 failed / 137 ignored** |
| 默认 `cargo test --locked --lib` | 101 | **446 passed / 1 failed / 5 ignored** |

本机 `--all-features` **可执行**（`all-features.log` 有 `Compiling cubecl-llvm` / `Finished test profile
in 1m 59s` / 测试已运行）——`CARGO_EXIT=101` 来自测试失败，不是构建失败。本机
`Get-Command dlltool` = 未找到、PATH 无 mingw/msys，**未复现** QA/Arch 环境报的
`error calling dlltool 'dlltool.exe': program not found`（如实记录为环境差异）。

**重钉前后对照**（本机 MSVC 日志）：重钉前 `all-features.log` lib 有 **2** 个失败
（`case_evidence::upstream_f32_within_4ulp` + `recapture::tests_host::preserves_current_sources_and_all_thirty_variants`）；
重钉后只剩 1 个（`case_evidence`）——**lineage/身份失败 18 → 0**；身份相关 target 在 MSVC 下亦全绿
（`t4_camlight_precision` 3/0、`t4_gravity_3d` 1+1ignored、`t4_gravity_nonzero` 1+1ignored、
`t4_input_precision` **36 passed/12 ignored/0 failed**、`t4_tendon_wake` 4/0）。

该测试 4 个关键输入与 HEAD **逐字节一致**（`git hash-object` == `git rev-parse HEAD:path`：
`case_evidence.rs` `a4100273…`、`case_compare.rs` `5b0cd05c…`、`case_data.rs` `b3b95951…`、
`fixtures/g10-p5/references/ellipsoid-box.json` `a3223a77…`），排除字面量/数据差异。

### 3.4 判定与处置

- **直接来源**已定位：构建工具链/target 差异（默认 GNU + `link-dead-code` vs 显式 MSVC）；
  **数值机制**（target ABI 或 `link-dead-code` 何者导致 frame 超限）**未定**，
  归独立诊断片 `2026-10-25-case-evidence-frame-diagnosis`（口径「构建环境敏感性」，`b47dcd8a`）。
- 本片未改断言/容差/排除清单；**不宣称跨环境全量绿**；r2 验收以 §2 仓库默认环境为准。
- **更正**（撤销此前的两处错误说法）：①「Arch 的 678/0 来自主 target 过期二进制」不成立
  （Arch 用全新 `arch-verify-clean`；§3.1 显示红来自 MSVC 构建）；②「时钟/指纹错乱导致复用旧
  二进制」不成立（sha 不同是 debug 构建不可位级复现，不是缓存问题）。差异见 `case-evidence-clean-build.log`。

## 4. 复现

```powershell
# 仓库默认环境（§2 验收口径；不带 +toolchain，默认即 GNU + link-dead-code）
cargo test --locked --no-fail-fast
cargo test --locked --features cuda-probe --lib --no-fail-fast
cargo test --locked --lib `
  physics::collision::gjk::case_evidence::upstream_f32_within_4ulp -- --exact --nocapture

# 本机对照：显式 MSVC 构建（§3.3 红日志来源）
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --lib `
  physics::collision::gjk::case_evidence::upstream_f32_within_4ulp -- --exact

# 本机 --all-features（§3.3；需 G01 环境）
$env:MJWARP_MUJOCO_ROOT = (Resolve-Path 'target/toolchains/mujoco-3.12.0/package').Path
$env:MJWARP_MUJOCO_DLL = Join-Path $env:MJWARP_MUJOCO_ROOT 'bin/mujoco.dll'
$env:CUDA_PATH = (Resolve-Path 'target/toolchains/cuda-12.8.1').Path
$env:PATH = (Join-Path $env:CUDA_PATH 'bin') + ';' + $env:PATH
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --no-fail-fast
```

## 5. 边界

- release 与 Linux 未跑（如实登记）。
- 历史捕获产物未重生成；仅 `previous-kernels.json` 的 product_sources 身份记录随 lineage 期望对齐。
- 未改任何数值语义/判据/产品路径；不宣称 T4 全量准入或精度结论。
- r2 验收 = 仓库默认环境（GNU）结果（§2）；显式 MSVC 构建的红数字保留作对照（§3.3）。
  **不宣称跨环境全量绿**；构建环境敏感性（数值机制）归诊断片。
