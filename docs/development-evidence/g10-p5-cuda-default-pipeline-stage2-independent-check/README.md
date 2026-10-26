# G10@P5 CUDA 默认管线接线（阶段 2）· QA 独立复核

- 任务：`2026-10-25-g10-p5-cuda-default-pipeline-stage2-independent-check`（p54，owner QA）
- 复核对象：`2026-10-25-g10-p5-cuda-default-pipeline-stage2-r2` 交付，提交 `5b27c69e`（父 `54615cd8`；原 stage2 流程性 abandon `e1154755`，登记 `54615cd8`）
- 复核 HEAD：`5b27c69e71e96f29173388f6930c99cba373f11e`
- 结论：**pass（阻断 0 / 观察 10）**。本片未开 GPU 窗口（纯 CPU 复核；窗口事实只做日志与产物核对），未改产品代码/判据/容差/证据语义，未 commit、未 done。
- 本目录只写 QA 自己的证据；所有数字除注明「读交付」外都是我自己重跑/自算。

## 0. 结论速览

1. 特性与依赖真值表成立：`default=["cuda"]`、`cuda=["cuda-probe"]`、单一依赖集合；默认外部依赖 **24**、`--no-default-features` **0**；`Cargo.lock` 零改动。
2. 三配置 clean 实测：默认 **971/0/369（112 binary；lib 698/0/137）**、no-default **605/0/7（lib 455/0/5）**、`--features cuda` **971/0/369**；三配置 build/test 全部 exit 0。交付叙述的 364/111 是**过时数字**（见 §3、观察 O1）。
3. 三态回退不 panic：4 条 API + 3 条 CLI 测试各自独立进程 exit 0；选择层 6 条测试 exit 0；**修前红**我在自己的 scratch worktree 复现 exit 101（panic 逃出守卫），恢复后 4 passed。
4. 历史 T4 审计隔离成立：默认 7 target 各 0 tests；MSVC `--all-features` 7 target 我自跑 **27/0**；`product_sources`/NP01 未 repin。
5. 等价性：17 个 dump 按 G5 行序归一后 run1/run2 **17/17 位级一致**（raw 数组仅 ellipsoid-box Contact 因行序不同）；记录 B 我独立重算与交付 `record-b.json` **逐项一致**（cylinder/ellipsoid/mesh-mesh 26/26、sdf-box 1040/1040、shallow 15/26 判据内、deep 8/26 12 失败、capsule 5/26 19 失败）。
6. 窗口：attempt4 唯一有效轮，B/C/D 四个设备二进制 exit 0；**E `stage2_equivalence_window` 窗口内 exit=101 如实披露**；采样器未落盘（samples=0）→ guard v2 未实际生效，已披露；post 基线同基线、无残留。
7. UNC-2 限定语与 r2 终版验收**逐字一致**；但 receipt/README 顶部/§7 末行仍留「窗口 0 / UNC-2 open」旧文（观察 O2）。
8. 未发现「抓凑绿」：`case_evidence.rs`、`case_compare.rs`、fixtures、G5/G6 参考、`gates.toml` 的判据/容差、已 verified 证据语义均未被改动（§6、§8）。

## 1. 复核方法与隔离

- 只读主树；所有构建用**自建** `CARGO_TARGET_DIR`（`target/qa-stage2/t-*`，先删后建），不复用交付/Backend 的 target；日志在 `target/qa-stage2/logs/`。
- 独立脚本：`scripts/qa-dump-check.ps1`（dump 确定性 + 记录 B 自算）、`scripts/run-default.ps1`、`scripts/run-fallback-tests.ps1`、`scripts/run-clippy.ps1`、`scripts/run-worktree-checks.ps1`（前修红）、`scripts/run-cleanclone-lf2.ps1`、`scripts/run-notarget2.ps1`。
- 环境：`rustc 1.99.0 (b940084d7)`，host `x86_64-pc-windows-gnu`，active toolchain `1.99.0-x86_64-pc-windows-gnu`；cargo `1.99.0 (5f94df478)`；Windows 11 `10.0.26200`；GPU 窗口 0。
- 证据目录约束：单文件 ≤500 行、UTF-8 无 BOM、无二进制。

## 2. 特性与依赖（独立核）

| 项 | 实测 |
| --- | --- |
| `Cargo.toml` features | `default=["cuda"]`；`cuda=["cuda-probe"]`；`cuda-probe=["dep:cudarc","dep:libloading","dep:serde","dep:serde_json","dep:sha2"]`；`t4-historical-audits=[]`（非默认） |
| `cargo metadata` | 与上表一致；无第二套默认特性、无隐藏 default |
| 默认 `cargo tree -e normal,build` | 原始行 33 / 去重行 29 / 外部包 **24**（含 `cudarc 0.18.2`） |
| `--no-default-features` tree | 原始行 1 / 外部包 **0**（`cudarc` 0 命中） |
| `Cargo.lock` | 父与 HEAD **逐字节无差异** |

## 3. 三配置 clean 实测（我自己的 target，逐一先删后建）

| 配置 | build | target | exe | 外部依赖 | 测试（pass/fail/ignored；binary） |
| --- | --- | --- | --- | --- | --- |
| 默认 | exit 0 / 17.7 s | 9087.7 MiB | 78.14 MiB | 24 | **971/0/369；112**（lib 698/0/137） |
| `--no-default-features`（PATH 去 CUDA/NVIDIA、CUDA_* 清空） | exit 0 / 6.9 s | 1816.7 MiB | 19.73 MiB | 0 | **605/0/7；112**（lib 455/0/5） |
| `--features cuda` | exit 0 / 18.1 s | 9087.6 MiB | 78.14 MiB | 24 | **971/0/369；112** |

- **交付叙述 vs 实测**：交付 README §0/§2、receipt、`logs/gates-final.txt` 写 971/0/**364**（111 binary）；交付自己的 `.normify/runs/...stage2-r2/unit.json` 与 r2 终版验收基线都是 **971/0/369**，与我一致。
- 差异根因（我核到）：交付 `unit-summary-default.log` 只有 111 行 result，缺 `tests/stage2_equivalence_window.rs` 这一行（默认运行 = 0 passed / 0 failed / **5 ignored**）。364+5=369、111+1=112，而该测试文件正是 `5b27c69e` 新增的 → 叙述数字早于窗口测试文件。no-default 的 605/7 两处一致，只是 binary 数也少 1。
- 体积/耗时与交付同量级（build +0.8~1.3 s、target +0.1%、exe +0.16 MiB），差异来自构建路径/缓存完整度，不作为判据。

## 4. 三态回退不 panic（独立进程 + 前修红）

| 路径 | 测试（每条单独进程 `--exact`） | 结果 |
| --- | --- | --- |
| 无驱动（API） | `device_context_without_driver_returns_nodevice_without_panic` | exit 0，1 passed |
| 有驱动无设备（API, seam） | `device_context_without_usable_device_returns_nodevice` | exit 0，1 passed |
| culib 初始化 panic（API, seam） | `device_context_initialization_panic_is_caught` | exit 0，1 passed |
| 有设备无 NVRTC（API） | `constructor_without_nvrtc_returns_compile_without_panic` | exit 0，1 passed |
| CLI 无驱动 | `prepare_device_without_driver_returns_driver_unavailable_without_panic` | exit 0 |
| CLI 无设备（seam） | `prepare_device_without_devices_returns_invalid_device` | exit 0 |
| CLI 无 NVRTC | `require_nvrtc_without_library_returns_nvrtc_unavailable` | exit 0 |
| 选择层 | `runtime::backend::tests::cuda_probe`（6 条） | exit 0，6 passed / 0 failed |

- **修前红（我自己做，不用交付日志）**：detached worktree `target/qa-stage2/wt-clean`（HEAD，无 target/），只在该副本把 `context.rs` 的驱动预检与 `catch_unwind` 去掉、保留 `#[cfg(test)]` panic 注入 → `device_context_initialization_panic_is_caught` **exit 101**，日志含 `injected cudarc culib panic` 与 `panic 逃出了 device_context（守卫缺失）`；恢复文件后同目标 **4 passed / exit 0**（`logs/pre-fix-red-scratch.log`、`logs/pre-fix-green-restored.log`）。
- 未覆盖：真实无驱动/无卡机器未实机；「有驱动无设备」「culib panic」为 `#[cfg(test)]` seam 模拟 —— 交付同样如实标注，没有写成实机验证。

## 5. seam 纪律与产品二进制扫描（独立）

- seam 标识在默认/`cuda`/no-default 三个产品 exe 中 **0 命中**：`DRIVER_OVERRIDE`、`NVRTC_OVERRIDE`、`DEVICE_COUNT_OVERRIDE`、`CONTEXT_PANIC`、`CONTEXT_ERROR`、`SeamGuard`、`set_*_for_tests`、`reset_test_overrides`、`FORCE_SOFT_FMA`。
- 但交付「产品二进制 `MJWARP_*` 全部 0 命中」**不成立**：默认/`cuda` exe 含 `MJWARP_PHYSICS_PTX_DIR`、`MJWARP_LAUNCHER_BASELINE_DIR`（都在**未改动**的 `src/runtime/transfer/kernel*` 产品代码里），以及内嵌 CUDA 源里的 `MJWARP_G01_EQUIVALENT`/`MJWARP_FIELD_BATCHES`/`MJWARP_MOCAP` 宏名。实质主张成立：无运行时 env 门控、改动文件 6 个里 `env::var`/`std::env`/`env!` **0 命中**、seam 全部 `#[cfg(test)]`。
- 结论：seam 纪律 pass；仅交付措辞需收窄（观察 O6）。

## 6. 门禁变更与可移植性

- `gates.toml` 与交付 §3 表逐项核对：`clippy-cuda` 改 `--features cuda`；新增 `unit-nocuda`（`cargo test --locked --no-default-features`）、`clippy-nocuda`；`gate.unit` 命令不变；`hash_inputs` 已含 `Cargo.toml`。old/new/理由/回滚五条齐全。
- G01 脚本 diff 最小：`g01-fast.ps1` 1 行（默认 feature 去冗余 `cuda-probe`）、`test-windows-g01.ps1` 2 行（feature 参数 + `clippy-gnu-cuda` 改 `--features cuda`），无其它 flag 变更。
- **门禁新鲜度**：交付六个 runs 在 `18:12:14–18:12:34`（提交 `18:13:02`，同一工作树）；`unit.json` 记录了 971/369、`unit-nocuda.json` 605/7（与我的重跑一致）。三个 clippy runs 的 wall 只有 403–404 ms，是 cargo **缓存命中（未重新 lint）**；所以我在**删除后的 fresh target** 里自己重跑：`fmt` exit 0（2.4 s）、`clippy` exit 0（24.8 s）、`clippy-cuda` exit 0（18.3 s）、`clippy-nocuda` exit 0（18.6 s）。
- **干净 clone 可移植性**：
  - 真正 `git clone`（默认 `core.autocrlf=true`）无 target/ 时会在 `tests/g10_p5_sdf_relmat.rs::sdf_rotation_fixture_is_well_formed` 红（脚本 sha 按 LF 冻结、checkout 变 CRLF）；
  - `core.autocrlf=false` clone 则在 `tests/launcher-warp-headers.rs::copied_headers_match_the_frozen_manifest` 红（header manifest 按 CRLF 冻结）。两者都是**父提交已存在**的 EOL 卫生问题，与本片无关。
  - 按本片真正要证的性质（**不依赖本机 target/ 冻结产物**）：把 13415 个 tracked 文件按主树现盘字节拷到 `D:\qa-nt`（无 `target/`、无 `.git`）+ 全新 clean target → `cargo test --locked` = **971/0/369，112 binary，exit 0**（85.9 s）。隔离 7 个历史审计 target 后默认门禁不再读本机冻结产物，**该性质成立**。
  - 因此交付用「干净 clone 可移植」表述偏强，建议改「不依赖 target/ 冻结产物」或登记 EOL 治理债（观察 O10）。

## 7. 历史 T4 审计隔离

| 项 | 实测 |
| --- | --- |
| 7 个 target 的 gating | `#![cfg(all(feature="cuda", feature="t4-historical-audits"))]`（7 个文件第 3 行） |
| 默认 `cargo test --locked` | 7 个 target 均 **0 passed/0 failed/0 ignored**（我的 clean 默认运行逐 target 抽出） |
| `--features cuda,t4-historical-audits` | 交付记 27/0；**我自跑 MSVC `--all-features` 7 target = 27/0（7 binary，0 ignored）**，逐 target 3/4/4/4/4/4/4 |
| T4 身份 | NP01 `inventory.json` 父/HEAD sha 相同；提交不含 `src/physics`/manifest 改动；`t4_identity_live_check` 在默认套件内 2 passed（`sha256_and_lf_normalization_match_known_vectors`、`live_product_sources_match_fixture`） |

→ 「默认不再跑历史审计、all-features 仍跑且 27/0、无 repin」三条均独立成立。

## 8. 等价性三条记录（独立重算，不读交付脚本）

**记录 A（host ↔ 上游 CPU 参考）**：交付引用既有 verified 的 fix r2（`50a0acd1`，G2 五 case 位级 0 差异）+ G3（sdf-box）；primitive 三 case 记「host 不适用」。我只读交叉引用（fix r2 已在本链路独立复核过），未重跑。

**dump 两轮确定性（17 个 dump）**：按 G5 行序口径（worldid 分组 + 稳定排序后行集合逐位比较）**17/17 一致**；raw 数组只有 `ellipsoid-box Contact` run1/run2 行序不同（worldid `1,0` vs `0,1`，数值多重集相同）。交付窗口 doc §4 有披露，但限定语未带行序前提（观察 O4）。

**记录 B（device dump vs G5 冻结 device 参考 `out1/`）**，我自己解析 `out1/*.bin` 并按 G5 判据重算：

| case | total | bit_equal | max_dist_ulp | max_pos_ulp | max_frame_abs | 失败条数 | 交付 record-b |
| --- | --- | --- | --- | --- | --- | --- | --- |
| cylinder-box | 26 | 26 | 0 | 0 | 0 | 0 | 一致 |
| ellipsoid-box | 26 | 26 | 0 | 0 | 0 | 0 | 一致 |
| mesh-mesh | 26 | 26 | 0 | 0 | 0 | 0 | 一致 |
| sphere-box-shallow | 26 | 15 | 1 | 0.25 | 6.985e-8 | 0 | 一致 |
| sphere-box-deep | 26 | 8 | 14 | 6600.75 | 1.215e-6 | 12 | 一致 |
| capsule-box | 26 | 5 | 12 | 2237052 | 1.0 | 19 | 一致 |
| sdf-box | 1040 | 1040 | 0 | 0 | 0 | 0 | 一致 |

- **G5 §11.2 依据成立**：`docs/development-evidence/g10-p5-gpu-kernels/README.md` §11.2（198–205 行）写明上游 dispatch 对 `(SPHERE,BOX)`、`(CAPSULE,BOX)` 走 `primitive_narrowphase`，探针 ccd 分支的 `out1` 未与上游参考绑定，且**同句点名 sphere-box-shallow/deep 与 capsule-box 三者**。
- **清单完整性（观察 O5）**：交付把 deep/capsule 归 `reference_not_comparable`，却把 shallow 计为「有效参考、判据内通过」。按同一条 §11.2 依据，三者同类；这是非对称使用同一排除规则（不影响「不是 B 全绿」）。我的重算显示 shallow 0 失败、deep/capsule 红，与交付数字一致，问题只在归类口径。
- **记录 C（跨 recipe）**：17 个报告、12 个 case/内核有差异，交付写「只记录不判 pass/fail」；我读 `result.json` 与其一致。host 对齐上游 CPU 非融合、device 对齐 pinned GPU 融合，两套 recipe 本来不同，**不是 host↔device 等价**；限定语已明确否认等价。
- 原始产物（attempt4 日志、17 dump、`record-b.json`/`result.json`/`three-records.json`）都在 gitignored `target/stage2-impl/window/`，未入库（观察 O8）。

## 9. 窗口 attempt4 与 E exit=101

- 基线/post（原始文件）：`4187 MiB used / 11875 MiB free` 前后相同；util `36% → 8%`；温度 `33 → 34 °C`；`query-compute-apps` 前后各 34 条桌面/系统上下文，**无本任务计算进程、无残留**。
- 设备二进制（我读原始日志）：B-nvrtc 1 passed（hfield-sphere 314/314、sdf-box 3122/3122、ellipsoid-box 80/80 全位等）；C-prim 1 passed（primitive 三 case 80/80）；C-multi 1 passed（314/314、391/392 含 1 ULP、80/80）；D-ccd 2 passed。四处 exit 0，与交付一致。
- **E `stage2_equivalence_window` 窗口内 exit=101 已如实披露**（facts.txt/README/window doc 都写 101，没有写成「5 个二进制全绿」）。
- **但根因表述与原始日志不符（观察 O3）**：E 日志显示 3 个 CPU 记录测试失败文本是 `读 dump ... os error 3（找不到路径）` —— libtest 按名字序先跑 3 个比对测试、最后才跑 `window_product_equivalence_dump`（该测试通过并写出 17 个 dump）；**直接原因是 dump 尚未落盘**。窗口 doc §2 把原因写成「harness 行序比较 bug」；行序问题实际是 dump 的 run1/run2 原始数组比较在 ellipsoid 上误判（排序修正后归一化一致，但窗口未重跑）。exit=101 与「dump 已落齐、CPU-only 重跑通过」的事实不被推翻，只是归因与日志不一致。
- 采样器 `samples.csv` 只有表头、facts `samples=0` → guard v2 未实际生效，交付已披露；attempt1/2/3 目录只有基线或 NVRTC 缺失日志，零 GPU 作废、未拼进矩阵。

## 10. UNC-2 与文档一致性

- `README §7.1` 的限定语与 r2 终版验收（Chief 批准）**逐字一致**：默认含 CUDA 入口、产品入口经 NVRTC launcher 在真实设备被调用（4 个设备二进制 exit 0、dump 两轮按 G5 行序一致）、记录 B 可评估子集全绿、2 例不可比、2 例无冻结参考、**不等于 B 全绿**、**默认选择层仍 Host、Device 需显式选择**、不代表 host↔device 等价/上游等价/指标结论。
- 陈旧/矛盾项（观察 O2）：`receipt.json` 仍 `status=in_progress / gpu_windows=0 / unc2=open / equivalence=pending ③b/③c`；README 顶部第 6–7 行与 §7 末行（149 行）仍写「窗口 0 / 不记 closed」；`tests/stage2_equivalence_window.rs` 的 `three_record_summary` 仍硬编码 `unc2_closed=false` 并在产物写 open。这些是**低估**而非提前宣称关闭，closed 的依据在 README §7/§7.1 与 r2 终版验收。
- 选择层可达性（观察 O7）：`src/runtime/backend.rs` 的 `BackendChoice/BackendStatus/device_permitted/device_program` **只被自己的测试引用**；CLI 探针走 `runtime::cuda::prepare_device` + 共享 presence 探测。这与 Chief 限定语「默认选择层仍 Host、Device 需显式选择」一致；若 r2 意图里的「把设备入口接进碰撞准相位」按实际 dispatch 接线理解，则尚未落地，建议登记为迁移债。

## 11. 观察清单（10 条，均不阻断）

1. **O1** 交付默认/`cuda` 测试基线写 971/0/364（111 binary），实际 971/0/369（112）；根因是 summary 日志漏了新增的 `stage2_equivalence_window` 的 0/0/5 行。建议收口前改为 369/112 或标注测量时点。
2. **O2** receipt/README 顶部/§7 末行与 harness 仍是「窗口 0 / UNC-2 open」，与 §7.1 closed 矛盾。建议统一或加注「closure 为治理决定」。
3. **O3** E exit=101 的根因写作「行序 bug」，原始日志显示直接原因是 dump 尚未落盘的 `os error 3`。建议分开写两条子事实。
4. **O4**「dump 两轮逐位一致」只在 G5 行序归一下成立（ellipsoid raw 行序不同）。建议补行序前提。
5. **O5** 记录 B 对 §11.2 的 primitive 三例使用不对称（shallow 计入绿子集、deep/capsule 排除）。建议同口径归类或注明理由。
6. **O6**「产品二进制 `MJWARP_*` 0 命中」不成立（未改动的 transfer 产品代码 env 名 + 内嵌 CUDA 宏）。实质 seam 结论成立，建议收窄措辞。
7. **O7** 选择层无产品调用方（仅测试）；与 Chief 限定语一致，但建议把「narrowphase dispatch 接线」登记为后续片。
8. **O8** 窗口/等价性原始产物未入库（仅在 gitignored `target/`），仓库外无法复算；本机可，我已重算。
9. **O9** `logs/selection-backend-tests-nocuda.log` 是旧版源码日志（含当前不存在的 unused import 警告）；当前 no-default 构建 0 warning、clippy-nocuda 通过。
10. **O10**「干净 clone 可移植」偏强：真正 git clone 会撞两个**父提交已存在**的 EOL 冻结测试（autocrlf=true 撞脚本 sha、autocrlf=false 撞 header manifest）；本片要证的性质（不依赖 target/ 冻结产物）已用 tracked-file 拷贝实测成立（971/0/369）。建议改措辞或登记 EOL 治理债。

## 12. 未覆盖 / 边界

- 未开 GPU 窗口：窗口事实只做原始日志/产物核对；device 段数字不重跑（`impact` 与 dump 是窗口内一次性产物）。
- 真实无驱动/无卡机器未实机；三态为 seam 模拟（与交付口径一致）。
- 未跑全量 G01（本任务只要求定向与本片 checks）；G01 脚本只有上述 3 行改动。
- 不宣称跨环境全绿、不宣称 host↔device 数值等价、不宣称上游等价；NaN payload 不作位级判据；边界计数 16。
- 记录 A 引用的 fix r2 / G3 数字为只读交叉引用（该链路此前已由 QA 独立复核），本片未重跑。

## 13. 复现命令（我的脚本在 `scripts/`）

```powershell
# 三配置（各自 clean target；no-default 需先清 CUDA 环境）
cargo build --locked; cargo test --locked
cargo build --locked --no-default-features; cargo test --locked --no-default-features
cargo build --locked --features cuda; cargo test --locked --features cuda

# 三态/选择层（每条单独进程）
cargo test --locked --lib runtime::collision_gpu::fallback_tests:: -- --test-threads=1
cargo test --locked --lib runtime::cuda::tests:: -- --test-threads=1
cargo test --locked --lib runtime::backend:: -- --test-threads=1

# 历史 T4 审计（默认 0 tests；all-features 27/0，MSVC + G01 环境）
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --test t4_camlight_precision --test t4_face_fusion_repair --test t4_face_normalization_control --test t4_hessian_generation --test t4_spatial_tendon_precision --test t4_tendon_wake_precision --test t4_trig_control_domain -- --test-threads=1

# 等价性自算（dump 确定性 + 记录 B）
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/qa-dump-check.ps1

# 门禁
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --all-targets --locked --features cuda -- -D warnings
cargo clippy --all-targets --locked --no-default-features -- -D warnings
```

## 14. 证据清单

- `README.md`（本文件）、`receipt.json`、`artifacts-sha256.json`
- `qa-features-deps.json`、`qa-three-configs.json`、`qa-fallback-seam.json`、`qa-t4-isolation.json`、`qa-equivalence.json`、`qa-window-e-log.json`、`qa-gates-portability.json`、`qa-docs-consistency.json`、`qa-observations.json`
- `logs/`：三配置与 lib 摘要、clippy fresh、clippy-nocuda、三态独立进程、前修红/恢复绿、MSVC all-features 27/0、两次 clone 失败摘录、no-target 通过、dump 重算、tree 计数、主树状态
- `scripts/`：上述 7 个独立检查脚本
