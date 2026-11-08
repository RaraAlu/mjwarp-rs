# QA 复核：perworld L v2 判据实体 `render_per_world_light`（spec v3 §10.1；零 GPU）

- 任务：`2026-11-08-render-perworld-l-v2-criteria-entity-check`（owner QA、p77、P8-渲染与BVH）
- 复核对象：Backend 卡 `2026-11-08-render-perworld-l-v2-criteria-entity`，提交 **`cde3e597`**（parent `88e4c296`）
- 依据：已签 spec v3 `bb064fe0a85ab38abba8043768666a0d9f73fc53d9f2f3dd72eace060c8a924d` / 39,379 B，§4（判据 A）、§5（B）、§6（C）、§8（失败条件）、§10.1（实体要求）
- 复核基线：HEAD `774a027b`、树 clean、`REPO_WRITE_LOCK_FREE`；**只读被复核内容 + 只写本卡证据；零 GPU、不采集、不开窗**
- 方法：读 spec 原文逐条对代码；用 G-L0 pilot 保留参考 npz（`target/render-bvh/perworld-l-v2-gl0/pilot-out/nw3-light-hinge02040/`）独立重算 criterion C；独立重跑 dumper 到 `target/` 比对字节；独立复跑自证与强制 `--ignored`；四门禁

## 0. 结论

**PASS-with-observations（实现结构、门控、dump provenance、边界、四门禁全对）+ 1 项须在 G-L2 冻结前修的判据口径偏差。**

- **F1（须修，中）**：C1 计数规则用「8-bit 字节量化后 `|Δbyte| ≥ 1`」，而已签 spec §6 写的是「`max_channel |Δrgb| ≥ 1/255`」（G-L0 pilot readout 即按 f32/f64 原值实现）。用 pilot 参考复算：实体口径 **43,604 / 41,815** vs 已签口径 **38,285 / 40,644**（+5,319 / +1,171）；差异全部来自 254↔255 饱和边界步进；本参考不影响 verdict（40× 余量），但计数规则与签名对象不一致、方向偏宽（多计），**建议在 G-L2 冻结前把 C1 改为按 f32 原值 `≥ 1/255`（与 `readout_c1_c2.py` 同式），并让自证同步**。
- 其余 A/B/C 结构、阈值、相机归属、诊断项只记、`#[ignore]` 门控、产品段（Card H）、两跑 sha、world 隔离、dump provenance、边界与门禁：**均通过**（逐条见 §2–§8）。
- 未发现：allowlist/容差放宽、C1/C2 互换、用诊断项冒充判据、提前短路、内核/ABI 改动、fixtures 写入、GPU 使用。

## 1. 对象文件与独立核对的 sha/bytes

| 文件 | 行数 | bytes | sha256（本卡独立复算） |
| --- | --- | --- | --- |
| `tests/render_per_world_light.rs` | 489 | 19,235 | `d064765b89052fd731b473bfa99319b3a37220bd0995eecb4cfba57c998a6ceb` |
| `tests/support/render_bvh/light.rs` | 390 | 17,580 | `70fd4e852736f1fdb3c99cbb27949f36e442c0db714c7ff91ae76ef8f62f3f7d` |
| `tests/support/render_bvh/light/tests.rs` | 446 | 15,225 | `c0da31656f9d87fd98e76a724c55f27e22bbffeb2b0c55f8305f1baa60e4bd43` |
| `tests/support/render_bvh/light/model-315.json` | 1 | 312,943 | `503bdabd3366034460639bfc7771e6cecc3381a950fbf40850c25e1b691dc67f` |
| `tests/support/render_bvh/light/model-315.provenance.json` | 73 | 2,209 | `e4553329e244845f7ac6648c580f141c8f58e656f4a9fabd9efacd381810a81b` |
| evidence `materials/dump_model_315_light.py` | — | 5,630 | `215fff8e0ade107c5af538626b7892aaa1a2f49e9c27a6ae9f2ea371534efb78` |
| evidence `README.md` / `receipt.json` | — | 9,641 / 6,757 | `5ebbafa1…` / `150bf17f…`（manifest 6/6 重算一致） |

行数 489/390/446 均 ≤500；全部 LF、无 BOM、裸 CR/TAB 0。

## 2. 判据 A（spec §4）— PASS

- `compare_frame`：rgb `mismatched_pixels == 0` 且 `max_byte_diff == 0`；depth 用 `to_bits()` 全等（`beyond_ulp` 仅作计数、要求 0，**无 ULP 容差、无 allowlist**）；seg `(id,type)` 逐像素全等；长度不同显式 `assert` 失败。
- 覆盖两台相机 × 3 world（`criterion_a` 返回 `[world][cam]`）；GPU 用例逐项断言 5 个统计量为 0。
- 参考 rgb 的 f32→packed 还原（`packed_rgb`：`round(v*255)`，断言落在 `byte/255` 网格 ±0.01 byte）与仓库既有口径一致（`tests/support/render_bvh/shadow.rs` / `sdf.rs` 同式）；对真实 G-L0 参考实测网格偏差 **1.75e-05 byte** ≪ 容差 → 还原精确。
- 未引用 criterion v2.2 清单、未做 1 LSB 放宽。

## 3. 判据 B（spec §5）— PASS（附观察 O3）

- `criterion_b` 逐 world × 3 灯比较 `light_xpos/light_xdir` 的 **f32 位型**（`to_bits()` 相等，0 例外）；单 bit 变异自证用例能检出。
- `world_light_poses` 从参考 `state.npz` 逐 f32 拷贝进 `WorldLightPoses`：无 FK、无 f64 重算、`xdir` 不重归一化。
- 边界：B 比较的是「由 state.npz 构造出来的 `WorldLightPoses`」与同一 state.npz —— 验证的是透传助手与数据面口径，无法独立观测产品内部上传后的灯数组（见 O3）。

## 4. 判据 C（spec §6）— **结构 PASS；C1 计数口径须修（F1）**

结构（全部正确）：

- C1 只取 **cam-persp**（`frames[..][0]`）、C2 只取 **cam-hinge**（`frames[..][1]`）的 depth 位型 + seg pair；`PAIRS = [(0,1),(1,2)]`；`ACCEPT=384`、`MARGIN=576`；`verdict`：任一 <384→NonDiscriminating、任一 <576→Insufficient、否则 PassMargin（**无提前短路、无 allowlist**）。
- 相机不可互换：正向/反向自证各一条（互换构造下 C1=C2=0 且判 NonDiscriminating）。
- 诊断项只记录：`diag_campersp_depth/seg`、`diag_camhinge_rgb`，不进门槛、不冒充 C1/C2。

独立复算（同一 pilot 参考）——C2 与 pilot 完全一致：

| 计数 | 实体/规范实现 | pilot 记录 | 一致 |
| --- | --- | --- | --- |
| C2 depth 0→1 / 1→2 | 35,828 / 33,883 | 35,828 / 33,883 | ✅ |
| C2 seg 0→1 / 1→2 | 16,041 / 16,754 | 16,041 / 16,754 | ✅ |
| 诊断 cam-persp depth/seg | 0/0、0/0 | 0/0、0/0 | ✅ |

**F1（C1 口径偏差）**：

| C1 口径 | 0→1 | 1→2 |
| --- | --- | --- |
| 已签 §6：f32 `max_channel|Δrgb| ≥ 1/255`（pilot readout 同式）**= 应然** | **38,285** | **40,644** |
| 实体实现：`round(v*255)` 后 `|Δbyte| ≥ 1` | 43,604 | 41,815 |
| 差值（实体多计） | +5,319 | +1,171 |
| 诊断 cam-hinge rgb（仅记录） | 54,380/54,250 vs pilot 50,363/52,329 | （同步偏差，不设门槛） |

- 根因：多计像素全部是 **254↔255**（饱和边界一步）——`1.0 - 254/255` 在 f32/f64 上为 `0.003921508789`，**小于** `1/255 = 0.003921568627`，故已签口径不计、字节口径计 1。样例像素 `(0,233)`：prev `[1,1,1]` → next `[1,0.99607849,1]`，byte `[255,255,255]`→`[255,254,255]`，Δ=0.003921508789。
- §6 明写「**饱和像素不计**」，与「字节一步也算」方向相反；且 pilot/capture 侧记录的 C1 就是 38,285/40,644，实体在 G-L2 复算会给出另一组数（当前无断言比对，故不改变 verdict）。
- 影响：本参考下 6 项计数均 ≫576，verdict 不翻转；但若未来参考落在 384–576 边界附近，字节口径最多可多计约 1/255 边界像素，方向为**放宽**。故建议 G-L2 冻结前修（1 处比较式 + 1 条自证），不要带入口径偏差进冻结 exe。
- 复算原始数据：`artifacts/c1-recompute.json`。

## 5. 运行门控（§10.1 / §8）— PASS

- 文件级 `#![cfg(feature = "cuda-probe")]`：默认门禁不编译空壳不失败；GPU 用例 `#[ignore]`。
- 强制 `--ignored` 独立复跑：**exit 101**，`tests/render_per_world_light.rs:353` panic 文本逐字 `not executed: reference missing (G-L1 capture pending): ../fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040/ (required rgb/depth/seg/state npz + capture.json)`（属预期，非失败）。日志 `artifacts/not-executed-rerun.log`。

## 6. 产品段与产品侧前提 — PASS

- 确用 Card H `render_per_world_with_lights(...)` + `WorldLightPoses`：9 条位姿（3 world × 3 灯）由参考 `state.npz` 的 `light_xpos/light_xdir` **逐 f32 位拷贝**，经 Card H 逐 world 重传到内核 1-D 灯实参。
- `camera_poses = CameraPose::from_fields(fields)` 长度 = ncam(3)；`world_camera_pose_arrays` 明确允许 `ncam` 广播（固定相机语义正确），不触发 `camera_pose_count`。
- 产品 per-world 路径每次发射前 `apply_poses → update_poses + refit_bvh`（P0-1 前提成立），与 §3.4「不 refit 则 C2 退化为 0」的采集前提一致。
- 两跑 sha：实体对 rgb/depth/seg 整块（全部 world×相机）各算 sha 并断言相等；world 隔离：仅改 world1 几何+灯，断言 world0/world2 两台相机逐位不变、world1 至少一台变化（不替代 A/B/C）。

## 7. dump provenance（独立重跑复算）— PASS

- 用同一 pinned venv 独立重跑 `materials/dump_model_315_light.py`（`--out/--provenance-out/--work-dir` 全部指向 `target/`，未触碰入库文件）：
  - 重 dump 与入库 `model-315.json` **逐字节相等**：`503bdabd…` / 312,943 B ✅（同一路径连续两次 dump 也相等 ✅）
  - mjb：`c5414b6b…` / 101,586 B，两次保存一致 ✅
  - 环境：python 3.12.13 / mujoco 3.15.0（3015000）✅；输入场景 `4b354054…` / 5,134 B ✅
  - dumper sha `215fff8e…`、`model_parity.py` sha `826bd790…` ✅
- 模型 dump 位于 `tests/support/render_bvh/light/`（**不在 fixtures**，与 hfield/SDF 先例一致）；`model_file` 仅名字串（mjb 未入库）如实标注。
- 模型相机数组独立读出核对：`cam_mode=[0,0,3]`、`cam_fovy=[73.73979462,30,40]`、`cam_resolution` 前两台 320×240、`cam_pos/quat` 与 §2.1 写死值逐位吻合、`cam-hinge` `sensorsize=[0,0]`、`intrinsic=[0.01,0.01,0,0]`（fovy 路径）。

## 8. 自证覆盖与 npz 解析 — PASS

- 独立复跑 `cargo test --locked --features cuda-probe --test render_per_world_light -- --test-threads=1` → **13 passed / 0 failed / 1 ignored**（exit 0；0.78 s）。日志 `artifacts/selfproof-rerun.log`。
- 覆盖核对：383/384/575/576 边界（含 6 个集合逐个边界）、某套 0/0 必败、C1/C2 互换必拒、1/255 字节口径、pair 顺序、A 三量位级+长度失败、B 位型+单 bit 变异、两跑 sha/隔离、npz 正负例、provenance 负例（两跑标记/时间/tag/nrender/相机字段缺失均拒）。
- **npz 解析器对真实参考布局的验证**（用 G-L0 pilot npz，不是合成数据）：`rgb/depth/seg` 各 6 个成员、`state` 5 个成员；全部 `ZIP_STORED`、`flag_bits=0`（无 data descriptor）、顺序 local-header 走位恰好停在 central directory（如 rgb：walk_end 5,530,638 / 文件 5,531,026，尾部 388 B 为 central+EOCD）→ 解析器假设成立；dtype `f4`/`f4`/`i4` 与实体断言一致（3×PIXELS / PIXELS / 2×PIXELS）。详见 `artifacts/npz-layout-check.txt`。

## 9. 边界与证据卫生 — PASS

- `cde3e597` 相对 parent `88e4c296` 共 17 个变更路径：卡 1、`runs` 4、本卡证据 7、新增 test/support 5；`src/`、`fixtures/`、`scripts/` **零 diff**（见 `artifacts/diff-face.txt`）。
- 被测证据目录 manifest **6/6** 独立重算通过；被测 5 个新文件 LF/无 BOM/裸 CR·TAB 0。
- 新增测试文件**不挂模块源**：全仓 `.normify/modules/**` 对 `tests/render_*` 命中 0，与既有 render 测试惯例一致（不是本卡漏登记）。
- 四门禁独立复跑（`normify gate …` 本卡）：fmt/clippy/clippy-cuda 通过；unit **1384 passed / 0 failed / 401 ignored**。

## 10. 观察项（分级）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| **F1/O1** | **中（G-L2 冻结前修）** | C1 计数用字节量化 `≥1` 而非 §6 的 f32 `≥1/255`，pilot 参考上 43,604/41,815 vs 38,285/40,644，方向偏宽；§6 明写「饱和像素不计」 | C1 保留 f32 原值按 `1.0/255.0` 比较（与 `readout_c1_c2.py` 同式），同步改自证断言；建议顺带断言重算值与 `capture.json` 记录的 `c1_rgb_changed_px` 一致 |
| O2 | 低 | `check_capture_provenance` 只查 10 个相机字段**存在**，未比对 §2.1 写死值（name/mode/fovy/resolution/xpos/quat/sensorsize）；元数据只查 captured_at/tag，未查 `command`、`per_world_scope` 新口径，也未交叉核对 capture.json 的判别力计数 —— §8 #8 与 §9.3-4 的「双相机快照/元数据四项」在实体层只落实了一部分 | G-L2 前补齐（值断言 + 4 项元数据 + 计数交叉核对）；当前不阻断，因为 A 判据会兜住相机换位/错误参考 |
| O3 | 低 | 判据 B 比较的是「由 state.npz 构造的 WorldLightPoses」与同一 state.npz，验证的是透传助手（无 FK/f64/重归一化）与 B 的数据面口径；产品内部上传值不可观测，B 的产品侧证据仍来自 Card H 代码复核 | README 已如实表述；G-L2 记录时注明 B 的独立力边界 |
| O4 | 低 | `light.rs` 里 `CAMERAS: usize = 3` 实际被当**灯数**用（`world_light_poses` 循环、`WorldLightPoses::new(WORLDS, 3, …)`、`criterion_b` 硬编码 3） | 改名 `LIGHTS` 并统一用常量，避免后续灯数变化时误用 |
| O5 | 信息 | 两跑 sha 用 rgb/depth/seg 三块**整块**哈希（跨全部 world×相机），spec §4 写的是「逐 world、逐相机」粒度；整块相等可检出任何部位差异，pass/fail 等价 | 可在 README 注明粒度差异，或按 world/camera 细化 |
| O6 | 信息 | 实体断言 `Verdict::PassMargin`（≥576，属 §6 采集前置），而 §6 验收门槛是 ≥384；方向更严、安全 | 保留；G-L2 材料写明这是前置门槛的复检 |
| O7 | 信息 | `model-315.json` / provenance 无 EOL 属性（`core.autocrlf=true`），与既有 `tests/support/render_bvh/{hfield,sdf}/model-312.json` 同姿态；fresh checkout 的 worktree 字节可能非 LF，届时 provenance 的 sha 只对应 LF blob | 现状与先例一致，不属本卡问题；若治理则统一为 `tests/support/render_bvh/**/*.json -text` |

## 11. 非声明

本卡只做只读复核与独立复算：**不宣称** G-L2 已通过、不宣称 per-world 灯验收/上游等价/渲染数值结论；不修改被复核内容、spec、pilot 证据或签名记录；F1 的修复与是否在 G-L2 冻结前修由 Chief 排程决定。所有数字均可由 `artifacts/` 下的脚本输出与 pilot 保留 npz 复现。
