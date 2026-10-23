# NP03 Windows 单平台数值等价证据

任务：`2026-10-23-np03-windows-evidence`（P11-精度门禁，owner QA）。日期：2026-10-23。
范围：Windows 单平台（RTX 4070 Ti SUPER）上的 NP03 两级数值判定证据汇总与当前 HEAD 轻量复跑；Linux 侧按用户 2026-10-23 决定 **deferred**。
本任务只做证据与复跑记录，**未改产品源码/测试/容差/fixtures/脚本/排除清单**。

## 1. 结论（先给）

1. **Windows 单平台已记录的数值门禁全部 0 失败**：G01 主链与低层、G02 RNE、G03 矩阵链、G26 能量全链在各自证据范围内 `failed=0 / pending=0`；当前 HEAD 的 all-probes 复跑汇总 9 个 NP03 scope、79,286 块、0 失败、0 待验收（§3.3）。
2. **有两类已知缺口，不能判 NP03 通过**：
   - **U038-3D 已知债务**：全链 `qfrc_gravcomp` **8/77 超 2e-6**（max 4.29e-6），用户 2026-10-23 裁决 B 登记为已知债务，**未验收、不宣称通过**；
   - **T4 历史捕获 10 项显式排除**（W-D5 3 项 + 历史捕获身份 6 项 + env/外部产物 1 项）：是历史捕获/归档工具与身份审计，不是数值验收探针，脚本显式排除、未静默跳过。
3. **当前 HEAD 复跑发现 3 个 all-probes 失败目标，均非数值回归**：2 个是 W14/W15 新 NVRTC 测试缺 `MJWARP_CUDA_INCLUDE` 环境接线，1 个是 U038-3D 测试缺 `MJWARP_GRAVITY_3D_EVIDENCE_DIR`（补环境后到达已知的 8/77 断言）。G01 脚本本任务范围外（unchanged 保护），建议另立小任务补接线（§5）。
4. **平台边界**：仅 Windows；Linux 未测（deferred）；**不宣称双平台等价通过、不宣称 NP03 整体通过**。NP01/NP02 清单仍未验收（3 条缺生成内核证据、30 条缺逐字段类型表）。
5. 容差、断言、排除清单全程未改动：RNE 2e-5/2e-5；G01 `2e-5+2e-5|expected|`；G03 `2e-4+2e-4|ref|`；T4 `2e-5+2e-5|expected|`；G26 `2e-5`；U038-3D `2e-6`。

## 2. 环境与代码身份

| 项 | 值 |
| --- | --- |
| 系统 | Windows 11 专业版 10.0.26200 build 26200 |
| CPU | AMD Ryzen 9 9950X3D 16-Core |
| GPU | NVIDIA GeForce RTX 4070 Ti SUPER（16,376 MiB） |
| 驱动 | 596.36 |
| CUDA / NVRTC | 12.8.1 工具链；NVRTC 12.8.93 |
| 原生参考 | MuJoCo 3.12.0 |
| Rust | 1.99.0（`b940084d7 2026-09-28`），MSVC |
| 策略 | `native_error_v2`（原生优先 + 5% 红线），schema 2 |
| 复跑 debug HEAD | `f14009f19168f6866f8db3f2b016ccfa88bca7f3` |
| 复跑 release HEAD | `621f36e96aa254a850d97f2af52bd88877f41063` |
| 历史 G01 双模式 HEAD | `f266802`（2026-10-23 01:24–01:48） |

debug 全脚本与 release 定向复跑之间仓库新增 1 个提交（`621f36e` 治理任务收尾），只改 `.normify/changes/`，产品/测试/fixtures 无变化；两次复跑均记录实际 HEAD。

## 3. 当前 HEAD 轻量复跑（2026-10-23）

### 3.1 debug 全脚本（`scripts/test-windows-g01.ps1 -AllFeatures -FullRegression`，HEAD `f14009f`）

运行目录 `target/g01-acceptance/e86e82e44eda435ebc894e362ad9f283`（本机日志 `target/np03-g01-debug.log`）。

| 阶段 | 结果 | 说明 |
| --- | --- | --- |
| fmt | 0 警告 | 通过 |
| native | 27 过 0 败 | 通过 |
| host-msvc | **722 过 0 败** | 通过（比 10-23 01:24 的 709 多 13 项新测试） |
| g01 | 7 过 0 败 | 通过 |
| native-g01 | 2 过 0 败 | 通过 |
| native-only-g01 | 1 过 0 败 | 通过 |
| resident | 102 过 0 败 | 通过 |
| all-probes | **344 过 / 3 个目标失败**（83 个 result 行） | 阶段失败，见 §3.2 |
| host-gnu、clippy、rustdoc、diff | 未执行 | 脚本在 all-probes 失败处中止（与脚本既有行为一致） |

阶段耗时（取运行目录内各 `<stage>.log` 完成时间，起点为目录创建 10:14:49）：native 31s、host-msvc 89s、g01 19s、native-g01 5s、native-only-g01 3s、resident 142s、all-probes 492s；到 all-probes 结束合计 **781s**（与 2026-10-23 01:24 全脚本 debug 789.1s 同量级）。

### 3.2 all-probes 3 个失败目标与归因（均非数值回归）

| 目标 | 失败测试 | 原始报错 | 归因 |
| --- | --- | --- | --- |
| `launcher-tile` | `nvrtc_compiles_static_shared_tile_kernel_with_verbatim_warp_headers` | `tests/launcher-tile.rs:208`：`set MJWARP_CUDA_INCLUDE to the CUDA 12.8 include directory: NotPresent` | W15 新 NVRTC 测试要求显式 include 环境；G01 脚本只设 `CUDA_PATH`/`PATH`，未设 `MJWARP_CUDA_INCLUDE` |
| `launcher-warp-headers` | `nvrtc_compiles_probe_with_verbatim_warp_headers` | `tests/launcher-warp-headers.rs:115`：同上报错 | 同上 |
| `t4_gravity_3d_product_chain` | `live::live_three_d_product_chain_matches_frozen_upstream` | `tests/t4_gravity_3d_product_chain/live.rs:87`：`Option::unwrap()` on `None`（`MJWARP_GRAVITY_3D_EVIDENCE_DIR` 未设） | U038-3D 新测试要求证据目录环境；补环境后到达已知 8/77 断言（见下） |

验证（同一 debug 构建、显式补环境后）：

- `MJWARP_CUDA_INCLUDE=<repo>\target\toolchains\cuda-12.8.1\include`：`launcher-tile` **6 过 0 败**、`launcher-warp-headers` **3 过 0 败** → 前两项是环境接线缺失，不是产品/测试回归。
- `MJWARP_GRAVITY_3D_EVIDENCE_DIR=<target 下不存在的新目录>`：`t4_gravity_3d_product_chain` 的契约测试 1 过、live 测试失败于 `assertion left == right failed: product chain exceeded frozen tolerance; left: 8, right: 0` → 与 `2026-10-23-u038-3d-known-deviation` 的 **8/77 超差**完全一致，属已登记已知债务（用户裁决 B），不是新回归。

### 3.3 当前 HEAD 的 NP03 数值判定汇总（debug all-probes 内 9 个 scope）

来源：`all-probes.log` 的 `NP03_SUMMARY` 行逐行解析（容差/断言/比较范围未改）。

| scope | evaluated | native_passed | mjwarp_passed（二级参考） | failed | pending |
| --- | ---: | ---: | ---: | ---: | ---: |
| G01-complete | 58352 | 58352 | 0 | 0 | 0 |
| G01-raw | 336 | 336 | 0 | 0 | 0 |
| G01-geom-motion | 27 | 18 | 9 | 0 | 0 |
| G03-factors-and-rhs | 240 | 234 | 6 | 0 | 0 |
| G03-batches | 6410 | 6410 | 0 | 0 | 0 |
| G03-native-snapshots | 240 | 234 | 6 | 0 | 0 |
| G03-rhs-counts | 152 | 134 | 18 | 0 | 0 |
| G03-recovery | 5 | 5 | 0 | 0 | 0 |
| new resident cached factor/RHS stream | 13524 | 13479 | 45 | 0 | 0 |
| **合计** | **79,286** | **79,202** | **84** | **0** | **0** |

合计比较标量 **8,527,795**。G01-complete 覆盖 2084 组状态、7,704,548 标量；G01-raw 50,512 标量（既有 audit 口径）。二级参考放行均属「对照放行/红线内放行」，5% 余量未用于任何物理字段（与 `docs/windows-t4-g03-retest.md` 的判定口径一致）。

### 3.4 release 定向复跑（HEAD `621f36e`，2026-10-23 10:30–10:31）

`cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --release --test <target> -- <args> --test-threads=1`：

| 目标 | 参数 | 结果 |
| --- | --- | --- |
| `g01` | `--include-ignored` | **7 过 0 败**（16.12s） |
| `resident_rne_acceleration` | `--ignored` | **2 过 0 败**（13.49s，1 filtered） |
| `mass_solve_probe` | `--ignored` | **6 过 0 败**（7.55s，2 filtered） |
| `g26_energy_chain` | `--ignored` | **1 过 0 败**（6.98s，1 filtered） |

release 与 debug 在四个数值门禁上计数与结论一致（G01 7/0、G02 RNE 2/0、G03 6/0、G26 1/0）。release 全脚本 14 阶段的完整证据仍是 2026-10-23 01:24 的 `f266802` 运行（340/0），本任务未重跑 release 全脚本（见 §5 边界）。

## 4. 逐项证据（Windows 单平台）

| # | 项 | 范围 | 来源（提交/证据） | 模式 | 计数与结论 |
| --- | --- | --- | --- | --- | --- |
| 1 | G01 双模式最终验收 | G01 主链与低层 | `f266802`；`docs/development-evidence/g01-fast-loop/README.md` §5、`docs/three-layer-acceptance.md` §4.5 | debug+release | 14 阶段全过；all-probes **340 过 0 败**；789.1s / 652.9s；排除项 10 个单列可审计 |
| 2 | G01 当前 HEAD 复跑 | 同上（含 10-23 新增测试） | 本文件 §3；`f14009f` | debug | 13 阶段过、all-probes 344/3（2 环境 + 1 已知债务），数值 gate 0 失败（§3.3） |
| 3 | G02 RNE 加速度重基线 | RNE 加速度 240 记录 | `b13d1c5`；`docs/development-evidence/g02-rne-acc-rebaseline/README.md` | debug（+本任务 release 定向） | 521 块 → **0 失败块**，scalars=900288；worst normalized 0.743；容差 2e-5 未改 |
| 4 | RNE 剩余超差归属审计 | 历史「false 模式 1042 块」 | `docs/development-evidence/rne-repair-audit/README.md` | debug | 1042 = 2×521；重基线后 0 失败；两组归属已区分，无产品缺口 |
| 5 | G03 矩阵链对齐 | 因子/求解/批/快照/右端项/错误恢复 | `d5b02d5`；`docs/development-evidence/g03-matrix-chain-alignment/README.md` | debug（+本任务 release 定向 6/0） | `mass_solve_probe` 6 过 0 败；MathDx potrf(产品矩阵) **0/121** 位差；产品解 vs 冻结上游解 **0/33** 位差；NP03 5 个 scope 0 失败 0 待验收 |
| 6 | G26 能量全链 | 9 场景 × 8 状态 × 2 向量 | `f594d4f` + repin `08e7983`/`2713483`；`docs/development-evidence/g26-complete/README.md` | debug（+本任务 release 定向 1/0） | 468 能量槽 **0 失败**；worst normalized 9.985e-3 < 1；144 条上游参考天然校验全过 |
| 7 | U038-3D 三维重力链 | 1477 项（运动学 1400 + 算子 77） | `2026-10-23-u038-3d-known-deviation`；`docs/development-evidence/u038-3d/known-deviation.md` | debug | 运动学 1400/0 超差；`gravity_operator` 77/77 0 超差；**全链 qfrc_gravcomp 8/77 超 2e-6（max 4.29e-6，总位差 122）** → 用户裁决 B 已知债务，**未验收** |
| 8 | T4 历史捕获排除项 | W-D5 3 + 历史身份 6 + env 1 | `d5b02d5`/`7cea496`；`docs/three-layer-acceptance.md` §5.1/§5.2、`docs/development-evidence/t4-historical-capture-archive/` | debug+release | 10 项单列显式排除，不是数值探针；历史捕获已归档 724 文件、sha256 `95e7b430…a915`；未伪造基线 |
| 9 | NP01 类型映射 | 30 条产品路径 | `3756487`/`9c411de`；`docs/development-evidence/np01-type-mapping/` | 静态 | 20 部分/10 待补 → 28 条有 W15 头文件路径；30 条仍缺逐字段类型表 → **未验收** |
| 10 | NP02 冻结清单 | 30 条路径、冻结 `71da24d…` | `721e85a`；`docs/development-evidence/frozen-inventory/` | 静态 | 24 条目/30 变体；688 份回执编译选项一致；**3 条缺生成内核证据**（`rne_acc`/`tendon_wrap`/`field_batch`）；complete 0 → **未验收** |
| 11 | 两级判定规则本体 | 宿主规则 | `tests/two_reference_policy.rs`（20 项） | debug | 20 过 0 败，随 host-msvc 722 项执行；规则 `native_error_v2` 未改 |
| 12 | Linux 侧 | — | 用户 2026-10-23 决定；`docs/three-layer-acceptance.md` §7.1 | — | **deferred**：未测、不宣称、不算通过 |

## 5. 覆盖、缺口与建议

**已覆盖（有实测记录）**：G01 主链与低层（双模式完整脚本 + 当前 HEAD 全脚本 debug）、G02 RNE、G03 矩阵链、G26 能量链、RNE 1042 块归属、T4 历史捕获排除项清单、NP01/NP02 文档覆盖计数。

**未覆盖/未验收（如实列出）**：

1. **U038-3D 8/77 超差**：已知债务（裁决 B），跟踪任务 `2026-10-23-u038-3d-gravity-propagation-debt-2`（proposed，未 start）；本任务只复述，不修复、不放宽容差。
2. **T4 历史捕获身份 6 项 + W-D5 3 项 + env 1 项**：显式排除，恢复需重采集/重基线（`2026-10-22-t4-historical-capture-rebaseline`，proposed）或重新生成 face-operator 产物；当前不伪造。
3. **NP01/NP02**：3 条路径缺生成内核证据、30 条缺逐字段类型表；本任务只引用，不补。
4. **Linux**：deferred，未测。
5. **G01 脚本环境接线缺口（新发现）**：10-23 新增的 `launcher-tile`、`launcher-warp-headers` 需 `MJWARP_CUDA_INCLUDE`，`t4_gravity_3d_product_chain` 需 `MJWARP_GRAVITY_3D_EVIDENCE_DIR`；脚本当前未设置，导致 HEAD 上 all-probes 阶段失败。**不在本任务范围（`scripts/test-windows-g01.ps1` 属 unchanged）**，建议另立小任务：在 all-probes 的 env 接线段补这两个变量（或按既有规则显式排除 U038-3D 已知债务项），并复跑双模式确认。

**给后续任务的复跑建议**：日常用 `scripts/g01-fast.ps1 -NoBuild -AllFeatures -Target <target> -IgnoredOnly` 迭代；完整双模式只在收口时跑，先确认 GPU 独占窗口（本机桌面图形进程常驻，非独占）。

## 6. 平台边界

- 仅 Windows x86_64 MSVC + RTX 4070 Ti SUPER（DarkSea 本机）。
- 本文件所有数字**不延伸到 Linux**；Linux 缺失按用户决定登记 deferred，不当作通过、也不当作失败。
- 结论只用于 Windows 单平台的 NP03 证据汇总；**不宣称双平台等价通过、不宣称 NP03 整体通过**。
- 产品内核、容差、断言、fixtures、`scripts/test-windows-g01.ps1` 排除清单均未改动。

## 7. 复现命令

```powershell
# debug 全脚本（本文件 §3.1；约 13 分钟，GPU 非独占）
target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -ExecutionPolicy Bypass `
  -File scripts\test-windows-g01.ps1 -AllFeatures -FullRegression

# release 定向数值门禁（本文件 §3.4；构建 + 约 1 分钟）
$env:CUDA_PATH = '<repo>\target\toolchains\cuda-12.8.1'
$env:PATH = "$env:CUDA_PATH\bin;$env:PATH"
$env:MJWARP_CUDA_INCLUDE = "$env:CUDA_PATH\include"
$env:MJWARP_MUJOCO_ROOT = '<repo>\target\toolchains\mujoco-3.12.0\package'
$env:MJWARP_MUJOCO_DLL = "$env:MJWARP_MUJOCO_ROOT\bin\mujoco.dll"
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --release --test g01 -- --include-ignored --test-threads=1
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --release --test resident_rne_acceleration -- --ignored --test-threads=1
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --release --test mass_solve_probe -- --ignored --test-threads=1
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --release --test g26_energy_chain -- --ignored --test-threads=1

# 3 个失败目标的环境归因验证（§3.2）
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --test launcher-tile -- --include-ignored --test-threads=1
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --test launcher-warp-headers -- --include-ignored --test-threads=1
$env:MJWARP_GRAVITY_3D_EVIDENCE_DIR = '<repo>\target\<不存在的新目录>'
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --test t4_gravity_3d_product_chain -- --include-ignored --test-threads=1
```

## 8. 证据索引

- `np03-windows.json`：机器可判定摘要（平台、HEAD、逐项证据、复跑计数、覆盖与缺口、边界）。
- 本文件：方法、逐项证据表、失败归因、覆盖与缺口、复现命令。
- 历史证据：`docs/development-evidence/g01-fast-loop/`、`g02-rne-acc-rebaseline/`、`g03-matrix-chain-alignment/`、`g26-complete/`、`rne-repair-audit/`、`u038-3d/known-deviation.md`、`t4-historical-capture-archive/`、`np01-type-mapping/`、`frozen-inventory/`。
- 本机复跑日志（gitignored）：`target/np03-g01-debug.log`、`target/g01-acceptance/e86e82e44eda435ebc894e362ad9f283/`、`target/np03-release-targeted.log`。
