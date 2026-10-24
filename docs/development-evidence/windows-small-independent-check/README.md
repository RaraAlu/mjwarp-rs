# Windows 小项交付独立复核（a10-p11 + U038，快验口径）

任务：`2026-10-24-windows-small-review`。日期：2026-10-24。执行人：QA。
复核 HEAD：`c0ac14c`（a10-p11 `245c461`、U038 `c0ac14c` 均已 verified）。
范围：**快验口径**——`scripts/g01-fast.ps1` 受影响 target + 定向测试 + 秒级 GPU 探针 + 只读脚本复算；**本轮未跑全量 G01**。
最近一次全量双模式全绿：**HEAD `4e07852`**（`docs/three-layer-acceptance.md` §4.13：debug/release 各 14/14 阶段 exit 0、host-msvc 835、all-probes 353/0、87 result 行、11 项排除、rustdoc 0）。
下一轮全量 G01 仅记录**预期**（未实测）：host-msvc 835→**840**（a10-p11 新增 5 单测）、all-probes 353→**354**（新增 1 项 ignored GPU 探针自动纳入）；本轮不宣称计数已达成。

## 结论（先给）

1. **a10-p11（`245c461`）交付声明成立**：CUDA event 计时与显存采样可用；独立复跑 3 次（debug×2 + release×1）两阶段中位数**均为 7168 ns**，显存检查点与证据**逐字节一致**（空闲 1264.5 MiB → 工作集 1296.5 MiB，+32 MiB → 释放回基线；阶段内峰值 = 基线）；debug/release 分档、记录 JSON 自解析校验与确定性均成立；不改 P10 语义/物理/容差/断言，新文件均 ≤500 行并已挂 `mjwarp.runtime.tooling`，门禁记录 unit 485/0（新增 5 项）。
2. **U038（`c0ac14c`）交付声明成立**：`git diff 3b22f5f..c0ac14c -- src tests fixtures Cargo.toml Cargo.lock` **为空**（无需 T4 repin）；run.json 为 **1477 项 / 122 位差 / 8 超差 / max 4.291534423828125e-6**，容差仍为 `2e-6`（f32 位 `0x360637BD`，测试常量未改）；`gravity_operator` 77/77 **0 超差**；首差结论（w2 body3 父姿态 rotate 2 ULP、w6 multiply/xmat、subtree_com 累加顺序、cdof 55 位差）与证据一致；有界两轮尝试（run7/run8）与三个选项（A/B/C）均记录在案；**未宣称 U038-3D 通过**，维持已知债务。
3. **两个文档建议项（不影响结论）**：a10-p11 证据/提交信息中的行数标注（462/197/256/429/183/79）与磁盘实测（491/220/275/445/198/84）不符（仍全部 ≤500）；设备总量标注 16,376.5 MiB 与实测 16,375.5 MiB 差 1 MiB。

## A. a10-p11（`245c461`）逐项核对

### 声明数字 vs 独立实测

| 项 | 交付声明 | 本轮独立实测 | 判定 |
| --- | --- | --- | --- |
| 温机 / 重复 | 10 / 50 | program 记录 `plan.warmup=10`/`repetitions=50`（3 份记录一致） | 一致 |
| 两阶段两档中位数 | 均 7168 ns | debug run1/run2、release 各：`affine_probe#1`/`#2` 中位数均 **7168 ns** | 一致 |
| min / max | min 6144–6560；max 7200–8192 | 独立实测 min 6144；max 7328–8192（事件分辨率噪声内） | 一致（区间内） |
| mean / total | 7059.84/7114.88（debug）、7106.56/7077.76（release） | 独立实测 6983.68/7061.12、7082.24/7074.56（debug）、7021.44/7091.20（release） | 量级一致（逐次运行正常波动） |
| 设备总量 | 16,376.5 MiB（17,170,956,288 B） | 记录值 17,170,956,288 B = **16,375.5 MiB** | **标注差 1 MiB（建议项）** |
| 空闲 / 工作集 / 释放 | 1264.5 → 1296.5（+32）→ 1264.5 MiB | 1,325,924,352 → 1,359,478,784（+33,554,432 = 32 MiB）→ 1,325,924,352 B，三次运行逐字节相同 | 一致 |
| 阶段内峰值 = 基线 | 是 | `records[].memory.baseline=peak=end=1,359,478,784` | 一致 |
| debug/release 分档 | 有 | release 独立运行记录 `environment.profile=release`，中位数 7168/7168 | 一致 |
| 记录 JSON 可回读 | serde_json 回读校验 | 探针写盘前 `to_json` 校验 + 自解析；单测 `report_json_round_trips_and_escapes_text` 断言两次导出逐字相等 | 一致 |

- 独立记录：`a10-probe-debug-run1.json`、`a10-probe-debug-run2.json`、`a10-probe-release.json`；g01-fast 汇总：`fast-loop-lib-tooling.json`、`fast-loop-probe-run1.json`、`fast-loop-probe-run2.json`。
- lib tooling 定向测试：`passed=10 failed=0`（新增 5 项：`build_profile_tracks_build_mode`、`plan_requires_warmup_and_repetitions`、`report_json_round_trips_and_escapes_text`、`report_validation_rejects_inconsistent_records`、`timing_stats_aggregate_matches_p10_recorder`）。
- GPU 探针：g01-fast debug 两次各 1 过 0 败；release 一次 1 过 0 败（墙钟 10.6s，含 release 构建）。

### 实现与边界核对

- 提交范围：仅 `src/runtime/tooling/{perf.rs,gpu.rs,mod.rs,tests.rs}`、`tests/a10_p11_gpu_perf_probe.rs`、`scripts/test-windows-a10-p11.ps1`、模块索引、证据与任务登记；**无** `src/physics`/`tests/support`/`fixtures`/`Cargo.*` 改动 → 不碰物理/容差/断言，未重做 P10。
- P10 语义：`mod.rs` 仅新增模块导出与错误枚举，P10 的 `BenchmarkRecorder`/`measure`/`StateExport` 保留；`timing_stats_aggregate_matches_p10_recorder` 通过；unit 门禁 485/0。
- 新文件行数（磁盘实测）：`perf.rs` **491**、`gpu.rs` **220**、`tests/a10_p11_gpu_perf_probe.rs` **198**、`scripts/test-windows-a10-p11.ps1` **84**，全部 ≤500。**证据标注 462/197/183/79 偏旧（建议项）**。
- 模块挂载：`.normify/modules/mjwarp/runtime/tooling.md` 已列 `perf.rs`、`gpu.rs`、`tests/a10_p11_gpu_perf_probe.rs`。
- 门禁记录：`.normify/runs/2026-10-11-roadmap-a10-p11/` fmt/clippy/clippy-cuda 通过、unit **485 过 0 败**。
- 边界如实：固定负载是 `affine_probe` 微基准、非物理阶段；不做性能门槛/main.rs CLI/Linux/双平台；显存为 `cuMemGetInfo` 采样口径。

## B. U038（`c0ac14c`）逐项核对

### 不变性与数字

- **产品/测试/fixtures 零改动**：`git diff 3b22f5f..c0ac14c -- src tests fixtures Cargo.toml Cargo.lock` 为空；改动仅在证据、文档、模块索引、任务登记与只读脚本 `scripts/authoring/compare-u038-first-diff.ps1` → unchanged 成立、无需 T4 repin。
- **run.json（本次实测复跑结果）**：`total_compared_values=1477`、`total_bit_differences=122`、`total_outside_tolerance=8`、`max_absolute=4.291534423828125e-6`；字段级与交付声明及历史 run5 逐项相同（xpos 4、xquat 7、xmat 17、xipos 4、xanchor 3、xaxis 4、subtree_com 9、cdof 55、qfrc_gravcomp 19 位差/8 超差）。
- **容差未改**：`absolute_tolerance=1.9999999949504854e-6`（= f32(2e-6)）；测试常量 `TOLERANCE_BITS = 0x360637bd`（`tests/t4_gravity_3d_product_chain.rs:30`）与 f32(2e-6) 位型一致；`git diff` 证明未改。
- **重力算子 vs 传播**：`gravity_operator` 77/77 **0 超差**（1 位差、max 1.19e-7）；`gravity_source_propagation` 77/77 **8 超差**、max 4.29e-6。
- **身份**：`package_sha256=95b6818d…` 与 repin-12 重钉值一致；run.json 身份哈希段完整。

### 首差与因果（只读脚本复算）

用 `scripts/authoring/compare-u038-first-diff.ps1`（只读、不跑 GPU）以已提交原始位文件复算，与 `debt-2/first-diff.json` **实质内容逐项相等**（`fields`/`gravity`/`world_diff_matrix`/`gravity_force_magnitudes`/合计全部 equal；仅 provenance 输入路径不同）。

| 字段 | 位差 | 首个差异（位置 / 产品位→上游位 / ULP） |
| --- | ---: | --- |
| xpos | 4 | #39 w2 body3 x / `3ecdb974`→`3ecdb976` / 2 |
| xquat_wxyz | 7 | #128 w6 body2 w / `3f3cf802`→`3f3cf803` / 1 |
| xmat | 17 | #288 w6 body2 (0,0) / `3f35d1d8`→`3f35d1da` / 2 |
| xipos | 4 | #39 w2 body3 x / `3eda4472`→`3eda4474` / 2 |
| xanchor | 3 | #30 w2 joint2(hinge, body3) x / `3f038dad`→`3f038dae` / 1 |
| xaxis | 4 | #55 w4 joint2(hinge, body3) y / `bf3e9e5f`→`bf3e9e60` / 1 |
| subtree_com | 9 | #32 w2 body0 z / `3d1418c3`→`3d1418b8` / 11（w3 root z 82 ULP） |
| cdof | 55 | #153 w2 dof3(free body1) linear x / `be1eae9d`→`be1eae9c` / 1 |

- world 矩阵：w2/w3/w6 的运动学差异与 qfrc 差异同域（w2 5 位差/3 超差、w3 7/4、w6 7/1）；w4 仅 xaxis/cdof 少数分量、qfrc 逐位一致；w0/w1/w5 全一致 — 与证据一致。
- 灵敏度：`|f|` = body1 14.83、body2 12.36、**body3 69.21**、body4 12.36；1 ULP cdof × body3 力臂 ≈ 4e-6 量级 → 需受影响量基本逐位一致，而非"减小误差"。
- 有界两轮：run7（源形态改写）8/77 不变、位差 122→123；run8（全显式非融合）8/77 不变、位差 122→147（更差）；记录见 `first-diff-round1.md`/`ptx-order-round2.md` 与 `run7-ptx-round1.json`/`run8-ptx-round2.json`。
- 三个选项（A 系统性对齐 / B 维持已知债务 / C 比较口径裁决）已写入 `debt-2/README.md` §7；本任务不自行选 A/C，维持 B 现状。
- 表述建议项：run.json 的 `gravity_operator` 位差为 1（历史 run5 为 0，均 0 超差），"与 run5 逐项一致"在总量/字段级成立，但该子项由 0→1；`known-deviation.md` 已注明"位差 0~1"，建议后续表述统一为"0 超差、位差 0~1"。

## 未满足项与边界

1. **本轮未跑全量 G01**（按用户新节奏，全量只在里程碑跑）；最近全绿 HEAD `4e07852`（§4.13）；下一轮全量预期 host-msvc 840、all-probes 354 **仅作预期、未实测**。
2. **U038-3D 验收目标未满足**：1477 项 0 超差未达成（8/77、max 4.29e-6），维持用户 2026-10-23 裁决 B 的已知债务；本任务不宣称通过，也不选 A/C。
3. a10-p11 两个文档建议项（行数标注偏旧、设备总量 1 MiB 标注偏差）——不影响工具能力与结论，需后续改证据文案（本任务不改他人证据）。
4. a10-p11 边界保留：非物理阶段、无性能门槛、无 Linux/双平台、显存为采样口径。
5. 未改产品/测试/容差/排除清单/G01 脚本/fixtures；GPU 仅本任务秒级探针（debug×2 + release×1）与既有证据，U038 未重跑 GPU 链（只读复算）。

## 证据索引

- `a10-independent-recheck.json`：a10-p11 三次独立运行数字与声明比对。
- `a10-probe-debug-run1.json` / `a10-probe-debug-run2.json` / `a10-probe-release.json`：独立探针记录。
- `fast-loop-lib-tooling.json` / `fast-loop-probe-run1.json` / `fast-loop-probe-run2.json`：g01-fast 汇总。
- `u038-independent-recheck.json`：U038 零改动、run.json 数字、容差位型、首差复算与有界轮次/选项核对。
- `commands.log`：本轮命令与结果摘要。
- 被复核证据：`docs/development-evidence/a10-p11-windows/`、`docs/development-evidence/u038-3d/debt-2/`；最终状态全量回归：`docs/development-evidence/g03-parity/`（HEAD `c4c1058`）与 `three-layer-acceptance.md` §4.13。
