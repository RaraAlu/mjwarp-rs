# R09-S3b 阴影数值验收测试（参考已切 capture-exact；窗口未开）

任务：`2026-10-28-render-bvh-r09-s3b-shadow-validation`（P8 渲染与 BVH；p60；owner Backend）
判据树 HEAD（作者阶段）：`cd1fd2e44e535476389ffccb71f570aaa213544a`
阶段：**作者完成 + 参考已切 `capture-exact/` + 窗口材料就绪**；本目录为窗口前证据，**窗口结果待开窗后补充**
资源：`gpu:windows-t4`（需单独签验证窗；本阶段未开窗、未跑任何 GPU 用例）

## 1. 文件与行数（LF 口径）

| 文件 | 行数 | 内容 |
| --- | --- | --- |
| `tests/render_shadow_reference.rs` | **307** | 4 个用例（1 host + 3 GPU `#[ignore]`）：shadow-on 判据、shadow-off 逐位回归、可见性对照、参考解析 |
| `tests/support/render_bvh/shadow.rs` | **416** | `capture-exact/` 同配方 npz 参考的最小只读解析器（ZIP_STORED + npy v1.0；不引入 npz 依赖）+ 参考层差异计数 + 冻结 sha256 断言 |
| `tests/support/render_bvh/shadow_scene.rs` | **238** | scene-shadow 产品场景装配（`fields_from_model` + `light_castshadow=[1,1,1]`）、比较/断言/运行哈希 |

三个文件均 ≤500 行；未改动任何既有测试/参考/夹具/判据。

## 2. 用例清单

| 用例 | 状态 | 内容 |
| --- | --- | --- |
| `shadow_references_parse_and_cross_checks_hold` | **host（默认执行）** | 两组 `capture-exact/` npz 解析 + sha256 冻结断言 + 形状（320×240 / 240×180，76 800 / 43 200 像素）+ 参考层交叉校验（depth/seg 差异 0、RGB 差异 > 0） |
| `shadow_on_matches_captured_reference` | `#[ignore]`（GPU） | 产品 `nworld=1` + scene-shadow + `use_shadows=true` 对 `capture-exact/shadow-on/`（**同配方**）：RGB 每通道 ≤1 LSB、depth ≤4 ULP（位级 `mismatched==0` 优先）、seg 全等；两次运行 rgb/depth/seg sha256 相同 |
| `shadow_off_bitwise_regression_to_pre_unlock_reference` | `#[ignore]`（GPU） | ① 对 `capture-exact/shadow-off/`（同配方）同判据；② **逐位回归解锁前既有同配方产品参考**：depth/seg 对 `s7b1::RECIPE_REFERENCE` 位级 `mismatched==0`、RGB 对 `s7b2::parse_rgb_fast_math_false_reference()` 逐字节 `max_byte_diff==0`（S8 解锁前实测口径）；③ 两次运行 sha256 相同 |
| `shadow_visibility_binary_control` | `#[ignore]`（GPU） | 语义交叉校验：产品 shadow-on vs shadow-off 的 rgb/depth/seg 差异像素数，与参考层同口径计数并列；断言产品 depth/seg 不随 `use_shadows` 变化、RGB 必须变化；**不作等价/精度结论** |

## 3. 判据与参考口径

- **参考来源（已切换）**：`fixtures/render-bvh/scene-shadow/capture-exact/{shadow-on,shadow-off}/`（R09-S3a-capture-exact 窗口，**产品同配方 `use_fast_math=false`**，`MJWARP_CAPTURE_AT=2026-10-28T19:44:02+08:00`）；参考**只读**（`include_bytes!` + sha256 冻结断言）
- **`capture/`（fast-math 组）明确不参与测试断言**：只作历史记录与跨配方诊断（其与 `capture-exact/` 的差异由 capture-exact 证据记录：depth cam0 19 074/max 2 ULP/超 4 ULP 0、cam1 4 064/max 4/0；rgb cam0 max byte 1/超 1 LSB 0、cam1 0/0）
- **场景**：`scene-shadow/scene.xml` 派生自 scene-a（几何/相机/材质相同），唯一语义差异是三灯 `castshadow=1`（scene-a 的 lamp 为 0）→ 产品字段复用 `s7b2::fields_from_model()`，只覆盖 `light_castshadow`
- **判据零改动**：复用 `s7b1`/`s7b2` 既有 diff 原语与容差（RGB ≤1 LSB、depth ≤4 ULP、seg 全等）；未新增/放宽任何容差；未删除或降级任何用例
- **解锁前逐位回归黄金**：shadow-off ② 用既有 `s7b/*_fast_math_false.json`（S8 解锁前产品实测 `mismatched=0 / max_ulp=0 / max_byte_diff=0`）；capture-exact 窗口已核验 `capture-exact/shadow-off/` 与该 JSON **位级一致**（seg 0 / depth 0 / rgb 0 mismatch）

## 4. 验证（作者阶段 + 参考切换后；不运行 GPU 用例、不声称窗口结论）

- `cargo test --locked --features cuda-probe --no-run --test render_shadow_reference` → 通过
- `cargo test --locked --features cuda-probe --test render_shadow_reference`（host 用例，不需要 GPU）→ `1 passed; 0 failed; 3 ignored`（参考已切 `capture-exact/`，冻结 sha 断言通过）
- `cargo clippy --all-targets -- -D warnings` → 通过；`cargo fmt --check` → 通过
- 四门禁：作者阶段在 in_progress 提交（`61b82f9a`）的树上执行过 `normify gate 2026-10-28-render-bvh-r09-s3b-shadow-validation`（fmt/clippy/clippy-cuda/unit 全通过、unit 1050 passed）；**参考切换后按最终树复跑**（窗口收口时）

## 5. 窗口材料（待签；9 实体 + runner + guards，均已冻结）

- **9 实体 exe（按最终树构建）**：

| 实体 | 二进制 | bytes | sha256 |
| --- | --- | --- | --- |
| `render_shadow_reference`（新） | `target/debug/deps/render_shadow_reference-8897ba50f1fe5d65.exe` | 99 082 328 | `87fe2018cdc084bf1c878edf461e4d03bd9f57c432b7a34ad95c81772c05c274` |
| `render_camera_rays` | `…-55a1bc63487c4d54.exe` | 89 694 775 | `0bf6cdeb7547a32e0ce2ddfc3da176d4cbf217397409e3bd797d8c1571adf4f6` |
| `render_context_lifecycle` | `…-16fa79202ab03000.exe` | 86 165 129 | `86ea45257aaf11a08b17b8d1b164dc5c27f1fef1fdfbae329c163de1a80516ef` |
| `render_e2e` | `…-48506fd461a155c9.exe` | 94 580 059 | `374ad4331c9d3dc22b89c6592eeed07846da3d5888ec853869684eea4523cba7` |
| `render_kernel_depth_seg` | `…-9349dfadfb02b210.exe` | 90 603 308 | `048e579b2027f8a57af7bf400e250eb87991071a2fb1429464e56b8f08e55d53` |
| `render_kernel_flags` | `…-c359af999d9d4cf9.exe` | 96 001 497 | `db2d67c519c125479116ba93774441d8d4765987e62b625541f98bd304808295` |
| `render_kernel_rgb` | `…-bfd65da348983af0.exe` | 94 050 109 | `d88136ab95e303d3fdd3471bafe3eac875a9c53b1960be39c7fbc140aa905dc3` |
| `render_r08_cameras` | `…-055444872456f9c8.exe` | 94 387 117 | `ad909b1cc40e3c040734de74b3e5993444783f55d12ba247afa8eed614cc90fe` |
| `render_r10_multi_world` | `…-82aedaa9008b6745.exe` | 94 152 658 | `e23a7d333217ab87e3cbbdc04df8bd4f9766722b4c5063ca8bf5cd6cf510c846` |

- **runner**：`target/s3b-validation-window/run-window-attempt1.ps1`（15 790 B / `1085ebf7…`，零编辑后签名）：5 s 采样 → 连续 60 样本干净 → guard precheck（`util<40`×3、`free≥10240`）→ 9 实体串行 `--ignored --test-threads=1 --nocapture` → **0.25 s 持续采样器**（`Start-Sleep -Milliseconds 250`，abort 阈值 util≥80×3）→ 3 点基线 + `floor=min(10240, baseline_free−3072)` → facts/entity-results/guard_aggregate；**窗内零 Rust 编译**（deps 快照校验）；窗后 9/9 exe sha 复查
- **guards**：`target/s3b-validation-window/guards.json`（18 472 B / `2a5051f3…`）：9 exe + 47 只读输入（38 既有 + `scene-shadow/scene.xml` + `capture-exact/` 8 文件）；runner `-GuardsOnly` 自检 **56/56、0 mismatch**
- **窗口命令**（runner 内固定）：`<exe> --ignored --test-threads=1 --nocapture`（每实体一次，串行）

## 6. 窗口硬要求（Chief 2026-10-28 追加，必须随证据保留）

- **S2b 解锁后的两条旧断言必须在窗口内真跑**：`render_kernel_flags` 的 `aa_supported_and_remaining_rejections_pre_launch` 与 `render_kernel_rgb` 的 `unsupported_configurations_are_rejected_before_launch` 要真正执行到「`use_shadows=true` → 接受 → `Ok(())`」断言；runner 会在对应实体日志中定位用例通过行与 marker（`AA + shadows supported` / `shadows supported; precomputed=false/skybox rejected`），写入 facts 的 `shadows_acceptance_<entity>` 并在缺失时记失败；六项回报单独列
- **0.25 s 连续采样器**：设备验证窗必须启用（authoring 例外不适用）
- 9 实体预构建冻结（sha+字节）、compute-apps 过滤 0、wall/attempt=1/≤2 有效轮、窗后 9/9 复查

## 7. 限制与口径（必须随证据保留）

- 本阶段**未运行任何 GPU 用例、未开窗**；不写任何渲染数值结论，不做窗口判定
- 不写任何窗口结论、覆盖声明、精度结论或等价结论（含上游与双平台）
- 参考只读：`capture-exact/`、`capture/`、`scene-a/reference`、`s7b/*.json` 零改动；判据/容差/夹具零改动；`capture/` 不参与断言
- 若窗口内 shadow-off ② 的逐位回归做不到（depth/seg `mismatched!=0` 或 RGB `max_byte_diff!=0`）→ 停手报 Arch，**不改判据、不放宽**
- 若窗口内 shadow-on 对 `capture-exact/` 参考出现超判据差异 → 如实记录，按 Arch 裁决处理；不擅自改判据
