# R06 多帧采集脚本实现（`capture_render_flex.py` 帧循环 + 变化像素自检）

- 任务：`2026-11-05-render-r06-flex-multiframe-capture-script`（p60；离线 authoring 工具实现卡）
- 冻结前提（运行时校验）：`2026-11-05-render-r06-flex-multiframe-criterion-spec` — spec sha256 `1d9fa5cef1ebc829747d63f648d3bab1ec935d8df8e3403bc52c2ac2aa7c40b6` / 10506 B
  （脚本 `verify_frozen_spec()` 读取该文件并核对 sha256/bytes，不一致直接拒绝运行；dry-run 输出亦含该块）
- 边界：本卡只做脚本实现与 CPU 侧参数/自检；**未执行采集、未开 GPU 窗口、未渲染、未改 fixtures/src/tests/Cargo.toml**；
  实现完成不产生任何验收结论。采集窗仍等 S-1 用户签名 + Chief GO。

## 1) 实现摘要（两个文件）

| 文件 | 行数 | 说明 |
| --- | --- | --- |
| `scripts/authoring/capture_render_flex.py` | 498（≤500） | **只加 7 行（0 删除）**：`from capture_render_flex_multiframe import add_frame_args, dispatch`、`add_frame_args(parser)`、main 内 3 行 dispatch；原有单帧逻辑一字未改 |
| `scripts/authoring/capture_render_flex_multiframe.py` | 446（≤500） | 新增模块：帧循环采集 / `--check-deltas` / CPU dry-run / 冻结 spec 校验 |

CLI（挂在原脚本入口）：`--frames 3 --frame-dz 0.1`（帧循环）、`--check-deltas`（读采集结果自检）、
`--dry-run`（CPU-only 计划检查）。单帧路径（不带这些参数）`dispatch()` 返回 None，原逻辑逐字节不变。

## 2) 与冻结 spec 的逐条对应

| spec 条款 | 实现 |
| --- | --- |
| §2 帧集/Δz | 常量 `FRAMES=3`、`FRAME_DZ=0.10`、`ANCHOR_BODIES=('flex-anchor0','flex-anchor1','flex-anchor2')`；`_render_frame()` 对第 k 帧把三个锚点 body 的 `body_pos.z += k*0.10` 后 `put_model → mj_forward → forward → render`；`capture_multiframe()` 拒绝其它帧数/Δz（须新 spec + 用户重签） |
| §3 相机 | `CAM2_INDEX=2`、`CAM2_WIDTH/HEIGHT=320/240`、`CAM2_PIXELS=76800`；`--check-deltas` 对 cam2 形状逐帧校验 |
| §5 阈值（384/576） | `DELTA_ACCEPT_MIN=384`、`DELTA_MARGIN_MIN=576` **硬编码**；`_verdict()` 输出 `pass-margin` / `insufficient-margin` / `non-discriminating`；**不提供任何改阈值/帧数/Δz 的 CLI** |
| §8.2 脚本扩展点 | 帧循环模式即上述模块；单帧输出路径/字段不改 |
| §8.3 产物布局 | `frame-{000,001,002}/{rgb,depth,seg,state}.npz` + 顶层 `capture.json` / `cam-poses.json` / `README.md`；输出根目录必须显式 `--out-dir`（脚本内不写默认 fixtures 路径） |
| §8.4 确定性/provenance | `--repeat 2` 两遍完整采集逐帧逐文件 sha256 对比，不一致非零退出并打印差异；`capture.json` 含帧数/Δz/逐帧文件 sha/两次一致标记/环境版本/pinned commit/GPU/recipe/时间/命令 |
| §8.5 自查顺序 | 采集完成即用 `_delta_report()` 算 cam2 相邻帧变化像素并打印判定（与 `--check-deltas` 同一实现）；非 pass-margin 时打印「不得落库」提示 |
| §6 allowlist | 本脚本只负责采集与变化量自检，**不实现、不扩展** allowlist（判定侧规则见 spec，脚本无放宽入口） |

## 3) CPU 侧自检（本次实际执行；零 GPU）

全部日志在 `logs/`（原样输出）：

- `--help`：exit 0，新增 3 个参数（`--frames` / `--frame-dz` / `--check-deltas`）可见（`logs/help.txt`）；
- `--frames 3 --frame-dz 0.1 --out-dir … --dry-run`：exit 0，CPU-only 计划（不渲染/不写采集产物），
  输出含冻结 spec sha 校验块、锚点 body id `[1,2,3]`、base z `[0.0,0.0,0.0]`、
  期望布局、阈值常量（`logs/dry-run-multiframe.json`）；
- 参数错误路径（期望拒绝）：见下表，均 exit 1 且打印原因；
- `--check-deltas`（喂合成采集数据，三组，`logs/check-deltas-synth-*.txt`）：
  - `synth-pass`：两组相邻帧 depth/seg 变化均 ≥576 → `pass-margin`，exit 0；
  - `synth-insufficient`：一组 depth 变化 500 落 [384,576) → `insufficient-margin`，exit 3；
  - `synth-non-discriminating`：一组 depth 变化 300 <384 → `non-discriminating`，exit 3（拒绝通过）；
  - 合成数据与期望计数一致（600/600、600/650、500/600、300/600），证明像素计数方向与阈值映射正确。

| 用例 | 命令参数 | exit |
| --- | --- | --- |
| `param-frames-2` | `--frames 2 --out-dir target/multiframe-work/never --dry-run` | 1 |
| `param-dz-02` | `--frames 3 --frame-dz 0.2 --out-dir target/multiframe-work/never --dry-run` | 1 |
| `param-no-out-dir` | `--frames 3 --frame-dz 0.1 --dry-run` | 1 |
| `param-repeat-1` | `--frames 3 --frame-dz 0.1 --out-dir target/multiframe-work/never --repeat 1` | 1 |
| `param-check-deltas-no-out-dir` | `--check-deltas` | 1 |
| `param-check-deltas-with-frames` | `--check-deltas --out-dir target/multiframe-work/synth-pass --frames 3` | 1 |
| `check-deltas-synth-pass` | `--check-deltas --out-dir target/multiframe-work/synth-pass` | 0 |
| `check-deltas-synth-insufficient` | `--check-deltas --out-dir target/multiframe-work/synth-insufficient` | 3 |
| `check-deltas-synth-non-discriminating` | `--check-deltas --out-dir target/multiframe-work/synth-non-discriminating` | 3 |

## 4) 单帧路径「逐字节不变」判定依据

- 改动是**纯增量**：`git diff scripts/authoring/capture_render_flex.py` 仅 +7 行（0 删除，见
  `artifacts/capture-script.diff`），无删除/改写；`parse_args` 只追加参数，原参数与默认值不变；
- 不带 `--frames`/`--frame-dz`/`--check-deltas` 时 `dispatch()` 返回 `None`，main 继续原流程
  （原 `warp` 配置、环境校验、`capture_once`、落盘、`capture.json`、`cam-poses.json` 全未触碰）；
- 因本卡不执行采集（无 GPU 窗口），不能提供「重跑与 base 采集同 sha」的动态实证；该点作为
  **静态判定**如实登记，待采集窗内首帧（frame-000）与 base 参考逐位对照复核。

## 5) 边界与遗留

- 未执行采集、未开窗、未跑上游渲染；未创建 `fixtures/render-bvh/scene-flex/multiframe/`；
- 未改 `src/`、`tests/`、`fixtures/`、`Cargo.toml`；未改 spec/判据/阈值/帧数/Δz/相机；
- 判据实现（新测试实体）本卡不做，等采集参考形状确定后另卡；
- 采集窗内仍需：S-1 签名 + Chief GO + 两点基线/floor + 两次采集逐位一致 + 变化像素 ≥576 才落库。

## 6) 证据清单

| 路径 | 说明 |
| --- | --- |
| `README.md` | 本文件 |
| `receipt.json` | 机器可读回执（`/task_id` 供 normify `json` 检查） |
| `artifacts/capture-script.diff` | `git diff scripts/authoring/capture_render_flex.py`（+5 行） |
| `artifacts/script-identity.json` | 两脚本路径/行数/sha256 与增量说明 |
| `logs/help.txt` / `logs/dry-run-multiframe.json` | CLI 帮助与 CPU 计划输出 |
| `logs/param-*.txt` | 参数错误路径（帧数/Δz/out-dir/repeat 等） |
| `logs/check-deltas-synth-*.txt` | 三种判定的合成数据自检 |
| `logs/cpu-checks.json` | 上述用例逐条 exit code 机器可读汇总 |
| `manifest.txt` | 本目录 sha256 + bytes + 证据目录内相对路径（base dir 见表头，不自列） |
