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
| `tests/render_per_world_light.rs` | **489** | 实体：参考 npz 解析/加载、capture provenance 自核、§4/§5/§6 + 两跑 sha + world 隔离的 `#[ignore]` GPU 用例 |
| `tests/support/render_bvh/light.rs` | **390** | 测试期 3.15 dump → `RenderFields`/`SceneState`；Card H 产品段构造；判据 A/B/C 纯函数；两跑 sha / 隔离助手 |
| `tests/support/render_bvh/light/tests.rs` | **446** | CPU 合成自证（边界/方向/互换/负例/npz 解析），由实体 `#[cfg(test)]` 挂载 |
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
| 读 §2 参考（rgb/depth/seg 各 6 个 world×相机键 + state） | `render_per_world_light.rs::load_reference` | 运行时读 `rgb/depth/seg/state.npz` + `capture.json`；缺目录/缺文件显式 `Err`（→「未执行」） |
| §4 判据 A：两台相机各自逐 world 严格位级 | `light.rs::criterion_a` / `compare_frame` | rgb **0 字节差**（`max_byte_diff==0`、`mismatched_pixels==0`）、depth 位级全等（`mismatched==0`、`beyond_ulp==0`）、seg `(id,type)` 全等；无 allowlist/容差 |
| §5 判据 B：灯字段数据面位型 | `light.rs::criterion_b` + `world_light_poses` | 参考 `state.npz` 的 `light_xpos/light_xdir` f32 位型与 `WorldLightPoses`（逐位透传、不 FK/f64/不重归一化）逐位相等，0 例外 |
| §6 判据 C：参考侧自算 C1/C2 | `light.rs::criterion_c` / `verdict` | C1=cam-persp `max_channel|Δrgb|≥1/255` 计数；C2=cam-hinge depth 位型 + seg pair 计数；阈值 `ACCEPT=384`/`MARGIN=576`；两对 world `[(0,1),(1,2)]`；相机归属写死（不可互换）；诊断项只记录 |
| 两跑 sha | `light.rs::sha256_frames` + GPU 用例 | 两次产品渲染 rgb/depth/seg 整块 sha256 相等 |
| world 隔离性断言（两台相机） | `light.rs::isolation_changed` + GPU 用例 | 仅改 world1 几何+灯位姿：world0/world2 两台相机逐位不变、world1 至少一台相机变化；不替代 A/B/C |
| 产品段 = Card H 接口 | GPU 用例 | `render_per_world_with_lights(...)` + `WorldLightPoses`（`state.npz` 逐位透传）；`camera_poses` 用产品 f32 quat 路径 |
| 运行门控（参考不存在） | `#[ignore]` + 显式错误 | 默认门禁不失败；`cargo test ... -- --ignored ...` 强制跑时显式 panic「not executed: reference missing（G-L1 pending）」 |
| QA 只读复核点 | 本 README §3/§4 | world/相机顺序、像素计数方向、阈值换算、`1/255` 口径、灯位型比较、C1/C2 不互换、无 allowlist 均可在 §4 对照与 CPU 自证定位 |

## 4. CPU 合成自证（零 GPU）

日志：`logs/selfproof.log` → `13 passed; 0 failed; 1 ignored`（`cargo test --locked --features cuda-probe --test render_per_world_light -- --test-threads=1`，exit 0）。
用例如下（覆盖 Chief 要求）：

- **verdict 边界 383/384/575/576**：`verdict_boundaries_383_384_575_576`（恰 576=pass-margin；575/384=insufficient；383=non-discriminating；六个集合逐个边界）。
- **某套 0/0 时 C2 必失败**：`zero_c2_set_fails_even_when_c1_is_large`（C1 再大也不能救 C2 0/0）。
- **C1/C2 不得互换**：`criterion_c_counts_c1_on_cam_persp_and_c2_on_cam_hinge`（正向）+
  `criterion_c_rejects_camera_swapped_reference`（反向构造：几何变化在 cam-persp、rgb 变化在 cam-hinge → C1=C2=0 且判 non-discriminating）。
- **统计方向/顺序**：`criterion_c_rgb_threshold_is_one_byte_and_uses_max_channel`（字节整数 ≥1/255、取通道最大差）+
  `criterion_c_pair_order_is_world_major`（pairs 顺序 [0→1, 1→2]）。
- **判据 A 位级**：`criterion_a_detects_rgb_depth_seg_bit_differences`（rgb 单字节差、max_byte_diff 取通道最大、depth 位差 0 ULP 口径、seg pair 差、长度不同显式失败）+
  `criterion_a_aggregates_worlds_and_cameras`（统计定位到 world/cam）。
- **判据 B 位型**：`criterion_b_compares_light_bits_and_reports_mutations`（逐位透传 0 例外；单 bit 变异必须被检出）。
- **两跑 sha / 隔离**：`sha_and_isolation_helpers_detect_only_world1_change`（任一 world 位变化改变整块 sha；world0/world2 不变、world1 变化）。
- **npz 解析**：`npz_parser_reads_members_and_npy_header`（合成 ZIP_STORED+npy 正例；缺成员/非 zip 负例）。
- **参考缺失显式报错**：实体 `missing_reference_reports_explicit_not_executed_error`；**强制 ignored 跑**见
  `logs/not-executed-check.log`（exit 101 + `not executed: reference missing (G-L1 capture pending)...`，属预期，非本卡失败）。
- **capture provenance 自核负例**：`capture_provenance_self_check_rejects_bad_or_missing_fields`（两跑标记/captured_at/tag/nrender/相机 10 字段缺失均被拒）。

## 5. 边界与门禁

- 只新增：`tests/render_per_world_light.rs`、`tests/support/render_bvh/light.rs`、
  `tests/support/render_bvh/light/tests.rs`、`tests/support/render_bvh/light/model-315.json`、
  `tests/support/render_bvh/light/model-315.provenance.json`、本证据目录。
- `unchanged fixtures/`、`unchanged src/`、`unchanged scripts/`、spec 目录与 pilot 材料目录不变；零 GPU、不采集、不建 CUDA。
- 四门禁 fmt/clippy/clippy-cuda/unit 全过；manifest 自核与编码卫生见 `manifest.txt`。
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
| `logs/selfproof.log` | CPU 合成自证原始输出（13 passed / 1 ignored，exit 0） |
| `logs/not-executed-check.log` | 强制 `--ignored` 跑 GPU 用例的显式「未执行」证据（预期失败，非卡失败） |
