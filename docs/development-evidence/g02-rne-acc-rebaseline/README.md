# G02 RNE 加速度参考重基线（用户裁决 A）

任务：`2026-10-23-g02-rne-acc-fixture-rebaseline`（登记 `46bdba7`）。
裁决来源：用户 2026-10-23 选择「重基线参考」（方案 A），由 Chief 转达；根因证据见 `docs/development-evidence/g02-rne-authoring-diff/`。

## 结论

`fixtures/rne-acceleration` 的 qfrc 期望值已从「冻结原生 MuJoCo 3.12 f64 参考」重基线为「冻结上游 MJWarp（`71da24d…`）同口径 f32 参考」。容差（ABS=REL=2e-5）、测试断言、产品 `src`/`tests`、脚本与排除清单**零改动**；原 f64 参考归档保留并标注历史。

重基线后 `resident_rne_acceleration` 的 521 块 native-tolerance 失败全部转绿（`failed_world_field_blocks=0`、scalars=900288）。

## 根因（来自 authoring 任务，供判读）

- 521 块全部是 `mixed-joints`/gravity2/state7/`flag=false`、worst_dof=3；本库 f32 `1.5473079681396484` vs 原生 f64 `1.5471062036731382`（差 2.02e-4，normalized 3.96，容差 2e-5）。
- 失败块产品 qfrc 位串与冻结上游 MJWarp 参考逐位相同（0/11 位差）；上游自身对同一原生参考也失败同一块（240 条中 1 条）。
- 即「原生 f64 参考 vs MJWarp f32 累加」的参考口径差异（与 G01 W-D4 同类），不是产品实现分歧。

## 做法与映射

- 期望值来源：`docs/development-evidence/g02-rne-authoring-diff/upstream-rne.json` 的 `records[model][*].qfrc_bias_hex`（f32 位串）。
- 映射：fixture `q.frc[flag][gravity][dof]` ← 上游记录 `{state=case.id, gravity, flag}` 的 `qfrc_bias_hex[dof]`；`flag` 维度 0/1 对应 `mj_rne` 的 `flg_acc=false/true`。
- 记录规模：5 models × 3 gravities × 8 states × 2 flags = 240 条；测试以 worlds `[1,2,5,513]`、8 轮、`state=(w+round)%8` 覆盖同一组记录。
- `zero-dof` 与 `massless-tree` 的 `nv=0`，qfrc 全为空数组，重基线前后文件字节一致；实际改动 `mixed-joints`、`inertial-tree`、`rotated-tree` 三个文件的 qfrc 行（每文件 8 行）。
- 输入（`qpos/qvel/qacc`、`nq/nv/nbody/gravities`、seed、容差）逐项校验未变（工具内 `Compare-Inputs` 断言）。
- 期望值以 f32 位串转 double 的最短往返十进制写出，解析回 f64 精确等于该 f32 值；失败块新期望即 `1.5473079681396484`（= 产品实际值）。

## 变更文件

| 文件 | 旧 SHA256 | 新 SHA256 |
| --- | --- | --- |
| `fixtures/rne-acceleration/mixed-joints.json` | `0bb8e995…` | `285f6a38b57ed1fd477c9b2d4eaea2a016656b1b2aab979d83d9681c8e1bf11a` |
| `fixtures/rne-acceleration/inertial-tree.json` | `f359787e…` | `3fc9b673530ae147c226b8a550c2070ee29dcfc8f56492d2ac487f4ba4104da8` |
| `fixtures/rne-acceleration/rotated-tree.json` | `66abdf7a…` | `a223f8153d39c57323c8234b5e42412325efe0eb00923467e035d7aa4c0dac5f` |
| `fixtures/rne-acceleration/zero-dof.json` | `92de890f…` | 不变 |
| `fixtures/rne-acceleration/massless-tree.json` | `c6bb013a…` | 不变 |
| `fixtures/rne-acceleration/manifest.json` | — | 同步 3 个 sha256，`scope` 更新，新增 `expected_reference` 与 `historical_native_f64_reference` 元数据；`native_version`、容差、`files` 语义未变 |

旧值全文归档在 `historical-f64/`（含 SHA256 与历史标注）；完整映射清单见 `rebaseline.json`。

## 工具与复现

```powershell
# 预演两次并逐字节比对（不写仓内文件，输出 target/rebaseline/out）
pwsh -NoProfile -File docs/development-evidence/g02-rne-acc-rebaseline/rebaseline-fixture.ps1 -VerifyTwice
# 落盘（fixtures + manifest + 本目录 rebaseline.json；同时归档旧 f64 参考）
pwsh -NoProfile -File docs/development-evidence/g02-rne-acc-rebaseline/rebaseline-fixture.ps1 -Apply
```

- 工具为 PowerShell（`System.Text.Json` 之外的 .NET 基础类型手写序列化），非 Python；离线证据工具，产品构建/测试/脚本不读取。
- 工具启动前校验 `upstream-rne.json` 的 SHA256（`42f76130…`），不一致拒绝运行；产物确定性由 `-VerifyTwice` 两次逐字节一致保证。
- 归档文件哈希与 `rebaseline.json.fixtures` 记录一致。

## 验证（debug，2026-10-23 01:04–01:16，RTX 4070 Ti SUPER / 596.36）

| 项 | 命令 | 结果 |
| --- | --- | --- |
| 定向 RNE GPU | `cargo +msvc test --locked --all-features --test resident_rne_acceleration -- --ignored --test-threads=1` | **2 过 0 败**（`computes_...`、`qacc_writes_...`；`failed_world_field_blocks=0`，此前 521；scalars=900288；测试墙钟≈14.5s、报告 elapsed_ms=12834）。日志 `logs/targeted-rne-gpu-debug.log`，报告 `logs/full-numerical-after-rebaseline.json` |
| 全量 all-features | `cargo +msvc test --locked --all-features --no-fail-fast` | **709 过 0 败 346 ignored**（78 个结果行，EXIT=0）。日志 `logs/all-features-msvc.log` |
| all-probes debug（等价命令，复刻脚本 106–215 行） | 见 `logs/all-probes-summary.json` | cargo 本体 **336 过 0 败**（78 行）；脚本口径含 collision-mocks 自检 4 过 → **79 行、340 过 0 败**（`-Minimum 335` 满足，退出码 0） |
| 预验证（落盘前） | scratch worktree `target/wt-rne`（HEAD `1d6ed1e` + 同款重基线产物） | 定向探针 0 失败块（elapsed 15.76s）；非 ignored 的 manifest/容差冻结测试通过 |

all-probes 等价命令的环境接线（t4_history/frozen_upstream/frozen_warp/sparse_preflight/collision_mocks）全部 wired，快照在 `logs/all-probes-env-wiring.json`；排除清单与脚本一致，快照在 `logs/all-probes-exclusions.json`，本任务未改动脚本与清单。

日志说明：`all-features-msvc.log`、`targeted-rne-gpu-debug.log`、`native-stage-prewarm.log` 首次捕获因编码问题已用**同一命令**重新捕获（结果不变：709/0/346、2 过 0 败 failed_blocks=0、native 27 过），并对全部日志去除行尾空白以通过 `git diff --check`；all-probes 日志未重跑，完整版保存在 gitignored 的 `target/g02-rebaseline-logs/all-probes-debug-full.log`，`logs/all-probes-result-lines.txt` 是 78 行结果摘要。

## 边界与未验收

- **不宣称完整 G02 验收**：release 双模式最终验收由 QA 执行；本任务仅跑 debug。Linux/性能未测。
- 产品与上游仍存在 84/240 条 ULP 级差异（首差 `cacc`，最大 normalized 0.743<1，远低于门槛）；若要求与上游逐位一致，应另立「RNE 加速路径逐位对齐」任务，不混入本次口径裁决。
- 本次只改期望值/哈希/证据与 `docs/three-layer-acceptance.md` 的 RNE 段落；未改容差、断言、产品代码、脚本与排除清单；未伪造参考。
- `fixtures/rne-acceleration/reference.cpp` 保留（继续作为输入与历史 f64 参考的作者工具），其内容与 manifest 哈希未变。

## 证据索引

- `rebaseline.json`：重基线映射、旧/新哈希、上游证据 SHA256、工具 SHA256、生成时间。
- `rebaseline-fixture.ps1`：生成/复核工具。
- `historical-f64/`：旧 f64 参考归档 + `README.md`。
- `logs/`：定向测试、all-features、all-probes、collision-mocks、env-wiring、exclusions、summary。
