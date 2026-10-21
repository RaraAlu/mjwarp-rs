# 超长文件拆分收口证据

任务：`2026-10-21-split-program-reconcile`（超长文件拆分收口）。
范围：`docs/development-evidence/split-program/`、`docs/module-splitting.md`。
基线 HEAD：`8aac086`（T4 源码身份哈希重钉第二轮之后）。
本任务只做收口扫描、文档与债务登记，不改产品代码、测试、夹具与脚本。

## 扫描

脚本：`scan-oversized.ps1`（本目录）。

```text
target/toolchains/powershell-7.6.7/bin/pwsh.exe -NoProfile -ExecutionPolicy Bypass -File docs/development-evidence/split-program/scan-oversized.ps1
```

口径：

- 受检扩展名：`rs`、`cu`、`c`、`h`、`cpp`、`sh`、`ps1`。
- 排除目录：`docs/`（证据）、`fixtures/`（样本）、`target/`（构建产物）、`.git/`。
- 行数为物理行（`Get-Content | Measure-Object -Line`）。
- 输出 `report.json`，列出全部受检文件的路径与行数，并按行数降序。

运行输出：

```text
report: D:\Rust\mjwarp-rs\docs\development-evidence\split-program\report.json
files: 675
total_lines: 113985
max_lines: 491 (tests/resident_fixed_tendon.rs)
over_limit_files: 0
```

`report.json` 的 `/over_limit_files` 为 0，`/files` 为全部 675 个受检文件。收口任务的 json 检查直接读取该字段。

## 拆分汇总

61 个切片任务全部验收，明细（每文件拆分前后行数、新增子模块数与最大子文件行数）见 `docs/module-splitting.md`。

| 指标 | 数值 |
| --- | --- |
| 拆分文件数 | 61（src 42、tests 19） |
| 拆分前总行数 | 68191 |
| 拆分后模块根合计 | 10029 |
| 新增子模块文件 | 397（合计 59075 行） |
| 拆分后总行数（根+子模块） | 69104 |
| 最大新增子文件 | 474（`src/physics/passive_force.rs` 组内） |
| 拆分后模块根最大行数 | 407（`src/runtime/warp_kinematics.rs`） |
| 全仓库当前最大单文件 | 491（`tests/resident_fixed_tendon.rs`，不在拆分清单内） |

## 模块源码登记

- `.normify/modules/` 的 `[[source]]` 条目：拆分前 125，当前 434，新增 309，移除 0。
- 42 个 src 切片提交新增 309 个 `.rs`，与新增登记数一致。
- 19 个 tests 切片提交新增 88 个 `.rs`；tests 文件按项目约定不登记模块源码。
- 登记一致性：`src/` 下 433 个 `.rs` 对应 434 条登记。`src/physics/rne_bias.rs`（264 行模块根）未登记，拆分前同样未登记，属既有缺口；`src/physics/rne.cu`、`src/runtime/probe.ptx` 已登记但文件不存在，基线即如此。
- `normify validate` 结果：0 错误（收口门禁前执行）。

## 门禁与检查

收口引用的命令：

```text
normify module source ...（本任务无新增源码）
normify validate
normify gate 2026-10-21-split-program-reconcile
normify check 2026-10-21-split-program-reconcile
normify done 2026-10-21-split-program-reconcile
```

- gate 覆盖 fmt、clippy-cuda、unit 三项；运行记录在 `.normify/runs/2026-10-21-split-program-reconcile/`。
- 拆分期间每片都跑过同样三项门禁并逐片通过；全量 `cargo test --locked --features cuda-probe --no-fail-fast` 在 `2026-10-22-t4-source-hash-repin-2` 中执行，无失败目标（lib 311 过、120 ignored）。
- 本任务开始的 `unchanged` 检查以 `8aac086` 为基线，确认 `fixtures`、`native`、`include`、`build.rs`、`Cargo.toml`、`Cargo.lock`、`rust-toolchain.toml` 未被收口改动。
- `forbid_claim` 检查要求提交说明与回执不得宣称未验收结论，见末节。

## 未完成项与债务

收口文档 `docs/module-splitting.md` 记录四项开放项：

1. 源码锚点分层：拆分后 T4 源码身份锚点改为「模块根 + 产品子模块组合」，组合口径分散在多个 support 模块，尚未收敛为单一清单来源。
2. 可见性收敛：跨子模块访问做了等价放宽（`pub(super)` → `pub(in crate::…)`/`pub(crate)`），尚未逐项收敛到最小范围，也没有可见性审计用例。
3. fixtures 契约：`fixtures/precision/*.json` 等夹具内嵌源码哈希，与源码文本强绑定；两轮重钉后一致，但缺少自动漂移告警，后续编辑需要重钉。
4. G01 债务：
   - gravity 冻结位证书 2 bit 差异，基线 worktree `360718f` 同样失败，登记 `2026-10-22-g01-numerics-audit`（proposed）。
   - `MJWARP_PHYSICS_PTX_DIR` 的报告路径与 PTX 捕获目录双语义冲突。
   - 门禁脚本历史过期已由 `2026-10-22-g01-gate-script-sync`（已验收）同步计数与 all-probes 环境接线；all-probes 剩余红项与根因是既有债务。
   - 历史失败数据：两轮全量 G01 耗时 261s / 182s，显存峰值 5532→5756 MiB、5521→5772 MiB。

`docs/README.md` 文档索引未加入本记录，属任务范围外；需要时由后续任务补。

## 不得宣称的验收

本收口只证明文件行数达标、既有测试目标在拆分后仍通过、源码身份锚点已重钉。以下均未验收，提交说明与本文不得宣称：

- 完整 G01 全量回归。
- NP01 至 NP04 精度验收。
- GPU 性能、Linux 平台、release 配置验收。
