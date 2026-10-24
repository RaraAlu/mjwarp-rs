# 渲染/BVH S2b 独立复核（GPU 逐射线 / 设备位姿参考 / fma / 双 world / 拒绝面）

- 任务：`2026-10-24-render-bvh-s2b-independent-check`（QA，GPU 45 分钟独占窗口由 Arch 排定）
- 复核对象：`c7b5d7f`（S2b 功能提交，任务 `2026-10-24-render-bvh-s2b`），复核基线 HEAD `f43b576`
- 日期：2026-10-24
- 结论：**复核面 9 项全部成立，未发现阻断问题**。383/382 条逐射线对照、geomid 全等、
  distance 最大 0 ULP、normal 最大 4.592e-8 rad；refit 变化集合一致；双 world 0 跨命中；
  4 项拒绝面成立；device 侧确认真 `fma.rn.f32`；参考确为设备侧 f32 位姿且 7/7 交叉校验成立；
  506 测试通过。mesh 命中（initial ray 91、moved ray 91/92）按 plan 明确留 S3，不含在通过口径内。
  未跑全量 G01，不宣称完整渲染/R01/上游全等。

## 1. 复核方式（不读交付测试断言，独立重算）

| 检查 | QA 独立做法 |
| --- | --- |
| 哈希/行数 | 直接从 git blob（HEAD 提交内容，LF 口径）重算 8 个产物的 sha256/bytes/lines，逐项比回执 |
| 参考来源 | 独立重算 7/7 交叉校验（自己比 fixture 位型与 S0b `rays.npz`、S2 `bvh_initial/moved.npz` 的 geomid/dist/normal） |
| 捕获可复现 | 把 authoring 捕获脚本复制到 `target/`（仅改输出路径），用 pinned venv 重跑两次（脚本内含双 pass 逐位一致断言），产物与入库夹具做语义逐字段比较，**仓库 fixtures 零写入** |
| 产品命中语义 | QA 自写 Rust harness（`target/qa-s2b-harness`，产品 crate 以 path 依赖方式引入、不导入交付测试/支持模块），用 **QA 用 Python 独立重算的 S2a AABB**（冻结上游 bvh.py 公式 + 文档化 fma 收缩顺序）建树，raw 输出（distance/geomid/normal 位型）落盘后由 QA Python 独立判定 |
| 精度路径 | 产品编译路径捕获 PTX（`MJWARP_PHYSICS_PTX_DIR`），核 source sha256 与场景源一致、fmad/use_fast_math 选项、fma.rn.f32 指令数、f64 指令数 |
| 门禁/边界 | `cargo test --locked` 全量 CPU 测试、交付 GPU 测试原样重跑、`git diff` 核 runtime 未改、新文件行数、产品测试无进程启动 |

## 2. 逐项结果

### ① 逐射线 initial 383/384、moved 382/384（成立）

QA harness 原始输出经 QA 独立比较：

```text
initial: compared=383 hits=157 misses=226 skipped_mesh=1 max_ulp=0 max_normal_angle=4.5923248e-08 geomid_mismatches=0
moved:   compared=382 hits=155 misses=227 skipped_mesh=2 max_ulp=0 max_normal_angle=2.8723007e-08 geomid_mismatches=0
```

- 跳过射线由 QA 从参考 geomid 独立判定：initial `[91]`、moved `[91, 92]`，均为 mesh 命中（enabled 集合中只有 geom 3 是 mesh 类型 7）。
- 未命中射线：QA 同时核产品 `distance == -1.0`（位型 `0xBF800000`）、`geomid == -1`、normal 全零，0 例外。
- distance 全部逐位相等（max ULP **0**，容差登记为 4 ULP），normal 最大夹角 **4.592e-8 rad**（moved 2.872e-8），远小于 1e-4 容差；夹角用 f64 `atan2(|cross|, dot)` 度量（交付说明的口径，非放宽）。
- 交付 GPU 测试原样重跑：`3 passed / 0 failed`，打印数字与上表一致（`product-gpu-test.log`）。

### ② geomid 全等 / distance max ULP 0 / normal ≤1e-4（成立）

见 ①：0 geomid mismatch、0 ULP、normal 两个 case 均 ≤4.6e-8 rad。

### ③ refit 变化集合 `[99,109,116,117,155]`（成立）

- QA 独立从产品 raw 输出重算：initial→moved 的 geomid 变化集合（排除 mesh 射线后）恰为
  `[99, 109, 116, 117, 155]`；参考侧全集合 `[92, 99, 109, 116, 117, 155]`，其中 ray 92 变为 mesh 命中，按 plan 留 S3。
- 追加独立检查：**refit 结果与新位姿下重新建树的结果逐条完全一致**（384 条 distance/geomid/normal 差异 0），refit 路径没有残留旧包围盒。

### ④ 双 world 384×2 查询 0 跨命中（成立）

- world 0 与单 world 结果逐条一致（0 差异），world 0 命中 geom id 全部 < 100；
- world 1 命中恰为 world 0 命中 +100，且 distance 位型逐条相同；未命中射线两 world 一致；
- QA 独立核对 384×2 条，`world0_errors=0`、`world1_errors=0`（跨 world 候选 0）。

### ⑤ 4 项拒绝面（成立，另含空场景构建层）

QA harness 直接调用产品 API，全部在打包/发射前返回 `Err`，不是空壳：

```text
reject empty_rays true
reject world_out_of_range true
reject geometry_length_mismatch true
reject hfield_type true
reject empty_scene_build true
```

代码路径核对：`SceneBvhQuery::query` 在分配/上传前依次校验 rays 非空、leaf geometry 长度、world 范围、
节点/leaf 计数、栈深度、geom 类型（hfield 显式拒绝）；`build_scene_bvh` 拒绝空场景。
交付测试 `rejects_invalid_query_inputs` 原样通过。

### ⑥ device 侧真 fma.rn（成立）

用产品编译路径（`SynchronousKernel::compile`，NVRTC）在捕获目录记录 PTX（`ptx-record.json`、`ptx-fma-summary.txt`）：

- `fmad: true`、`useFastMath: false`、options `--std=c++17 --gpu-architecture=compute_70`、NVRTC 12.8；
- PTX 的 `sourceSha256 = 8dad4d61…56d`，与入库 `src/render/bvh/cuda/scene.cu` 的 blob 哈希逐字一致（编译的确实是提交源码）；
- PTX 中 `fma.rn.f32` **166** 条、`mul.rn.f32` 0 条、`mul.f32`（未标舍入，默认 rn）132 条、`sqrt.rn.f32` 13 条、`div.rn.f32` 15 条；
- **f64 指令 0 条**（PTX 文本里出现的 "f64" 只是 `%f640` 这类寄存器名）；fast-math 未开启。
- S2a 主机侧 f64 复刻不在 device 路径（该文件只有 f32 运算；PTX 精度门禁 `check_physics_ptx` 通过）。

### ⑦ 参考必须用设备侧 f32 位姿 + 7/7 交叉校验（成立）

- QA 独立重算 7/7（不读夹具 `cross_check_s0b_s2` 布尔值）：initial vs S0b `rays.npz` 的 geomid/dist 位型/normal 位型、
  initial vs S2 `bvh_initial.npz` 的 geomid、moved vs S2 `bvh_moved.npz` 的 geomid/dist 位型/normal 位型 —— **7/7 全 true**。
- 夹具里的 `geom_xpos/xmat` 确为设备侧位姿：与 S2 npz 存的 CPU 位姿相比，xpos 21 个分量中 8 个不同（最大 31 ULP）、
  xmat 63 个分量中 39 个不同（最大 11 ULP）。这些差异足以让 cylinder 类命中距离漂移超过 4 ULP 预算，
  交付“必须用设备位姿”的结论成立。
- 重跑捕获（`target/` 隔离输出，未写仓库夹具）：两次 pass 逐位一致（第二遍 0.139 s），产物与入库
  `reference_hits.json` 的**全部语义字段逐项相等**（ray_count、enabled、rays 位型、两 case 的位姿/命中位型、
  stats、cross_check、sources；`semantic_mismatches=[]`、`environment_equal=true`），
  文件哈希不同仅因环境时间戳/显存数字/复核任务 id 字段。

### ⑧ 证据 / 门禁 / 边界（成立）

- 回执 8 个产物（scene.cu、cuda/mod.rs、bvh/mod.rs、GPU 测试、support、捕获脚本、fixture、README）
  **8/8 哈希、字节、行数与 git blob 一致**；新文件全部 <500 行（462/283/188/336/251）。
- `src/runtime` 在 `f9adf14..c7b5d7f` **零改动**；`src/render/bvh/mod.rs` 相对 S2a **纯加 4 行**（cuda 挂载与条件 re-export）。
- 产品测试不启动 Python：测试/支持模块只读 `include_str!` 夹具，`src/render/bvh/cuda/` 与 `build.rs` 无进程启动。
- `cargo test --locked`（无 cuda-probe）：**506 passed / 0 failed**（S2a 主机测试无回归）；交付 GPU 测试 `3 passed / 0 failed`。

## 3. GPU 窗口记录

- 开始：`3522 MiB`，驱动 `596.36`、CUDA 13.2、RTX 4070 Ti SUPER（16 GiB）；开始前无其他满载任务。
- 捕获重跑：baseline `3484 MiB` → 进程内 `3693 MiB` → 结束 `3470 MiB`；
- 产品 GPU 测试与 QA harness：`3782 MiB` / `3813 MiB` 峰值级，含 NVRTC 编译，结束 `3521 MiB`；
- 复核结束后无本任务 CUDA 计算进程，显存回桌面基线（`3518–3521 MiB`，桌面应用波动范围；Arch 给的基线约 3.2 GiB）。

## 4. 未满足项 / 限制（如实登记）

- mesh 三角形求交与 mesh 命中语义留 S3：initial ray 91、moved ray 91/92 不计入通过口径。
- hfield 未实现（显式拒绝）；SDF 仅按 MESH 分支返回 miss；旋转 plane 命中路径无独立 GPU 参考。
- 单一冻结场景 scene-a + 384 条固定射线，无 fuzz/随机射线集；不支持参数化剔除（bodyexclude/geomgroup/flg_static/不可见材质）。
- 固定 128 层遍历栈；更深的 BVH 在打包前拒绝（本 fixture 远低于上限，未做深树实测）。
- 参考是 3.15 冻结产物（产品原生保持 3.12）；不宣称 3.12/3.15 渲染等价。
- 本轮未跑全量 G01（快验口径）。
- 观察项（非阻断）：参考捕获用 `rc.use_fast_math=True`，而产品内核编译 `use_fast_math=false`；本 fixture 上 distance 0 ULP 说明未产生分歧，但属编译口径差异，建议后续切片关注。

## 5. 复核产物

- `static-check.json`：git blob 哈希/行数、7/7 交叉校验、命中统计、refit 集合、设备位姿 vs CPU 位姿
- `rerun-compare.json`：捕获重跑 vs 入库夹具的语义比较
- `harness-compare.json`：QA harness raw 输出的独立判定（ULP/夹角/geomid/refit/双 world/拒绝面）
- `harness-product-hits.txt`：QA harness 的 raw 产品输出（4×384 条 + 拒绝面）
- `ptx-record.json` / `ptx-fma-summary.txt`：产品编译路径 PTX 与 fma 统计
- `product-gpu-test.log`、`unit-tests-summary.txt`：交付 GPU 测试与 506 测试记录
