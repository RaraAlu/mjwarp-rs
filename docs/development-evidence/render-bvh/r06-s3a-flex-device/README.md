# R06-S3a：flex 渲染设备侧（`render_flex.cu` + 设备 cast/BVH/seg）+ 1D 端到端接线

任务：`2026-11-04-render-r06-s3a-flex-device`（p74，owner Backend；依赖 `2026-11-04-render-r06-s3pre-flexhost-wiring`）。
日期：2026-11-04（空闲段）。基线 HEAD：`55d2c264`。**离线、零 GPU 窗口、零参考采集**；窗口/数值结论归 S3b。

范围按 Chief 2026-11-04 收窄：**1D flex 端到端**（数据齐、可跑通整链）+ **2D/3D 机制**（显式拓扑入参、
宿主/设备机制 + standalone 编译核，**不接产品数据**）；host 对 `dim != 1` 显式 `Unsupported`。
不宣称 flex 渲染可用，不产任何渲染数值结论。

## 1. 交付面（全部 ≤ 500 行）

| 文件 | 行数 | 内容 |
| --- | --- | --- |
| `src/render/bvh/flex.rs` | 302 | 1D flex 场景叶包围盒（上游 `_compute_flex_bvh_bounds`）；叶序 world-major、`group = worldid` |
| `src/render/bvh/flex_mesh.rs` | 417 | 2D/3D flex mesh 面生成 + `MeshBvh` 构建/重拟合（机制；显式拓扑入参） |
| `src/render/bvh/flex_mesh/tests.rs` | 116 | 2D/3D 机制单测（拆分出来守 500 行棘轮，`#[path]` 同款先例） |
| `src/render/kernel/cuda/render_flex.cu` | 157 | `FlexView` + `warp_length3`/`quat_z2vec`/`quat_to_mat`（上游 `math.py` 逐式）+ `flex_leaf_hit`（1D 胶囊） |
| `src/render/kernel/cuda/render_common.cu` | 432 | `cast_ray` 增加 flex 叶分派（`leaf_geom < 0`）、统一背面剔除与 tie-break（flex `candidate_id = -2`） |
| `src/render/kernel/cuda/render_megakernel.cu` | 398 | `FlexView` 组装；flex 着色（`flex_rgba[mesh_id]`、`mat_id = -1`）；seg `(flexid, 9)` |
| `src/render/kernel/cuda/render_shade.cu` | 458 | `compute_lighting` 透传 `FlexView`（阴影可见度走同一 `cast_ray`） |
| `src/render/kernel/cuda/render_shadow.cu` | 32 | `shadow_visibility` 透传 `FlexView` |
| `src/render/kernel/mod.rs` | 477 | `MEGAKERNEL_SOURCE` 拼接顺序加入 `render_flex.cu`（scene.cu → render_math.cu → render_flex.cu → …） |
| `src/render/kernel/cast.rs` | 487 | `SceneDevice::pack` 支持 flex 叶编码（`leaf_geom = -1 - flex_local`、`leaf_mesh = -1`、size/pose 零占位） |
| `src/render/kernel/megakernel_arrays.rs` | 361 | `geom_data_poses` 跳过负 `leaf_geom`（flex 叶无 geom 位姿） |
| `src/render/kernel/megakernel.rs` | 450 | `render_with_flex(Some)` 逐帧把 1D flex 叶并入本帧 `SceneBvh`；`None` 路径仍用 context 拥有 BVH |
| `src/render/bvh/mod.rs` | 284 | 模块注册与 re-export |

## 2. §pre-check：pinned 上游 flex 渲染语义对表（`71da24d`）

pinned 源码取自 S0 恢复的 `target/mjwarp-parallel/upstream`（`git status` clean，HEAD `71da24d…`）与仓库
内冻结副本 `docs/evidence/d04-complex-physics/authoring-runs/run-5/frozen-source/_src/`。

| 语义 | 上游锚点 | 本片落点 |
| --- | --- | --- |
| flex 图元映射 | `render_util.py` L490-512 | 1D：每边一胶囊图元（`edgeid = flex_edgeadr[f] + e`）；2D/3D：每 flex 一 mesh 图元（`edgeid = -1`）。数据侧沿用 S2b `FlexGeomMapping` |
| scene BVH 叶序/分组 | `bvh.py` L314-379 | 上游叶 = geom 叶 + flex 叶，`group = worldid`，`bvh_nflexgeom = len(flex_geom_flexid)`；本片 flex 叶块追加在场景叶之后、`group` 同样按 world 分组（节点布局不参与验收） |
| flex 叶包围盒 | `bvh.py` L265-311 `_compute_flex_bvh_bounds` | `flex.rs::one_dimensional_scene_leaves`：`min(v0,v1) - r` / `max(v0,v1) + r`（1D）；2D/3D `± r` 的顶点整体包围盒（机制内实现，未接产品数据） |
| 命中分派 | `render.py` L762-800 | 设备叶 `leaf_geom < 0` → `flex_local = -1 - leaf_geom`；`flexid = flex_geom_flexid[flex_local]`、`edgeid = flex_geom_edgeid[...]` |
| 1D 胶囊 | `render.py` L770-784 | `pos = 0.5*(v0+v1)`、`vec = v1-v0`、`quat_z2vec` → `quat_to_mat`、`size = (radius, 0.5*|vec|, 0)`、`ray_capsule`（复用 `scene.cu` 逐式副本） |
| flex mesh BVH（2D/3D） | `bvh.py` L994-1109 / L1112-1179 | `flex_mesh.rs`：2D `nface = 2*nelem + 2*nshell`、3D `nface = nshell`；2D 顶点法线累积（`dim==1/3` 跳过）；每个 `(flex, world)` 单独 `MeshBvh`，`group_root =` 树根；**设备命中复用既有 `MeshBvhQuery`**（世界空间、identity 位姿、`cull_backfaces = false`） |
| BVH 求交语义 | `ray.py` L759-802 | flex 已在世界空间：无 `_ray_map`、无多段 march、无内部背面剔除 |
| 背面剔除 | `render.py` L802-806 | 命中后统一判 `cull && dot(ray_dir_world, n) > 0 → d = -1`；1D 胶囊有真实法向，与 primitive 同路 |
| tie-break | `render.py` L813-823 | `tol = RAY_TOL_REL * max(dist, 1)`；flex 的 `hit_geom_id = -2`，**近共面平局永不赢**（`-2 > geom_id` 不成立）——设备 `candidate_id = -2` 复刻 |
| 透明跳过 | `render.py` L766-767 | `flex_rgba[flexid][3] == 0` 直接 miss（胶囊与 BVH 两路都跳） |
| 着色 | `render.py` L1303-1305 | `geom_id == -2` 时 `color = flex_rgba[mesh_id]`（`mesh_id = flexid`）、`mat_id = -1` → 无纹理/高光/自发光，走默认值；normal 照常参与光照/阴影 |
| depth | `render.py` L1285-1294 | 与 geom 同式：`dist * -ray_dir_local_cam[2]` |
| seg | `render.py` L1245-1249 | `geom_id == -2` → `vec2i(flexid, ObjType.FLEX)`；`mjOBJ_FLEX = 9`（mujoco `mjtype.h` L292-301 枚举序）；只 `ray_base == 0`（首样本）写 |
| 类型常量 | `types.py` L532 / `mjtype.h` L301 | `GeomType.FLEX = mjGEOM_FLEX`；`mjOBJ_FLEX = 9` |

**给 S3b 的采集口径（同一份已单发 Arch）**：

- 采集脚本用 pinned editable 安装（`target/mjwarp-parallel/upstream`，HEAD `71da24d…`），**不得用 PyPI 轮子**
  （S0 §1 已留陷阱：PyPI 3.15.0 与 pinned commit 有 5 个 `.py` 逐字节不一致）。
- 场景建议 **1D flex + 刚性几何共存**：读取 `flex_geom_flexid/edgeid` 口径、`bvh_nflexgeom`、
  `flexvert_xpos`（one-step 后）与相机参数一并落 fixture；两跑必须固定 one-step 状态与 sample 顺序。
- 判据沿用 R05/R02 家族口径（S3b 卡已写死）：rgb ≤1 LSB（bit-first）、depth ≤4 ULP 且 0 beyond
  （bit-first）、**seg 精确相等**（含 flex：`(flexid, 9)`）、两跑 sha 相同；flex 与刚性共存时 world 隔离成立。
- 注意 tie-break：`flex` 在近共面平局永不赢，采集场景**不要**用与刚性面近共面/贴脸的 flex 边。

## 3. 实现要点（ABI 与数据流）

- **81 槽 ABI 不变**：9 个 flex 形参全部沿用 S2 通道（`megakernel_args.rs` 实参装配未改），无新增槽；
  `bvh_ngeom` / `bvh_nflexgeom` 语义沿用上游。
- **场景叶编码**：`SceneDevice::pack` 对 flex 叶写 `meta.leaf_geom = -1 - flex_local`、`leaf_mesh = -1`；
  设备 `cast_ray` 以 `leaf_geom < 0` 分派（等价上游 `local_id >= bvh_ngeom`），`meshid` 语义等价。
- **数据流**：`render_with_flex(context, fields, model, poses, rays, Some(&FlexHostAssembly::inputs()))`：
  `megakernel.rs` 用 `bvh::flex::one_dimensional_scene_leaves` 逐帧算 flex 叶包围盒（上游每帧 `refit_scene_bvh`
  语义；本实现逐帧重建场景 BVH，节点布局只服务自有打包），拼成含 flex 叶的 `SceneState`/`SceneBvh` 后
  交给既有 `SceneDevice::pack`；flex 通道 9 数组仍走 S2 上传。
- **`None` 路径**：不进入任何 flex 分支、不重建 BVH、实参/设备槽与 S7b2 完全一致（逐字节等价由数据门控
  保证：`bvh_nflexgeom == 0` 时场景叶不可能出现负 `leaf_geom`；见 §4 第 4 条）。
- **2D/3D**：`host` 侧对 `edgeid < 0`（2D/3D mesh 图元）显式 `Unsupported`（不 panic、不静默降级）；
  `flex_mesh.rs` 的 2D/3D 面生成/BVH/重拟合作为机制存在并有单测，待 S2c 数据层（`flex_shell*` +
  2D `flex_elem*` additive + T4 重钉）落地后在 S3c 接线。
- 精度：设备侧只做 f32；`sinf`/`cosf` 的 f64 归约属渲染门禁精确放行项；`warp_length3` 用显式
  `fmaf` 固定 NVRTC 收缩顺序（与 `normalize3_rays` 同口径），不依赖推断收缩。

- **R06 nvrtc-eol 修（2026-11-04）**：`tools/nvrtc_precheck.py` 的 `.ptx` 从文本写改为**二进制写**
  （`write_bytes(ptx.encode("utf-8"))`，`.txt`/`.nvrtc-log.txt` 显式 `newline="\n"`），磁盘 `.ptx` 遂为 LF；
  修后实跑：磁盘 `.ptx` 字节数/sha256 与 `nvrtc-precheck.txt` 的 `ptx_bytes`/`ptx_sha256` 一致。
  本目录记录值（S3a 期 7 段源码）**保持不变**；当前源码已含 S3c 改动，实跑得到的是当前源的身份。
  治理卡：`2026-11-04-governance-nvrtc-precheck-ptx-eol`。

## 4. 验证

1. **门禁**（`.normify/runs/2026-11-04-render-r06-s3a-flex-device/`）：fmt / clippy / clippy-cuda /
   unit **全过**；unit **1298 passed / 0 failed / 398 ignored**（基线 1290 + 8 条新单测）。
   NP01 源面棘轮跟随（外部前置，非本片改动）：`cuda-surface.json` 登记 `render_flex.cu`
   （files 41→42、globals 22 不变、该文件 0 个 `__global__`，快照卡 `5077e44e`）；
   `tests/precision_host_source_gate.rs` 硬编码 41→42 由 NP01 卡
   `2026-11-03-precision-np01-host-source-gate`（`07941c7c`）跟随修复。
2. **新增单测 8 条**（`cargo test --locked --lib "render::bvh::flex"` → 8 passed / 0 failed）：
   1D 叶包围盒外扩、多 world 叶序/分组、2D/3D 图元显式拒绝、长度/映射不一致拒绝；
   2D 面数与顶/底/侧面点及绕序、3D shell 面点、重拟合跟随世界顶点、维度/拓扑越界拒绝。
3. **离线 NVRTC 预检**（与产品同 recipe，纯 NVRTC 编译，**不建 CUDA 上下文、不发射内核**）：
   命令见 `tools/nvrtc_precheck.py` 头注释；结果 `nvrtc-precheck.txt`：
   - nvrtc 12.8；选项 `--std=c++17 --gpu-architecture=compute_70 --fmad=true
     --device-as-default-execution-space -Iinclude/warp-headers`（`artifact/nvrtc.rs` L92）；
   - 7 段拼接源码 sha256 `be7c2ac1…`；PTX **314257 B**（sha256 `e32d2fce…`）；
   - `check_render_ptx` 等价规则（证据侧独立复算）：W15 两条 `_svd_config<double>` 常量各 1 次、
     `sin/cos` 归约三元组 5 处、f16/bf16/tf32 **0**、入口 `render_megakernel` 存在。
4. **`None` 路径位级不变（离线可证部分）**：设备 flex 分支由数据门控（`bvh_nflexgeom == 0` ⇒ 无负
   `leaf_geom`），`None` 路径的实参、上传、场景打包与 S7b2 逐项不变；表达式未改动（新增代码不在
   `None` 路径执行）。**未跑 GPU**：`render_with_flex(None)` 与既有 render 的逐位相等留 S3b 窗口复验
   （与 S3pre 三条口径一致，不在此重复）。
5. **`checks.unchanged`**：`fixtures/` 与 `docs/evidence/` 零 diff（`git status` 仅本片源码 +
   证据目录 + 卡/runs）；不改上游 pin/数值参考/判据/容差。
6. **零 GPU、零窗口、零参考**：本片未启动任何 GPU 内核、未开验证窗口、未生成参考采集。

## 5. 边界与遗留

- **不宣称 flex 渲染可用**，不产像素/深度/seg 数值结论；参考采集、窗口验收（rgb/depth/seg/两跑 sha）
  全部归 S3b，走 Chief go 制 + 重钉守卫。
- 2D/3D 未接产品数据：缺 `flex_shellnum`/`flex_shelldataadr`/`flex_shell`（3D 表面 + 2D 侧边）与
  2D `flex_elem*`（render 桶）。已登记 `2026-11-04-render-r06-s2c-flex-shell-data`（owner Backend，
  depends S2b merged）承载 additive 数据层 + T4 重钉；接线在 S2c 之后（预计 S3c）。
- 上游 2D 顶点法线在并行 kernel 里 `+=` 累加（元素间顺序不定，数据竞争）；本实现按元素下标升序
  确定性累加，2D 着色法线最低位可能与某次上游运行不同——2D/3D 窗口验证时按 S2c/S3c 卡口径处理。
- `flex_render_smooth` 只影响上游 2D refit 的顶点法线分支；1D 产品路径无关（S0 已记）。
- 三角 BVH 节点布局是本实现的自主分箱 SAH（叶容量 8），上游 `sah`/`leaf_size=2`；验收只断言语义命中，
  不断言节点布局（沿用既有 `MeshBvh` 口径）。
