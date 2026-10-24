# 渲染/BVH S6a 独立复核（28 参数 / 34 resolved / 偏移形状 / znear-zfar / 15 拒绝面 / 531）

- 任务：`2026-10-24-render-bvh-s6a-independent-check`（QA；宿主侧复核，**全程无 GPU/NVRTC/Python**）
- 复核对象：`dad622f`（S6a 功能提交，任务 `2026-10-24-render-bvh-s6a`），复核基线 HEAD `3fa5e32`
- 日期：2026-10-25
- 结论：**验收面全部成立（0 阻断）**。34 个 `resolved` 字段、2 条 `camera_paths`、输出偏移/形状与 S0b `context.json` 逐项相等；28 个公开参数默认值与上游 `create_render_context` 签名逐项一致；`znear=0.024999999441206455`、`zfar=125.0` 与独立重算（f32 `vis.map.*` × f64 `stat.extent`）逐位相等；15 组拒绝全部在分配前拒绝并映射到产品错误；531/0；纯宿主代码无 GPU/NVRTC/Python 依赖。

## 1. 复核方式（独立重算，不读交付测试断言）

| 检查 | QA 独立做法 |
| --- | --- |
| 34 resolved | QA harness（`target/qa-s6a-harness`，产品 crate path 依赖、**默认 features 无 cuda-probe**、不导入交付测试）从 S0b `context.json` 的 28 参数 + S1 `model-312.json` 的 116 渲染字段还原输入，调用产品 `resolve_context`，导出全部 resolved/camera_paths/output；QA Python 逐字段与 `context.json` 比对 |
| 28 参数默认 | QA Python 直接解析冻结上游 `render_util.py::create_render_context` 签名默认值，与产品 `ContextParams::default()` 逐项比对；并核对 S0b 调用相对默认值的非默认项 |
| znear/zfar | QA 独立按上游口径重算：f32 `vis.map.znear/zfar` × f64 `stat.extent`，与产品 f64 结果逐位比对 |
| 偏移/形状 | `rgb_adr/depth_adr/seg_adr/total_rays` 与 fixture 比对；rgb_shape 由产品 plan 分辨率独立推导比对；depth/seg 形状按 dtype 语义（depth `[h,w]`、seg `[h,w,2]` vec2i）核对 |
| 15 拒绝面 | QA 用产品公开 `ContextParams` 构造 15 个用例（相机 8、splat 4、背景色 2、模型字段 1）验证全部 Err 并记录错误变体/字段 |
| 证据/边界 | 7 个产物 git blob 哈希/行数；context 代码与测试 grep GPU/NVRTC/Python 令牌；`cargo test --locked` 531/0；harness 用默认 features 构建运行 |

## 2. 逐项结果

### ① 34 resolved 字段 + 2 camera_paths（成立）

- 34/34 字段全等（整数/布尔精确，f32 来源按 f32→f64 加宽比对）：nrender=2、cam_id_map=[0,1]、cam_res=[[320,240],[240,180]]、render_rgb/depth/seg=[T,T]、rgb_adr=[0,76800]、depth_adr=[0,76800]、seg_adr=[0,76800]、total_rays=76800、use_* 开关、background_color_unpacked=[0,0,0,255]、bvh_ngeom=**7**、enabled_geom_ids=[0..6]、znear/zfar、headlight_active/ambient/diffuse/specular、splat_count=0、geom_ray_types；
- 2 条 camera_paths 全等：cam 0 intrinsic 分支（320×240、sensorsize `[0.036,0.024]`、intrinsic `[0.018,0.016,0,0]`、fovy 73.73979461915735°）；cam 1 fovy 分支（240×180、fovy 40.0°）。

### ② 28 参数默认值（成立）

QA 从上游源码签名解析出 28 个默认值，与产品 `ContextParams::default()` **24 项检查全过**：`nworld=1`、`cam_res=None`、`render_*=None`、`use_textures/use_fast_math=True`、`use_shadows=False`、`use_ambient_lighting=True`、`enabled_geom_groups=[0,1,2]`、`cam_active=None`、`background_color=(0,0,0,1)`、`flex_render_smooth=True`、`use_precomputed_rays=True`、`render_skybox=False`、`enable_backface_culling=True`、`shadow_light_fraction=0.3`、`samples_per_pixel=1`、其余 `enable_*=True`、6 个 splat 参数 None；参数总数 **28**。S0b 调用相对默认值的非默认项仅为 `render_rgb/depth/seg=True`（`None`→按模型推导仍是真）与 `enabled_geom_groups=[0,1,2,3]`，与 resolved 结果一致。

### ③ 偏移 / 形状（成立）

- `rgb_adr/depth_adr/seg_adr = [0, 76800]`、`total_rays=76800` 与 fixture 逐项相等；rgb_pixels=76800；
- `rgb_shape = [[240,320,3],[180,240,3]]` 与 fixture 相等；depth/seg 形状按 dtype 语义推导（`[240,320]`/`[180,240]`、`[240,320,2]`/`[180,240,2]`）与 fixture 记录一致。

### ④ znear / zfar（成立）

- 产品：`znear=0.024999999441206455`、`zfar=125.0`（f64），与 fixture **逐位相等**；
- QA 独立重算：`extent=2.5`、`f32(vis.map.znear)=0.01`、`f32(vis.map.zfar)=50.0` → `0.024999999441206455` / `125.0`，**与产品完全相同**（f32 输入升 f64 后相乘，无额外舍入）。

### ⑤ 15 组拒绝面（成立）

全部在分配前 `Err`，且映射到产品错误变体/字段：

| 用例 | 产品错误 |
| --- | --- |
| bool 掩码长度不符 | `Camera(ActiveMaskLength{expected:2,..})` |
| 相机名不存在 | `Camera(UnknownCameraName{position:0})` |
| 相机 id 越界 | `Camera(CameraIndexOutOfRange{position:0})` |
| cam_res 数量不符（3 vs 2） | `Camera(ResolutionCount{expected:2,..})` |
| render 开关列表长度不符 | `Camera(RenderFlagLength{field:"render_rgb",..})` |
| spp=0 | `Camera(SamplesPerPixelZero)` |
| spp>1 + 非预计算 | `Camera(SamplesRequirePrecomputedRays{..})` |
| spp>1 + 无 RGB | `Camera(SamplesRequireRgb)` |
| splat 只给 position | `SplatMissing{field:"splat_rotation"}` |
| splat 缺 scale | `SplatMissing{field:"splat_scale"}` |
| splat rotation 形状不符 | `SplatShape{field:"splat_rotation",..}` |
| splat group_id 长度不符 | `SplatGroupCount{expected:1,actual:2}` |
| 背景色分量越界（1.5） | `BackgroundColor{channel:0}` |
| 背景色非有限（NaN） | `BackgroundColor{channel:0}` |
| 模型字段缺失/长度错 | `Camera(LengthMismatch{field:"stat.extent",..})` |

- 代码路径核对：`resolve_context` 在构建 `ResolvedContext`/输出布局前完成相机、模型派生、splat、背景色校验；无设备分配。

### ⑥ 证据 / 门禁 / 无 GPU（成立）

- 7 个产物 **7/7 sha256+行数** 与 git blob 一致（415/183/151/213/255/363/409 行，全部 ≤500）；
- `cargo test --locked`：**531 passed / 0 failed**；`cargo test --locked --test render_context_params`：**3 passed / 0 failed**；
- 纯宿主：产品 `src/render/context*` 与测试中 `SynchronousKernel|TransferSession|nvrtc|cuda|python|Command::new` **0 命中**；QA harness 用默认 features（无 cuda-probe）在无 `CUDA_PATH` 的情况下构建运行成功；
- 未启动 Python、未加载 NVRTC、未分配显存（窗口内 GPU 显存保持桌面基线）。

## 3. GPU 窗口

- 本片为宿主侧切片，**未使用 GPU**：开始/结束桌面显存均 `2898 MiB`，无本任务 CUDA 进程/分配；实际复核耗时约 20 分钟（窗口 30 分钟）。

## 4. 未满足项 / 限制

- 资源分配/所有权/生命周期属 S6b（本片 `ResolvedContext` 只是宿主解析结果，不含设备缓冲）；
- 渲染内核/AA 图像/阴影/天空盒着色/渲染发射 ABI 属 S7a/S7b；
- 模型输入受 S1 `RenderFields` 字段集限制（无 flex/hfield/splat 模型侧数组，`bvh_nflexgeom`、hfield BVH 等派生量未覆盖；splat 只做参数校验与计数）；
- 调用矩阵期望值来自上游规则语义（S0b 只捕获默认调用），默认调用与 `context.json` 逐项对照，矩阵用例是行为断言；
- 背景色越界/非有限拒绝是产品防御性硬化（上游不校验）；
- 未跑全量 G01（fast-loop 口径）。

## 5. 复核产物

- `harness-compare.json`：34 resolved/2 camera_paths/默认 28 参数/独立 znear-zfar/偏移形状/15 拒绝/产物哈希
- `no-gpu-proof.txt`：宿主侧与无 GPU/NVRTC/Python 证据
- `unit-tests-summary.txt`、`gpu-window.txt`
