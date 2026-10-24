# 渲染/BVH S5 独立复核（17 context 全网格 / 128 抽样 / spp / dynamic / 正交 / 拒绝面）

- 任务：`2026-10-24-render-bvh-s5-independent-check`（QA；Arch 30 分钟 GPU 独占窗口，实测 GPU 约 2 分钟）
- 复核对象：`1df634d`（S5 功能提交，任务 `2026-10-24-render-bvh-s5`），复核基线 HEAD `6c16d9c`
- 日期：2026-10-25
- 结论：**验收面全部成立（0 阻断）**。17 个相机 context 的射线全网格 SHA-256 全部一致、每相机 128 抽样位型全等（dir/offset 位等）、spp=1/2 的条数/子样本/顺序正确、dynamic 两对与 precomputed 逐位相同、正交单/混合覆盖、相机筛选 bool/name/index/空列表与异构分辨率正确、7 项拒绝面全部映射到产品错误字段（另设备打包前 4 项拒绝）、夹具 1,465,865 B 真实数据、516/0。AA 图像级验收按 Arch 裁决延后 S7b/S8，本片只验射线层。

## 1. 复核方式（独立重算，不读交付测试断言）

| 检查 | QA 独立做法 |
| --- | --- |
| 全网格位等 | QA harness（`target/qa-s5-harness`，产品 crate path 依赖、不导入交付测试/支持模块）从夹具的相机模型位型与 request 规格重建 17 个 plan、用产品 `RayBuilder` 生成射线，把 `canonical_bytes()` 原样写入 `target/` 二进制文件；**SHA-256 由 QA Python 计算**（不是产品 `sha2` 结果） |
| 抽样位型 | harness 导出每相机 128 像素×全部子样本的 dir/offset 位型，Python 与夹具逐项比对；并从二进制文件按 `先 dirs 再 offsets、LE f32` 独立解析 8 个下标复核布局 |
| 元数据 | plan 的相机选择/分辨率/render 标志/total_rays/rgb_pixels/samples/spp/znear/zfar/条数 与夹具 expected 逐项比对 |
| 拒绝面 | 7 项 resolve 层错误用产品 API 重建并核 `CameraError` 字段映射；4 项 builder 层（samples 不一致、相机越界、零分辨率、dynamic+spp>1）核写前拒绝 |
| 证据 | 8 个产物从 git blob 重算 sha256/行数；夹具字节/行数；无关 runtime 改动 |
| 精度路径 | 产品编译路径捕获 PTX，核 source sha256 与 `build_rays.cu` blob、fmad/use_fast_math、fma/f64 指令 |

## 2. 逐项结果

### ① 17 context 全网格 SHA-256（成立）

- 17 个 context 的 canonical 字节总量 **37,916,712 B**（每条 24 B）；
- QA Python 计算的 SHA-256 与夹具 **17/17 全等**；
- 文件长度与 `count × 24` 全等；独立按「先 dirs 再 offsets、LE f32」解析抽样下标，**17/17 布局正确**。
- 上下文哈希关系符合语义（11 个不同哈希）：`render_rgb_first`/`znear_x2_default`/`dynamic_default` 与 `default_all` 同射线（只改输出标志/znear/模式）；`names_track` 与 `mask_second` 同（都只选 camera 1）；`dynamic_ortho` 与 `ortho_mixed` 同；`empty_active`/`empty_mask` 均为空字节 SHA；
- 抽样条目数符合「每相机 128 像素」：单相机 128、双相机 256、`spp2_default` 1024（128×4×2）、异构 `res_hetero` 191（128+63，第二相机 7×9=63 像素）。

### ② 128 抽样位型 + 元数据（成立）

- 每个 context 的全部抽样条目（camera/sample/x/y/全局下标 + dir/offset 位型）**逐项全等**；
- plan 元数据逐项全等：`cameras`、`cam_res`、`render_rgb/depth/seg`、`total_rays`、`rgb_pixels`、`samples_per_pixel`、`samples`、`znear`/`zfar` 位型、条数。
- 射线条数覆盖：120,000 / 76,800 / 43,200 / 10,000 / 5,063 / 38,400 / 480,000 / 0 等，与夹具一致。

### ③ spp=1/2、顺序与子样本（成立）

- `spp2_default`：`samples=4`（spp²），条数 **480,000 = 4 × 120,000**，`znear` 与 `default_all` 相同；
- 子样本顺序（`px=xid*spp+sample%spp`、`py=yid*spp+sample//spp`）由二进制布局 + 抽样条目逐项核对通过；
- 布局 `[sample][camera][row][column]`（x 最快）成立。

### ④ dynamic == precomputed（成立）

- `dynamic_default` vs `default_all`、`dynamic_ortho` vs `ortho_mixed`：canonical SHA-256 **逐位相同**；产品 `RayMode::Dynamic` 走设备内核逐像素计算路径，结果与预计算一致。

### ⑤ 正交（成立）

- `ortho_single`（单正交 43,200 条）与 `ortho_mixed`（正交+透视 120,000 条）均标记并有正交相机；全网格位等；
- `ortho_mixed` 的 offsets 非零覆盖（抽样条目中含 offset 位型），验证正交偏移路径。

### ⑥ 相机筛选/异构/空列表（成立）

- 选择矩阵：bool 掩码（mask_first/mask_second）、name（`cam-track`）、index（`indices_reversed`，模型相机 id `[1,0]`）、空列表（`empty_active`/`empty_mask` → 0 条、空 SHA）；
- 分辨率：None→模型值、单元素复制（`res_tuple` 100×50 → 10,000 条）、异构（100×50, 7×9 → 5,063 条）、单元素列表复制（160×120 → 38,400 条）；
- 输出解析：`render_rgb_first`（rgb=[T,F]、depth=[F,T]）plan 标志位等且射线与 default 相同（只影响输出）。

### ⑦ 拒绝面 7 项（成立）

| 上游异常 | 产品字段 | 结果 |
| --- | --- | --- |
| ValueError spp=0 | `samples_per_pixel` | 拒绝，字段一致 |
| ValueError spp>1 + dynamic | `precomputed_rays_required` | 拒绝，字段一致 |
| ValueError spp>1 无 RGB | `render_rgb_required` | 拒绝，字段一致 |
| ValueError cam_res 数量不符 | `cam_res` | 拒绝，字段一致 |
| AssertionError bool 掩码长度不符 | `cam_active` | 拒绝，字段一致 |
| ValueError 相机名不存在 | `cam_active_name` | 拒绝，字段一致 |
| **IndexError** 相机 id 越界（上游未校验，直接下标崩溃） | `cam_active_index` | 产品写前拒绝（硬化差异，已记录） |

- 另 builder 设备打包前 4 项：samples 不一致、相机越界、零分辨率、dynamic+spp>1 **4/4 拒绝**。

### ⑧ 夹具与证据（成立）

- 夹具 `fixtures/render-bvh/scene-a/s5/camera_rays.json`：**1,465,865 B、69,762 行、sha256 `545a1442…2022`**，与回执/git blob 一致；3 个模型（default/ortho/znear_x2）、17 context、7 拒绝项，真实位型非占位（跨 context 一致性：mask_first camera0 的 128 个抽样与 default_all 全等、znear_x2 的 256 个抽样与 default_all 全等，0 mismatch）；
- 回执 8 个产物 **8/8 sha256+bytes+lines** 与 git blob 一致；代码文件全部 <500 行（469/299/169/357/498/345/184）；
- `cargo test --locked`：**516 passed / 0 failed**；
- 交付 GPU 测试原样重跑：**2 passed / 0 failed**（`camera-ray contexts checked: 17`、`builder rejection surface ok`）；
- PTX：`source_sha256 = e38cdf2a…` 与 `build_rays.cu` blob 一致；`fmad=true`、`use_fast_math=false`、compute_70；`fma.rn.f32` 15 条、`mul.rn.f32` 2 条、`mul.f32`（默认 rn）21、`sqrt.rn.f32` 1、`div.rn.f32` 9；f64 token 4 处**全部是 CUDA-12.8 sinf/cosf 整数角归约三元组**（`cvt.rn.f64.s64` + `mul.f64 0d3BF921FB54442D19` + `cvt.rn.f32.f64`），正是精度门禁记录在案的唯一例外，无物理 f32 量被提升到 f64。

## 3. GPU 窗口

- 开始：`2899 MiB`（桌面），驱动 596.36 / CUDA 13.2 / RTX 4070 Ti SUPER；无其他满载任务。
- harness 运行约 `2880 MiB`；交付 GPU 测试 `2878 → 2871 MiB`。
- 结束：`2871 MiB`，无本任务 CUDA 计算进程，已回桌面基线；实际 GPU 用时约 2 分钟。

## 4. 未满足项 / 限制

- **AA 图像级验收延后 S7b/S8**（Arch 裁决；产品尚无渲染内核），本片不宣称 AA 图像一致；
- 世界系射线变换（`cam_xmat @ dir`、正交 `origin + cam_xmat @ offset`）与 `max_cam_dist` 属 S7b，不在本片；
- `znear_x2` 与默认 context 的射线位型相同（归一化方向对 znear 整体缩放不变），只验证 plan 的 znear 位型（`0x3ccccccc → 0x3d4ccccc`）与公式路径；不是独立数值证据（README 已承认）；
- 单 fixture（scene-a 2 相机 + 17 context），无 fuzz/随机相机属性；相机名仅 ASCII；
- 参考为 3.15 冻结产物；正交参考来自 3.15 MjModel 副本覆盖（provenance 在夹具），产品输入来自 S1 已验收的 3.12 字段；
- 本轮未重新运行 authoring 捕获（夹具声明两次捕获逐位一致，第二遍 1.6456 s；QA 侧独立验证的是产品输出与夹具位等）；
- 未跑全量 G01（fast-loop 口径）。

## 5. 复核产物

- `harness-compare.json`：17 context 的 QA 计算 SHA、抽样位型、布局、元数据、dynamic/spp/ortho、7+4 拒绝面
- `static-check.json`：产物哈希/行数、夹具内部一致性、context 哈希关系
- `product-gpu-test.log`：交付 GPU 测试原样重跑
- `ptx-record.json` / `ptx-fma-summary.txt`：产品编译路径 PTX 与指令统计
- `unit-tests-summary.txt`、`gpu-window.txt`
