# R09-S3b 阴影数值验收测试（作者阶段：只写测试，未开窗）

任务：`2026-10-28-render-bvh-r09-s3b-shadow-validation`（P8 渲染与 BVH；p60；owner Backend）
判据树 HEAD（开写时）：`cd1fd2e44e535476389ffccb71f570aaa213544a`
阶段：**authoring（测试作者 + 编译验证）**；本目录为作者阶段证据骨架，**窗口结果待签窗后补充**
资源：`gpu:windows-t4`（不随本阶段开窗；开窗需 Arch 报材料、Chief 签字后执行）

## 1. 新增文件与行数（LF 口径）

| 文件 | 行数 | 内容 |
| --- | --- | --- |
| `tests/render_shadow_reference.rs` | **307** | 4 个用例（1 host + 3 GPU `#[ignore]`）：shadow-on 判据、shadow-off 逐位回归、可见性对照、参考解析 |
| `tests/support/render_bvh/shadow.rs` | **414** | S3a 捕获 npz 参考的最小只读解析器（ZIP_STORED + npy v1.0；不引入 npz 依赖）+ 参考层差异计数 + 冻结 sha256 断言 |
| `tests/support/render_bvh/shadow_scene.rs` | **238** | scene-shadow 产品场景装配（`fields_from_model` + `light_castshadow=[1,1,1]`）、比较/断言/运行哈希 |

三个文件均 ≤500 行；未改动任何既有测试/参考/夹具/判据（新增文件只读引用 `capture/` npz 与既有 `s7b/*.json`）。

## 2. 用例清单

| 用例 | 状态 | 内容 |
| --- | --- | --- |
| `shadow_references_parse_and_cross_checks_hold` | **host（默认执行）** | 两组 capture npz 解析 + sha256 冻结断言 + 形状（320×240 / 240×180，76 800 / 43 200 像素）+ 参考层交叉校验（depth/seg 差异 0、RGB 差异 > 0） |
| `shadow_on_matches_captured_reference` | `#[ignore]`（GPU） | 产品 `nworld=1` + scene-shadow + `use_shadows=true` 对 `capture/shadow-on/`：RGB 每通道 ≤1 LSB、depth ≤4 ULP（位级 `mismatched==0` 优先）、seg 全等；两次运行 rgb/depth/seg sha256 相同 |
| `shadow_off_bitwise_regression_to_pre_unlock_reference` | `#[ignore]`（GPU） | ① 对 `capture/shadow-off/` 同判据；② **逐位回归解锁前既有同配方产品参考**：depth/seg 对 `s7b1::RECIPE_REFERENCE` 位级 `mismatched==0`、RGB 对 `s7b2::parse_rgb_fast_math_false_reference()` 逐字节 `max_byte_diff==0`（S8 解锁前实测口径）；③ 两次运行 sha256 相同 |
| `shadow_visibility_binary_control` | `#[ignore]`（GPU） | 语义交叉校验：产品 shadow-on vs shadow-off 的 rgb/depth/seg 差异像素数，与参考层同口径计数并列；断言产品 depth/seg 不随 `use_shadows` 变化、RGB 必须变化；**不作等价/精度结论** |

## 3. 判据与场景口径

- **参考来源**：R09-S3a 捕获窗产物 `fixtures/render-bvh/scene-shadow/capture/shadow-on|shadow-off/`（pinned MJWarp 3.15.0 + warp 1.15.0，`MJWARP_CAPTURE_AT=2026-10-28T19:09:01+08:00`），参考**只读**（`include_bytes!` + sha256 冻结断言）
- **场景**：`scene-shadow/scene.xml` 派生自 scene-a（几何/相机/材质相同），唯一语义差异是三灯 `castshadow=1`（scene-a 的 lamp 为 0）→ 产品字段复用 `s7b2::fields_from_model()`，只覆盖 `light_castshadow`
- **判据零改动**：复用 `s7b1`/`s7b2` 既有 diff 原语与容差（RGB ≤1 LSB、depth ≤4 ULP、seg 全等）；未新增/放宽任何容差；未删除或降级任何用例
- **参考配方事实（如实记录）**：S3a 捕获是 **fast-math 采集配方**（`capture.json::parameters.use_fast_math=true`），产品内核为精确 f32（既有「同配方」= `use_fast_math=False` 采集）。因此：
  - `capture/` 参考的断言按 ≤1 LSB / ≤4 ULP 判据执行（跨配方性质记录在窗口结果中，不单独宣称等价或精度结论）；
  - **同配方硬判据**由 shadow-off ② 承担（既有 `s7b1`/`s7b2` 的 `fast_math_false` 参考，S8 解锁前产品实测 `max_byte_diff=0`）
- **shadow-off 与 S0b 参考的关系**：`capture/shadow-off/` 的 depth/seg sha 与 `scene-a/reference/` 的 depth/seg 逐字节相同（S3a 记录的管线交叉校验信号）；`shadow-off` 的 RGB sha `9379f5d3…` 与 `scene-a/reference/rgb.npz` 相同

## 4. 作者阶段验证（不运行 GPU 用例、不声称窗口结论）

- `cargo test --locked --features cuda-probe --no-run --test render_shadow_reference` → 通过（产物见 `logs/build.log`）
- `cargo test --locked --features cuda-probe --test render_shadow_reference`（host 用例，不需要 GPU）→ `1 passed; 0 failed; 3 ignored`（`logs/host-test.log`）
- `cargo clippy --all-targets -- -D warnings` → 通过；`cargo fmt --check` → 通过
- 四门禁：作者阶段在 in_progress 提交的树上执行 `normify gate 2026-10-28-render-bvh-r09-s3b-shadow-validation`，结果记录在 `.normify/runs/2026-10-28-render-bvh-r09-s3b-shadow-validation/`（fmt/clippy/clippy-cuda/unit 全通过；unit 1050 passed / 0 failed）；窗口收口时按最终树复跑

## 5. 冻结目标候选（窗口材料用；窗口前按提交树重建复核）

- 新实体：`tests/render_shadow_reference.rs` → 二进制 `target/debug/deps/render_shadow_reference-8897ba50f1fe5d65.exe`
  - 作者阶段构建：**99 082 328 B** / sha256 `79056193301391c9dca1696f6a21f54d2e741fdf6d56807e8252b18d9d501957`
  - **窗口前需按提交树重新构建并复核 sha/字节**（本值只作作者阶段记录）
- 回归实体（8 个既有）：`render_camera_rays`、`render_context_lifecycle`、`render_e2e`、`render_kernel_depth_seg`、`render_kernel_flags`、`render_kernel_rgb`、`render_r08_cameras`、`render_r10_multi_world`
- 窗口命令（拟，待 Arch 签）：
  - `cargo test --locked --features cuda-probe --test render_shadow_reference -- --ignored --test-threads=1 --nocapture`
  - 8 个回归实体同参数各跑一次（`--ignored --test-threads=1`）

## 6. 限制与口径（必须随证据保留）

- 本阶段**不运行任何 GPU 用例、不开窗**；不写任何渲染数值结论，不做窗口判定
- 不写任何窗口结论、覆盖声明、精度结论或等价结论（含上游与双平台）
- 参考只读：`capture/` 与 `scene-a/reference`、`s7b/*.json` 零改动；判据/容差/夹具零改动
- 若窗口内 shadow-off ② 的逐位回归做不到（depth/seg `mismatched!=0` 或 RGB `max_byte_diff!=0`）→ 停手报 Arch，**不改判据、不放宽**
- 若窗口内 shadow-on 对 fast-math 参考出现超判据差异 → 如实记录，并按 Arch 裁决处理（可能需要同配方补充捕获；不擅自改判据）
