# 渲染/BVH S3 独立复核（436×5 射线 / 0 ULP / refit / 退化网格 / 设备位姿）

- 任务：`2026-10-24-render-bvh-s3-independent-check`（QA；Arch 30 分钟 GPU 独占窗口，实测 GPU 使用约 2 分钟）
- 复核对象：`8efd4ad`（S3 功能提交，任务 `2026-10-24-render-bvh-s3`），复核基线 HEAD `8efd4ad`
- 日期：2026-10-25
- 结论：**验收面全部成立（0 阻断）**。436 条射线在 5 个场景用例上 geomid/face 全等、distance/u/v max ULP **0**、normal max **4.0e-8 rad**；`tight_above` 29 命中 / `tight_below` 0 命中；`set_points`+refit 变化集合 32 条与参考一致且与重新建树逐条一致；双 world 436×2 零跨、重复查询逐位一致；退化网格 16 条/9 命中、distance max **2 ULP**、无 panic；S2b 冻结交叉校验（位姿位型、mesh ray 91/92 的 distance/normal）全部成立；两次捕获逐位一致、QA 隔离重采与入库夹具语义全等；512/0；10/10 产物哈希行数一致。
- 同时发现 **2 个非阻断问题**（见 §6）：退化网格用例存在**未覆盖的有效面**（face 1），且「刻意避开精确穿顶点/边」的表述与实现不符；被替换的首版射线**未归档位型**，无法逐位审计。

## 1. 复核方式（独立重算，不读交付测试断言）

| 检查 | QA 独立做法 |
| --- | --- |
| 哈希/行数 | 从 git blob（HEAD 提交内容，LF 口径）重算 10 个产物的 sha256/bytes/lines，逐项比回执 |
| 设备位姿/顶点 | 独立核对夹具逐用例 `geom_xpos/xmat` 与 S2b 设备位姿捕获（geom 3 切片）逐位相等；`model-312.json` 的 `mesh_vert` f64→f32 与夹具顶点逐位相等 |
| 捕获可复现 | 把 `capture_s3_mesh.py` 复制到 `target/`（仅改输出路径），pinned venv 重跑（脚本内含双 pass 逐位断言），产物与入库夹具做语义逐字段比较，**仓库 fixtures 零写入** |
| 产品命中语义 | QA 自写 Rust harness（`target/qa-s3-harness`，产品 crate path 依赖、不导入交付测试/支持模块），用夹具位型建网格/查询，raw 输出（distance/geomid/face/u/v/normal 位型）落盘，由 QA Python 独立判定 |
| 精度路径 | 产品编译路径捕获 PTX（`MJWARP_PHYSICS_PTX_DIR`），核 source sha256 与 `mesh.cu` blob、fmad/use_fast_math、fma/f64 指令计数 |
| 平局/覆盖 | 用独立 f64 Möller–Trumbore（含边界）复算 436+16 条射线：是否存在「同一最近 t 上多个非退化三角形」的布局相关平局；并核对退化用例的有效面覆盖与精确穿边/顶点命中 |

## 2. 逐项结果

### ① 436×5 射线：geomid/face 全等、distance/u/v 0 ULP、normal ≤1e-4（成立）

QA harness 原始输出经 QA 独立比较：

| case | compared | hits | misses | max distance ULP | max u/v ULP | max normal angle | face/geom mismatch |
| --- | --- | --- | --- | --- | --- | --- | --- |
| initial | 436 | 29 | 407 | 0 | 0 | 3.9995e-8 | 0/0 |
| moved | 436 | 27 | 409 | 0 | 0 | 3.4396e-8 | 0/0 |
| no_cull | 436 | 40 | 396 | 0 | 0 | 3.9995e-8 | 0/0 |
| dynamic | 436 | 28 | 408 | 0 | 0 | 0 | 0/0 |
| dynamic_moved | 436 | 24 | 412 | 0 | 0 | 3.2277e-8 | 0/0 |

- 未命中判定同时核 `distance == -1.0`、`face == -1`、`geom_id == -1`，0 例外。
- `no_cull` 40 命中 ≥ `initial` 29，背面案例覆盖成立（hit tags：initial 中 front 4/back 1/axis 13/graze 10/s0b 1；no_cull 中 back 4）。
- 交付 GPU 测试原样重跑：**6 passed / 0 failed**，打印数字与上表一致（`product-gpu-test.log`）。

### ② tight_above 29 命中 / tight_below 0 命中（成立）

- `tight_above`（max_t = 1.01×参考距离）29 命中且每条 `distance < max_t`；`tight_below`（0.99×）436 条全 miss。两条边界均为严格 `<` 语义，未放宽。

### ③ set_points+refit 变化集合 32 条（成立）

- QA 独立按交付同规则（hit/face 变化或 distance >4 ULP）重算：产品 initial→refit 变化集合 = `[91,384,386,387,388,392,394,396,397,398,399,401,404,405,406,407,410,412,414,417,418,420,422,423,425,426,427,430,431,433,434,435]`（32 条），与参考侧变化集合、夹具 `stats.set_points_changed_rays` **三者一致**。
- 追加独立检查：**refit 结果与用同一顶点重新建树的查询结果逐条完全一致**（436 条全字段差异 0），说明 refit 路径没有残留旧包围盒。

### ④ 双 world 436×2 零跨（成立）

- `moved`（geom 103）436 条查询命中 27 条，geom id 全为 103（或 -1），**0 条越 world**；
- world0 重复查询与首次逐位一致（产品查询无状态）；world0/world1 分别与 initial/moved 参考逐条一致。

### ⑤ 退化网格 16 条 / 9 命中 / 2 ULP / 无 panic（成立，覆盖有缺口见 §6-1）

- 16 条：9 命中、7 miss、face 全等、u/v 0 ULP、distance max **2 ULP**、无 panic/挂起；
- 退化/零面积三角形参与构建与查询：无崩溃、无 NaN 传播。

### ⑥ S2b 冻结交叉校验（成立）

- mesh ray：initial `[91]`、moved `[91,92]`，与 S2b `geomid == 3` 集合一致；这些射线 distance 位型与 S2b 逐位相等，normal 夹角 ≤1e-6（按位型差 0 ULP 的 normal 在 product 侧可能差几个 ULP，用角度口径核对）。
- 逐用例设备位姿位型 == S2b 设备位姿捕获（geom 3 的 xpos/xmat，initial/moved 全 12 分量逐位相等）。

### ⑦ 两次捕获逐位一致 + 捕获脚本 bug 修复真实（成立）

- 隔离目录重跑：脚本内部双 pass `bitwise_identical: true`（第二遍 0.2498 s）；QA 重跑产物与入库夹具 **semantic_mismatches = []**（mesh/rays/cases/degenerate/stats/cross_check/sources 全等），environment 全等；文件哈希不同仅因环境时间戳/显存/任务 id 字段。
- 首版 KeyError（漏写逐用例位姿）修复真实性：入库夹具**每个 case 都带 `geom_xpos_bits`/`geom_xmat_bits`**，且与 S2b 设备位姿逐位相等（不是占位/零值）；退化用例的 identity 位姿是其合成网格的独立字段，非占位。
- 夹具 sha256 `4cf448c1…a753`、434,588 B、31,331 行与回执、git blob 三者一致。

### ⑧ 测试侧 world_to_local 修正（成立，修的是测试不是产品）

- 上游 `_ray_map`（冻结 `71da24d…` ray.py L37-56）：`lpnt = matT @ (pnt - pos)`、`lvec = matT @ vec`；
- 产品 `mesh.cu` 的 `ray_map` 同样做 `R^T` 映射，且 `MeshBvhQuery::query` 的入参是**世界系射线**（宿主 ABI 注释一致）；
- 主机候选覆盖检查里原先直接用世界系射线查**局部系**网格，属测试侧错误；`tests/support/render_bvh/mesh.rs::world_to_local` 按上游公式补上映射后通过。断言本身没有被削弱（仍要求「命中三角形必须在宿主候选内」，只是输入坐标系修正）。

### ⑨ 证据 / 门禁 / 边界（成立）

- 回执 10 个产物 **10/10 sha256+bytes+lines** 与 git blob 一致；新代码文件全部 <500 行（485/371/383/244/493/320/498/199/146）。
- `cargo test --locked`：**512 passed / 0 failed**（S2/S3 主机测试无回归）。
- `src/runtime` 在 `c7b5d7f..8efd4ad` **零改动**；`src/render/bvh/mod.rs` +57/−1（mesh 挂载、`vertices_from_f64`、`MeshBvhError` Display/Error）。
- 产品测试不启动 Python（测试只读 `include_str!` 夹具）；离线 NVRTC 预检与 PTX 精度门禁通过：
  - PTX `sourceSha256 = 75ae35ac…` 与 `mesh.cu` blob 一致；`fmad=true`、`use_fast_math=false`、compute_70；
  - PTX 中 `fma.rn.f32` **36** 条、`mul.rn.f32` 0、`mul.f32`（默认 rn）40、`sqrt.rn.f32` 2、`div.rn.f32` 8、**f64 指令 0**。

## 3. GPU 窗口

- 开始：`3026 MiB`（桌面），驱动 596.36 / CUDA 13.2 / RTX 4070 Ti SUPER；无其他满载任务。
- 捕获重跑：baseline `2868` → 进程内 `3111` → 结束 `2867`；交付 GPU 测试 `2891 → 2867`；harness 峰值约 `2890`。
- 结束：`2884 MiB`，无本任务 CUDA 计算进程，已回桌面基线（桌面应用波动 2.87–3.0 GiB）。

## 4. 未满足项 / 限制（如实登记）

- 上游 Woop 的 f64 兜底分支未翻译（产品 f32-only）；夹具上静态重放 0 次判定变化，但「会改变判定的精确退化输入」不在覆盖内，不宣称泛化等价。
- hfield/flex/SDF/splat BVH 不实现；纹理/材质/光照/相机不在本片。
- 单 fixture（4 顶点 4 三角形网格 + 436 条射线），无 fuzz/大网格/随机射线集；深树（>128 层）栈限制未实测。
- 参考为 3.15 冻结产物；动态 refit 参考用上游 `sah`、静态用 render context `cubql`，节点布局不同（按验收不断言布局）。
- 本轮未跑全量 G01。

## 5. 复核产物

- `static-check.json`：git blob 哈希/行数、mesh 顶点转换、S2b 位姿交叉、冻结命中交叉、统计/refit/重放、平局分析
- `rerun-compare.json`：隔离重采 vs 入库夹具的语义比较与确定性
- `harness-compare.json` / `harness-product-hits.txt`：QA harness raw 输出与独立判定
- `coverage-analysis.json`：命中 tag 覆盖、退化网格有效面覆盖、精确穿边/顶点命中清单
- `ptx-record.json` / `ptx-fma-summary.txt`；`product-gpu-test.log`；`unit-tests-summary.txt`；`gpu-window.txt`

## 6. 非阻断问题（建议修，不影响本切片通过口径）

1. **退化网格有效面���盖缺口（一般）**：退化网格 3 个有效面为 face 1 `(0,1,3)`、face 2 `(0,2,4)`、face 3 `(3,4,0)`；16 条射线实际只命中 face 2/3，**face 1 从未成为最近命中**。原因：face 1 位于 z=0 平面，正面命中需要 `-z` 方向，而 12 条中心射线方向只有 `±x / +y / +z`；其余射线也到不了它。README「3 个有效面中心 × 4 方向」的表述暗示三面都覆盖，实际只覆盖两面。建议补 1 条 `-z` 方向射线（或 no_cull 变体）并重采。
2. **「避开精确穿顶点/边」表述与实现不符（建议）**：独立复算显示 16 条退化射线中至少有 10 处精确穿边/顶点命中（如 ray 13 精确穿共享顶点 `(0,0,0)`、u=v=0；ray 3 在两个面同一 `t=3.3333` 上各有一处精确命中，但两者都被背面剔除 → 结果唯一）。这些输入**不产生歧义**是因为另一侧三角形退化/共面/被剔除，而不是射线真的避开了顶点/边。建议改文档为「避开的是**在剔除后仍存在两个有效候选的歧义平局**」。
3. **被替换的首版射线未归档（建议）**：仓库与证据中没有首版那条射线的位型/坐标记录，只有叙述；无法逐位复核「参考 face 2 / 产品 face 1」的原始现象。建议后续把替换前后的射线位型记入 receipt（或 `removed_rays` 字段）。

## 7. 关于「换样本规避」的裁定

- **不是产品缺陷，也不是无条件规避**：QA 用独立精确数学构造了「精确穿过场景网格共享顶点」的射线，它同时命中 3 个非退化面且 `t` 完全相同（0.9999999999999989）——face 索引此时只由 BVH 遍历顺序决定，属**布局相关平局**；S3 验收明确「不做节点布局断言」，因此这类输入不能作为 face 判据。
- 已入库的 436 + 16 条射线经独立复算**不存在歧义平局**（没有「剔除后仍有 ≥2 个非退化候选同 t」的射线），所以当前判定集没有跳过射线、没有放宽容差（u/v 预算 4 ULP、distance 0 ULP、face 精确）。
- 替换后仍保留退化网格语义覆盖（16/9 命中、face/u/v 精确、distance ≤2 ULP、无 panic），但如 §6-1 所述 face 1 未覆盖、如 §6-2 所述文案不准、如 §6-3 所述首版射线未留档。
