# R06-S3c：2D/3D flex 设备接线（真实 topology → 设备 flex mesh BVH + host 解锁 `dim 2/3`）

任务：`2026-11-04-render-r06-s3c-flex-mesh-device`（p74，owner Backend；依赖 `2026-11-04-render-r06-s2c-flex-shell-data` +
`2026-11-04-render-r06-s3a-flex-device`）。基线 HEAD：`3b8e0d7f`。
**离线、零 GPU 窗口、零参考采集**；窗口/数值结论归 S3b（owner Arch）。

## 1. 交付面（全部 ≤ 500 行；`megakernel.rs` 守 450 行死线）

| 文件 | 行数 | 内容 |
| --- | --- | --- |
| `src/render/bvh/flex_scene.rs` | 470 | S3c 宿主装配：`FlexMeshChannel`/`FlexMeshAssembly`（场景叶 + 设备段 + 条目基址）、`prepare` 形状校验、`append_to_mesh_device`、调用点拓扑自校验 |
| `src/render/bvh/flex_scene/device.rs` | 186 | 与 `MeshDevice` 同布局的 `FlexMeshDevice`（`meta`/`params`/`entries`）、`append_to`、逐 `(flex, world)` `MeshBvh` 构建与打包 |
| `src/render/bvh/flex_scene/tests.rs` | 259 | 4 条单测：混合 1D/2D/3D 装配、world 隔离、短拓扑向量不 panic、覆盖既有拒绝路径 |
| `src/render/bvh/mod.rs` | 288 | 模块注册 + `FLEX_NO_MESH`/`FlexMeshAssembly`/`FlexMeshChannel`/`FlexMeshDevice`/`FlexSceneError` re-export（+3 行） |
| `src/render/kernel/cuda/render_flex.cu` | 159 | `FlexView` 增 `bvh_id`（2D/3D 条目基址）；1D 胶囊路径不变 |
| `src/render/kernel/cuda/render_common.cu` | 473 | `flex_mesh_leaf_hit`（世界空间三角 BVH）；`cast_ray` flex 叶按 `edgeid` 分派胶囊 / mesh；shadow any-hit 下 mesh 路径不 cull |
| `src/render/kernel/cuda/render_megakernel.cu` | 401 | `FlexView` 组装补 `flex_bvh_id`（+3 行） |
| `src/render/kernel/megakernel.rs` | **449** | `render_with_flex` 增第二通道（2D/3D mesh）；mesh 段追加到 `MeshDevice` 末尾；`flex_bvh_id` 槽覆盖；`pack_flex_scene` 迁出守 450 行 |
| `src/render/kernel/megakernel_arrays.rs` | 411 | `pack_flex_scene`（S3a 的 flex 场景叶并入 + `SceneDevice` 打包从 `megakernel.rs` 挪入，纯移动零语义变化） |
| `src/render/kernel/cast.rs` | 487 | **零改动**（`SceneDevice::pack` 的负 `leaf_geom` 编码 S3a 已就位） |

`unchanged`：`fixtures/` 零 diff；`docs/evidence/` 零 diff（`git status` 仅本片源码 + 证据目录 + 卡/runs）。

## 2. §pre-check：pinned 上游 2D/3D flex mesh 语义对表（`71da24d`）

承接 S3a `r06-s3a-flex-device/README.md` §pre-check（同一份采集口径已由 Arch 抄进 S3b 卡，不再重复）。
本片只补 2D/3D **mesh 路径**的设备语义：

| 语义 | 上游锚点 | 本片落点 |
| --- | --- | --- |
| 2D/3D 图元映射 | `render_util.py` L490-512 | 每 flex 一个 mesh 图元（`edgeid = -1`）；设备叶按 `geom_edgeid < 0` 分派 mesh 路径 |
| mesh BVH 构建 | `bvh.py` L994-1109 / L1112-1179 | 复用 S3a `bvh::flex_mesh`（2D `nface = 2*nelem + 2*nshell`、3D `nface = nshell`）；每个 `(flex, world)` 一棵世界空间三角 BVH |
| BVH 根 | `bvh.py` 分组树 | 上游 `group_root` = 该 world 分组子树根；本实现每个 `(flex, world)` 单独建树，`meta[3]` 即该树根（语义等价，节点布局不参与验收） |
| 求交语义 | `ray.py` L759-802 | flex mesh 已在世界空间：无 `_ray_map`、无多段 march、**无内部背面剔除**（设备 `ray_mesh_leaf(..., cull=false)`） |
| 命中数据 | `ray.py` L759-802 | 返回距离/法向/face/bary；`candidate_id = -2`、`mesh_id = flexid` 与 S3a 一致 |
| 背面剔除 | `render.py` L802-806 | primary ray 统一后判据；**shadow any-hit 下按 mesh 语义 n 为零向量 ⇒ 不 cull**（设备用 `flex_mesh_path` 豁免，与 gtype 7/1/8 同列） |
| 透明跳过 | `render.py` L766-767 | `flex_rgba[flexid][3] == 0` 直接 miss（胶囊与 mesh 两路都跳，`active`/条目不产生常量 0 面） |
| refit 语义 | `bvh.py` L1112-1179 | 上游每帧 `refit_flex_bvh`；本实现每帧从当前 `flexvert_xpos` 重建（等价实现，`FlexMeshBvh::refit` 留持久 registry） |
| 采集口径 | S3a README §2 | pinned editable 安装（禁 PyPI 轮子）、场景避开贴脸/近共面 flex 边（tie-break flex 永不赢）、seg 精确含 `(flexid, 9)`、两跑 sha 相同 |

## 3. 实现要点（数据流 / ABI / 行数）

- **81 槽 ABI 不变**：不新增槽；2D/3D 复用既有 `MeshDevice` blob 段与 9 个 flex 形参。
  `flex_bvh_id` 槽（`u64[nflex]`）由宿主覆盖为**本帧 `MeshDevice.offsets` 内的绝对条目号**，
  设备条目索引 = `flex_bvh_id[flexid] + worldid`；1D/无 mesh 的 flex 为 `u64::MAX` 哨兵
  （设备侧直接走胶囊路径，不访问 mesh 段）。
- **设备段布局与既有 `MeshDevice` 逐字段一致**（`meta = [n_nodes, n_tri, n_vert, root, links(2n),
  order(n_tri), indices(3n)]`、`params = [bounds(6n), vertices(3n_vert)]`），追加在既有 mesh 条目之后：
  既有条目索引与 `(meta, params)` 偏移全不变，`pack_blob` 签名/上传路径零改动。
- **宿主数据流**：`FlexMeshAssembly::prepare(flex_mesh, nworld, nflex)` 校验 `worlds == nworld` 且
  `flex_dim.len() == nflex`（不一致返回显式 `FlexSceneError`，不 panic）→ `build` 生成场景 flex 叶
  （1D 边胶囊包围盒 / 2D/3D 顶点整体包围盒 ± 半径，上游 `_compute_flex_bvh_bounds`）与设备段 →
  `megakernel.rs` 把设备段 `append_to_mesh_device` 到本帧 `MeshDevice` 并取回绝对条目号 →
  以 struct-update 复制 `FlexHostInputs` 覆盖 `flex_bvh_id` 后进既有上传路径。
- **host 解锁 `dim 2/3`**：`render_with_flex(..., Some(&mesh_channel))` 时 2D/3D 不再命中
  `one_dimensional_scene_leaves` 的 `UnsupportedMeshPrimitive`；无 mesh 通道时 2D/3D 仍返回显式
  `flex-scene-leaves: ...` 错误（不 panic、不静默降级），1D 端到端行为与 S3a 逐字节一致。
- **调用点硬化（QA S2c `3b8e0d7f` 的 observation）**：`FlexMeshAssembly::build` 先做
  `channel.topology.validate()`（S2c `FlexTopologyFields` 自校验：长度/计数/非负），再按 flex 索引读取；
  短拓扑向量返回 `FlexSceneError::Topology`（新增单测覆盖，不 panic）。函数内部硬化另立
  `2026-11-04-render-r06-s2c-topology-render-validate-fix`（owner Backend，files 仅 `src/io/flex/topology.rs`）。
- **`None` 路径逐字节不变**：`render_with_flex(..., None)` 与 `render` 的 `RenderPass` 全 `None` 完全同构；
  S3c 新增代码只在 `mesh_assembly.is_some()` 时执行，`pack_flex_scene` 为 S3a 逻辑纯移动。
- **行数纪律**：`megakernel.rs` 450 行死线 → S3c 的 host 装配收进 `bvh/flex_scene.rs`，S3a 的
  flex 场景打包挪进 `megakernel_arrays.rs::pack_flex_scene`（纯移动），`megakernel.rs` 从 521 压回
  **449 行**。`render_with_flex` 8 参数（S3c 必须显式区分 1D/无为 `None`）用仓库既有
  `#[allow(clippy::too_many_arguments)]` 先例 + 理由注释。
- 精度：设备侧只做 f32；mesh 路径复用既有 `ray_mesh_leaf`/`mesh_query_ray`（S3 位级口径不变），
  未新增 `sin/cos`/f64 归约；隐藏段与胶囊路径共享统一后判据。

### 3.1 S3a 相对索引 → S3c 绝对索引（修复说明）

S3c 是首个把 flex mesh 段并入既有 `MeshDevice` 的路径。S3a 只走 1D 胶囊（`u64::MAX` 哨兵、
不碰 mesh 段），所以下面这处语义差在 S3a 从未暴露，S3c 接线时必须修掉：

- **Before（S3a 遗留语义）**：`bvh/flex_scene.rs::mesh_device` 产出的 `bvh_ids[flex]` 是**该 flex
  段内的相对条目号**（`device.entries.len()`）。若原样写进设备 `flex_bvh_id` 槽，设备侧
  `flex_bvh_id[flexid] + worldid` 会拿相对号索引 `MeshDevice.offsets`——当本帧既有 mesh 条目数
  `> 0` 时指向错误条目（错命中/越界），既有条目越多偏得越远。
- **After（S3c 修复）**：`FlexMeshDevice::append_to` 返回 `base = offsets.len()`（**追加前**的既有
  条目数），`FlexMeshAssembly::append_to_mesh_device` 输出 `base + 相对id`；`u64::MAX`（1D/无 mesh）
  原样保留。设备条目索引因此是 `MeshDevice.offsets` 的**绝对条目号**；相对号只存在于
  `mesh_device` 内部，转换点唯一在 `append_to_mesh_device`。
- **一致性三件套**：`mesh.meta`/`mesh.params` 追加后，本段条目偏移 `[meta_base, params_base]` 加同一
  基址，设备 `flex_bvh_id` 槽值加同一 `base`——三处同源，`append_to` 返回值是唯一契约
  （`base` 的语义写进函数文档与 `FlexMeshAssembly::append_to_mesh_device` 文档）。
- **可复核点（QA）**：①`mesh_device` 内 `bvh_ids` 只做段内相对计数（不改写为绝对号）；
  ②`append_to` 的 `offsets.push([entry[0] + meta_base, entry[1] + params_base])` 与
  `mesh.meta`/`mesh.params` 的 `extend_from_slice` 同序同基址；③`append_to_mesh_device` 对每 flex
  输出 `base + id`（`u64::MAX` 哨兵保留）；④单测 `packs_mesh_bvh_into_mesh_device_layout` 覆盖
  2D/3D 条目数、条目偏移与顶点值。

## 4. 验证

1. **四门禁**（`.normify/runs/2026-11-04-render-r06-s3c-flex-mesh-device/`）：fmt / clippy /
   clippy-cuda / unit 全过；unit **1302 passed / 0 failed / 398 ignored**（S3a 基线 1298 + 4 条新单测）。
2. **新增单测 4 条**（`cargo test --locked --lib render::bvh::flex_scene` → 4 passed / 0 failed）：
   混合 1D/2D/3D 装配（叶数/条目数/条目基址/2D/3D 顶点偏移）、world 隔离（同 flex 两 world 顶点差 +10）、
   1D-only 通道零 mesh 条目、拒绝路径（nflex/短向量/非有限半径/拓扑宽度/地址链/位置长度/worlds=0/
   非法维度/短 S2c 拓扑字段不 panic）。
3. **离线 NVRTC 预检**（与产品同 recipe；纯 NVRTC 编译，**不建 CUDA 上下文、不发射内核**）：
   `tools/nvrtc_precheck.py`（S3a 工具同源副本）→ `nvrtc-precheck.txt` / `nvrtc-precheck-log.txt`：
   - nvrtc 12.8；选项 `--std=c++17 --gpu-architecture=compute_70 --fmad=true
     --device-as-default-execution-space -Iinclude/warp-headers`（`artifact/nvrtc.rs` L92）；
   - 7 段拼接源码 sha256 `d48fc74e64a4a5ab92a603e27e205d0eee11a72708a32c1fd9cd2af7a882e330`；
     PTX **343993 B**、sha256 `9118d1eae20efd8d9721112bd50ad2f3403c7011028d422cd1ed5b42807e4fb8`；
   - `check_render_ptx` 等价规则（证据侧独立复算）：W15 两条 `_svd_config<double>` 常量各 1 次
     （`f64_globals = [1, 1]`）、`sin/cos` 归约三元组 5 处、f16/bf16/tf32 **0**、入口
     `render_megakernel` 在；编译日志仅 compute_70 弃用告警（与 S3a 同）。
   - 说明：本片改动只新增被内联的设备分支，PTX 字节数与 S3a（314257 B）不同属预期，判据看门禁规则
     不变而不是字节相等。
4. **`None` 路径位级不变（离线可证部分）**：设备 flex 分支由数据门控（无 flex 叶 ⇒ `leaf_geom >= 0`
   ⇒ 不进入 flex 分支）；`render_with_flex(None)` 的实参/上传/场景打包与 S3a 逐项一致。
   **未跑 GPU**：1D/2D/3D 像素/深度/seg 数值留 S3b 窗口。
5. **零 GPU、零窗口、零参考**：未启动内核、未开窗口、未生成参考；不改上游 pin/判据/容差。

## 5. 边界与遗留

- **不宣称 2D/3D flex 渲染可用**：无 GPU 数值结论；S3b 首窗仍是 1D flex + 刚性几何共存
  （2D/3D 数值验收需 S2c 数据齐 + 单独窗口材料，归 Arch/Chief 排）。
- **API 变化**：`MegakernelRenderer::render_with_flex` 增第 8 参数 `Option<&FlexMeshChannel<'_>>`；
  本仓当前无调用点（窗口/harness 在 S3b 之后才接）。`src/render/context/flex_host.rs` 文档注释里的
  `render_with_flex(..., Some(&assembly.inputs()))` 旧签名示例属该文件 scope，未在本片回改（不影响编译）。
- 上游 2D 顶点法线在并行 kernel 里 `+=` 累加（顺序不定）；本实现按元素下标升序确定性累加，
  2D 窗口验证时按 S2c/S3c 卡口径处理（沿用 S3a §5）。
- `FlexMeshBvh::refit` 未接持久 registry（每帧重建，上游 `refit_flex_bvh` 的等价语义）；
  大场景性能结论不在本片。
