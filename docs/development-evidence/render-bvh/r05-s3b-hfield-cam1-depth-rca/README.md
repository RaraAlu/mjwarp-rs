# R05-S3b hfield cam1 depth 判据红只读定位（RCA）

任务：`2026-10-28-render-bvh-r05-s3b-hfield-cam1-depth-rca`（只读 RCA；不改判据/容差/参考/fixture，不开窗）。
全部数字来自**本地调试跑**，不是窗口验收结论。

## 1. 结论（四选一）

**② 射线方向构造 / 相机本地帧层**：`nmesh=0` 发射修复后暴露的 cam1 depth 差异，根因是
**固定相机的世界位姿（`cam_xmat`）派生在我们 host 侧与 pinned 上游采集时使用的位姿相差 1–2 ULP**。
不是 ① 深度合成、不是 ③ hfield/mesh 相交算术、不是 ④ 参考语义。

- 上游采集输入：`mujoco_warp` 的 `d.cam_xmat`（GPU 上由 `smooth.py` 相机 kernel 计算，内部调用
  `mujoco_warp/_src/math.py::quat_to_mat`，编译选项 `fast_math=false` + `fuse_fp=true` → NVRTC `--fmad=true`）。
- 本产品输入：S3b 目标用 `CameraPose::from_fields`（host Rust）派生同一 `cam_xmat`，公式为
  `src/math/mod.rs::quat_to_mat`（代数展开式，与 `mujoco_warp math.quat_to_mat` 源码逐项相同），
  但 **host Rust f32 不自动收缩（无 FMA）**，而 GPU 编译产物会收缩 → 同一公式、不同舍入。
- cam0：`cam-top` 四元数恰为 `(1,0,0,0)`，两种算术都得精确单位矩阵 → **零差异**；
  cam1：`cam-side` 由 `xyaxes` 派生的非平凡四元数 → 9 个矩阵元素中 **5 个差 1–2 ULP**（近零元素残差也不同），
  world 射线方向逐像素 1–2 ULP → 命中 `t` 差 1–7 ULP → depth 判据红。

## 2. 机器证据（逐层排除）

先复现基线（本地调试，非窗口；命令见 §8）：
`render_hfield_reference` 两用例：RGB/seg 位级全等，**cam0 depth mismatch=0 / max_ulp=0**，
**cam1 depth mismatch=4423（hfield-only）/4389（hfield-primitive）、max_ulp=7、beyond_ulp=20、首差异 cam1 pixel=16913 (x=113,y=70)**。

| 层 | 检查 | 结果 |
| --- | --- | --- |
| hfield 几何 | 我们 `build_hfield_mesh` 顶点/索引/`half` vs 上游 `build_hfield_bvh` 同函数产物 | **逐位全等**：顶点 bit diffs=0（13 顶点）、indices equal=true、half equal=true（12 三角形）；输入 16 个高度位型及 size bits 也全等 |
| 射线（相机本地） | 我们的 `RayBuilder` dir/offset vs 上游 `create_render_context` 的 `rc.ray`/`rc.ray_offset` | **逐位全等**：120000 条 dir bits 全等、offset bits 全等 |
| 输出布局 | `rgb_adr/depth_adr/seg_adr/cam_res/cam_id_map/total_rays` | 全等（[0,76800]、120000 像素、两相机分辨率一致） |
| 参考语义 | pinned 上游同 env 重渲 `depth` 与夹具 `depth.npz` 逐位比较 | **0 差异**（cam0 76800、cam1 43200，max_abs_diff=0）→ 参考就是 `dist * -ray_dir_local_cam.z` 口径，自洽可复现 |
| 深度合成源码 | 上游 `render.py:1285-1294` vs 我们 `render_megakernel.cu:201-204` | 同式 `hit.dist * -ray_dir_local_cam.z`，背景 0 一致 |
| 相交公式（Arch 候选） | 上游 `intersect.h` 的 `U/V/W==0` 双精度兜底分支命中率：host 端用同一 Woop 公式逐像素 f32 模拟（cam0 47024 / cam1 6150 命中像素） | **0 次命中（cam0 0/47024、cam1 0/6150）** → 排除该候选；`det` 与 depth ULP 差无相关（cam1 `corr(log|det|,ulp)=0.094`、`corr(1/|det|,ulp)=-0.065`，det 四分位不匹配率 0.71–0.74 平坦） |
| 三角形内位置 | cam1 mismatch 像素 vs matched 命中像素的 CPU f64 反投射 bary 分布 | 几乎相同（`min bary` p50 0.0967 vs 0.0964；边缘<1e-3 占比 0.70% vs 0.81%）→ 不是「不同三角形/tie-break」 |
| **相机世界位姿** | 我们 `CameraPose::from_fields` 的 `cam_xmat` vs 上游 `d.cam_xmat` | **cam0 逐位相同（9/9）**；**cam1 5/9 元素差 1–2（整数位差 [+1,−1,+2,−2040873,+1]），近零元素残差 −1.19e-8 vs −1.48e-8**；`cam_xpos` 两边全等 |
| 位姿差 → 射线差 → depth 差 | 用两边 `cam_xmat` 各自把同一批本地射线转 world（f32），逐像素比 | cam0：world dir 位差 **0/76800**；cam1：世界方向位差 **42449/43200 像素**，命中像素中 **6110/6150** 有方向差；cam1 depth 不匹配 4423 像素中 71.8% 落在「方向不同」集合（cam0 全等） |

补充：把内核编译换到 **NVRTC 12.9.1**（与 pinned 上游 Warp 同版本）重跑，cam1 结果不变（4423/4389、max_ulp=7、beyond=20；
`source_sha256` 不变、`ptx_sha256` 变化 `97467d83…`→`812b4fe7…`）→ 排除「我们的 NVRTC 版本/内核代码生成」这一层。

## 3. 为什么此前没有暴露

- `scene-a` 等既有深度参考实体**直接用参考 `camera_data` 位姿**（`tests/support/render_bvh/s7b1.rs::camera_data_poses`，
  测试里断言我们的 pose bits == `reference.camera_data[*].xmat_bits`；`tests/render_kernel_depth_seg.rs` 只对
  `CameraPose::from_fields` 的 quat 路径做 **≤1.0e-6 容差**检查，从未要求位等）。
- R05-S3b hfield 目标实体是**第一个**用 `CameraPose::from_fields` host 派生位姿 + **位级 depth 判据**的实体；
  该 1–2 ULP 位姿差在这个判据下暴露。
- 与本次零 mesh 发射修复无关：占位数组在零 mesh 场景不被解引用；RGB/seg/cam0 位级全等也印证发射路径正确。

## 4. 最小修复方案预核（**待 Arch/Chief 立项批准，本片未改任何代码**）

- **方案 A（推荐，host 侧对齐 GPU 收缩序）**：在 `src/render/kernel/cast.rs` 内新增一个 **warp 编译产物对齐**的
  `quat_to_matrix`（与 `mujoco_warp math.quat_to_mat` 同公式，按上游 GPU 实际 fma 收缩序用显式
  `f32::mul_add` 固定；不从 `src/math` 改全局函数），`CameraPose::from_fields` 改用它。
  - 依据：本片用 host 逐式模拟（`materials/bruteforce-summary.txt`）在 18 个矩阵元素中已能复现上游 **16 个位型**，
    仅 2 个对角项仍差 1 ULP → 缺口就是「收缩序未逐指令对齐」；精确序列需从仓库内 pinned Warp 缓存
    `target/render-bvh/r05-s3a-hfield-cache/1.15.0/wp_mujoco_warp._src.smooth_*/…sm89.ptx` 反读（只读）。
  - 影响面：`src/render/kernel/cast.rs` 当前 411 行（+~30–40 行，≤500）；不改 `src/math/mod.rs`（物理路径共用，避免大影响面）；
    不改 `.cu`、81 槽 ABI、`DEVICE_SOURCES`、`CollisionArgs`、判据/容差/参考/fixture。
  - 验证链：改后重跑 `render_hfield_reference`（期望 cam1 depth mismatch=0）→ 8 回归 exit 0 → fmt/clippy/clippy-cuda/unit/
    `--no-run`/内核门/`normify validate` → 重冻结 guards + stash + Chief 重签 → attempt=2 窗口。
- **方案 B（设备侧）**：相机位姿改由设备 kernel 计算（与上游同编译配方）；改动更大（新增/改 `.cu`、`DEVICE_SOURCES`），
  建议后续片评估，不作本片修复。
- **方案 C（测试侧，不可行/仅备选）**：S3a `capture.json` 的 `context` 块**不含** `camera_data` 位型，无法像 scene-a 那样直接取参考位姿；
  且用参考位姿会绕过产品派生路径，窗口就测不到该缺口（只可作「已知限制」记录，不做修复）。

## 5. 本片未做/局限

- 只读定位；**未改** `src/`、`tests/`、`fixtures/`、`scripts/`（临时探针已还原，`git status -- src tests` 为空）。
- 未从 PTX 逐指令反推完整收缩序（已定位到公式/收缩层，给出 16/18 的复现率）；方案 A 落地前需按 PTX 补齐。
- 未开窗、未重冻结、未改任何冻结物；`target/render-bvh/r05-s3b-window/attempt1/` 原样保留。
- 本片**零渲染数值验收结论**。

## 6. 复现与临时探针（全部在 `target/`，不入库）

- Rust 临时 dump 探针（已还原）：在 `tests/render_hfield_reference.rs` 临时加 `rca_tmp_dump_cam1_depth_inputs`，
  写出 `our-dump.json`（网格/射线/位姿/两位参考 depth/两次渲染 depth bits）后 `git checkout --` 还原。
- pinned python（warp 1.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4）：
  `target/render-bvh/r05-s3b-window/rca-cam1-depth/upstream_dump.py` 导出上游 mesh/射线/`d.cam_xmat`，
  并重渲 depth 与夹具逐位比较（0 差异）。
- 对比/模拟脚本：`analyze.py`、`stats2.py`、`campose_link.py`、`bruteforce_mjw_quatmat.py`（结果在
  `analysis-summary.json`、`step1-det-summary.json`、`campose-link.json`、`upstream-camworld.json`）。
- NVRTC 12.9.1 A/B：`target/toolchains/cuda-12.9.1-nvrtc/bin` 前置 PATH 跑 `render_hfield_reference --ignored`，
  结果与 12.8.1 相同（日志 `nvrtc129-run.log`）。
