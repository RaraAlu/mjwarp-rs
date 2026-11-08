# QA 复核：判据实体 F1/O2 修正（`0c770bb2`；零 GPU）

- 任务：`2026-11-08-render-perworld-l-v2-criteria-entity-f1-o2-fix-check`（owner QA、p78、P8-渲染与BVH）
- 复核对象：F1/O2 修正提交 **`0c770bb2`**（parent `7d089165`；即我上一张复核卡证据提交）
- 依据：已签 spec v3 `bb064fe0…`/39,379 B §2.1/§2.4/§6/§9.3；我的复核卡 `2026-11-08-render-perworld-l-v2-criteria-entity-check`（F1 中、O2 低、O3 低）
- 基线：HEAD `0c770bb2`、树 clean、锁 FREE；**只读被复核内容 + 只写本卡证据；零 GPU、不采集、不开窗**
- 方法：读 diff 与代码→独立复算 C1/C2（pilot 保留 npz）→独立跑仓内 ignored 交叉核对用例（真实 pilot 数据）→强制 `--ignored` 验路径与「未执行」→四门禁

## 0. 结论

**PASS。可用于 G-L2 冻结。**

- **F1 修法正确**：参考 rgb 保留原始 f32（不量化）；C1 = 逐通道 f32 差提升 f64 → `max_channel` → `>= 1/255`，与已签 §6 及 `readout_c1_c2.py`（`4a054aba…`）同式；A/B/C2 语义、384/576、相机归属、无 allowlist 未动。
- **独立复算**与修正卡 artifact 逐值一致：新 C1 **38,285 / 40,644**、旧字节法 **43,604 / 41,815**、差 **+5,319 / +1,171**、`float_only_px = 0`；C2 depth **35,828 / 33,883**、seg **16,041 / 16,754**、verdict pass-margin。
- 仓内 ignored 用例 `pilot_c1_crosscheck_matches_readout` 在**真实 pilot 数据**上独立跑通（并交叉 `window/readout.json`）。
- O2 provenance 加强版在**真实脚本输出**（pilot `capture.json`）上通过（该加强自核被上述用例调用），G-L1 计划命令含全部所需 argv → 无假阴性风险。
- 路径修复生效：强制 `--ignored` 报仓内绝对路径、文案与 exit 101 不变。
- 两条低观察（不阻断）：证据里 `401 ignored` 应为 **402**；`capture.json` 判别力计数字段仍缺（spec §2.4 要求），使 capture-C1 交叉核对在真参考上会静默跳过。

## 1. 对象与文件

| 文件 | 行数 | bytes | sha256（本卡复算） |
| --- | --- | --- | --- |
| `tests/render_per_world_light.rs` | **469**（改前 489） | 23,475 | `b2123ab0c223d456bb83c90dc3ecba44d6fc7b16d827bed2c80653e424f32b97` |
| `tests/support/render_bvh/light.rs` | **500**（改前 390） | 25,883 | `6cf668cd688516fa9faa15bd23fc379c0089c95be822ffd111b1d93cb31d1c2e` |
| `tests/support/render_bvh/light/tests.rs` | **465**（改前 446） | 17,746 | `c5a08ae17c8acc2dc9459f5ceda04d20775717d91fe9802a8a4fd0174ab06fbb` |

均 ≤500；UTF-8/LF、无 BOM、裸 CR/TAB 0。

## 2. F1：C1 口径修正（PASS）

- 代码：`Reference.rgb_f32` 加载时逐值保留（`rgb_values.chunks(3)`，不量化）；`criterion_c` 的 C1 与 `diag_camhinge_rgb` 均改用 `rgb_above_threshold(left,right)`：
  `max_c( |left_c as f64 - right_c as f64| ) >= RGB_THRESHOLD`，`RGB_THRESHOLD = 1.0/255.0`。与 `readout_c1_c2.py` 的 `np.abs(x.astype(float64)-y.astype(float64)).max(axis=-1) >= 1.0/255.0` 逐位同口径。
- 未动：`Frame.rgb`（packed 字节）仍只服务判据 A（严格位级）；`verdict`/`ACCEPT=384`/`MARGIN=576`/`PAIRS`/相机归属/诊断项只记录 — diff 中无相关改动。

**独立复算（G-L0 pilot 保留参考 npz；npz 在 `target/`、不入库）**：

| 量 | 值 | 与修正卡 artifact |
| --- | --- | --- |
| C1 f32（新，规范口径） | **38,285 / 40,644** | 一致 |
| C1 字节法（旧实现） | 43,604 / 41,815 | 一致 |
| 差（旧−新） | +5,319 / +1,171 | 一致 |
| `float_only_px` | 0 / 0 | 一致 |
| 差集构成 | 0→1：254↔255 = **977**，其余为 240–249 等高位相邻码步进的 f32 舍入（240↔241 406、246↔247 349、241↔242 331、245↔246 327、247↔248 309…）；1→2：**全部** 254↔255（1,171） | 一致 |
| C2 depth / seg | 35,828/33,883、16,041/16,754 | 一致 |
| 诊断 cam-persp depth/seg | 0/0、0/0 | 一致 |
| verdict | PassMargin（六项均 ≥576） | 一致 |

- 另独立执行仓内 `#[ignore]` 用例 `pilot_c1_crosscheck_matches_readout`（真实 pilot 数据）：PASS，输出 `C1=[38285, 40644] C2 depth=[35828, 33883] seg=[16041, 16754] verdict=PassMargin`，并交叉 `window/readout.json` 的 C1。
- **对上一卡表述的更正（记录用，不回改历史证据）**：我上一张复核卡写「差集全部是 254↔255」过于狭窄；实测 0→1 的 5,319 中只有 977 是 254↔255，其余是同为「字节差恰 1 而 f32 差 < 1/255」的高位相邻码步进。F1 结论（口径偏差、方向偏宽、须修）不受影响。

## 3. 自证（PASS）

独立复跑 `cargo test --locked --features cuda-probe --test render_per_world_light -- --test-threads=1` → **15 passed / 0 failed / 2 ignored**（exit 0）：

- 新用例 `criterion_c_c1_counts_f32_threshold_1_over_255_not_byte_steps` 真能区分两套口径：1/255 步进计入、半步不计、`254/255 → 1.0` **不计**（字节法会多算），并构造像素级计数断言 `c1=[2,0]`。
- `pilot_c1_crosscheck_matches_readout`（`#[ignore]`，pilot npz 在 `target/`）与 `capture_c1_cross_check_requires_recorded_counts_match`、`capture_scope_text_handles_string_and_list` 在位。
- 负例自证 `capture_provenance_self_check_rejects_bad_or_missing_fields` 从 5 项扩到 **20 项**（逐项必须被拒）。

## 4. O2：provenance 补强（PASS）

- **值与 §2.1/§2.2 比对**：两相机 `name/slot/mujoco_cam_id/mode/resolution/xpos/quat(归一化后)/fovy/sensorsize/intrinsic`（容差 1e-6，sensorsize/intrinsic 按 3.15 f32 位型）+ `recipe.scene_xml` sha256 `4b354054…`/5,134 B；相机条目字段数恰 10。
- **command**：`command.argv` 非空且含 `capture_render_perworld.py`、`--scene`、场景路径、`--reference-tag`、tag、`--worlds`/`3`、`--repeat`/`2`、`--use-fast-math`/`false`、`--use-shadows`/`true`。
- **per_world_scope**：`covers` 字符串/列表归一化（与 readout `scope_text` 同口径）后须覆盖 per-world 灯位姿（`light_xpos/light_xdir`）+ per-world 几何位姿（`geom_xpos/geom_xmat`）。
- **resolved 交叉**：`nworld=3`、`nrender=2`、`cam_id_map=[0,1]`、`cam_res=[[320,240],[320,240]]`、`parameters.cam_active=[true,true,false]`、`legacy_manifest_entries`（或 `legacy_reference_files`）=112、`legacy_reference_untouched=true`。
- **真实数据验证（关键）**：上述加强版自核在 pilot `capture.json`（同脚本真实输出）上**通过**——由 `pilot_c1_crosscheck_matches_readout` 内的 `check_capture_provenance` 调用实际执行；且 S-2 计划命令（`materials/command.txt` §5）逐字含全部 argv 需核项（`--scene fixtures/... --reference-tag … --worlds 3 --use-fast-math false --use-shadows true --repeat 2`）→ G-L1 参考不会被新自核误拒。
- **交叉核对（capture C1）**：`check_capture_c1` 在 `discriminability.c1_rgb_changed_px` / 顶层 / `readout` 三处取字段，存在则断言相等、不存在返回 false（跳过）。用例覆盖「一致 / 缺失跳过 / 不一致拒绝」。

## 5. 路径修复（PASS）

- `reference_dir()` = `env!("CARGO_MANIFEST_DIR")/fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040`，与测试进程 cwd 解耦。
- 强制 `--ignored` 独立复跑 exit **101**，逐字文案：`not executed: reference missing (G-L1 capture pending): D:\Rust\mjwarp-rs\fixtures\render-bvh\scene-a-light\perworld\nw3-light-hinge02040 (required rgb/depth/seg/state npz + capture.json)` —— 路径在**仓内**、不再是 `../fixtures` 的仓外解析。
- `missing_reference_reports_explicit_not_executed_error` 同用 `reference_dir()` 基准构造缺失目录。

## 6. 边界 / 证据 / 门禁（PASS）

- `0c770bb2`（parent `7d089165`）只动：3 个 tests 文件 + 本卡（原实体卡）证据（README/receipt/manifest/logs/artifacts/c1-crosscheck）+ 卡 addenda + runs；`fixtures/`、`src/`、`scripts/`、spec 目录、pilot 材料目录 **零 diff**（见 `artifacts/diff-face.txt`）。
- 被测证据目录 manifest **8/8** 独立重算通过；编码卫生通过（`artifacts/encoding-checks.txt`）。
- 四门禁独立复跑：fmt exit 0、clippy exit 0、clippy-cuda exit 0、unit **1386 passed / 0 failed / 402 ignored**。

## 7. 观察项（均不阻断）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 低（证据 erratum） | 修正卡 README §5 与 `receipt.json` 写 `unit … 401 ignored`；实测（含该卡自己的 `runs/unit.json`）是 **402** —— 新增 1 条 `#[ignore]` pilot 用例使 ignored +1 | 一次 `normify fix` 改两处数字（1386/0/402） |
| O2 | 低–中（跨文档，需决策） | `check_capture_c1` 的「实体 C1 == capture.json 记录」在**真 G-L1 参考上会静默跳过**：现脚本不把 `discriminability.c1_rgb_changed_px` 等写入 capture.json（pilot `capture.json` 实测无此键）；而 spec v3 §2.4 要求 capture.json 记录判别力计数 | G-L1 前二选一：①采集脚本补写这些计数（小改，满足 §2.4，交叉核对生效）；②明示该跳过并以 readout 产物为准（§2.4 缺口登记） |
| O3 | 信息 | `tests/support/render_bvh/light.rs` 已达 **500 行**上限（ratchet 无余量） | 若后续再往该文件加自核，先规划拆分 |
| O4 | 信息 | 上一卡「差集全部 254↔255」表述过窄（实为「字节差恰 1 且 f32 差 < 1/255」的相邻码步进；254↔255 为 977/5,319） | 本卡记录更正；历史证据不回改 |

## 8. 非声明

本卡只做只读复核与独立复算：**不宣称** G-L2 已通过、不宣称 per-world 灯验收/上游等价/渲染数值结论；未修改被复核内容、spec、pilot 证据与签名记录；O1/O2 是否为 G-L2 冻结前置由 Chief 决定（本卡结论：**实体本身可用于 G-L2 冻结**）。
