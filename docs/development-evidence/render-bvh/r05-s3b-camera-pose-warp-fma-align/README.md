# R05-S3b 相机世界位姿对齐上游 FMA 收缩（`cast.rs` 内 warp 收缩序 `quat→matrix`）

任务卡：`2026-10-28-render-bvh-camera-pose-warp-fma-align`（priority 61，Owner Backend）
前置根因：RCA `2026-10-28-render-bvh-r05-s3b-hfield-cam1-depth-rca`
（证据 `docs/development-evidence/render-bvh/r05-s3b-hfield-cam1-depth-rca/`，收口 `a62cc6aa` / 追加 `a7beafae`）

本片**只改 host 侧相机世界位姿派生**，不动 `src/math`、不动 `.cu`、不动判据/容差/参考/fixture。
本片**零渲染数值验收结论**：数值验证留给 S3b validate 的 attempt=2 窗口。

---

## 1. 结论（一句话）

`CameraPose::from_fields` 原先用未收缩的代数展开 `crate::math::quat_to_mat` 派生 `cam_xmat`；
上游产品路径是 GPU 上 `quat_to_mat(mul_quat(world_quat, cam_quat))`，其矩阵运算在 PTX 里只有
`mul/add/sub`，**真正的 FMA 收缩发生在驱动 JIT（ptxas→SASS）**。本片用 pinned GPU 参考在 1003 条
四元数上逐元素**唯一反推**出该收缩序，在 `cast.rs` 内实现 `warp_quat_to_mat` 并替换调用点。

结果：hfield cam0/cam1 `cam_xmat` **9/9 位级相等**；world dir 位差 cam1 **42449/43200 → 0/43200**、
cam0 保持 **0/76800**；hfield 两组 cam0/cam1 depth mismatch **0**，RGB/seg 保持 0；
scene-a `cam-persp` 重放 **9/9**（修复前 2/9 差 1 ULP）；8 条回归 exit 0；内核编译门三 sha 不变。

## 2. 根因链（承接 RCA）

1. 上游采集时 `d.cam_xmat` = `mujoco_warp math.quat_to_mat` 在 GPU 上的结果，编译选项
   `fast_math=false + fuse_fp=true`（`--fmad=true`），存在 FMA 收缩；
2. 我方 host Rust 用 `crate::math::quat_to_mat`（未收缩代数展开）派生固定相机世界位姿；
3. hfield cam1 的 `cam_quat` 由 `xyaxes` 派生（非平凡），9 元素中 5 个差 1–2 ULP；
   cam0 的 `quat` 为精确恒等，两种算术都得到精确单位矩阵 ⇒ 之前一直没暴露；
4. 相机世界矩阵差 1–2 ULP → world dir 位差 42449/43200 → hfield cam1 depth 判据红
   （4423/4389、max_ulp=7、beyond=20，RCA 数字）。
5. 旧场景（scene-a / scene-shadow）的位等实体走 `tests/support/render_bvh/s7b1.rs::camera_data_poses`
   **直传参考 `camera_data` 位姿**，产品 quat 公式只在容差断言下被检查
   （`render_e2e` ≤1 ULP、`render_kernel_depth_seg` ≤1e-6），从未位级验证 —— 见 §9。

RCA 已排除项（机器证据在 RCA 证据目录）：mesh/hfield 数据位等、相机 local 射线 120000 条位等、
深度公式同式、参考语义自洽、U/V/W==0 分支命中 0、det 无关、NVRTC 12.9 版本无关、
3.15/3.12 加载期 `cam_quat` 8/8 与 `cam_mat0` 18/18 位等（版本假设否证）。

## 3. 实现依据：PTX 只有 mul/add/sub，收缩序由 GPU 参考唯一反推

### 3.1 PTX 反证（`materials/upstream-camera-kernel-ptx-excerpt.txt`）

pinned 上游相机 kernel `_cam_local_to_global_*_cuda_kernel_forward`
（`target/render-bvh/r05-s3a-hfield-cache/1.15.0/wp_mujoco_warp._src.smooth_ca013d6/…sm89.ptx`，
993 976 B / 28 448 行（`Get-Content` 计数；SHA-256 `5ff8714d…`），entry @ line 13887）：

- Slice A（`mul_quat`，line 14719–14733）：**有** `fma.rn.f32`；
- Slice B（`quat_to_mat` 矩阵运算，line 14734–14782）：只有 `mul.f32`/`add.f32`/`sub.f32` + 9 条
  `st.global.f32`，**`fma.rn.f32` 计数 = 0**。

⇒ 矩阵部分在 PTX 层没有 FMA，单次舍入的收缩是驱动 JIT 在 SASS 层做的；因此收缩序**不能从 PTX 直接
读出**，必须用 pinned GPU 输出反推。

### 3.2 反推方法（`materials/scripts/`）

- 参考链（整链，不是单四元数）：`materials/scripts/gpu_quat_reference.py`
  = pinned 环境发射 chain kernel `mul_quat(world_quat, cam_quat) → quat_to_mat`，导出
  `mul_quat_bits` 与 9 元素 `mat_bits`；world 体单位四元数下 `mul_quat` 与 `cam_quat`
  **1003/1003 行逐分量位等**（证明方式见 `materials/reference-chain.md`）。
- 覆盖：hfield cam0/cam1、scene-a cam0 + 1000 条 `default_rng(20261029).standard_normal`（f32，
  内存序 (w,x,y,z)）。
- 候选枚举（源序固定为 `mujoco_warp math.quat_to_mat`，只枚举 FMA 收缩选择）：3 个对角元各
  **12 候选**、6 个非对角元各 **3 候选**，全局组合空间 **12³×3⁶ = 1 259 712**。
- 唯一性（`materials/contraction-search-uniqueness.json`）：逐元素恰好 **1 个**候选在 1003 条上全中；
  收敛到唯一解最多只需 **16 条四元数**（各元素分别 2–16 条）。命中序：
  对角元 `d111`（`fma(-q3,q3,fma(-q2,q2,fma(q0,q0,±q11)))`），非对角元 `fmaA`
  （`2*fma(q_a0,q_a1,±乘积)`）。
- 全量对账：1003 × 9 = **9027 项 0 差异**；同一批上未收缩展开版差 **2614 项 / 947 条**
  （`materials/verify-contraction.log`、`materials/sampling-20.tsv` ≥20 条抽样）。

## 4. 代码改动（唯一生产文件 `src/render/kernel/cast.rs`）

- 新增私有 `fn warp_quat_to_mat(quat: [f32;4]) -> [[f32;3];3]`：
  `q=(w,x,y,z)`；`q11=q1*q1`；对角元 = `fma(-q3,q3,fma(-q2,q2,fma(q0,q0,±q11)))` 链；
  非对角元 = `2*fma(q_a0,q_a1,±乘积)`（`q01/q02/q03` 为被加乘积）；
- `CameraPose::from_fields` 调用点从 `crate::math::quat_to_mat` 改为 `warp_quat_to_mat`；
- FMA 使用项目既有精确实现 `crate::math::fma::fma_f32`（硬件 `vfmadd231ss` 主路 + 整数软兜底），
  **不用 `f32::mul_add`**：本工具链 debug 构建的软件路径实测差 1 ULP（机器证据
  `materials/fma-toolchain-note.md`），会导致 debug/release 位型漂移；
- `src/math/mod.rs`/`src/math/fma.rs` **零改动**（物理路径共用，fma 只被调用）；
- `cast.rs` 内 `#[cfg(test)]` 新增 1 条位型单测（4 个案例：恒等 / hfield cam-side / scene-a cam-persp /
  随机抽样反例），无 CUDA、普通 `cargo test --locked` 可跑；
- 行数：`cast.rs` **411 → 481 行（LF 口径，+72/−2）**，≤500 棘轮；`materials/line-counts.md`。

## 5. 位级对照（`materials/cam-xmat-bits.json`）

| 相机 | 修复前（未收缩展开） | 修复后（本片） | 上游 GPU/capture | 差异 |
| --- | --- | --- | --- | --- |
| hfield cam0 | 单位矩阵 9/9 | 单位矩阵 9/9 | 单位矩阵 9/9 | 0/9 |
| hfield cam1 | 5 个元素差 1–2 ULP（如近零元素 `0xb3000000`） | **9/9 位等**（近零 `0xb31f2429`） | **9/9** | **0/9** |
| scene-a cam-persp | 2 个元素差 1 ULP（index 0/7） | **9/9 位等** | 已 verified `camera_data_check[0].xmat_bits` | **0/9** |

## 6. 决定性验证（**本地调试、非验收**）

- world dir 位差（同一 local dirs、同一 plain 行点积口径）：cam1 **42449/43200 → 0/43200**；
  cam0 **0/76800 → 0/76800**；
- `render_hfield_reference` 两组：cam0/cam1 depth **mismatch=0、max_ulp=0、beyond=0**，
  RGB/seg 0；修复前 cam1 为 4423/4389、7、20；
- 8 条回归 8/8 exit 0；scene-a `cam-persp` 重放 9/9；
- 三条硬门：hfield cam0/cam1 9/9、1003 条 9027 项 0 差异、scene-a 9/9。
- 全部数字标注「本地调试、非验收」（`materials/local-debug-summary.md`）。

## 7. 门禁（`materials/gates.txt`，全部 exit 0）

fmt / clippy / clippy-cuda / `cargo test --locked`（**1092 passed / 0 failed / 389 ignored**，
父提交 1091 + 本片 1 条）/ `--no-run` / 内核编译门 / `normify validate --fingerprints`（错误 0）。
内核三 sha 与父提交逐位相同：`source_sha256=72bc364c…`、`ptx_sha256=97467d83…`、`ptx_bytes=264 015`
⇒ 只改 host Rust，未触碰设备源（`DEVICE_SOURCES`/81 槽/`CollisionArgs` 不变）。

环境说明：工作树 `src/render/kernel/cuda/render_megakernel.cu` 曾因环境原因呈 CRLF（git blob 仍 LF、
`git status` 一度误报 M）；按 committed blob 归一化回 LF 后内核门 source_sha256 回到 `72bc364c…`
（过程见 `materials/kernel-gate-after.log` 与 `materials/zero-change-paths.md`）。

## 8. 影响面清单

1. **改哪些文件/行数**：只 `src/render/kernel/cast.rs`（411→481，+72/−2）；`src/math` 不动。
2. **哪些 exe/目标会变 sha**：只影响走 `CameraPose::from_fields` 的 host 派生路径。9 个 render
   exe 中实际引用该路径的：`render_camera_rays`、`render_context_lifecycle`、`render_kernel_depth_seg`、
   `render_kernel_flags`、`render_kernel_rgb`、`render_r08_cameras`、`render_r10_multi_world`、
   `render_e2e`、`render_hfield_reference`；非 render 目标不引用（grep `CameraPose::from_fields`
   调用方 = `src/render/**` + `tests/render_*`）。**任何 exe 的 sha 变化都来自 host Rust 重新编译，
   与设备源/PTX 无关**（内核门三 sha 不变）。
3. **既有参考影响**：scene-a / scene-shadow 的位等实体走 `s7b1.rs::camera_data_poses` 直传参考
   `camera_data` 位姿（`tests/render_kernel_depth_seg.rs` 的 `poses[*].xmat` 位等断言仍对参考位型），
   不经过 `from_fields` ⇒ **零影响**；`render_e2e`/`render_kernel_depth_seg` 的容差断言重跑通过。
4. **必须重跑的回归**：已全部重跑并 exit 0 —— R08 `render_r08_cameras`、R09 `render_e2e` +
   `render_kernel_depth_seg`、`render_camera_rays`、`render_context_lifecycle`、`render_kernel_flags`、
   `render_kernel_rgb`、`render_r10_multi_world`、`render_hfield_reference`。
5. **红线核对**：`.cu` 零改动；`DEVICE_SOURCES=28`、81 槽、`CollisionArgs=504 B` 不变；
   判据/容差/参考/fixture/`hfield_scene.rs` 零改动。
6. **语义定性**：本片是**补齐上游 GPU 的 FMA 收缩语义**（host 派生口径与产品链一致），
   不是放宽判据、不是绕过产品路径（测试仍走 `CameraPose::from_fields` 的产品派生）。

## 9. 历史口径（精确措辞）

- **版本假设否证**：pinned `mujoco 3.15.0` vs 3.12 dump 的 `cam_quat` **8/8**、`cam_mat0` **18/18**
  f64 位型全等（cam0/cam1 均 0 差）⇒ 改夹具/重采参考不能修复本问题。
- **旧位等实体从未把产品 quat 公式放进权威比较**：scene-a `cam-persp` 重放显示旧 host 公式
  vs 已 verified 参考 **2/9 项差 1 ULP**（index 0/7）。但旧结论仍有效 —— 因为位等实体直传参考
  `camera_data` 位姿、不经过该公式；**不得写成「公式没问题」**，本片正是补这条派生口径。

## 10. 局限（`receipt.json::limits`）

1. 该收缩序是「**本栈的 ptxas/driver 对 pinned 上游代码在 SASS 级的收缩**」的实测结果；驱动/toolchain
   变化后可能需要重推（参考脚本与种子已入库，可复现重推）。
2. `CameraPose::from_fields` 这条派生路径**只覆盖固定相机**（world body 恒等四元数）；挂在运动体上的
   相机上游还会乘 `xquat_world`，本片**未覆盖**（文档限制，不扩范围；backlog
   `2026-10-29-render-bvh-host-quat-derivation-audit` 跟踪其它 host 侧 quat→matrix 派生同口径问题）。
3. 本片不产生渲染数值验收结论；hfield 数值判据以 S3b validate attempt=2 窗口为准。

## 11. 复现命令（仓库根目录）

```text
$env:PATH = 'D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1\bin;' + $env:PATH
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --all-targets --locked --features cuda -- -D warnings
cargo test --locked
cargo test --locked --no-run
cargo test --locked --features cuda-probe --test render_kernel_rgb -- --ignored megakernel_source_compiles_under_render_gate --test-threads=1 --nocapture
cargo test --locked --features cuda-probe --test render_hfield_reference -- --ignored --test-threads=1 --nocapture

# GPU 参考（pinned venv；seed 固定 20261029）
target\mjwarp-parallel\venv\Scripts\python.exe -B docs\development-evidence\render-bvh\r05-s3b-camera-pose-warp-fma-align\materials\scripts\gpu_quat_reference.py --out target\render-bvh\r05-s3b-window\fix-local-logs\gpu-reference-regen.json
target\mjwarp-parallel\venv\Scripts\python.exe -B docs\development-evidence\render-bvh\r05-s3b-camera-pose-warp-fma-align\materials\scripts\verify_contraction.py --ref target\render-bvh\r05-s3b-window\fix-local-logs\gpu-reference-regen.json
```

GPU 参考文件确定性：重生成 JSON 的 `quat_bits/mat_bits/mul_quat_bits` 与首次参考语义全等
（sha256 因 JSON 键顺序不同而不同，逐字段相等）；参考文件摘要见 `materials/reference-chain.md`。

## 12. 收口

- 代码提交 `6f246b46`（`Normify-Task: 2026-10-28-render-bvh-camera-pose-warp-fma-align`）：`src/render/kernel/cast.rs`
  + 任务卡；证据提交：本目录；closing：任务状态 + 模块指纹。
- 本片**未开窗、未重冻结 guards**；attempt=2 与 guards 重冻结等 Arch/Chief 裁决（重签材料另报）。
- QA 复核：由 Arch 在该卡登记 check 卡（`2026-10-28-render-bvh-camera-pose-warp-fma-align-check`）。
