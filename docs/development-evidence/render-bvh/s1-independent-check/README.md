# 渲染/BVH S1 独立复核（渲染模型输入，快验口径）

- 任务：`2026-10-24-render-bvh-s1-independent-check`（owner QA）
- 日期：2026-10-24
- 复核对象：`f314cad`（登记）/ `d642c03`（功能：116 渲染字段原生只读 + 宿主校验）/ `e83823d`（证据哈希改 blob 口径）
- 复核时 HEAD：`75166cf`（S2 场景 BVH 捕获证据）。复核期间 S2（`da6a135`/`75166cf`）落库，但
  `git diff e83823d..75166cf` **未触碰任何 S1 的 src/include/native/build.rs/tests 文件**
  （S2 只新增 `fixtures/render-bvh/scene-a/s2/*`、`scripts/authoring/render-bvh/capture_s2_bvh.py`
  与其任务/证据），故本轮结论仍绑定 S1 的代码提交 `d642c03`。
- 口径：快验独立复核，CPU/离线，**未用 GPU、未跑全量 G01**；只对 Windows 已覆盖范围下结论。
- 结论：**6/6 项成立**，未发现阻断问题；1 条证据小偏差与 3 条建议见 §8。

## 1. 逐元素结论（成立）

**独立原生转储**（不经过 S1 的 Rust/native 桥）：QA 自写 C++ 程序用 MuJoCo 3.12 C API 加载
`fixtures/render-bvh/scene-a/scene-312.mjb`，按参考 JSON 的 schema 打印全部 **133 个数组**
（`tools/dump_model_generated.cpp`，由 `tools/gen_dump.py` 从参考 JSON 生成），再与
`model-312.json` 逐标量比较（整数精确、float32 比 f32 位型、float64 比 f64 位型）：

| 口径 | 字段 | 标量 | 差异 |
| --- | ---: | ---: | ---: |
| 全部参考数组 | 133 | 90,256 | 1 个数组：`geom_sameframe` 3/7 值（1 vs 3） |
| S1 覆盖的渲染字段 | **116** | **89,941** | **0 字段 / 0 标量** |

- `geom_sameframe` 是 3.12（BODY）与 3.15（BODYROT）的枚举语义差异；它是 S1 的**排除字段**
  （碰撞 frame-reuse 标志，渲染器不读），不影响 116 字段结论（`field-parity.json`）。
- 同时重跑交付的 ignored 原生测试（pinned 3.12 DLL + 冻结 MJB，证据写到 `target/` 临时目录）：
  `matches_scene_a_reference_model_arrays` 与 `name_lookup_matches_mj_id2name` 均通过；
  重新生成的 `field-diff.json` 与入库证据 **逐字节相同**
  （sha256 `8ed81083cef9f2658d7d942a0dc16287f578311c6b4be3277b1b692c19d5ed3f`）。
- 字段的 kind/count（unit×width）另按 MuJoCo 3.12 头文件（`mjmodel.h`/`mjvisualize.h` 的类型与
  `(nx x N)` 注释）独立核对：**116/116 一致**，`116 + 17 = 133`（`header-spec-check.json`）。

## 2. 排除面自洽（成立）

- `116 + 17 = 133`：参考 JSON 共 133 个数组，S1 读 116，排除 17 个 geom 字段
  （contype/conaffinity/condim/priority/plugin/sameframe/solmix/solref/solimp/aabb/rbound/
  friction/margin/gap/surfacevel/adhesion/fluid）。
- 上游 `mujoco_warp@71da24d` 渲染管线的 `_src/render.py`、`render_util.py`、`ray.py`、`bvh.py`
  及其传递依赖（`math.py`、`warp_util.py`）中，**17 个排除字段一个都没有出现**（逐字段正则
  全文扫描，`exclusion-surface.json`）；`upstream-references.tsv` 同样没有任何一条命中。
- 重点核 `geom_aabb`/`geom_rbound`/`geom_margin`：它们只出现在 `collision_convex.py`、
  `collision_core.py`、`collision_driver.py`、`collision_flex.py`、`collision_primitive.py`、
  `collision_sdf.py` 与 `types.py` 的 Model 声明中——属于碰撞/约束路径，**ray/BVH/render 不读**。
  BVH 的 mesh/hfield 尺寸由 `bvh.build_mesh_bvh/build_hfield_bvh` 从顶点数据计算
  （`render_util.py` L455–488），不依赖 `geom_aabb/rbound/margin`。
- 正向对照：渲染管线实际读取的 geom 字段是 `geom_type/dataid/group/matid/rgba/size`（及 data 侧
  `geom_xpos/xmat`），全部在 S1 的 116 字段内；**没有被渲染器读取的字段被误排除**。

## 3. 名称表语义（成立）

- 独立 oracle：QA 自写 C++ 程序调用**加载库的 `mj_id2name`**，覆盖 9 类对象
  （body/joint/geom/site/camera/light/mesh/texture/material）的全部 id 与边界 id；
  Python 侧独立解析 `name_*adr + names` 缓冲（不经过 Rust `table_name`）。
- 复算 **87 次比较（63 次边界）0 mismatch**（`name-lookup-check.json`），与交付声明的 87 一致；
  该 87 = 每类 7 个边界 id + 该类全部 id 的断言次数（Σ(7+count)=63+24）。
- 语义核对（代码 + 宿主测试）：
  - `id < 0` 或 `id >= count` → `None`；
  - `name_*adr` 为负或越界 → **构造期即拒绝**（`validate_render_tables`），查找函数对负偏移
    仍防御性返回 `None`；
  - 名称缓冲末字节非 NUL → 构造期拒绝（`missing_name_terminator`）；条目内缺 NUL →
    查找返回 `None`；
  - 空名（偏移指向 `\0`）→ `Some(&[])`；
  - 非 UTF-8 名称可用 `name_bytes`，`name()` 返回 `None`（已文档化）。
- 宿主测试 `name_lookup_matches_mj_id2name_table_semantics` 覆盖空名/负 id/越界 id；本轮重跑通过。

## 4. 拒绝面（成立，非空壳）

- 11 个构造用例（`tests/render_model_input.rs::rejects_invalid_counts_lengths_kinds_and_ranges`）：
  负计数、计数溢出、长度不符、kind 不符、负分辨率、纹理 id 越界、纹理地址越界、名称偏移越界、
  网格范围超容量、未终止 names、空 names —— 逐条读代码确认，重跑宿主测试 **4 passed / 2 ignored**
  （每个用例都断言 `is_err()`）。
- 「分配/写入前拒绝」的路径核对（`rejection-surface.json`）：
  - 宿主 `RenderModelInput::new`：info/counts/字段 kind+长度 → `validate_render_tables` →
    `check_mesh_ranges`，全部在返回对象前完成；S1 **不分配任何设备缓冲**；
  - 原生快照 `NativeRenderSnapshot::capture`：先 `render_field_info` + `checked_count`
    （`checked_mul` 溢出检查），通过后才为该字段分配宿主 `Vec`；
  - 原生桥 `render_stage`：`unit<0`、`width<=0`、`unit*width` 溢出、`count*scalar_bytes` 超
    `SIZE_MAX`、非空字段空指针全部在写前返回 8；`copy_render_field` 写前重查 kind/count/target；
    `mjwarp_native_render_name` 校验指针与 capacity 后才 memcpy。

## 5. G01 既有路径不受影响（成立）

- S1 代码范围 `f314cad..d642c03`：既有 5 个文件**纯加**（143 insertions / 0 deletions）——
  `build.rs` +1（rerun-if-changed）、`include/mjwarp_native_probe.h` +3（末尾 include）、
  `native/model_probe.cpp` +2（尾部 include 子模块）、`src/io/mod.rs` +24、`src/model/mod.rs` +113；
  `native/g01_fields.inc`、`native/model_probe_collision.cpp`、`native/model_probe_sparse.cpp`
  **零改动**；G01 45 字段 ABI 与语义未改。
- 定向复跑（`g01-path-check.json`）：用既有原生验收脚本
  `scripts/test-windows-native.ps1`（构建 15 个 mock DLL + 可信 3.12 DLL）→ **22 passed / 0 failed**
  （CPU/native-model-probe 期望值 22），G01 原生路径与 mock 拒绝面全部通过。
- 另用 `scripts/g01-fast.ps1 -Target native_model_probe -IncludeIgnored` 试跑得到 20 passed / 7 failed：
  失败全部是 `MJWARP_NATIVE_MOCKS` 未设置（g01-fast 不建 mock DLL），**不是产品回归**；
  以 `test-windows-native.ps1` 为准（见上）。该差异已如实记入证据。

## 6. 边界（成立）

- **无 GPU**：只用 `native-model-probe` feature（无 cuda-probe）；新文件无 CUDA/内核调用；
  复核期间 `nvidia-smi` 为 3,340 MiB 桌面基线，无捕获/计算进程。
- **产品测试不启动 Python**：`src/`、`build.rs` 无 `Command::new`/进程启动；产品路径里出现的
  “python” 仅是冻结证据字段名与“不启动 Python”注释；原生验收报告
  `scriptInvokesPython=false`、`scriptInvokesModelCompiler=false`。
- **新文件 ≤500 行**（blob 行数）：`src/model/render.rs` **正好 500**（上限，未超）、
  `src/io/render.rs` 453、`tests/render_model_input.rs` 384、`native/model_probe_render.cpp` 233、
  `include/mjwarp_render_probe.h` 50；既有文件 `native/model_probe.cpp` 284、`build.rs` 51、
  `src/model/mod.rs` 333、`src/io/mod.rs` 418 均在限内（`boundary-check.json`）。
- **模块源码登记**：`mjwarp.model.render-model` → `src/model/render.rs`；
  `mjwarp.io.render` → `src/io/render.rs`；两者都在 `.normify/modules/` 下。`normify validate`
  为 **0 errors / 5 warnings**（两个叶子 `apis` 为空、两处 layout order 未覆盖新叶子、
  31 个叶子偏大建议拆分）——均为架构树建议项，不是错误。

## 7. 证据回执自洽

- 交付 `receipt.json` 的 12 个 artifact：**12/12 sha256 与提交 blob 一致、12/12 字节数一致、
  11/12 行数一致**（`receipt-check.json`）。
- 唯一行数偏差：`field-diff.json` 记 725 行，实际 blob 有 725 个换行、末行无换行（按 splitlines
  为 726 行）；哈希与字节数完全一致，仅行数计数口径差 1（建议项，见 §8）。

## 8. 观察项 / 建议项（不影响结论）

1. `s1-model-input/receipt.json` 里 `field-diff.json` 行数 725 与实际 726 差 1（按换行数 vs
   splitlines 口径）；建议下次按统一口径计数（不涉及数据）。
2. `normify validate` 对两个新叶子模块给 5 条 advisory 警告（`apis` 为空、layout order 未覆盖、
   叶子偏大）；建议在 S2/S3 顺手补齐 layout 顺序，避免后续 `validate` 噪声。
3. `src/model/render.rs` 正好 500 行（达到上限）；后续给该文件加功能前应先拆分，否则会超限。
4. S1 README 的复现命令只覆盖两个 ignored 原生验收测试（可单独跑通）；mock 依赖的 45 字段测试
   必须走 `scripts/test-windows-native.ps1`，建议在 README 里点明，避免误用 g01-fast。

## 9. 未满足项（如实）

- 只覆盖 `scene-a` 单个冻结场景（2 相机/3 光源/3 材质/3 纹理/1 网格）；无任意 MJB 的属性测试/模糊测试。
- 名称对照只验证 ASCII 名称；非 UTF-8 走 `name_bytes`。
- `geom_dataid` 只校验 ≥ -1（hfield 不在 S1 范围）。
- 不做 GPU 上传、纹理对象、BVH、相机射线/渲染内核、context 参数（S2–S7）。
- 不宣称完整渲染/产品渲染验收、S2 已实现或 R01 整体完成；本轮无全量 G01。

## 复现命令

```powershell
$py = 'target\mjwarp-parallel\venv\Scripts\python.exe'
$env:MJWARP_MUJOCO_ROOT = (Resolve-Path 'target\toolchains\mujoco-3.12.0\package').Path
$env:MJWARP_MUJOCO_DLL  = (Resolve-Path 'target\toolchains\mujoco-3.12.0\package\bin\mujoco.dll').Path
# 交付测试
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features native-model-probe --test render_model_input
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features native-model-probe --test render_model_input -- --ignored --test-threads=1
# G01 既有原生路径
target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -ExecutionPolicy Bypass -File scripts\test-windows-native.ps1
# QA 独立检查（脚本在 tools/，生成物在 target/qa-render-bvh-s1/）
& $py -B target\qa-render-bvh-s1\gen_dump.py            # 生成 dump_model.cpp
# 用 MSVC 编译 dump_model.cpp（MuJoCo 3.12 include/lib）并 dump scene-312.mjb -> dump.json
& $py -B target\qa-render-bvh-s1\parity_and_exclusion.py
& $py -B target\qa-render-bvh-s1\header_spec_check.py
& $py -B target\qa-render-bvh-s1\name_lookup_check.py
& $py -B target\qa-render-bvh-s1\evidence_gen.py
```

## 证据索引

- `field-parity.json`：独立 3.12 转储 vs `model-312.json` 的 133/116 字段逐标量结果
- `exclusion-surface.json`：17 个排除字段在上游渲染管线与 `upstream-references.tsv` 的命中
- `header-spec-check.json`：116 字段 kind/count 与 MuJoCo 3.12 头文件的独立对照
- `name-lookup-check.json`：87 次 `mj_id2name` 独立对照
- `rejection-surface.json`：11 个拒绝用例与「写前拒绝」路径
- `g01-path-check.json`、`g01-native-tests.log`、`g01-native-report.json`、`g01-fast-summary.json`：G01 既有路径
- `receipt-check.json`：交付回执 12 个 artifact 的 blob 哈希/字节/行数核对
- `boundary-check.json`：行数上限、无 GPU、无 Python、模块登记
- `tools/`：QA 独立检查脚本与原生 oracle 源码
