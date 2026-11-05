# R06 多帧采集窗（G-1）执行记录：结果 non-discriminating、按 spec 停手、未落库

- 任务：`2026-11-05-render-r06-flex-multiframe-capture`（owner Arch，唯一执行会话 arch-chat）
- 日期/时间：2026-11-05 12:00–12:07（12:00–14:00 空闲窗）
- 结论一句话：**采集窗按 G-1 材料执行完毕；两次采集逐位一致，但 `001→002` 相邻帧变化低于采集前置余量门槛 → 脚本 exit 3、按 spec 停手；三帧参考不作为验收材料落库（已删除），结论不构成多帧通过。**

## 1. 签名 / 授权事实（窗口卡面写死）

| 项 | 值 |
| --- | --- |
| S-1 用户签名对象 | `docs/development-evidence/render-bvh/r06-flex-multiframe-criterion/spec.md`，sha256 `1d9fa5cef1ebc829747d63f648d3bab1ec935d8df8e3403bc52c2ac2aa7c40b6` / 10,506 B（2026-11-05 签，冻结；本窗未改 spec） |
| S-2 | 已预批（常规 util&lt;40 不达标时允许 authoring-only util&lt;60）——**本窗未启用**（常规阈值达标） |
| Chief GO | 2026-11-05 08:43；窗口 12:00–14:00 |
| 写范围 | `fixtures/render-bvh/scene-flex/multiframe/` + 本证据目录 + 卡/runs；未碰 `src/`、`tests/`、其它 `fixtures/`、现有单帧参考、冻结 spec、`scripts/` |

## 2. 开窗前基线（5 分钟内，间隔 2.5 s）

| 点 | 时刻 | used | free | util | temp |
| --- | --- | --- | --- | --- | --- |
| 1 | 12:00:26 | 1,730 MiB | **14,332 MiB** | 3% | 27°C |
| 2 | 12:00:28 | 1,730 MiB | **14,332 MiB** | 2% | 27°C |
| 3 | 12:00:31 | 1,730 MiB | **14,332 MiB** | 2% | 27°C |

- **floor = max(min_free − 3,072, 11,688) = max(11,260, 11,688) = 11,688 MiB**（取严）；三点 free 均 ≥ floor、util &lt; 40 ✅
- 构建进程 0（cargo/rustc/cl/nvcc/link）；无本任务 CUDA 计算任务（仅桌面 WDDM 图形上下文）；树 clean（除窗口卡 JSON）。
- 原始输出：`baseline-three-points.txt`

## 3. monitor（开窗前）

- 命令：`monitor-until-clean.ps1 -Required 60 -MaxMinutes 30 -OutCsv target/r06-multiframe-capture/monitor-capture.csv`
- 结果：**samples=60 / longest_clean=60 / ready=True**（12:00:44–12:05:44，util 2–4%，**free_min 14,329 MiB**）
- 人工逐点核：60 行 **free 最低 14,329 ≥ floor 11,688，0 行低于 floor**（脚本内置 10,240 仅为下限，不替代 floor）
- 原始 CSV：`monitor-capture.csv`（本目录副本）
- S-2 例外：**未启用**（常规 util&lt;40 达标）

## 4. dry-run 计划检查

- 命令：`venv\Scripts\python.exe -B scripts\authoring\capture_render_flex.py --frames 3 --frame-dz 0.1 --out-dir fixtures/render-bvh/scene-flex/multiframe --dry-run`
- 结果：exit 0；spec 校验 `1d9fa5ce…`/10,506 B 一致；锚点 `flex-anchor0/1/2` body id `[1,2,3]`、base z `[0.0,0.0,0.0]`；`repeat=2`；expected layout 三帧 + capture.json/cam-poses.json/README.md；**未写产物、未渲染**。
- 原始输出：`dry-run.log`（注意：入口是 `capture_render_flex.py`；`capture_render_flex_multiframe.py` 是模块、无 `__main__`）

## 5. 采集执行与结果

- 命令：`venv\Scripts\python.exe -B scripts\authoring\capture_render_flex.py --frames 3 --frame-dz 0.1 --out-dir fixtures/render-bvh/scene-flex/multiframe --repeat 2`
- 时刻：12:06:38 → 12:06:44（约 6 s），**exit 3**
- 两次采集逐位一致：**true**（`two_captures_bitwise_identical=true`）
- `legacy_reference_untouched=true`（现有单帧参考未被改动）
- **cam2 变化像素自检（阈值：accept 384 / margin 576）**：

| 相邻帧对 | depth changed | seg changed | verdict |
| --- | --- | --- | --- |
| 000→001 | **697** | **616** | pass-margin |
| 001→002 | **244** | **183** | **non-discriminating** |

- **overall verdict = `non-discriminating`** → 按 spec §5 与 GO 条款：**停手、不落库为验收材料、不调阈值、不改步进参数**。
- 原始日志：`capture.log`；`capture.json` 诊断副本：`capture-diagnostic.json`；相机位姿诊断副本：`cam-poses-diagnostic.json`

## 6. 处置：不落库

- 采出的三帧目录 `fixtures/render-bvh/scene-flex/multiframe/` **已删除**，`git status --porcelain fixtures/` 为空；本目录只保留诊断日志与 `capture.json` 副本，**不包含任何参考帧数据**。
- 删除前记录的产物 sha256（诊断用，不构成参考）：

| 文件 | sha256 |
| --- | --- |
| frame-000/rgb.npz | `5ed502872b869bb1f37659ec22816f9f1c04405c1666f3d366b28fd298678207` |
| frame-000/depth.npz | `c3ee4adca6cf9874f40ab87b7f72a16b17f26647e45758342895919b725d8deb` |
| frame-000/seg.npz | `f3b00fa82df1e54d4c89309d820212d9808038e00ae25f25841f2b40a6277bef` |
| frame-000/state.npz | `7f2fff501bfa98f41fb8d4b689337e9e62e1ec54d98c576dc0a6e8b880b61769` |
| frame-001/rgb.npz | `df1345237c46178675b3ef30d5a839798cf9abed1bdd339985b4e3442da2661e` |
| frame-001/depth.npz | `bd5887f2f02f2b4f53ab4929c62cff2240fbf1ac9cdb09e5ee5214a83190a22a` |
| frame-001/seg.npz | `fc18c5c6da2843ee94f3b7f06cf68a5d4bc6ee05d536b329420012b608b4282d` |
| frame-001/state.npz | `2cb3cb84c8101a8d1e8d4a97c16051dea992204c05e0e4f7be99e54a2aaccae2` |
| frame-002/rgb.npz | `9a41e9094ad53d4e631cd607ec969ea851657a13e1e93157d5ce17a976079cdc` |
| frame-002/depth.npz | `f42a28e4d2faeb19ef847d5956905bd268bbff1815ad526942baededd89e037c` |
| frame-002/seg.npz | `cc34db8bb8d45432c45cd5c033128403dc3b9c4cc8d6344c539b3e0787a4b4a5` |
| frame-002/state.npz | `e797fb70fa99cafa169b80e916ab46e24bba5c11b0b81b261f10a41dff98ef68` |

- 先兆（诊断，不替代正式对照）：**frame-000 四个文件 sha 与 `reference/base/` 现有参考完全相同**（rgb `5ed50287…`、depth `c3ee4adc…`、seg `f3b00fa8…`、state `7f2fff50…`），说明无偏移帧与单帧参考同源。
- **frame-000 vs base 的数组级位级对照未执行**：按 spec/GO「自检非 pass-margin 即停手」，本窗在自检处停止，未进入后续对照步骤；该对照需在新 spec 的采集窗内执行。

## 7. 失败归因（诊断级，未改判据）

- `000→001` 变化充足（697/616，均 ≥576）→ 第一步位姿变化把 cam2 覆盖投影位移做出来了。
- `001→002` 变化骤降（244/183，<384）→ 第二步在 cam2 上**投影变化不足**：可能原因（未证实，需新采集验证）：a) 第二步上移后 flex 簇部分移出 cam2 视锥/被相机取景裁切；b) 二次刚性上移对该相机几何上接近沿视线方向，投影位移变小；c) 锚点偏移只移动了部分顶点边界框，遮挡关系变化减小。
- **未做任何参数调整**（Δz/帧数/阈值/相机均未改）；本窗不重试。

## 8. 结论与建议

- **结论**：采集窗按材料与 spec 正常执行；脚本按冻结判据正确判定 `non-discriminating` 并 exit 3；**三帧参考不落库、不构成多帧验收材料**；单帧证据与冻结 spec 均未受影响。
- **建议（需用户重签新 spec，阈值 0.5% 不变）**：改帧集设计使其在 cam2 上两段都留余量，例如 ①Δz 减半（如 `+0.05 m/步`，3 帧 0/+0.05/+0.10，降低第二步出视锥风险）；②或用更居中/更贴近 cam2 主轴的方向偏移；③或改用 2 帧 `0 → +0.10 m`（若只需单段可见性）——三者都要新 spec sha + 用户签名，本卡不预选。
- 16 条 guards 未被触发/未重冻（本窗无验证实体、无编译）。

## 9. 非声明

- 不声明多帧已覆盖、不声明多帧验收结论、不声明动画一致性、不声明与上游一致、不作对外发布依据；
- 本目录仅为**窗口执行记录 + 负结果**，不含参考数据；不替代 `spec.md`（冻结签名对象）。

## 10. 锁时序与执行纪律（如实记录）

- 11:59:25（**窗口正式开始前**）执行过一次 `acquire`（早于 12:00 边界，属我的时序失误）；**未做任何仓库内容写**（仅锁文件）。
- 12:00:03 边界后：`release` → `acquire`（12:00:07，owner=arch-chat）→ `status` 确认后才开始写（卡 JSON 于 12:00 后落盘）。
- 窗口内所有写均在锁持有下进行；12:06:44 采集结束后释放锁。

## 11. 证据修订记录（2026-11-05，`normify fix`）

QA 复核（`2026-11-05-render-r06-flex-multiframe-capture-check`，提交 `5dcc3597`，PASS-with-observations / 0 阻断）拎出 4 条低观察，处置如下（**结论、数值、停手与未落库判定均不变**）：

- **O1（已修）**：4 个原始控制台输出转码为 **UTF-8 / LF / 无 BOM**（正文逐字符不变，仅编码与行尾变化），`manifest.txt` 重算：

  | 文件 | 修订前 | 修订后 |
  | --- | --- | --- |
  | `capture.log` | ANSI/cp936 + 混合行尾（100 CRLF + 42 单 LF），9,265 B，`3b6030c7…` | UTF-8 / LF，9,199 B，`e24859cc…` |
  | `dry-run.log` | ANSI/cp936 + 混合行尾（88 CRLF + 6 单 LF），2,537 B，`3f600052…` | UTF-8 / LF，2,470 B，`9cfee38a…` |
  | `baseline-three-points.txt` | UTF-8 BOM + CRLF，93 B，`ab6206a9…` | UTF-8 / LF，87 B，`3e60cf2e…` |
  | `post-window-gpu.txt` | UTF-8 BOM + CRLF，33 B，`b5a8f8af…` | UTF-8 / LF，29 B，`f522324e…` |

- **O2（记已知，不返工）**：窗口卡 `created_at` 12:01 晚于基线 12:00:26 / monitor 12:00:44；卡与全部窗口证据同在 12:07:59 的 `484852ca`（标题「登记 + start」）落库，晚于采集结束 12:06:44 —— 记账时序瑕疵，**不影响**基线/数值/停手/不落库结论；锁时序见 §10。
- **O3（记已知，不返工）**：`capture.log` 仅含脚本 stdout，无命令行与 exit code；命令与 `exit 3` 由 receipt/README + 已复核代码路径判定。**后续采集窗日志模板要求**：wrapper 输出首行写完整命令行、末行写 `exit_code`（本窗为回溯记录，不改写原始日志）。
- **O4（记已知，不返工）**：卡验收第 5 条 `frame-000 vs base` 数组级位级对照**仅在自检 pass-margin 后执行**（与 spec §8.5 顺序一致）；本窗在自检处停手，故该对照 NOT RUN，不属于缺项。
- 复核锚点：本次 fix 只改本证据目录（4 个原始文件转码 + 本 README + `manifest.txt` 重算）；不改帧产物（本窗未落库）、不改 `capture-diagnostic.json` / `cam-poses-diagnostic.json` / `monitor-capture.csv` / `receipt.json`；`src/`、`tests/`、`fixtures/`、`scripts/`、冻结 spec 零 diff。
