# R06 多帧 refit 修复 r2：每帧强制 refit + 三条硬自查 + provenance（authoring-only）

- 任务：`2026-11-05-render-r06-flex-multiframe-refit-fix-r2`（owner Backend、p70、P8-渲染与BVH）
- 依赖：H1 诊断卡 `2026-11-05-render-r06-flex-multiframe-pose-diag`（提交 `77aeb6da`）——A/B 定论
  「采集脚本帧循环缺 `mjw.refit_bvh(m, d, rc)`」；r1 卡 `2026-11-05-render-r06-flex-multiframe-refit-fix`
  （登记 `251d2c5b`、start `0ee85384`）因卡面缺 Chief 4 条硬要求与 spec 漂移审计，已 abandon（`b95c33b6`）
  并重登记本 r2（一次写对）。
- 边界：**authoring-only、零 GPU、不执行采集、不开窗口、不写 `fixtures/`、不改判据/阈值/帧设计/相机/spec**；
  单帧主脚本 `capture_render_flex.py` **逐字节不变**（498 行）。

## 1. 结论

| 项 | 结果 |
| --- | --- |
| 每帧强制 refit（**两遍采集逐帧都做**） | 已实现：`_render_frame` 在 `mjw.forward` 后、`capture_once` 前**无条件** `mjw.refit_bvh(m, d, rc)`；`--refit-bvh` 开关与「默认关」路径已删除（未知参数按 argparse exit 2） |
| refit 计次断言（防 `--repeat 2` 第二遍漏 refit 的假通过） | 计次必须 = `帧数 × 遍数 = 3 × 2 = 6`，顺序 `[0,1,2,0,1,2]`，无分支可跳过（CPU 自检 B1–B3） |
| 三条硬自查（采集路径内建，失败 **exit 3** 停手） | 已实现（拆到新模块 `capture_render_flex_multiframe_checks.py`）：base 幂等 / 位移 = k×0.100 m（±1e-6，读采集侧 state）/ 两段变化像素均 ≥576 |
| provenance | `capture.json.provenance`：`spec_sha256`（`1d9fa5ce…`/10,506 B）+ `authoring_script_sha256`（修复后三脚本）+ refit 标记 |
| spec 漂移 | `spec.md` sha/bytes **未变**（`1d9fa5ce…`/10,506 B），不触发用户重签；G-2 沿用 S-1 签名 |
| 500 行棘轮 | 主模块 493→**427**、新自查模块 **296**（均 ≤500）；单帧主脚本 498 **零 diff** |
| CPU 自检（零 GPU） | **28/28 PASS**（`scripts/verify_refit_fix.py`，见 §6） |

## 2. 改动清单（diff 见 `artifacts/refit-fix.diff`）

| 文件 | 行数（HEAD→now） | 说明 |
| --- | --- | --- |
| `scripts/authoring/capture_render_flex_multiframe.py` | 493 → **427**（+44/−110） | 帧循环无条件 refit + 计次记录；采集末尾调用三条硬自查并以 `checks.ok` 决定 exit 0/3；`capture.json` 增 `provenance`/`capture_checks`/`refit_calls`；delta 自检函数整体迁出（见下） |
| `scripts/authoring/capture_render_flex_multiframe_checks.py` | **新增 296** | 自检模块：refit 计次、三条硬自查、delta 自检（阈值 384/576、打印、退出码）、provenance |
| `scripts/authoring/README.md` | 57 → **99**（+43/−0） | 新增「为什么多帧必须每帧 refit」与「采集路径三条硬自查」两节 + spec 漂移审计 + 非声明 |
| `scripts/authoring/capture_render_flex.py` | 498 → **498（0 diff）** | 单帧路径逐字节不变 |

## 3. 三条硬自查（采集路径内建；任一失败 exit 3 停手）

| # | 检查 | 规则 | 数据源 |
| --- | --- | --- | --- |
| ① | base 幂等 | `frame-000` 的 rgb/depth/seg/state 与 `fixtures/render-bvh/scene-flex/reference/base/` **数组级位级全等**（15 个数组） | 内存采集结果 vs base npz |
| ② | 位移断言 | 附着到非 world body 的顶点相对 base 帧**平均位移 = k×0.100 m**（k=0,1,2；容差 ±1e-6 m） | **采集侧 `state.npz.flexvert_xpos`（设备回读）**，非 CPU `mjd` |
| ③ | 变化余量 | `000→001` 与 `001→002` 的 depth 与 seg 变化像素**均 ≥ 576**（=1.5×384，pass-margin 口径） | cam2 相邻帧 depth 位级 / seg 对比较 |

失败语义：三项结果与 `refit` 计次一起写入 `capture.json.capture_checks`；任一 `ok=false` 时脚本打印
`R06 capture self-check FAILED: <项>（不得落库，exit 3）` 并以 **exit 3** 结束（不再有「静默成功」路径）。
另：`frame-000 == base` 的位级对照同时补上 G-1 QA 观察 **O7**（「frame-000 vs base 动态证据」）的缺口。

## 4. refit 计次断言（口径写死）

- 期望：`frames × passes = 3 × 2 = 6`（**两遍 `--repeat 2` 采集都计**）；顺序必须 `frame 0,1,2` 重复两遍；
- 反例（CPU 自检覆盖）：只记一遍（3 次）→ FAIL；乱序 `[0,2,1,0,1,2]` → FAIL；
- 实现：`record_refit(frame)` 在 `mjw.refit_bvh` 调用处逐帧记录；`capture_multiframe` 开头 `reset_refit_log()`，
  采集末尾 `run_capture_checks` 内 `refit_summary(3, 2)` 判定；主模块无 `if refit` 分支、无开关形参（AST 自检 A1/A2）。

## 5. provenance（`capture.json.provenance`）

```json
{
  "spec_sha256": "1d9fa5cef1ebc829747d63f648d3bab1ec935d8df8e3403bc52c2ac2aa7c40b6",
  "spec_bytes": 10506,
  "authoring_script_sha256": {
    "capture_render_flex.py": "…",
    "capture_render_flex_multiframe.py": "…",
    "capture_render_flex_multiframe_checks.py": "…"
  },
  "refit_policy": "every-frame",
  "refit_bvh": true
}
```

供 QA 区分采集轮次（修复前后脚本 sha 不同）并证明同一签名对象（spec sha 不变）。
`capture.json` 另有 `refit_policy` / `refit_calls: 6` 与 `capture_checks`（逐项自查结果）。

## 6. CPU 自检 28/28（零 GPU、不采集）

`scripts/verify_refit_fix.py`（可复跑）→ **`RESULT: PASS (28/28)`**，机器可读结果 `logs/verify-refit-fix.json`：

| 组 | 内容 | 证据 |
| --- | --- | --- |
| A（静态/AST） | `_render_frame` 内无条件 refit 且在 `capture_once` 之前；无 `if refit`/开关形参；无 `--refit-bvh`；主模块 427 ≤500；自查模块 296 ≤500；单帧主脚本零 diff | `logs/verify-refit-fix.json` A1–A6 |
| B（refit 计次） | 6 次且顺序正确 → PASS；只记一遍（3 次）→ FAIL；乱序 → FAIL | B1–B3 |
| C（三条硬自查） | base 幂等 PASS / 改 1 bit FAIL；位移精确 k×0.1 PASS / +1e-3 FAIL / +5e-7 边界 PASS；变化余量 600 PASS / 500 FAIL / 300 FAIL；`run_capture_checks` 全 PASS 与失败项检出 | C1–C10 |
| D（provenance） | spec sha/bytes 与常量一致；三脚本 sha 与实际文件一致；`refit_bvh=true` | D1–D2 |
| E（CLI） | `--help` exit0 且无 `--refit-bvh`；传 `--refit-bvh` exit2；`--dry-run` exit0、不写产物、`refit_policy=every-frame`/`refit_calls=6`；`--check-deltas` 合成三态 0/3/3 | E1–E5，`logs/dry-run.log`、`logs/check-deltas-synth-*.log` |

注：真实采集路径的端到端行为（真实 GPU 帧循环 + 自查通过/停手）需 G-2 采集窗口验证；本卡执行器为
Backend（Arch 已转只读），窗口 token 由 Chief 在 QA 复核 PASS 后发放。

## 7. spec 漂移审计

判据 spec §8.2 只写 `put_model → forward → 渲染`、未提 BVH；本修复补齐隐含前提「渲染必须消费当前
state（几何变动后 BVH 需 refit）」，属于实现层修正：**不改判据文本/阈值/帧设计/相机/输出布局**，
`docs/development-evidence/render-bvh/r06-flex-multiframe-criterion/spec.md` 的 sha256/bytes 保持
`1d9fa5ce…` / 10,506 B 不变 → **不触发用户重签**，G-2 采集窗沿用 S-1 签名。

## 8. 与 G-1 负结果的关系（supersede 预告）

G-1 采集窗（`2026-11-05-render-r06-flex-multiframe-capture`，提交 `dbe8d2b7`）的 `non-discriminating`
结论在**旧脚本（无每帧 refit）**下成立，但其原因已由诊断卡 A/B 定为「渲染消费链缺 refit」，而非帧集设计。
本修复落地后，G-2 采集窗将用**同一签名 spec / 同一帧集**重采：若自查全过（base 幂等、位移精确、
两段 ≥576、refit 6/6），G-2 证据将写明「采集有效 → 可进入验证窗 G-2」，并**显式 supersede** G-1 的
「帧集不足」归因；G-1 负结果证据保留不删。

## 9. 边界与非声明

- 零 GPU、未执行采集、未开窗；未写 `fixtures/`（`git status --porcelain fixtures/` 为空）；未改
  `src/`、`tests/`、`Cargo.*`；未改判据/阈值/帧设计/相机；spec sha 未变。
- 本卡只声明「采集基础设施修复完成 + CPU 侧自检通过」；不声明多帧判据已通过、动画一致性覆盖结论、
  与上游实现的等价结论或任何发布依据；真实采集有效性由 G-2 窗口判定。

## 10. 证据清单

| 路径 | 说明 |
| --- | --- |
| `README.md` | 本文件 |
| `receipt.json` | 机器可读回执（`/task_id` 供 normify `json` 检查） |
| `artifacts/refit-fix.diff` | 本次 `Normify-Fixes` 修正提交的 `git diff HEAD -- scripts/authoring/`（纯文本路径修正；完整实现 diff 见提交 `ff8135e5`） |
| `artifacts/script-identity.json` | 四脚本本次修正 HEAD→now 的行数/sha256（行数不变：498/427/296/99） |
| `scripts/verify_refit_fix.py` | CPU 自检 28 项（可复跑） |
| `logs/verify-refit-fix.json` | 28 项逐条结果（`RESULT: PASS`） |
| `logs/line-counts.txt` | git blob 口径行数（498 / 427 / 296） |
| `logs/dry-run.log` | `--dry-run` 原始输出（refit_policy/refit_calls=6，spec sha 校验） |
| `logs/check-deltas-synth-{pass,insufficient,non-discriminating}.log` | `--check-deltas` 合成三态 0/3/3 |
| `manifest.txt` | 本目录 sha256 + bytes + 证据目录内相对路径（base dir 见表头，不自列） |

## 11. 复现

```powershell
# CPU 自检（零 GPU；仓库根，pinned venv）
target/mjwarp-parallel/venv/Scripts/python.exe -B docs/development-evidence/render-bvh/r06-flex-multiframe-refit-fix-r2/scripts/verify_refit_fix.py
# 单点：dry-run 观察 refit_policy/refit_calls
target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/authoring/capture_render_flex.py --frames 3 --frame-dz 0.1 --out-dir target/r06-multiframe-refit-fix-dry/plan --dry-run
```

## 12. G-2 采集窗操作提醒（Arch 只读旁证 2026-11-05；登记 G-2 卡面时并入 acceptance）

1. **exit 3 失败路径会先写出产物再判定**：采集路径的顺序是「逐帧落盘 npz → 写 capture.json/README →
   跑三条硬自查 → 各自判定 → exit 0/3」。因此 `exit 3` 时 `--out-dir` 里**已有** npz/capture.json 残留。
   G-2 窗口流程必须在判定失败后**立即删除该 out_dir 残留**，并在窗口证据里留痕（`git status`/目录不存在
   的检查输出）；不得把失败轮残留误当参考材料（G-1 先例：窗口产物删除后 `git status fixtures/` 为空并记录）。
2. **`check_frame000_vs_base` 只比对第一遍，不重复做两遍一致性**：该硬自查只拿第一遍的 frame-000 与
   `reference/base/` 做位级对照；「`--repeat 2` 两遍采集逐帧逐文件 sha 一致」由既有 `first["files_sha256"]
   != second["files_sha256"]` 比对负责（不一致直接 fail）。两者职责不同、**不要叠改或互相替代**；G-2 证据
   分别记录两项结果即可。

## 13. 证据修订记录（2026-11-05，`normify fix`）

Arch 只读旁证第⑦条指出两处文档引用路径错误（**纯文本修正，不动行为/阈值/逻辑**；由本 `Normify-Fixes:` 提交落地）：

| 文件 | 修订前 | 修订后 |
| --- | --- | --- |
| `scripts/authoring/README.md`（原 :64） | `mujoco_warp/_src/record.py:110-111` | **`mujoco_warp/record.py:110-111`** |
| `scripts/authoring/capture_render_flex_multiframe_checks.py`（docstring） | `mujoco_warp/_src/record.py:110-111` | **`mujoco_warp/record.py:110-111`** |

- pinned `71da24d9` 的 `mujoco_warp/` 包内无 `_src/record.py`（`record.py` 在包根）；`contrib/render.py:452-453`
  一处原本正确、未改。
- 同笔修正还新增本节 §12（G-2 操作提醒）；行数与行为不变（主模块 427 / 自查模块 296 / 单帧主脚本 498），
  `manifest.txt` 与 `artifacts/script-identity.json`、`artifacts/refit-fix.diff` 已按修正后文本重算。

