# T4 源码身份重钉（第 13 轮 r2）· QA 独立复核

- 复核任务：`2026-10-25-t4-source-hash-repin-13-independent-check`（QA）
- 被复核任务：`2026-10-25-t4-source-hash-repin-13-r2`（Backend）
- 复核时 HEAD：`eddad9c024dab798bcaac41782d3cfdc9f565e91`（内容提交 `61fb6baf`，其后 `eddad9c0` 仅证据注记）
- 结论：**pass（阻断 0 / 发现 0 / 观察 7）**；所有数字来自本目录脚本的独立复算与自建 target 的独立复跑，未引用交付脚本或交付日志作为自己的证据。
- 证据入口：本 README、`receipt.json`、`qa-identity-enum.json`、`qa-diff-boundary.json`、`qa-env-matrix.json` 及三个复算脚本。

## 1. 独立身份枚举（不读交付结论）

自写 `qa_identity_enum.py`（未导入交付脚本），算法逐条对齐 Rust 测试源码（`tests/support/t4_evidence/sources.rs`
的 `verify_sources`、`tests/support/t4_rigid_capture*` 的 `digest(normalized(..))`、`lineage()`）：

- 单文件：`sha256(LF(文件字节))`；
- `src/physics/mod.rs`：**physics_core 组合摘要** = `sha256(LF(concat(锚点 include_str! 列表, 以 \n 连接)))`，
  锚点 `docs/development-evidence/t4-resident-anchors/physics.rs` 的 include_str! 列表由脚本自行解析
  = `mod.rs, com_position.rs, device_checks.rs, history.rs, kinematics.rs`；
- 组合摘要其它项：resident（`resident-source-anchor.rs` 的 include_str! 列表）、transfer（三文件以 `\n` 连接）。

### 1.1 结果：20/20 命中，漂移恰好 9 条

- `fixtures/precision/t4-current-kernels.json` 的 **20 条 `product_sources` 全部**等于本次独立计算值（20/20）。
- 与父提交（`61fb6baf~1`）的同一 fixture 逐条比较：**恰好 9 条变化**，路径集合与交付声明完全一致
  （attached / com_velocity / mass_matrix / mass_solve / mod.rs / resident / rne_bias /
  `src/runtime/transfer.rs` / `src/runtime/transfer/kernel.rs`）；其余 11 条未变。
- 交付 §1.1 的 9 行「旧→新」逐条核对：**9/9 旧值 = 父提交 fixture 值，9/9 新值 = 本次独立计算值**。

### 1.2 physics_core 三对象（fixture 记录值 / 锚点文件 / live 单文件）

| 对象 | 值 | 与交付描述 |
| --- | --- | --- |
| fixture 记录（=组合摘要） | `33a6e55f8f758528db9c21d82ff639ce794ce80408c9271f4394f3667036e546` | 一致 |
| 锚点文件自身 sha256（raw 与 LF 相同） | `b4bb7ed91de02622dee146c6fb47b42a3dd4578f9a8eaf8fd0222b16d6079053` | 一致（非组合值） |
| live `src/physics/mod.rs` 单文件 LF 摘要 | `f840ddfe7a8741f8900504d0776eb53f7298bfae2d5387298a135e9a95c632f9` | 一致（该条不使用单文件值） |

三对象互不相同（3 distinct），交付 §1.2 的辨析与实测一致；组合摘要由锚点 define 的 include_str! 列表独立重算命中。

### 1.3 其它身份与传播点（抽核）

| 对象 | 独立计算 | 交付声明 |
| --- | --- | --- |
| transfer 组合（transfer.rs+buffer.rs+session.rs，`\n` 连接） | `3e5c23b79995857fcc3c7ff3d0bb67aaaf05f187a7a26bd37226c8e0a44f81ea` | 一致 |
| resident 组合（锚点 include_str! 列表） | `3c4de13bdce432a18fc0e76e3162e8cb2b433a7e9919efccfea4a8d4ebb588c6`（未变） | 一致 |
| `fixtures/precision/g01-inputs.json` 的 18 条 product_sources（含 3 个 native 文件） | **18/18** | 一致 |
| three-d / nonzero 两包 `PACKAGE_SHA256`（测试常量） | `708ced7c…dc795` / `96bd92af…f79b`，均 = `sha256(包文件原始字节)` | 一致 |
| 两包 `product_sources` 字段 | physics_core=`33a6e55f…`、resident=`3c4de13b…`、gravity_compensation=`sha256(原始文件字节)` 全部命中 | 一致 |
| `inventory_sha256` | 新 `61bddcd0…98e3` = `sha256(当前 fixture 原始字节)`（该 checkout 无 CRLF，raw 与 LF 同）；previous `8a4b6e1f…` = `sha256(父提交 fixture blob)` | 一致 |

## 2. 边界复核：只改身份期望字面量

`qa_diff_boundary.py` 对内容提交 `61fb6baf~1..61fb6baf` 全量逐文件核对（34 个文件）：

- 组成：**17 个内容文件**（与交付清单逐项一致）+ 10 个证据文件新增 + 4 个 gate 运行记录新增
  + 2 个 `.normify` 模块指纹 + 1 个任务记录（M）。
- 禁止面：`src/`、`native/`、`include/`、`scripts/`、`build.rs`、`Cargo.toml`、`Cargo.lock`、`.cargo/` **零改动**；
  范围外路径 0 个；新增文件全部落在证据目录或 `.normify/runs/`。
- 16 个被改文本文件：把 64/40 位 hex 掩码后逐行相同（**全部为身份字面量替换**）；内容文件合计 **28+/28-**
  （逐文件行数与交付 `diff-stat.txt` 完全一致）。
- 3 个 `-text` JSON（`previous-kernels.json`、两个 gravity 包）：字节长度不变，结构差异**仅**
  `product_sources[].sha256` / `product_sources.physics_core.sha256`（9 + 1 + 1 处）。
- 任务记录（`.normify/changes/...r2.json`）结构差异仅 `status`、`revision.before`、`updated_at`、`closed_at`
  —— acceptence / intent / scope 文本未变（无口径放宽）。
- 断言/容差/排除清单/G01 脚本：`TOLERANCE_BITS` 等未改；3 个 tests 文件仅改内联常量
  （旋转谱系 digest、两个 `PACKAGE_SHA256`、physic_core 期望值），断言表达式未动。
- `diff-stat.txt` 聚合值 `18 files / 47+ / 36-` 是当时（HEAD `da12cc96` + 未提交改动，含任务记录中间态）采集，
  不能从最终提交精确复现；**内容 17 文件的集合与逐文件行数可复现**，边界结论不变（见观察 2）。

## 3. 构建环境敏感性（自建干净 target，GNU 与 MSVC 并列）

按「干净构建判定标准」自建**专用 target**（`target/verify-clean-qa`、`-cuda`、`-msvc`、`-msvc-all`，以及
可执行性探针用 `-gnu-all`、`-gnu-native`），**未复用 Backend 的 `verify-clean-001/002`，未把其日志当本复核证据**。记录：

- 清空 `CARGO_*`/`RUST*`：清空前 0 项、清空后 0 项、实际移除 0 项（`env-before.txt` / `env-after-clear.txt`）；
- 仓库默认工具链 `rustc -vV`：`1.99.0 (b940084d7)`，host **`x86_64-pc-windows-gnu`**，LLVM 23.1.1；
  显式 `+1.99.0-x86_64-pc-windows-msvc` 同版本、host **msvc**；`cargo 1.99.0`；
- 各目录 `debug/deps` 文件数：默认全量 281（从 0 全新构建）、cuda lib 74、MSVC lib 86；target 体积 1763.6 / 701.3 / 758.8 MiB。

| # | 环境 | 命令 | 退出码 | 结果 |
| --- | --- | --- | --- | --- |
| 1 | GNU（仓库默认） | `cargo test --locked --no-fail-fast` | **0** | 全部 target 0 失败；lib **447 passed / 0 failed / 5 ignored** |
| 2 | GNU | `cargo test --locked --lib …upstream_f32_within_4ulp -- --exact` | **0** | **1 passed / 0 failed**（最小复现：默认环境不红） |
| 3 | GNU | `cargo test --locked --features cuda-probe --lib` | **0** | **678 passed / 0 failed / 137 ignored**（含谱系测试 `preserves_current_sources_and_all_thirty_variants` **ok**） |
| 4 | MSVC | `cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features native-model-probe,cuda-probe --lib` | **101** | **689 passed / 1 failed / 137 ignored**；唯一失败 `physics::collision::gjk::case_evidence::upstream_f32_within_4ulp`，`ellipsoid-box frame 6.351978e-7` |
| 5 | GNU | `--features cuda-probe` 跑交付枚举出的 5 个集成 target | **0** | 3/0、1+1ign、1+1ign、**36/0/12ign**、4/0，全绿 |
| 6 | MSVC | 同 #5（`native-model-probe,cuda-probe`） | **0** | 与 #5 同计数，全绿 |
| 7 | MSVC | `cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --no-fail-fast` | **101** | lib **693 passed / 1 failed / 137 ignored**；110 个测试结果行合计 **997 passed / 1 failed / 422 ignored**，唯一失败同上（`frame 6.351978e-7`）；构建 2m32s，wall 171.1 s |

结论与交付一致并**并列**：仓库默认（GNU）口径全绿；显式 MSVC 口径下 `case_evidence` 红（`frame 6.351978e-7`），
属**既有问题、非本片重钉造成的失败**，也不因 GNU gate 绿而消失。枚举出的 6 个身份失败 target（1 lib + 5 集成）
在 GNU 与 MSVC 两侧均 **0 失败**——「身份失败 18→0」的失败集已独立覆盖。

补充事实（本次实测）：

- GNU 默认特性下，5 个身份集成 target 仍会被编译运行但 **0 用例**（测试被 cuda-probe 门控），
  因此「默认全量 exit 0」不能当作身份证据——与交付 §2.1 的盲区结论一致（见观察 3）。
- GNU 口径下 `--all-features` 不可执行，本次实测两种独立原因：
  ① `cargo test --locked --all-features --no-run` → exit 101，
  `error calling dlltool 'dlltool.exe': program not found`（`getrandom` / `windows-sys` 编译失败，本机无 dlltool）；
  ② 单独 `--features native-model-probe` → `build.rs:12` panic：`native-model-probe requires Windows MSVC or Linux GNU`。
  故「可执行最大集合」两口径不同：GNU = default（+cuda-probe），MSVC = all-features（见观察 6）。

## 4. 门禁新鲜度

- 交付引用的 `61fb6baf` 门禁记录：fmt `17:13:56Z` wall 1515 ms、clippy `17:13:48Z` wall 2676 ms、
  clippy-cuda `17:13:53Z` wall 4844 ms、**unit `17:14:21Z` wall 24801 ms / 595 passed / 0 failed / 7 ignored**；
  提交时间 `17:14:49Z`（紧邻记录之后）。clippy-cuda 4.8 s 与 unit 24.8 s 说明是**真跑**（非缓存记录）。
- `eddad9c0`（注记提交后）重跑记录：unit `17:16:06Z` wall 3476 ms / **595/0/7**（测试确实执行）；
  clippy 405 ms、clippy-cuda 354 ms、fmt 1463 ms —— 产品源码自 `61fb6baf` 未变，clippy 属 cargo 缓存命中
  （结论不变，但不是全新分析），作为「门禁仍绿」的证据需带此限定（见观察 7）。
- 本复核自跑门禁 `normify gate 2026-10-25-t4-source-hash-repin-13-independent-check`（本任务首次运行、无缓存记录）：
  fmt / clippy / clippy-cuda 全部通过、**unit 595 passed / 0 failed / 7 ignored**，EXIT 0；
  各门禁的时间戳/耗时记录在 `.normify/runs/2026-10-25-t4-source-hash-repin-13-independent-check/`。
  另在自建干净 target 上跑默认全量（#1 命令，`--no-fail-fast`）exit 0，合计计数同为 **595/0/7**。

## 5. 口径核对（含 `eddad9c0` 加注）

- **GNU 限定**：README §2 标题已标注「GNU 口径」，§6 明确「本片所有门禁绿均为 GNU 口径」；
  receipt 新增 `env_scope_note.gate_and_acceptance_scope`、`boundaries.env_scope`，`gate.env` 也写明默认工具链；
  `drift.json` 的 `repo_default_env_acceptance` 与 claim 均把验收绑定在「仓库默认环境」。逐处核对未见
  「门禁绿」被写成 G01 绿或跨环境绿；§2.1 单独说明默认门禁盲区。
- **MSVC 红如实保留**：README §3.3 / §6、receipt `env_scope_note.g01_host_msvc_status` 均写明
  `+msvc --features native-model-probe,cuda-probe --lib` = 689/1/137、`frame 6.351978e-7`，
  并明确定性「既有问题、非重钉失败、不因 GNU gate 绿消失」；本次复跑数字与之一致，未发现把环境差异
  写成重钉成果、或把既有红算成重钉失败。
- **未发现**把「身份重钉正确性」与「全量可执行性」混写：§1/§2 与 §3 分节陈述，drift.json 用
  `pre_repin/post_repin` 与 `executability` 两块分开。
- 证据未使用宣传式结论表述（整体实现、跨平台、完全等价、放行、数值精度结论等）；
  边界章节明确未跑 release/Linux、未重生成历史捕获。

## 6. 观察（均非阻断，编号对应上文）

1. **`head`/`head_note` 记录口径**：交付 README「HEAD：`4ba11df5`（验证主体）」且 head_note 只列其后的
   `da12cc96`/`b47dcd8a`/`c0b4ab64` 三个 .normify 治理提交，未提**真正的身份字面量内容提交 `61fb6baf`**。
   本次已验证内容级等价：`git diff 4ba11df5..61fb6baf -- src native include scripts Cargo.toml Cargo.lock` 为空、
   17 个内容文件集合与交付清单一致、身份枚举以 HEAD 树计算 20/20 —— 结论不受影响。建议后续文档批次把
   `head` 补成最终内容提交并注明 `diff-stat.txt` 的采集时点（`da12cc96` + 未提交改动）。
2. **`diff-stat.txt` 聚合值不可精确复现**：`18 files / 47+ / 36-` 含任务记录当时的中间态；
   内容 17 文件 `28+/28-` 与交付逐文件行数完全一致，边界结论成立，属证据采集时点差异。
3. **默认全量不是身份证据**：GNU 默认特性下身份集成 target 被编译但 0 用例（cuda-probe 门控）；
   交付 §2.1 的「默认门禁抓不到」结论正确，建议补一句「target 在、用例 0」使其更明确。
4. **20 条 product_sources 的活体校验覆盖不均**：非 ignored 测试中只有 `mod.rs` 由谱系测试对 live 组合摘要校验，
   其余 19 条的 live 校验在 `#[ignore]` 的 recapture 测试内；本次独立枚举 20/20 命中，但建议后续补一个
   非 ignored 的 live 校验，避免同类漂移再次潜伏。
5. **旧 hash 仍存在于历史/冻结证据**（`frozen-inventory`、`layers-m3` frozen registry、其它片 evidence map、
   `fixtures/precision/t4-kernels.json` 基线、`face-current-compile`/`face-operators` 历史捕获）——按设计保留为
   历史记录；本次确证它们不是 live 身份期望（非 ignored 身份 target 两侧全绿 + live 枚举 20/20），无需清理。
6. **「可执行最大集合」随口径不同，且 dlltool 归因可更精确**：交付 §3.3「`--all-features` 可执行」仅指 MSVC
   （本次 MSVC all-features 复跑成立：997/1/422），GNU 口径下 `--all-features` 实测 exit 101 ——
   先因 `getrandom`/`windows-sys` 缺 `dlltool` 失败，且即使有 dlltool 也会被 `build.rs:12` 的
   「native-model-probe 仅 Windows MSVC」断言挡住。交付 `dlltool_note` 把它称为「环境差异」，
   更准确的归因是**工具链差异**（GNU 目标需要 dlltool，MSVC 不需要）；不影响验收口径（验收只覆盖
   GNU default+cuda-probe），建议后续措辞对齐。
7. **门禁复跑的缓存限定**：最终树（`eddad9c0`）门禁记录中 clippy/fmt 为 cargo 缓存命中（产品源码未变），
   unit 真跑 595/0；不影响「门禁绿」结论，但引用时应带「GNU 口径 + 代码未变」限定。

## 7. 覆盖与未覆盖

已覆盖：20 条 product_sources 独立枚举与 9 条漂移旧→新；physics_core 三对象；transfer/resident 组合；
g01-inputs 18 条；两个 gravity 包身份与 PACKAGE 常量；inventory 新旧值；内容提交边界逐文件；
GNU 默认全量、GNU cuda-probe lib、MSVC native+cuda lib、身份集成 target 两侧、MSVC all-features（见 §3 #7）；
门禁记录新鲜度与本复核自有门禁。

未覆盖（如实登记）：MSVC `--all-features` 之外的其它特性组合未逐一穷举；Linux 构建未跑；
release 未跑；`case_evidence` 超限的**数值机制**未定（归 `2026-10-25-case-evidence-frame-diagnosis`，本复核只确认现象与工具链相关性）；
历史捕获未重生成（按交付边界）；`--all-features` 下单个 ignored 用例未逐个解释（按 libtest 汇总口径）。

## 8. 复现与显式路径

```powershell
# 本目录脚本（均以 repo 根为 cwd）
& target\mjwarp-parallel\venv\Scripts\python.exe docs\development-evidence\t4-source-hash-repin-13-independent-check\qa_identity_enum.py
& target\mjwarp-parallel\venv\Scripts\python.exe docs\development-evidence\t4-source-hash-repin-13-independent-check\qa_diff_boundary.py
& target\mjwarp-parallel\venv\Scripts\python.exe docs\development-evidence\t4-source-hash-repin-13-independent-check\qa_env_matrix_parse.py
# 环境矩阵复跑（自建 target，见 run_env_matrix.ps1）
& target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -File docs\development-evidence\t4-source-hash-repin-13-independent-check\run_env_matrix.ps1
```

本目录入库文件（`docs/development-evidence/t4-source-hash-repin-13-independent-check/`）：
`README.md`、`receipt.json`、`artifacts-sha256.json`、`qa-identity-enum.json`、`qa-identity-enum.log`、
`qa-diff-boundary.json`、`qa-diff-boundary.log`、`qa-env-matrix.json`、`qa_identity_enum.py`、
`qa_diff_boundary.py`、`qa_env_matrix_parse.py`、`qa_artifacts_sha256.py`、`run_env_matrix.ps1`。

不入库（可由脚本/命令重建）：`target/qa-repin13/*.log|*.txt`（原始构建/测试日志）、
`target/verify-clean-qa*/**`（本复核专用 target）。
