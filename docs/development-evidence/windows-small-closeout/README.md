# Windows 小项收口里程碑 T2：全量 G01 双模式终验

任务：`2026-10-24-windows-small-closeout`。日期：2026-10-24。执行人：QA。
授权：用户 2026-10-24 批准 T2 里程碑（**全量只跑一次**）。
最终 HEAD：**`959be10`**（debug 运行期间并行落地 U038 原生 f64 两级判定证据提交 `2412c11`/`959be10`；`git diff 5a0a348..959be10 -- src tests fixtures Cargo.toml Cargo.lock` 为空，两轮测试的产品源码身份一致）。
平台：Windows 11、RTX 4070 Ti SUPER、`rustc 1.99.0`、`--all-features`。

## 结论

- **G01 debug/release 双模式各 14/14 阶段 exit 0**：host-msvc **840 过 0 败**、all-probes **354 过 0 败（88 result 行；11 项文档化排除）**、host-gnu **485 过**、clippy 三配置 0 警告、rustdoc 0、diff 通过。
- 墙钟：debug **833.8s**、release **649.9s**；运行目录 `target/g01-acceptance/d4a4364b0f584f5685181b1e93a7f385`（debug）、`f6e41b7e46984686af4e03663558dc79`（release）。
- 与上一次全绿（§4.13，HEAD `c4c1058`）差异：host-msvc 835→**840**（a10-p11 新增 5 项单测）、all-probes 353→**354**（新增 1 项 ignored GPU 探针自动纳入）、result 行 87→**88**、host-gnu 480→**485**；排除仍 **11 项**、两模式逐项一致、未新增。
- **Windows 小项全部收口**：a10-p11（`245c461`，CUDA events 计时 + 显存采样，证据小修 `a50b288`）、U038 债务（`c0ac14c`，最小改动不可对齐；修正记录 `dc18c68`；用户 2026-10-24 裁决 **B 维持已知债务、不宣称通过**）、工具链门禁（`3b22f5f`，17/17 Windows 可判条款）、独立复核（`894ce28`，两笔声明成立）均 verified。
- 只对 Windows 已覆盖范围下结论：**不宣称双平台、不宣称完整上游等价、不宣称完整 g03-parity 全量通过**。

## 终验数字（两模式一致）

| 阶段 | debug | release |
| --- | --- | --- |
| fmt | 通过 | 通过 |
| native | 27 过 | 27 过 |
| host-msvc | **840 过 0 败** | **840 过 0 败** |
| g01 | 7 过 | 7 过 |
| native-g01 | 2 过 | 2 过 |
| native-only-g01 | 1 过 | 1 过 |
| resident | 102 过 | 102 过 |
| all-probes | **354 过 / 0 败**（88 result 行；11 项排除） | **354 过 / 0 败**（88 result 行；11 项排除） |
| host-gnu | **485 过** | **485 过** |
| clippy-gnu / clippy-gnu-cuda / clippy-msvc | 0 警告 | 0 警告 |
| rustdoc（`-D warnings`） | 0 警告 | 0 警告 |
| diff | 通过 | 通过 |
| 全脚本墙钟 | **833.8s** | **649.9s** |

- 数值报告（两模式相同）：`completeStates=2084`、`completeScalars=7,704,548`、`maxAbsoluteError=1.25147918803026e-06`；`rawScalars=50,512`、`rawMaxAbsoluteError=5.55349298547725e-07`。
- 排除清单 11 项（W-D5 3、历史捕获身份 6、env 未接线 1、U038-3D 已知债务 1），两模式逐项一致、本轮未新增；脚本写入 `<run>/probe-reports/exclusions.json`，显式排除而非静默跳过。
- `report.json` 的 `gitRevision=959be10`；两轮之间无产品源码变化（见 HEAD 说明），`unchanged` 门禁覆盖 `src`/`tests`/`fixtures`/`Cargo.*`。

## 文档更新

- `docs/development-evidence/g01-fast-loop/README.md` 新增 §5.6（T2 终验）与 `final-acceptance-t2.json`（机器记录）。
- `docs/three-layer-acceptance.md`：新增 §4.14（T2 终验）、§5.3 追加 debt-2 有界判定与用户裁决 B、§7 G01 现状更新到 §4.14。
- `docs/compatibility-matrix.md`：Windows 收口行更新为 T2 数字（840/354/88、485、833.8s/649.9s）；U038 状态段更新为「债务跟踪任务 verified（有界判定：最小改动不可对齐）＋用户 2026-10-24 裁决 B 维持已知债务」。

## 未满足项（逐条）

1. **U038-3D 已知债务**：全链 `qfrc_gravcomp` **8/77 超 2e-6**（max 4.29e-6），用户 2026-10-24 裁决 B 维持已知债务、**不宣称通过**；A/C 选项留档待用户另行裁决。
2. **NP01**：内部布局字段 20 个未做语义映射（191 行映射、类型偏离 0）。
3. **NP02**：9 条已知漂移（裁决 B 归档、不重采不重基线）与 51 份历史捕获 PTX 实体不在仓；未实现子系统 7、部分覆盖 3。
4. **MathDx 再分发口径**未裁决（不影响本地证据与运行）。
5. **性能与完整上游等价未验收**；产品固定 `compute_70` 与设备 SM89 架构差异未单独评估。
6. **Linux deferred**（并入 `2026-10-24-linux-batch-retest`），不宣称双平台。
7. **G02 完整验收未宣称**；本次 G01 双模式只代表 Windows 已覆盖范围。

## 证据索引

- `README.md`（本文）、`receipt.json`（含 `task_id`）、`final-acceptance-t2.json`（两轮 14 阶段、墙钟、排除项、报告指标与 HEAD 说明）。
- 运行目录（gitignored）：`target/g01-acceptance/d4a4364b0f584f5685181b1e93a7f385`、`target/g01-acceptance/f6e41b7e46984686af4e03663558dc79`；脚本日志：`target/windows-small-closeout/{g01-debug,g01-release}.log`。
- 关联证据：`docs/development-evidence/g01-fast-loop/README.md` §5.6、`docs/three-layer-acceptance.md` §4.13/§4.14/§5.3/§7、`docs/compatibility-matrix.md`、`docs/development-evidence/windows-small-independent-check/`、`docs/development-evidence/a10-p11-windows/`、`docs/development-evidence/u038-3d/debt-2/`、`docs/toolchain-clippy-acceptance.md`。

## 复现

```powershell
cd D:\Rust\mjwarp-rs
scripts\test-windows-g01.ps1 -AllFeatures -FullRegression
scripts\test-windows-g01.ps1 -AllFeatures -FullRegression -Release
```
