# R05-S2：设备侧 hfield 三角化 + mesh BVH 求交接入（无窗口；数值验收留 S3 验证窗）

任务：`2026-10-28-render-bvh-r05-s2-hfield-intersect-r2`（P8 渲染与 BVH；p57；owner Backend）
拆分前树 HEAD：`59d1b2a4`（拆分② O1 证据修正后）
> **重登记说明**：本片由原卡 `2026-10-28-render-bvh-r05-s2-hfield-intersect` 重登记（原卡因 files 未含两条必然失效的旧断言、运行中声明冻结无法扩 scope 而 abandon）；实现与证据目录不变。
阶段：**宿主三角化 + 注入 + 槽语义 + 设备分派全部接线完成**；本片**零渲染数值结论**，
数值验收（RGB ≤1 LSB 位级优先 / depth ≤4 ULP 位级优先 / seg exact / 两次 sha 相同）留 R05-S3 验证窗。

## 1. 行数口径（S2 rebase，Chief 批准 + Arch 转达）

拆分①实际交付 `megakernel.rs` 490→**406**（预核曾按 ≈345 估，QA 已验收），旧上限「366」失效。
Chief 批准的 S2 口径：**`megakernel.rs` 零改动或 ≤500**（不为凑 345/366 再拆），
槽填充/上传优先落 `megakernel_uploads.rs`；`cast.rs` ≤450；其余 ≤500。

最终实测（LF 行数，0x0A 计数）：

| 文件 | S2 前 | S2 后 | 预算/说明 |
| --- | --- | --- | --- |
| `src/render/kernel/megakernel.rs` | 406 | **404** | **净 −2**（hfield 打包整体下沉 `cast_mesh.rs`，本文件只留一次调用 + 1 行槽实参） |
| `src/render/kernel/megakernel_uploads.rs` | 384 | **405** | +21：`hfield_bvh_id` 槽上传（u32 字对承载 u64）+ `hfield_bvh_count` |
| `src/render/kernel/cast.rs` | 411 | **411** | 零改动（仅 `pub use cast_mesh::{HfieldDevice, MeshDevice, pack_mesh_devices};` 的 re-export 行原样更新，行数不变） |
| `src/render/kernel/cast_mesh.rs` | 102 | **223** | +121：`HfieldDevice` + `pack_mesh_devices`（S2 宿主胶水） |
| `src/render/bvh/hfield_mesh.rs`（新） | — | **266** | 上游三角化移植 + 单测宿主 API |
| `src/render/bvh/aabb.rs` | 406 | **457** | +51：`GeomKind::Hfield` + 中心偏移 bounds + 单测 |
| `src/render/bvh/mod.rs` | 244 | **278** | +34：hfield 子模块/导出/BvhError 变体 |
| `src/render/context.rs` | 427 | **453** | +26：`ResolvedContext.hfield` + `set_hfield` builder |
| `src/render/context/hfield.rs` | 372 | **449** | +77：`HfieldStatic`/`HfieldContextInput`（拥有型注入输入） |
| `src/render/context/owner.rs` | 414 | **415** | 文档（SceneLeaf hfield 支持） |
| `src/render/mod.rs` | 384 | **385** | +1：re-export `HfieldContextInput`/`HfieldStatic` |
| `src/render/kernel/cuda/render_common.cu` | 393 | **410** | +17：`cast_ray` gtype==1 分派（哨兵跳过）+ cull 口径 |
| `src/render/kernel/cuda/render_shade.cu` | 359 | **383** | +24：`texture_hfield` + gtype==1 UV 分派 + shadow 参数透传 |
| `src/render/kernel/cuda/render_megakernel.cu` | 340 | **347** | +7：`hfield_bvh_id`/`geom_dataid` 实参 + 顶点法线 gtype gate |
| `src/render/kernel/cuda/render_shadow.cu` | 30 | **32** | +2：`geom_dataid`/`hfield_bvh_id` 参数透传 |
| `tests/render_hfield_mesh.rs`（新） | — | **240** | 12 测试：三角化顺序 + 槽语义 |

**红线零改动**：`render.cu` **499** / `launch.rs` **500** / `model/render.rs` **500** / `camera_output.rs` **499**。
`DEVICE_SOURCES`=28（collision 侧）、入口=6、`CollisionArgs` 504 B、`MEGAKERNEL_SLOTS`=81 均未改动。

## 2. 三角化逐条对齐上游（`bvh.py` 行号 → 移植）

上游锚点：冻结 MJWarp 3.15 `71da24d` 的 `_src/bvh.py`。

| 上游 | 语义 | 本片实现（`src/render/bvh/hfield_mesh.rs`） |
| --- | --- | --- |
| L472-484 | `get_point_index(r,c)`：`x=sx*(c/width-1)`、`y=sy*(r/height-1)`、`z=data[r,c]*sz_scale`，按 `(r,c)` 首次出现插入去重 | `point_index` 闭包 + `HashMap<(usize,usize),u32>`；插入顺序 = 首次访问顺序 |
| L486 | `visited[nr-1, nc-1]` | `vec![false; (nr-1)*(nc-1)]` |
| L499-500 | 共面判据 `abs((z00+z11)-(z01+z10)) < 1e-5` | `planar()`：f32 差值与 f32 阈值（见 §2.1） |
| L504-513 | 非共面单格：`TL,TR,BR` + `TL,BR,BL` | 同序（`[i00,i01,i11]` + `[i00,i11,i10]`） |
| L517-545 | `slope_x=z01-z00`、`slope_y=z10-z00`；`fits_plane`：共面 + 同平面预测 `z00+(rr-r)*slope_y+(cc-c)*slope_x` + 斜率一致 | 同式（f32 左结合累加） |
| L547-563 | 先扩宽 `w`（`c+w < nc-1`、未访问、`fits_plane`），再整行扩高 `h` | 同序 |
| L564-565 | `visited[r:r+h, c:c+w]=True` | 同 |
| L567-576 | 合并矩形：`TL,TR,BL,BR` 取点顺序；索引 `TL,TR,BR` + `TL,BR,BL` | 同序 |
| L595-596 | `width=0.5*max(nc-1,1)`、`height=0.5*max(nr-1,1)` | 同（f64 计算） |
| L608-610 | `pmin/pmax` 后 `half=0.5*(pmax-pmin)` | `half_extents()`：逐轴 f32 减、乘 0.5（2 的幂无舍入） |
| L612-620 | `wp.Mesh(points, indices, bvh_constructor="sah", bvh_leaf_size=2)` | `MeshBvh::build`（见 §3 差异） |

### 2.1 精度口径（pinned 上游环境 numpy 2.5.4，NEP 50）

- `sx`/`sy`/`sz_scale` 是 `np.float32`；`width`/`height` 是 Python float。NEP 50 弱提升下
  Python float 先转 f32 再参与 f32 运算：`x = sx * f32(c/width - 1)`（括号内 f64 → 一次 f32 舍入）、
  `z = data * sz2`（f32 乘）。
- 共面/斜率判据在 f32 下算，**阈值 `1e-5` 也先转 f32 再比较**：`f32(1e-5) = 9.999999747378752e-06`
  （严格小于十进制 1e-5）。因此 `diff` 与阈值都用 f32——差值恰为 `f32(1e-5)` 时上游判**非共面**
  （`<` 不成立）。**原证据写「阈值不落在相邻 f32 之间 ⇒ 与 f64 比较等价」不成立**（错在阈值本身被
  翻译成 f32），已按 QA O1 修正为实现与证据，并用边界单测钉住：
  `diff == f32(1e-5)`（位型 `0x3727C5AC`）→ 非共面/不合并（2×3 → 6 顶点/12 索引）；
  `diff == f32::from_bits(0x3727C5AB)`（低 1 ULP）→ 共面/合并（4 顶点/6 索引）。
- `half` 的 `0.5*` 是 2 的幂缩放 → 无额外舍入。
- 网格退化（`nrow<2`/`ncol<2` 无三角形）上游会在 `np.min` 处抛异常；本片给可读错误
  `BvhError::HfieldEmptyMesh`（不 panic）。

**单测钉顺序**（`tests/render_hfield_mesh.rs`，16 条）：2×2 flat（点/索引顺序）、2×2 非共面单格、
3×3 线性坡面整体合并、2×3 贪心在首个非共面邻格停、`half` 含非零 `size[2]`、退化/数据不足/非有限错误面、
f32 阈值边界（恰为阈值 → 非共面；低 1 ULP → 共面）、三角化确定性（同输入两次结果相同）。

## 3. BVH 复用与策略差异（实测记录，不改参考/容差）

- **复用**：`src/render/bvh/mesh.rs::MeshBvh::build`（分箱 SAH `SAH_BINS=12`、`LEAF_CAPACITY=8`），
  hfield 与既有 mesh 共用同一设备打包/遍历（`render_common.cu::ray_mesh_leaf`）。
- **与上游差异**（记录，本片不做数值结论）：
  1. 上游 `wp.Mesh(bvh_constructor="sah", bvh_leaf_size=2)` 允许 leaf_size=2；`MeshBvh` 固定
     `LEAF_CAPACITY=8`，**无 leaf_size 配置入口**（本片不改 API，避免影响既有 mesh 回归）。
  2. 上游 SAH 是 warp 内部分箱实现；本片是自研分箱 SAH（12 bins，节点先子后父）。两者只影响
     遍历顺序/候选顺序，最近命中语义应由测试覆盖；**若 S3 判据出现 tie-break 差异 → 停手报 Arch，
     不改容差/参考**（卡面要求）。
- 场景 BVH 的 hfield leaf：`LeafGeometry::hfield(half)` + `compute_leaf_aabb` 的
  `center = pos + rot[:,2]*half[2]`（`bvh.py` L225-233，`fused_mul_add` 复刻 fma.rn.f32），
  `size` 传 `half` 而非 `geom_size`；`SceneLeaf` 文档已同步。

## 4. 槽语义替代（`MEGAKERNEL_SLOTS` 81 槽签名零变化）

- 槽 64（0-based）= `hfield_bvh_id`，声明 `Array(U64)`，**槽表零改动**（81 槽断言保持）。
- 值语义：**blob 内 mesh 索引**（hfield mesh 追加在既有 mesh 末尾，`mesh_count + order`；
  既有 mesh 索引不变）；长度 = `nhfield`；未被启用 hfield geom 引用的槽位 = `u64::MAX` 哨兵；
  无 hfield geom 时数组长度 1、值为 `u64::MAX`（占位）。
- 设备侧：`hfield_bvh_id[geom_dataid[geom]]` 取索引，`0xFFFF...FF` 跳过；`gtype==1` 走
  `ray_mesh_leaf`，与 mesh 同 cull 规则（阴影射线不剔背）。
- **与 warp 运行时 BVH id 的差异**：上游 `hfield_bvh_id` 存 warp `Mesh` 的运行时句柄
  （opaque `wp.uint64`，经 `wp.mesh_query_ray` 消费），未启用槽位为 `0`（null handle → 无命中）；
  本片存的是自有 blob 内 mesh 段索引（设备侧直接从 blob 读 meta/params），未用占位是
  `u64::MAX` 哨兵。两者都满足「每 hfield 一个 mesh id」的渲染语义，但**不是同一编号空间**。
- 上传表示：`TransferElement` 无 `u64` 实现（u64 类型不在本卡 files scope），故槽数据以
  **u32 低/高字对（小端）** 承载 u64 值；ABI 上等价于 `Array(U64)`（内核按 `wp::uint64` 读）。

## 5. 注入 A（非破坏 API；`None` 路径零差异）

- `RenderContext::create` / `resolve_context` **签名零改动**。
- 新增 builder：`ResolvedContext::set_hfield(&HfieldInputs<'_>)`：
  复用 S1 `resolve_hfield` 的全部校验（长度/边界/溢出/非正行数），再复制成拥有型
  `HfieldContextInput`（`HfieldStatic`：size/nrow/ncol/按 adr 切好的 data，单份、不随 world 复制）。
- 用法（证据口径；不新增 native）：
  ```text
  VolumeModelInput --(VolumeHfieldInputs::from_volume)--> HfieldInputs
    -> resolved.set_hfield(&inputs)          // 注入（None/空 = 不注入，合法）
    -> 每个 hfield: build_hfield_mesh(...)   // 得 vertices/indices/half
    -> SceneLeaf { geom_type: 1, size: half, .. } -> SceneState -> RenderContext::create(...)
  ```
- **`None` 路径零行为差异实测**：未注入时 `HfieldDevice::build` 返回 `Ok(None)`、`hfield_bvh_id`
  槽落 1 个哨兵占位、设备侧 `gtype==1` 不会被命中（模型含 hfield geom 时会先被 host 拒绝）；
  全量测试 **1086 passed / 0 failed**（含 O1 修复后 4 条边界/确定性新测试；S7b1/S7b2/S7b3/S8 位等回归随 unit 记录）。
- **QA O2（如实记录）**：`None` 路径的槽 64 上传值由旧占位 `u64(0)` 变为 `u64::MAX`（形状 `[1]`/类型 `Array(U64)` 不变）；设备只在 `gtype==1` 时读该槽，而该组合由 host 前置拒绝 ⇒ **渲染行为零差异**；但「上传内容不变」字面**不成立**（上传字节确实变了，只是不被读取）。
- 含 hfield geom 但未注入 → `LauncherError::Unsupported`（可读拒绝，不读未初始化数据）；
  `geom_dataid` 越界 → `InvalidArray`。

## 6. 设备分派（`render_common.cu` / `render_megakernel.cu` / `render_shade.cu` / `render_shadow.cu`）

- `cast_ray` 新增 `geom_dataid` + `hfield_bvh_id` 入参；`gtype==1` 与 `gtype==7` 同走 `ray_mesh_leaf`
  （hfield mesh 索引经哨兵检查）；`gtype==8`（SDF）仍由 host 拒绝（设备侧保守不判交）。
- 阴影可见度：`shadow_visibility` 透传两参数；阴影射线对 `gtype∈{1,7}` 不做背面剔除
  （上游 any-hit 语义，`n` 为零向量），primitive 仍 cull。
- 顶点法线分支追加 gtype gate（`7|8`），与上游 `render.py` L1209-1211 一致（hfield 不套顶点法线）。
- 纹理：`sample_texture` 增加 `gtype==1` → `texture_hfield`（`_texture_hfield` 逐式翻译：
  `u=(local.x+size[0])/(2*max(size[0],1e-6))`、`v=(size[1]-local.y)/(2*max(size[1],1e-6))`、
  `tex_uniform` 时 `rep *= size`、`offset=0`）。
- **不新增 `.cu`**；`render.cu`（S7b1 遗留内核私有 `cast_ray`）零改动。

## 7. 模块源与指纹

- `normify module source mjwarp.render --add src/render/bvh/hfield_mesh.rs`：**38 → 39**
  （拆分②后现盘 38；新增 1 项；不新增 `.cu` 所以 `.cu` 源清单不变；`cast_mesh.rs` 已在 38 内）。
- 指纹过期由本任务 `normify done` 刷新；收口后 `validate --fingerprints` 错误 0。

## 8. 验证

- `cargo test --locked` → **1086 passed / 0 failed**（133 个测试二进制；含 16 个 hfield 三角化/槽语义
  新测试（其中 QA O1 修复新增 4 条：f32 阈值常量、恰为阈值→非共面、低 1 ULP→共面、确定性）、1 个 hfield bounds 新测试；S7b1/S7b2/S7b3/S8 位等回归随 unit 记录）
- 两个既有测试的断言随支持面更新（行为变更的必要后果，非新增测试）：
  - `tests/render_context_lifecycle.rs`：原「hfield 由 `LeafGeometry` 拒绝」→ 改为「gtype=1 支持，
    bounds 按 `center=pos+rot[:,2]*half[2]` 断言」（旧断言已失效）；
  - `tests/render_scene_bvh_host.rs`：原 `from_geom_type(1) == UnsupportedGeomType{1}` →
    改为 gtype 9（未支持面）断言。
- `cargo clippy --all-targets --locked -- -D warnings` → 通过
- `cargo fmt --check` → 通过（`logs/fmt.log` 空）
- 内核编译门（`megakernel_source_compiles_under_render_gate`）：
  S2 前 `source_sha256=2d2c27df…`/`ptx_sha256=e955bb4b…`/258 660 B →
  S2 后 `source_sha256=72bc364c…`/`ptx_sha256=97467d83…`/**264 015 B**
  （**预期变化**：CUDA 分派/纹理/参数透传改动；`MEGAKERNEL_SOURCE` 与 PTX 新值入证据）
- **QA O1 修复后内核门复跑**：`source_sha256`/`ptx_sha256`/`ptx_bytes` 与 O1 前**完全一致**
  （`72bc364c…`/`97467d83…`/264 015 B）——O1 修复只在 host Rust 三角化，不触碰任何 CUDA 源。
- 日志：`logs/{fmt,clippy,test,kernel-gate-before,kernel-gate-after}.log`

## 9. 限制与未覆盖项（如实记录）

- **本片零渲染数值结论**：RGB/depth/seg 判据、阴影、纹理采样正确性全部留 **R05-S3 验证窗**
  （authoring 捕获 2 组与验证窗 9 实体材料各自单独报 Chief 签，本片不开窗）。
- hfield mesh/BVH **每帧重建**（`megakernel::render` 每帧重打包 scene/mesh 的既有设计）：
  大 hfield 有构建开销，未做缓存（留 S3/后续性能片）。
- `MeshBvh` 固定 `leaf_size=8`（上游 2）：tie-break/遍历顺序差异未做数值测量（见 §3）。
- 三角化 1e-5 判据为 **f32 结果 × f32 阈值**（numpy 2.5.4 NEP 50 口径，见 §2.1）；边界差异已由单测钉住；
  若 S3 参考仍出现边界差异 → 停手报 Arch，不改容差/参考。
- **遗留 O3（带进 S3）**：设备侧「used hfield 集合」来自模型 geom 列表（`geom_type==1` 且 `geom_dataid>=0`），
  未含上游 `geom_enabled_mask`；当前 S7b2 要求 `enabled_geom_ids` 覆盖全部 geom（host 侧前置校验），故当前路径与上游语义一致；
  未来支持部分使能时需把启用 mask 纳入 used 集合计算。
- **遗留 O5（带进 S3）**：调用方构造 `SceneLeaf` 时 `size` 必须是与三角化**同一份** `half`；S3 fixture 侧需保证，
  或由调用方约定加 debug 断言（本片不改变既有 API，故未加运行期校验）。
- 未覆盖：flex/splat/skybox/SDF 求交；hfield 多 world 复制（本片单份，按 S1 口径）。
- **S2 hfield 设备通路未验证（2026-10-29 附录）**：`nmesh=0` 场景（hfield-only 与 hfield+box 两组，MJCF 无 `<mesh>` 元素）
  的发射在 R05-S3b 验证窗 attempt1 被记录为 invalid round：`render_hfield_reference` 在发射阶段报
  `s7b2-span: 传输输入错误：维度无效：launcher_buffer`（同窗口 8 个回归实体 exit 0）。该缺口属既有实现缺口，
  由独立片 `2026-10-28-render-bvh-r05-s3b-hfield-empty-mesh-fix` 记录处理；**本卡的限制结论不变**，
  也不改判据/容差/参考。hfield 数值链路（RGB/depth/seg 判据）仍需 R05-S3b 验证窗 attempt=2 及后续片结论。
