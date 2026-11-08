# perworld L v2 判据实体 `render_per_world_light`（spec v3 §10.1；零 GPU 实现；G-L2 才真跑）

- 任务：`2026-11-08-render-perworld-l-v2-criteria-entity`（owner Backend、p60、P8-渲染与BVH）
- 依据：已签 spec v3 `bb064fe0a85ab38abba8043768666a0d9f73fc53d9f2f3dd72eace060c8a924d` / 39,379 B
  §10.1（判据实体）、§4（判据 A）、§5（判据 B）、§6（判据 C）；产品段用 Card H 卡
  `2026-11-08-render-perworld-l-v2-host-light-entry`（实现 `773cbe99`）的 `render_per_world_with_lights` + `WorldLightPoses`。
- 先例：`tests/render_per_world.rs`（per-world 实体/GPU ignore）、`tests/render_flex_validate.rs`
  （CPU 自证 + `#[ignore]` GPU 用例）；测试期模型 dump 放 `tests/support/render_bvh/`（hfield/SDF 先例，不进 fixtures）。
- 本卡**纯 CPU**：零 GPU、不采集、不建 CUDA；参考（G-L1 采集）尚不存在 → GPU 用例 `#[ignore]`，
  强制执行时**显式报未执行**（不静默 skip）。G-L2 冻结 exe 才真跑。

## 1. 交付文件与行数（git-blob 换行口径，均 ≤500）

| 文件 | 行数 | 作用 |
| --- | --- | --- |
| `tests/render_per_world_light.rs` | **469**（F1/O2 修正前 489） | 实体：参考 npz 解析/加载（保留原始 f32 rgb）、capture provenance 自核、§4/§5/§6 + 两跑 sha + world 隔离的 `#[ignore]` GPU 用例、本机 pilot C1 交叉核对 |
| `tests/support/render_bvh/light.rs` | **500**（修正前 390） | 测试期 3.15 dump → `RenderFields`/`SceneState`；Card H 产品段构造；判据 A/B/C 纯函数；两跑 sha / 隔离助手；capture provenance 值比对 |
| `tests/support/render_bvh/light/tests.rs` | **465**（修正前 446） | CPU 合成自证（边界/方向/互换/阈值/负例/npz），由实体 `#[cfg(test)]` 挂载 |
| `tests/support/render_bvh/light/model-315.json` | 312,943 B | 离线 3.15 dumper 生成的产品格式模型 dump（**不进 fixtures**） |
| `tests/support/render_bvh/light/model-315.provenance.json` | 2,209 B | dump provenance（见 §2） |

## 2. 测试期模型 dump（provenance）

- dumper：`materials/dump_model_315_light.py`（sha256 `215fff8e0ade107c5af538626b7892aaa1a2f49e9c27a6ae9f2ea371534efb78`）；
  复用 `scripts/authoring/render-bvh/model_parity.py`（sha256 `826bd7904bad18447e594876d21373c154e4c01f8b61729d480a79e70f79fbaf`）的
  `compile_315` + `dump_model`。
- 逐字命令：`target/mjwarp-parallel/venv/Scripts/python.exe -B docs/development-evidence/render-bvh/perworld-l-v2-criteria-entity/materials/dump_model_315_light.py`
  （argv/cwd 见 provenance `command`）。
- 环境：pinned venv **python 3.12.13 / mujoco 3.15.0**（与参考采集同版本；不混 3.12 产物）。
- 输入场景：`fixtures/render-bvh/scene-a-light/scene.xml` sha256 `4b354054116ddb81c6209ed10c021d133ea30d6fd82c1a6f866bafff9f625716` / 5,134 B。
- 输出：`tests/support/render_bvh/light/model-315.json` sha256 `503bdabd3366034460639bfc7771e6cecc3381a950fbf40850c25e1b691dc67f` / 312,943 B；
  **同路径连续两次 dump 逐字节相等 = true**；`compile_315` 两次 mjb 保存一致 = true（mjb sha `c5414b6b…`/101,586 B，未入库）。
- `model_file` 字段说明：dump 的 `model_file` 只是名字字符串（`scene-a-light-315.mjb`），该 mjb 生成在 `target/` 下、不入库，
  因此该字段不指向任何已提交文件；模型身份以本文件 sha 与场景 sha 为准。
- dump 规模：133 个数组（116 个 `RENDER_FIELDS` 全覆盖 + 场景几何/网格/纹理数组；`counts`: ngeom=7、ncam=3、nlight=3、nbody=3）。

## 3. 判据实现 ↔ spec v3 §10.1 对照

| spec v3 §10.1 要求 | 实现位置 | 说明 |
| --- | --- | --- |
| 读 §2 参考（rgb/depth/seg 各 6 个 world×相机键 + state） | `render_per_world_light.rs::load_reference` | 运行时读 `rgb/depth/seg/state.npz` + `capture.json`；同时保留**原始 f32 rgb**（`Reference::rgb_f32`，供 C1）与量化字节（供 A 的 rgb 0 字节差）；缺目录/缺文件显式 `Err`（→「未执行」） |
| §4 判据 A：两台相机各自逐 world 严格位级 | `light.rs::criterion_a` / `compare_frame` | rgb **0 字节差**（`max_byte_diff==0`、`mismatched_pixels==0`）、depth 位级全等（`mismatched==0`、`beyond_ulp==0`）、seg `(id,type)` 全等；无 allowlist/容差 |
| §5 判据 B：灯字段数据面位型 | `light.rs::criterion_b` + `world_light_poses` | 参考 `state.npz` 的 `light_xpos/light_xdir` f32 位型与 `WorldLightPoses`（逐位透传、不 FK/f64/不重归一化）逐位相等，0 例外 |
| §6 判据 C：参考侧自算 C1/C2 | `light.rs::criterion_c` / `rgb_above_threshold` / `verdict` | C1=cam-persp **原始 f32** rgb 的 `max_channel|Δrgb| ≥ 1/255` 计数（f32 差值提升 f64 后比较，逐位复刻 G-L0 `readout_c1_c2.py`；**不以量化字节差代替**，254↔255 类相邻码步进不计）；C2=cam-hinge depth 位型 + seg pair 计数；阈值 `ACCEPT=384`/`MARGIN=576`；两对 world `[(0,1),(1,2)]`；相机归属写死（不可互换）；诊断项只记录 |
| 两跑 sha | `light.rs::sha256_frames` + GPU 用例 | 两次产品渲染 rgb/depth/seg 整块 sha256 相等 |
| world 隔离性断言（两台相机） | `light.rs::isolation_changed` + GPU 用例 | 仅改 world1 几何+灯位姿：world0/world2 两台相机逐位不变、world1 至少一台相机变化；不替代 A/B/C |
| 产品段 = Card H 接口 | GPU 用例 | `render_per_world_with_lights(...)` + `WorldLightPoses`（`state.npz` 逐位透传）；`camera_poses` 用产品 f32 quat 路径 |
| 运行门控（参考不存在） | `#[ignore]` + 显式错误 | 默认门禁不失败；`cargo test ... -- --ignored ...` 强制跑时显式 panic「not executed: reference missing（G-L1 pending）」 |
| QA 只读复核点 | 本 README §3/§4/§8 | world/相机顺序、像素计数方向、阈值换算、`1/255` 口径、灯位型比较、C1/C2 不互换、无 allowlist 均可在 §4 对照与 CPU 自证定位；§8 记录 QA F1/O2 修正 |

## 4. CPU 合成自证（零 GPU）

日志：`logs/selfproof.log` → `15 passed; 0 failed; 2 ignored`（`cargo test --locked --features cuda-probe --test render_per_world_light -- --test-threads=1`，exit 0；
F1/O2 修正前为 `13 passed; 1 ignored`）。用例如下（覆盖 Chief 要求）：

- **verdict 边界 383/384/575/576**：`verdict_boundaries_383_384_575_576`（恰 576=pass-margin；575/384=insufficient；383=non-discriminating；六个集合逐个边界）。
- **某套 0/0 时 C2 必失败**：`zero_c2_set_fails_even_when_c1_is_large`（C1 再大也不能救 C2 0/0）。
- **C1/C2 不得互换**：`criterion_c_counts_c1_on_cam_persp_and_c2_on_cam_hinge`（正向）+
  `criterion_c_rejects_camera_swapped_reference`（反向构造：几何变化在 cam-persp、rgb 变化在 cam-hinge → C1=C2=0 且判 non-discriminating）。
- **C1 阈值口径（F1 回归）**：`criterion_c_c1_counts_f32_threshold_1_over_255_not_byte_steps`（f32 1/255 步进计入、半步不计、`254/255→1.0` 饱和步进**不计**；量化字节法会把它多算，正是 F1 差值来源）。
- **统计方向/顺序**：`criterion_c_pair_order_is_world_major`（pairs 顺序 [0→1, 1→2]）。
- **判据 A 位级**：`criterion_a_detects_rgb_depth_seg_bit_differences`（rgb 单字节差、max_byte_diff 取通道最大、depth 位差 0 ULP 口径、seg pair 差、长度不同显式失败）+
  `criterion_a_aggregates_worlds_and_cameras`（统计定位到 world/cam）。
- **判据 B 位型**：`criterion_b_compares_light_bits_and_reports_mutations`（逐位透传 0 例外；单 bit 变异必须被检出）。
- **两跑 sha / 隔离**：`sha_and_isolation_helpers_detect_only_world1_change`（任一 world 位变化改变整块 sha；world0/world2 不变、world1 变化）。
- **npz 解析**：`npz_parser_reads_members_and_npy_header`（合成 ZIP_STORED+npy 正例；缺成员/非 zip 负例）。
- **参考缺失显式报错**：实体 `missing_reference_reports_explicit_not_executed_error`；**强制 ignored 跑**见
  `logs/not-executed-check.log`（exit 101 + `not executed: reference missing (G-L1 capture pending)...`，属预期，非本卡失败）。
- **capture provenance 自核负例（O2 扩到 20 项）**：`capture_provenance_self_check_rejects_bad_or_missing_fields`（两跑标记/captured_at/tag/scene sha/bytes/nrender/nworld/cam_id_map/cam_res/cam_active/legacy 计数/camera 字段缺失/fovy/sensorsize/intrinsic/quat/顺序/argv/scope/legacy 未改动，逐项必须被拒）。
- **capture C1 交叉断言 / scope 形状**：`capture_c1_cross_check_requires_recorded_counts_match`（capture 记录 C1 时必须逐值一致，缺失则跳过）+
  `capture_scope_text_handles_string_and_list`（per_world_scope 字符串与列表两种形状同口径）。
- **本机 pilot 交叉核对（`#[ignore]`，非默认门禁）**：`pilot_c1_crosscheck_matches_readout` 读 `target/…/pilot-out/nw3-light-hinge02040`
  （pilot 产物不入库）→ C1 `[38_285, 40_644]`、C2 depth `[35_828, 33_883]`、seg `[16_041, 16_754]`、verdict pass-margin，并额外比对窗内 `window/readout.json`；见 `logs/c1-crosscheck.log` 与 `artifacts/c1-crosscheck.json`。

## 5. 边界与门禁

- 只新增：`tests/render_per_world_light.rs`、`tests/support/render_bvh/light.rs`、
  `tests/support/render_bvh/light/tests.rs`、`tests/support/render_bvh/light/model-315.json`、
  `tests/support/render_bvh/light/model-315.provenance.json`、本证据目录。
- `unchanged fixtures/`、`unchanged src/`、`unchanged scripts/`、spec 目录与 pilot 材料目录不变；零 GPU、不采集、不建 CUDA。
- 四门禁 fmt/clippy/clippy-cuda/unit 全过（F1/O2 修正后 **unit 1386 过 / 0 败 / 401 ignored**；修正前 1384 过）；manifest 自核与编码卫生见 `manifest.txt`。
- 停手条款核查：**未触发**——实体完全基于 Card H 已有公开接口与现有 `src/` API，无需改内核/ABI/`src/`。

## 6. 非声明

本卡只实现判据实体与测试期模型 dump；**不宣称** per-world 灯验收通过、不宣称与上游实现等价、
不宣称判别力恢复、不宣称任何渲染数值结论。**G-L2 真跑前不得声称实体通过**；参考缺失时本实体显式报「未执行」。
pilot 结论（`pass-margin`，C1 38,285/40,644；C2 depth 35,828/33,883、seg 16,041/16,754）来自窗卡
`2026-11-08-render-perworld-l-v2-pilot-g-l0-window`（`268380d8`/`66bac4f6`），仅作本实体判据 C 的
参考侧口径背景，不替代 G-L2 实测。

## 7. 证据清单

| 路径 | 说明 |
| --- | --- |
| `README.md` / `receipt.json` / `manifest.txt` | 本文件 / 机器可读回执 / sha256+bytes 清单 |
| `materials/dump_model_315_light.py` | 离线 3.15 dumper（provenance 见其运行输出与 `artifacts/model-315.provenance.json`） |
| `artifacts/model-315.provenance.json` | dump provenance 副本（与 `tests/support/…/light/model-315.provenance.json` 相同） |
| `artifacts/c1-crosscheck.json` | F1 交叉核对：pilot 参考上 old（量化字节）/new（f32 1/255）C1、差集构成、pilot npz sha（npz 在 `target/`、不入库） |
| `logs/selfproof.log` | CPU 合成自证原始输出（15 passed / 2 ignored，exit 0；F1/O2 修正后重跑） |
| `logs/not-executed-check.log` | 强制 `--ignored` 跑 GPU 用例的显式「未执行」证据（预期失败，非卡失败） |
| `logs/c1-crosscheck.log` | 本机 pilot C1 人工交叉核对原始输出（38,285/40,644；exit 0；pilot npz 在 `target/`、不入库） |

## 8. F1/O2 修正（QA 复核 PASS-with-observations，2026-11-08；fix 提交）

QA 复核卡 `2026-11-08-render-perworld-l-v2-criteria-entity-check` 判 **PASS-with-observations**；本卡按
Chief 派工在 **G-L2 冻结前**修两处（**判据 A/B/C 阈值与 A/B/C 语义不动**，只改 C1 计数口径与 provenance 自核强度）：

**F1（中）：C1 计数口径与已签 §6 不一致（已修）**

- 旧实现：参考 rgb 量化回 8-bit 后比 `|Δbyte| ≥ 1`。pilot attempt 3 参考上得 **43,604 / 41,815**。
- 已签 spec v3 §6 原文：f32 `max_channel |Δrgb| ≥ 1/255`（与 G-L0 `readout_c1_c2.py` 同式），
  参考侧应为 **38,285 / 40,644**。旧法方向偏宽，差 **+5,319 / +1,171**。
- 修法：`Reference` 新增**原始 f32 rgb**（`rgb_f32`，加载时逐值保留，不量化）；C1/诊断项改用
  `rgb_above_threshold`（逐通道 f32 差提升 f64 → 取 max → `>= 1/255`，逐位复刻 readout）。
- **交叉核对**（pilot attempt 3 参考，npz 在 `target/`、非入库；`artifacts/c1-crosscheck.json`）：
  - 新 C1 = **38,285 / 40,644**，与修复后 `readout_c1_c2.py` 及 `window/readout.json` 逐值一致（`logs/c1-crosscheck.log`）；
  - 旧法 43,604 / 41,815 亦复现 QA 数值；差集 **全部是量化字节差恰 1 而 f32 差 < 1/255 的相邻码像素**
    （`float_only_px = 0`）：0→1 的 5,319 中 254↔255 饱和步进 977、其余为 240–249 等高位相邻码步进的 f32 舍入；
    1→2 的 1,171 **全部**是 254↔255。
  - 差集归因的精确分布（top 字节对）见 artifact；GPU 用例在参考可用时另断言「实体 C1 == capture.json 记录的 C1（若字段存在）」。
- 新增 CPU 自证 `criterion_c_c1_counts_f32_threshold_1_over_255_not_byte_steps`：1/255 步进计入、半步不计、
  `254/255 → 1.0` 饱和步进不计（量化字节法会多算——F1 差值来源）。

**O2（低）：capture provenance 自核只查字段存在（已补强）**

- **值与 §2.1 比对**：`cam-persp`/`cam-hinge` 的 `mode/fovy/resolution/pos/quat(归一化后)/sensorsize/intrinsic`
  逐值比对（容差 1e-6，sensorsize/intrinsic 按 3.15 f32 位型 `0.036f32 as f64` 等），并核 `recipe.scene_xml` 的
  sha256 `4b354054…`/5,134 B（场景身份）。
- **command**：`capture.command.argv` 非空且含 `capture_render_perworld.py`、`--scene`、场景路径、`--reference-tag`、
  tag、`--worlds 3`、`--repeat 2`、`--use-fast-math false`、`--use-shadows true`。
- **per_world_scope**：`covers` 归一化（字符串保持原样、列表逐元素 join，与 readout `scope_text` 同口径）后必须含
  per-world light poses + per-world geom poses（含 `light_xpos/light_xdir`、`geom_xpos/geom_xmat`）。
- **交叉核对**：`resolved` 的 `nworld/nrender/cam_id_map/cam_res`、`parameters.cam_active`、
  `legacy_manifest_entries`（或 `legacy_reference_files`）=112、`legacy_reference_untouched == true`；相机条目字段数恰 10、slot 顺序 0/1。
- 负例自证从 5 项扩到 20 项（字段缺失、值错、顺序换、argv 缺、scope 缺、legacy 错均必须拒绝）。
- **附带修正**：运行时参考路径改为 `CARGO_MANIFEST_DIR` 解析（原相对 `../fixtures/…` 在 `cargo test` 工作目录下
  会解析到仓外 → G-L2 会误报「未执行」）。pilot 交叉核对同用 `CARGO_MANIFEST_DIR/target/…`。

**修正后状态**：行数 469 / 500 / 465（≤500）；自证 `15 passed / 0 failed / 2 ignored`；四门禁全过（unit 1386/0）；
未跑 GPU、未采集，本卡仍**不宣称**任何渲染数值结论。

