# 渲染/BVH S1：渲染模型输入（原生只读 + 宿主校验）

任务：`2026-10-24-render-bvh-s1`（proposed 登记提交 `f314cad`）。日期：2026-10-24。执行：Backend。
依据：`docs/development-evidence/render-bvh-precheck/plan.json` 的 S1 切片与 README §6
「S1 渲染模型输入」；用户已批准 D1–D4 口径（产品 native 保持 MuJoCo 3.12.0，3.15 只在
authoring 侧）。**CPU/离线，无 GPU；产品测试不启动 Python；不改 G01 既有 45 字段路径与语义。**

## 1. 结论（先给）

1. **逐元素对照通过**：116 个渲染字段、共 **89,941 个标量**与
   `fixtures/render-bvh/scene-a/model-312.json` 在线加载的 `scene-312.mjb` 上
   **完全相等（0 差异）**；`field-diff.json` 逐字段记录 count 与 mismatches。
2. **渲染字段覆盖**：geom 9 / cam 15 / light 22 / mat 10 / tex 8 / mesh 34 / vis 6 /
   stat 2 / 名称表 10（含 `names` 字节缓冲）= 116。其余 17 个模型数组是碰撞/约束/求解
   专用（`geom_contype/conaffinity/condim/priority/plugin/sameframe/solmix/solref/solimp/aabb/
   rbound/friction/margin/gap/surfacevel/adhesion/fluid`），渲染器不读取，登记为排除面，
   由既有 G01 / 碰撞探针路径覆盖（`field-diff.json.excluded_fields`）。
3. **名称语义通过**：独立 `mj_id2name` 对照（从加载的 MuJoCo 3.12 库解析符号）对 9 类对象
   （body/joint/geom/site/camera/light/mesh/texture/material）覆盖全部 id 与越界 id
   （-2/-1/0/…/count/count+1），Rust 名称查找与之一致；语义：越界或负偏移返回 `None`，
   空名返回空字节，均与库一致。
4. **宿主校验**：计数非负、期望标量数 = 基数×宽度并检查乘加溢出；名称偏移/终止符、
   相机分辨率非负、纹理 id、纹理地址、网格 adr/num 范围在**分配与写入前**拒绝。
   拒绝面 11 个构造用例 + 名称语义断言全部通过。
5. **无 GPU**：只启用 `native-model-probe` feature 与可信 MuJoCo 3.12 DLL；不创建纹理对象、
   不建 BVH、不分配设备缓冲。

## 2. 交付物与实现

| 文件 | 行数 | 说明 |
| --- | ---: | --- |
| `include/mjwarp_render_probe.h` | 50 | 独立 ABI：`mjwarp_render_info`（19 项计数）、`mjwarp_render_field_info`/`copy_render_field`（field id + kind 1..5：int32/double/uint8/int64/float32）、`mjwarp_native_render_name`（`mj_id2name` 对照）。schema 与 G01/碰撞/柔体 ABI 独立。 |
| `src/model/render.rs` | 500 | 渲染字段规范表（116 项，顺序与 C++ `render_stage` 一致）、`RenderModelCounts`、`FieldData`/`RenderFields` 独占快照、名称表语义（`table_name`）与对象类型。 |
| `src/io/render.rs` | 453 | 原生快照 `NativeRenderSnapshot`：查询计数与逐字段 info → 按期望长度分配宿主缓冲 → 拷贝 → 校验后构造 `RenderModelInput`；`native_name` 为 `mj_id2name` 对照入口。 |
| `tests/render_model_input.rs` | 384 | 纯宿主测试（字段表/参考计数、构造接受、拒绝面、名称语义）+ 2 个 ignored 原生测试（逐字段对照写 `field-diff.json`、`mj_id2name` 对照）。 |
| `native/model_probe.cpp`（扩展） | 284 | 既有 TU：仅在尾部包含渲染桥子模块，既有 G01/碰撞/柔体/sparse 代码与路径未改。 |
| `native/model_probe_render.cpp`（新增） | 233 | 只读渲染桥：19 项计数、逐字段 `unit×width` 选择（含溢出检查）、拷贝预检、`mj_id2name` oracle；被 model_probe.cpp 在同一 TU 包含。 |
| `build.rs`（扩展） | 51 | 增加 `rerun-if-changed=native/model_probe_render.cpp`。 |
| `src/model/mod.rs`、`src/io/mod.rs` | +119 / +24 | 模块注册与导出；`validate_render_tables`（名称/纹理/网格地址与范围前置校验）+ `NativeModelProbe::{render_snapshot, render_name_oracle}`。 |
| `include/mjwarp_native_probe.h` | +3 | 末尾包含渲染探针头（单一原生 include 根），G01 ABI 未动。 |

字段覆盖（116，括号内为字段数）：

- geom（9）：`geom_type/bodyid/dataid/matid/group/size/pos/quat/rgba`
- cam（15）：`cam_mode/bodyid/targetbodyid/projection/resolution/output/fovy/ipd/sensorsize/intrinsic/pos/quat/poscom0/pos0/mat0`
- light（22）：`light_mode/bodyid/targetbodyid/type/texid/castshadow/active/bulbradius/intensity/range/attenuation/cutoff/softness/exponent/ambient/diffuse/specular/pos/dir/poscom0/pos0/dir0`
- mat（10）：`mat_texid/texuniform/texrepeat/emission/specular/shininess/reflectance/metallic/roughness/rgba`
- tex（8）：`tex_type/colorspace/height/width/nchannel/adr(int64)/data(uint8 89,088 B)/pathadr`
- mesh（34）：顶点/法线/纹理坐标/面与面法线/面纹理坐标/图/极值/缩放/位姿/路径地址 + 多边形与顶点映射 `mesh_*adr/num/vert/normal/texcoord/face/facenormal/facetexcoord/graph/extrema/scale/pos/quat/pathadr/polynum/polyadr/polynormal/polyvertadr/polyvertnum/polyvert/polymapadr/polymapnum/polymap`
- vis（6）：`vis.headlight.{ambient,diffuse,specular,active}`、`vis.map.znear/zfar`
- stat（2）：`stat.extent/center`
- 名称表（10）：`name_{body,jnt,geom,site,cam,light,mesh,tex,mat}adr` + `names`

## 3. 逐元素对照（`field-diff.json`）

- 参考：`fixtures/render-bvh/scene-a/model-312.json`（sha256 `cdd490f0…`）与
  `scene-312.mjb`（sha256 `c6f5f9f1…`，101,328 B），均来自 S0a 冻结夹具。
- 方法：原生 3.12 读取值 vs 参考 JSON 数值文本；整数按原生宽度精确解析，`float64/float32`
  用 Rust std 正确舍入解析（避免通用 JSON f64 解析的 1 ULP 误差），逐标量严格相等。
- 结果：**116/116 字段 0 mismatch、89,941/89,941 标量相等**；逐字段 count 与 mismatches
  见 `field-diff.json.rows`。分组标量数：geom 133、cam 74、light 120、mat 69、tex 89,109、
  mesh 203、vis 12、stat 4、名称表 24、names 193。
- 排除面 17 字段与参考 JSON 的键数自洽：116 + 17 = 133（参考数组总数）。

## 4. 名称查找实现（`mj_id2name` 语义）

- Rust 侧：`name_*adr` 偏移表 + `names` NUL 分隔缓冲；`id` 越界或偏移为负 → `None`；
  偏移指向空串 → 空字节切片；无终止 NUL → `None`。`RenderFields::name_bytes/name` 与
  `RenderModelInput::name_bytes/name` 暴露该语义。
- 对照侧：`mjwarp_native_render_name` 从**加载的 MuJoCo 3.12 库**解析 `mj_id2name`
  符号并调用，避免"自实现对照自实现"。
- 测试：9 类对象 ×（全部 id + 边界 id）共 87 次比较，Rust `Option<Vec<u8>>` 与库返回值一致
  （含越界 None、空名空字节）；`render_model_input::native::name_lookup_matches_mj_id2name` 通过。

## 5. 拒绝面与溢出校验

- 原生桥：每个字段 `count = unit × width`，`unit<0`、`width<=0`、乘积溢出、
  `count × 标量宽度` 超 `SIZE_MAX`、非空字段空指针均在写前返回 8；拷贝前重查 kind/count。
- 宿主 `validate_render_tables` + `RenderModelInput::new`：
  - 19 项原生计数非负；字段长度必须等于期望值（不等即 LengthMismatch）。
  - 名称表：长度=对象数、偏移在 `names` 内、缓冲以 NUL 终止。
  - 相机分辨率非负；`light_texid/mat_texid` ∈ {-1} ∪ [0,ntex)、`geom_matid` ∈ {-1} ∪ [0,nmat)。
  - `tex_adr` ∈ [0,ntexdata]；`mesh_bvhadr/octadr/pathadr` ≥ -1。
  - 网格 adr/num：vert/face/normal/texcoord/poly/polyvert/polymap 的
    `start ≥ 0（texcoord 无坐标时允许 -1）` 且 `start+num ≤ 对应容量`。
- 纯宿主拒绝面测试：11 个构造用例（负计数、计数溢出、长度不符、kind 不符、负分辨率、
  纹理 id 越界、纹理地址越界、名称偏移越界、网格范围超容量、未终止 names、空 names）
  全部被拒绝；构造合法参考布局被接受。

## 6. 测试与门禁

- 纯宿主（默认 ignore 之外，需要 `native-model-probe` 编译）：
  `cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features native-model-probe --test render_model_input`
  → 4 passed / 2 ignored。
- 原生对照（可信 MuJoCo 3.12 DLL + 冻结 MJB）：
  `cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features native-model-probe --test render_model_input -- --ignored --test-threads=1`
  → 2 passed（逐字段对照、名称语义）。
- 门禁：`normify gate 2026-10-24-render-bvh-s1` 运行 fmt / clippy / clippy-cuda / unit，
  结果绑定本证据与源码范围；具体耗时与通过数见本任务提交正文与 `normify` 运行记录。
- 不自称产品渲染能力：本片只做宿主输入与校验，无 GPU 内核、无纹理对象、无 BVH。

## 7. 复现

```powershell
$env:MJWARP_MUJOCO_ROOT = (Resolve-Path 'target/toolchains/mujoco-3.12.0/package').Path
$env:MJWARP_MUJOCO_DLL  = (Resolve-Path 'target/toolchains/mujoco-3.12.0/package/bin/mujoco.dll').Path
$env:MJWARP_S1_EVIDENCE_DIR = (Join-Path (Get-Location) 'docs/development-evidence/render-bvh/s1-model-input')
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features native-model-probe --test render_model_input
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features native-model-probe --test render_model_input -- --ignored --test-threads=1
```

`MJWARP_S1_EVIDENCE_DIR` 存在时原生对照测试写 `field-diff.json`；未设置时只断言不落盘。

## 8. 未满足项与边界

- 只覆盖 S0a 冻结的 `scene-a` 单个场景（2 相机/3 光源/3 材质/3 纹理/1 网格）；未对任意 MJB
  做属性测试或模糊测试。
- `geom_dataid` 只校验 ≥ -1：它可同时指向 mesh 或 hfield，S1 不读 hfield，故不做上限判定。
- 名称表对照只验证 ASCII 名称（夹具内名称均为 ASCII）；非 UTF-8 名称可用 `name_bytes`，
  `name()` 会返回 `None`（已文档化）。
- 原生桥拆分为既有 TU `native/model_probe.cpp`（282 → 284 行，仅加包含）与新增子模块 `native/model_probe_render.cpp`（233 行）；所有新文件满足 ≤500 行。
- 不做 GPU 上传、纹理对象、BVH、相机射线/渲染内核、context 参数（S2–S7 范围）。
- 不宣称完整渲染通过、产品渲染验收通过、S2 已实现或 R01 整体完成。
