# G01 脚本新测试接线（W14/W15/U038）· 证据

任务：`2026-10-23-g01-script-new-tests-wiring`。
目标：修复当前 HEAD 上 `scripts/test-windows-g01.ps1` 的 all-probes 3 个 target 失败，使 G01 debug 全脚本恢复通过；
release 双模式由 QA 做最终验收。只改脚本、证据与文档计数；不改产品/测试/容差/fixtures。

结论：debug 全脚本 **14 个阶段全部 exit 0**；all-probes **346 过 / 0 败**（83 个 result 行），
仅剩 11 项文档化排除（W-D5 3、历史捕获身份 6、env/外部产物 1、U038-3D 已知债务 1）；墙钟 686.4s。

## 1. 修复前的 3 个 target 失败

| target | 失败 | 根因 |
| --- | --- | --- |
| `launcher-tile` | `nvrtc_compiles_static_shared_tile_kernel_with_verbatim_warp_headers` panic：`set MJWARP_CUDA_INCLUDE ... NotPresent` | W14 ignored 探针需要 CUDA SDK include，all-probes env 未接线 |
| `launcher-warp-headers` | `nvrtc_compiles_probe_with_verbatim_warp_headers` 同上 | W15 ignored 探针需要同一 include，未接线 |
| `t4_gravity_3d_product_chain` | `live_three_d_product_chain_matches_frozen_upstream` panic：`MJWARP_GRAVITY_3D_EVIDENCE_DIR` 缺失；接线后到达 U038-3D `total_outside==0` 断言 | 新 live 探针需要证据目录，且最终断言就是 8/77 已知债务 |

失败日志与哈希见 `repro.json` 与 `before-*.log`（本机复现；命令在 `repro.json`）。

## 2. 修复内容

1. **env 接线（`scripts/test-windows-g01.ps1` all-probes 段）**
   - 新增 `MJWARP_CUDA_INCLUDE = target/toolchains/cuda-12.8.1/include`；
   - 显式确保 `target/toolchains/cuda-12.8.1/bin` 在 `PATH`；
   - 新增 `MJWARP_GRAVITY_3D_EVIDENCE_DIR = <run>/gravity-3d-chain`（每次运行前清空，唯一目录）；
   - `probe-reports/env-wiring.json` 记录 `cuda_include=wired`、`cuda_bin_path=wired`、`gravity_3d_evidence=fresh-directory`。
2. **U038-3D 已知债务显式排除**
   - `scripts/test-windows-g01.ps1` 新增 `$script:excludedKnownDebt = @('live::live_three_d_product_chain_matches_frozen_upstream')`，
     理由写入 `probe-reports/exclusions.json` 的 `known_debt`/`known_debt_reason`（U038-3D 8/77、用户 2026-10-23 裁决 B）；
   - 不静默跳过：排除项在 exclusions 清单单列，手动复核命令写入 `docs/three-layer-acceptance.md` §5.3；
   - 断言、容差、测试文件均未改。
3. **计数与文档同步**
   - all-probes `-Minimum` 由 335 同步为 **346**；
   - `docs/three-layer-acceptance.md` 新增 §4.6（本次 debug 复跑与 14 阶段数字）、§5.3（known-debt 排除与手动命令），
     更新 §7 的 G01 现状/计数与 §4.5 接线前说明（release 待 QA 复验后更新双模式声明）。
4. **本机 PowerShell 5.1 可执行性**（说明：本机只有 Windows PowerShell 5.1.26100.7920、codepage 936，无 pwsh7 且无法联网安装）
   - 4 个 G01 脚本补 UTF-8 BOM（否则 5.1 以 ANSI 误读既有中文注释导致解析失败）；
   - 新增 `Invoke-WithNativeStderr` 包装：重定向的原生 stderr 在 EAP Stop 下会终止脚本，包装内用 Continue 并把
     ErrorRecord 还原为原始文本；显式 `$LASTEXITCODE` 检查与断言不变；
   - `test-windows-collision-input.ps1`：`#Requires -Version 5.1`，SHA-256 十六进制改用 `SHA256.Create()+BitConverter`
     （等价改写）；原生输出先捕获再写日志；避免 `$args` 在包装函数内被自动变量遮蔽。

## 3. 验证

| 阶段 | 结果 |
| --- | --- |
| fmt | exit 0 |
| native | 27 过（受限 27） |
| host-msvc | 722 过 |
| g01 / native-g01 / native-only-g01 | 7 / 2 / 1 过 |
| resident | 102 过 |
| all-probes | **346 过 / 0 败**（83 result 行；11 项文档化排除） |
| host-gnu | 406 过 |
| clippy-gnu / clippy-gnu-cuda / clippy-msvc | 0 警告 |
| rustdoc（`-D warnings`）/ diff | 通过 |

- 命令：`scripts\test-windows-g01.ps1 -AllFeatures -FullRegression`（debug）
- 运行目录：`target/g01-acceptance/d53081155f704909b098627b0936a57c`；墙钟 686.4s；GPU RTX 4070 Ti SUPER（驱动 596.36）。
- 证据文件：`debug-run.json`（阶段/接线/排除/脚本 sha256）、`checks.json`、`env-wiring.json`、`exclusions.json`、
  `all-probes-summary.txt`（83 行 result 摘要）、`repro.json`、`before-*.log`。
- 目标测试定向复验（接线后）：`launcher-tile` 2 过、`launcher-warp-headers` 1 过；
  `t4_gravity_3d_product_chain` 按 `--skip live::live_three_d_product_chain_matches_frozen_upstream` 0 跑（显式排除）。

## 4. 复现

```powershell
# 完整 debug 验收（本机 PowerShell 5.1；脚本已带 BOM）
scripts\test-windows-g01.ps1 -AllFeatures -FullRegression
# release 双模式（QA 最终验收）
scripts\test-windows-g01.ps1 -AllFeatures -FullRegression -Release
```

前提：本机 CUDA 12.8.1 / MuJoCo 3.12.0 工具链、冻结捕获产物、`target/t4-g03-retest` 等环境（脚本内已接线）。
只读检验：对比 `all-probes-summary.txt` 的 83 行 result 与 `debug-run.json` 的 346/0。

## 5. 边界

- 本次只完成 debug 全脚本；release 双模式由 QA 执行，执行前不宣称双模式通过（§4.5 的旧声明仅对应接线前版本）。
- source 拆分/产品代码/测试/容差/fixtures 均未改；U038-3D 债务仍为已知债务，未被"修复"。
- 脚本兼容性改动只为在本机 PowerShell 5.1 可执行，未改变断言、容差或检查语义；Windows 单平台。
