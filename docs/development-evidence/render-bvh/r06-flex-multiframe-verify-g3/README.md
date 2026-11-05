# R06 多帧验证窗 G-3：真冻结 28 条 + 零编译跑 GPU 多帧实体（PASS）

- 任务：`2026-11-05-render-r06-flex-multiframe-verify-g3`（owner Backend、p82、P8-渲染与BVH）
- 三事实：QA 复核 B PASS = `8c9e7192`（0 阻断）；批准时间 **2026-11-05 22:36**；执行者/唯一写者 = **Backend**（Arch 只读）
- 前置：实体卡 A verified（`2a27c2fd`/`241d702f`）+ 小修卡 `2026-11-05-render-r06-flex-multiframe-entity-fix` verified（`a2d7d4c5`）+ QA 复核 B PASS
- **历史原文说明**：实体卡 A 的 intent/acceptance #4 为**历史原文**（normify 冻结不可改动，原文仍写「按 spec §2 用偏移模型建 3 个 context」）；选项 A 的**当前口径**以实体卡 addenda `a175c7f8` + 小修卡 acceptance #2 + 两份 README（实体/小修）为准。
- **实体文件冻结**：窗口期未修改 `tests/render_flex_multiframe.rs`（499）、`tests/support/render_bvh/multiframe.rs`（496）、`tests/support/render_bvh/allowlist.rs`（210）；`unchanged tests/`；500 行棘轮未动。

## 1. 判定（G-3 判据门）

- **PASS**：冻结 exe 的 GPU `#[ignore]` 用例**单次**通过 —— `running 1 test`、`1 passed; 0 failed`、exit 0、wall 2.76 s，日志 `r06-multiframe PASS: 3 frames, per-frame + cross-frame + option-A 3-context audit`。
- 该 3 帧集逐帧口径 + 跨帧可见性成立：frame-000 allowlist 像素 `(134,128)` `ulp=3`（≤4 且 `beyond4=0`，参考 `flexid=0`）；frame-001/002 严格 bit-first 全过；逐帧两跑整块多相机缓冲 sha 相同；跨帧 **cam2-only** 变化 `000→001 = 1167/1074`、`001→002 = 1262/1126`（均 ≥384）；`refits()==0`；选项 A 三 context 刚性签名一致 + 同 context 渲染对照。
- **非声明**：只覆盖该 3 帧集逐帧口径 + 跨帧可见性；**不写** R06 整族通过 / 动画一致性已覆盖 / 与上游实现的等价结论 / 发布依据。历史原文说明与实体文件冻结按卡面第 2/3 条写入本材料。

## 2. 冻结（窗口前完成，窗口内零编译）

| 项 | 值 |
| --- | --- |
| 条目 | **28/28**（2 exes + 26 inputs；`classification=input=25,guard_script=1`） |
| `aggregate_inputs_sha256` | `b8110cee547a6a6a12dbc4b8201f1571bebb31f7450e2d252f0a4015484afb7f`（与 entity 卡 pending 一致） |
| `aggregate_sha256` | `6a6cca42554967291c29b5030ffc3f98e50581a7956bea8fbf51e0df6e144978` |
| guards 文件 | sha256 `64bfecdfd179c4b7160c4f853660671ec9fb570b4d6c7702aa6c35c2c3cb352d` / 11,899 B |
| attempt / status | `1` / `frozen`（exe 不再 `pending_freeze`） |
| guards generated_at | 2026-11-05T22:39:25.462+08:00 |
| freeze 完成 | 2026-11-05T22:39:25.539+08:00 |

| 条目 | sha256 | bytes | LastWriteTime（同刻记录） |
| --- | --- | --- | --- |
| `mjwarp-rs`（bin） | `a75cf7db8393c5990c4b98beafdf61af885530aa757ff2964314d1d27ebbf38b` | 85,041,741 | 2026-11-05T22:37:39.661+08:00 |
| `render_flex_multiframe`（test） | `0f76f9e14d1e0bd685f6331655e8b41c18ef6e38a97c0d3d03c4c8365412b0f2` | 118,098,954 | 2026-11-05T22:37:41.381+08:00 |

- 构建（窗口前，一次性）：`cargo build --locked --features cuda-probe`（bin，Chief token 命令）+ 冻结器同口径
  `cargo build --locked --bin mjwarp-rs`（默认 feature 含 `cuda-probe`，产物同 sha）+ 
  `cargo test --locked --features cuda-probe --no-run --message-format=json`（test exe）；构建指纹记录时刻
  2026-11-05T22:39:08.311+08:00。
- **等价核验 live==guard==stash 2/2**（逐条 sha/bytes 相同）；freeze 后**零编译/零 relink**：构建指纹与
  guards 记录的 sha 逐字节一致，窗口内 build 进程 before/after = 0/0（见 `artifacts/freeze-verify.json`、
  `artifacts/entity-exe-identity.json`）。
- 证据分别记 **freeze 时刻**（22:39:25）与**窗口时刻**（22:46:50–22:46:53）：freeze 在窗口前完成、窗口内零编译。

## 3. 基线 / monitor / 窗口时序

| 阶段 | 时间（+08:00） | 结果 |
| --- | --- | --- |
| 三点基线 | 22:39:40 / 22:39:55 / 22:40:10 | free 13,726 / 13,727 / 13,696 MiB ≥ **floor 11,688**；util 37/36/38% <40；compute_filtered=0；build=0 |
| floor | — | `max(min_free − 3072, 11,688) = max(10,624, 11,688) =` **11,688 MiB**（取严） |
| monitor | 22:40:31.770 → 22:46:33.228 | 72 samples，**longest_clean=60 = required**，ready=true，exit 0；`util_min=6 / util_max=58`（58 出现在干净 streak 之前的样本），`free_min=13,442` ≥ floor |
| 窗口（实体） | 22:46:50.555 → 22:46:53.322 | 冻结 test exe 单次、exit 0、wall 2,764 ms |
| post-GPU | 22:46:53.631 | 2,282 MiB used / 13,780 free / 5% util / 29 °C |

- 基线逐点满足 free ≥ floor、util < 40、无本任务 CUDA 任务、build=0；树状态仅 `guards-r06-multiframe.json`
  一处声明内变更（freeze 产物）+ 本证据目录（窗口写前 clean）。
- 窗口内零编译：live/stash exe sha 前后不变，`build_procs_before=0 / after=0`。
- monitor CSV：`logs/monitor-g3.csv`（72 行样本，显式 `-OutCsv target/r06-multiframe-verify-g3/monitor-g3.csv`，窗口后复制）。

## 4. 逐帧逐相机结果（冻结 exe 日志原文）

| 帧 | 相机 | depth `(mismatched, beyond4, max_ulp)` | 说明 |
| --- | --- | --- | --- |
| frame-000 | cam0 | `(0, 0, 0)` | 严格区全位等 |
| frame-000 | cam1 | `(0, 0, 0)` | 严格区全位等 |
| frame-000 | cam2 | `(1, 0, 3)` | 仅 allowlist 像素 `(134,128)` 差 3 ULP；`bits=0x400015ba expected=0x400015b7 ulp=3 beyond4=0 flexid=0` |
| frame-001 | cam0/cam1/cam2 | 严格 bit-first 通过 ⇒ `(0, 0, 0)` | 无打印（严格断言：逐像素 f32 位级相等） |
| frame-002 | cam0/cam1/cam2 | 严格 bit-first 通过 ⇒ `(0, 0, 0)` | 同上 |

- rgb（逐帧逐相机）：断言口径 `max_byte_diff ≤ 1` 且 `beyond_1_lsb_pixels = 0`，全部通过。
- 两跑：frame-000/001/002 各渲染两次，整块多相机缓冲 rgb/depth/seg sha256 相同；`refits=0`（每帧）。
- 跨帧（cam2-only，验收口径）：`000→001` depth 1,167 / seg 1,074；`001→002` depth 1,262 / seg 1,126；
  全相机合计同为 `1167/1074`、`1262/1126`（仅诊断打印，不参与判定）。
- 说明：冻结实体**不输出** rgb 的精确 `max_byte_diff`/变化像素数（实体无该仪表；窗口内禁止改实体文件，
  且 499 行仅剩 1 行余量）。上表数值以日志可得项 + 断言语义（strict 通过 ⇒ 全 0）为准。

## 5. 参考 15 文件校验（冻结 exe，窗口前，无 GPU）

- `reference_integrity_15_files`：exit 0、`running 1 test`、`1 passed`；12 个 npz sha256 对齐
  `capture.json.files_sha256` + 3 文本夹具 sha256 锚定 = **15 文件**（`artifacts/reference-integrity.log`）。

## 6. 门禁 / 边界 / 释放

- 本卡无 Rust 改动；`unchanged src/, tests/, fixtures/, scripts/authoring/`，
  `unchanged docs/.../criterion/`（spec `1d9fa5ce…`/10,506 B）、实体证据目录、小修证据目录。
- 写范围仅 `scripts/validate/render-bvh/guards-r06-multiframe.json` + 本证据目录 + 卡/runs；
  未改判据/阈值/allowlist/脚本、未重采、未返工数据。
- 实体文件 499/496/210 窗口期零改动；`render_flex_validate.rs` 498 行零 diff。
- 收尾释放写锁；GPU 已无本任务进程（post-GPU 记录）。

## 7. 已知项（QA 复核 B 的 observation，不阻断、不影响判据与数值）

O1 全相机函数在 CPU 自证⑦参与确定性断言（非判据断言）；O2 裁决字段路径 `criteria.option_a` vs 顶层
`option_a`；O3 漂移 panic 缺 `flat`（strict 路径仍有 flat）；O4 `refits()==0` 只在渲染前断言；
O5 freeze 脚本抛错文案「expected 16」（实际检查 28）；O6 实体 receipt 非声明句含「与上游实现等价」原词；
O7 实体卡历史原文旧短语（不可改写，裁决已留痕）。验证窗后按 Chief 口径决定小修或 backlog。

## 8. 证据清单

| 路径 | 说明 |
| --- | --- |
| `README.md` | 本文件 |
| `receipt.json` | 机器可读回执（`/task_id` 供 normify `json` 检查） |
| `logs/entity-gpu.log` | 冻结 exe GPU 运行原始输出（command/env/start/end/wall/exit + 逐帧打印） |
| `logs/monitor-g3.csv` | monitor 72 样本原始 CSV（60/60 干净 streak） |
| `artifacts/build-fingerprint.json` | 窗口前构建同刻指纹（两 exe 路径/sha/bytes/LastWriteTime） |
| `artifacts/build-summary.txt` | 构建输出摘要（三次 cargo 命令） |
| `artifacts/freeze.log` | 冻结器原始输出（`GUARDS_OK=28/28 …`） |
| `artifacts/freeze-verify.json` | 冻结等价核验（live==guard==stash、aggregate 复算、guards sha） |
| `artifacts/guards-frozen-snapshot.json` | 冻结 guards 文件快照（**LF 规范化副本**；原文件字节 sha256/bytes 与 aggregate 见 `freeze-verify.json`） |
| `artifacts/baseline.txt` | 三点基线原始输出 + floor 计算 |
| `artifacts/reference-integrity.log` | 冻结 exe 的 15 文件参考完整性 CPU 校验 |
| `artifacts/entity-exe-identity.json` | 实体运行前后 exe 指纹（零 relink、build 进程 0/0） |
| `artifacts/post-gpu.txt` | 窗口后 GPU 状态 |
| `artifacts/git-after-freeze.txt` | freeze 后 `git status` 留痕（仅 guards JSON） |
| `manifest.txt` | 本目录 sha256 + bytes（首行 `#`，不自列） |

## 9. 复现

```powershell
# 冻结（窗口前；需 token）
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/render-bvh/freeze-guards-r06-multiframe.ps1 -Attempt 1
# 基线 + monitor（窗口前 5 分钟 / 窗口）
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/render-bvh/monitor-until-clean.ps1 -OutCsv target/r06-multiframe-verify-g3/monitor-g3.csv -Required 60 -MaxMinutes 12 -MaxUtil 40
# 窗内零编译跑冻结 exe（单次）
$env:CUDA_PATH='D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1'; $env:PATH="$env:CUDA_PATH\bin;$env:PATH"
target/render-bvh/r06-multiframe-window/frozen-exes/render_flex_multiframe.exe --ignored --test-threads=1 --nocapture
```
