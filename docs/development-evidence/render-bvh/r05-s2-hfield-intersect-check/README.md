# R05-S2（hfield 三角化 + mesh BVH 求交接入）独立复核

任务：`2026-10-28-render-bvh-r05-s2-hfield-intersect-check`（P8 渲染与 BVH；p56；owner QA）
被复核任务：`2026-10-28-render-bvh-r05-s2-hfield-intersect-r2`（已 verified；由原卡 `…-r05-s2-hfield-intersect` 重登记）
被复核提交：`0656bd6e`（实现 + 证据，32 文件）+ `c84c1760`（verified 收口）
S2 前树：`59d1b2a4`；复核时 HEAD：`c84c1760`
范围：只读复核 + 本证据目录；**不开 GPU 窗口、不重跑窗口**；不改代码/判据/参考/夹具。

## 1. 结论

**verdict：pass / 阻断 0 / 观察 7**（O1 一般级建议修、O2 建议级、O3–O7 记录级；均不阻断本片验收）。

本片零渲染数值结论，复核口径是「接线与几何构造是否与上游一致 + ABI/回归是否零变化」，不是"能跑"。关键结论：

| 复核点 | 结果 |
| --- | --- |
| 三角化对齐上游 | 我对 `bvh.py` L453-622 独立复刻（numpy 2.5.4/NEP 50 f32 口径）并与实现逐位比对 **16 例**：14 例顶点/索引/`half` 位型**完全一致**（含 2×2/3×3/2×5/4×4 混合、非零 `size[2]`、缩放尺寸、贪心停边界）；**2 例在阈值边界（差值恰为 `f32(1e-5)`）不一致** → O1 |
| bounds | `GeomKind::Hfield`：`center = pos + rot[:,2]*half[2]`（fma）后走 box 公式，与 `bvh.py` L225-233 同式；`size` = `build_hfield_bvh` 的 `half`（上游 `render_util.py` L479-488 `hfield_bounds_size[hid]=hhalf` 佐证）；我独立复算两个 pinned fma 值均逐位一致 |
| 槽语义/ABI | `args.rs`（81 槽表）blob 与 S2 前**相同**；内核形参 `wp::array_t<wp::uint64> hfield_bvh_id`；host 以 u32 低/高字对（LE）承载 u64，字节数 8×count 与槽形状一致；`u64::MAX` 哨兵/长度=nhfield/空时 1 哨兵逐项核 |
| 两条旧断言 | diff 仅「hfield 由拒绝变支持」的必要后果：lifecycle 改 bounds 断言、scene_bvh_host 把 gtype 1 换成 gtype 9；**未放宽任何判据/容差**，其余断言逐行未动 |
| `None` 零差异 | 未注入时槽形状/类型不变（1 元素 Array(U64)）、设备侧不可达、全量 1082 测试全绿；**但槽值由 `u64(0)` 变为 `u64::MAX`** → O2 |
| PTX/门禁 | 我自跑内核编译门：`source_sha256=72bc364c…` / `ptx_sha256=97467d83…` / `ptx_bytes=264015`（与实现证据一致）；fmt/clippy/clippy-cuda exit 0；`cargo test --locked` **1082 passed / 0 failed / 387 ignored**（133 result 行） |
| 行数/红线/模块 | 全部行数与声称一致；`render.cu`/`launch.rs`/`model/render.rs`/`camera_output.rs` blob 未改；无新增 `.cu`；模块源 **39**、`validate --fingerprints` 错误 0 |
| 证据 | 实现片 8 文件、manifest **7/7 对 committed blob**、LF 无 BOM、禁词 0、README 163 行、receipt `/task_id` = r2 且含 `re_registered_from`；README §1–§8 段落齐全、零渲染数值结论 |

## 2. 三角化逐条对齐（自己复刻上游后逐位比对）

我把上游 `bvh.py::_optimize_hfield_mesh`/`build_hfield_bvh` 按 **numpy 2.5.4 + NEP 50** 口径独立复刻（`np.float32` 标量运算、Python float 阈值转 f32、`half=0.5*(pmax-pmin)` f32），再用临时 Rust 探针（`tests/_qa_s2_hfield_probe.rs`，运行后已删除、未入库）对 16 个自造样例逐位比对：

- **完全一致（14 例）**：`2×2` flat / 非共面单格、`3×3` 线性坡整体合并、`3×3` 贪心在首个非共面邻格停、`2×5` 纯宽向合并、`4×4` 混合、`3×3` 非零 `size[2]` 与非 1 缩放 `sx/sy`、`2×3` 阈值以下合并、`2×2` 阈值边界（对称不可区分）、退化 `1×1`/`1×5`/数据不足的错误面。
  比对内容：每个顶点 `(x,y,z)` 的 **f32 位型**、索引序列、`half` 三轴位型（含 `0x00000000` 的 z 例）。
- **不一致（2 例，见 O1）**：差值**恰为 `f32(1e-5)`** 时，实现合并、上游不合并（2×3 边界样例：上游 6 顶点/12 索引，实现 4 顶点/6 索引）。
- 行号对照：我逐段核对 README §2 表（L472-484/L486/L499-500/L504-513/L517-545/L547-563/L564-565/L567-576/L595-596/L608-610/L612-620）与 `bvh.py` 原文，**行号与语义均正确**。
- 退化面：`nrow<2 || ncol<2` 上游会在 `np.min(空数组)` 抛错，实现给 `BvhError::HfieldEmptyMesh`（可读错误、不 panic）；数据不足 `HfieldDataLength`、维度非正 `HfieldDimensions`、`nrow×ncol` 溢出 `HfieldCountOverflow` 均只在进 `MeshBvh` 前返回。

## 3. bounds（`bvh/aabb.rs`）

- 实现：`GeomKind::Hfield` → `center[axis] = fused_mul_add(rot[axis][2], size[2], pos[axis])`，再 `box_bounds(center, rot, size)`；`size` 语义为 `half`（`LeafGeometry::hfield(half)`）。
- 上游（`bvh.py` L225-233）：`did>=0` 时 `size = hfield_bounds_size[did]`、`hfield_center = pos + rot[:,2]*size[2]`、`lower/upper = _compute_box_bounds(hfield_center, rot, size)`；`hfield_bounds_size[hid] = hhalf`（`render_util.py` L479-488），即 `build_hfield_bvh` 的 `half` → **与实现同式**。
- `fused_mul_add` = `(a as f64 * b as f64 + c as f64) as f32`（单次 f32 舍入）；我用精确有理数 fma 独立复算证据中两个 pinned 位型（`0xbd0602f1`、`0xbe055442`），**逐位一致**；hfield center 与既有 capsule bounds 同款 fma 约定（S2a 56 位姿冻结参考已验）。
- `SceneLeaf` 文档已同步（gtype 1 支持、`size` 语义）；`gtype==8`（SDF）仍在 host 前置拒绝（`megakernel::render` 只拒 8），设备侧 `gtype != 8` 才走 primitive 分支、`geometry` 映射 7|8→Mesh（S2a 既有），与"hfield 已支持、SDF 未支持"现状一致。

## 4. 槽语义与 ABI 等价

- 槽表零改动：`src/runtime/launcher/render/args.rs` 相对 S2 前 blob **相同**（81 槽表未动）；`megakernel_slots_signature_is_unchanged` 在我 unit 复跑中通过。
- 内核形参 `wp::array_t<wp::uint64> hfield_bvh_id`（元素 8 字节）；host 侧 `upload(session, &hfield_words)` 用 `TransferBuffer<u32>`，每 id 压 **低 32 位在前 + 高 32 位在后**（LE），`hfield_bvh_count = 元素数`，宿主以 `array(&u.hfield_bvh_id, RenderElem::U64, &[count])` 绑定 → 字节数 8×count，`RenderArrayValue::contiguous` 校验 `required(8×count) <= span.bytes(8×count)` 通过；设备/宿主同为小端，cudaMalloc 基址对齐 ≥8 ⇒ **ABI 上等价于 u64 数组**。
- 值语义：blob 内 mesh 索引（hfield mesh 追加在既有 mesh 末尾，`mesh_count + order`），未用槽 `u64::MAX`，无 hfield geom 时 1 个哨兵；设备 `hfield_bvh_id[geom_dataid[geom]]` 取值并以 `0xFFFF...FF` 跳过；与上游 warp 句柄编号空间不同，README §4 已如实说明。
- 设备分派：`cast_ray` 对 `gtype==1` 与 `7` 同走 `ray_mesh_leaf`（hfield 用上述索引；`mesh_index` 对 hfield 不适用）；`hit.mesh_id` 对 hfield 保持 -1（顶点法线/网格纹理分支不适用）；`texture_hfield` 逐式对照 `render.py` L332-350（`s0/s1=max(size,1e-6)`、`u/v`、`tex_uniform` 时 `rep *= size`、offset=0），其 `size` 取 `geom_size`（不是 half，与上游同）。
- 阴影：`shadow_visibility` 透传 `geom_dataid`/`hfield_bvh_id`；阴影射线对 `gtype∈{1,7}` 不做背面剔除（上游 any-hit 语义，`n` 为零向量），primitive 仍 cull —— 与上游 `cast_ray` L802-806 的 `dot>0` 规则一致。

## 5. 两条旧断言必要性（逐行读 diff）

- `tests/render_context_lifecycle.rs`（+7/−7）：把「gtype=1 由 `LeafGeometry` 拒绝」改为「gtype=1 支持，bounds 按 `center=pos+rot[:,2]*half[2]` 断言」（`size=[1.5,2.5,0.5]`、单位位姿 → `[-1.5,-2.5,0]..[1.5,2.5,1]`）。**这是 hfield 由拒绝变支持后的必然失效断言**，断言强度未降低（改为具体数值断言）。
- `tests/render_scene_bvh_host.rs`（+3/−2）：`from_geom_type(1) == UnsupportedGeomType{1}` → `from_geom_type(9) == UnsupportedGeomType{9}`（改到仍未支持的面）。
- 其余断言无改动；`git diff` 中两文件仅上述 hunk。非「放宽判据/容差」。

## 6. `None` 路径与注入 A

- `RenderContext::create`/`resolve_context` 签名零改动；新增 `ResolvedContext::set_hfield(&HfieldInputs)`（复用 S1 `resolve_hfield` 全量校验，复制为拥有型 `HfieldContextInput`/`HfieldStatic`，单份不随 world 复制）；`hfield` 字段默认 `None`。
- 无 hfield geom → `HfieldDevice::build` 返回 `Ok(None)`、槽落 1 个哨兵、设备侧 gtype==1 不可达 ⇒ 渲染行为零差异（1082 测试全绿含 S7b1/S7b2/S7b3/S8 位等回归）。
- **但**：槽 64 的上传值由 S2 前 `u64(0)` 变为 `u64::MAX`（形状/类型不变）→ O2（字面"上传内容不变"不成立，行为差异不可达）。

## 7. 行数与红线（我的口径：committed blob LF 行数）

| 文件 | 行数 | 文件 | 行数 |
| --- | --- | --- | --- |
| `kernel/megakernel.rs` | **404** | `kernel/megakernel_uploads.rs` | **405** |
| `kernel/cast.rs` | **411** | `kernel/cast_mesh.rs` | **223** |
| `bvh/hfield_mesh.rs`（新） | **266** | `bvh/aabb.rs` | **457** |
| `bvh/mod.rs` | **278** | `context.rs` | **453** |
| `context/hfield.rs` | **449** | `context/owner.rs` | **415** |
| `render/mod.rs` | **385** | `render_common.cu` / `render_shade.cu` | **410 / 383** |
| `render_megakernel.cu` / `render_shadow.cu` | **347 / 32** | `tests/render_hfield_mesh.rs`（新） | **240** |

- 红线：`render.cu` **499** / `launch.rs` **500** / `model/render.rs` **500** / `camera_output.rs` **499**：相对 S2 前 blob **未改**；`args.rs` 494 未改；**无新增 `.cu`**。
- `DEVICE_SOURCES`=28、入口 6、`CollisionArgs` 504 B、`MEGAKERNEL_SLOTS` 81：由 unit 中断言全绿佐证（`device_sources_place_*`、`all_six_entries_take_args_by_value`、`abi_collision_args_matches_device_layout_504`、`megakernel_slots_signature_is_unchanged`）。
- 模块源 `mjwarp.render` **39** 条且含 `bvh/hfield_mesh.rs`；`normify validate --fingerprints` 错误 0（警告 5 条为既有结构类）。

## 8. 回归与证据

- 我独立复跑四门禁：`cargo fmt --check` exit 0、`cargo clippy --all-targets --locked -- -D warnings` exit 0、`--features cuda` exit 0、`cargo test --locked` **1082 passed / 0 failed / 387 ignored**（133 result 行）。
- 内核编译门（我自跑）：`72bc364c…` / `97467d83…` / 264 015 B，与实现证据的 after 值一致（before 为 `2d2c27df…`/`e955bb4b…`/258 660）。
- 实现证据：8 文件、manifest **7 条 7/7 对 committed blob**、不自引用、LF 无 BOM、禁词 0、README 163 行 ≤500、`receipt.re_registered_from` 正确。
- 提交范围：`0656bd6e` 32 文件、`c84c1760` 2 文件，全部落在卡面允许面（`src/render/`、`tests/`、证据、`.normify/`）。

## 9. 观察（7 条，均不阻断）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| **O1** | **一般** | **三角化阈值比较口径与上游不一致**：实现用 `threshold = 1.0e-5f64` 与 f32 差值做 f64 比较；上游 numpy 2.5.4 NEP 50 下 `np.float32 < python float` 按 **f32** 比较（`f32(1e-5)=9.999999747378752e-06`，**小于** 1e-5）。因此差值**恰为 `f32(1e-5)`（位型 `0x3727C5AC`）**时：上游判非共面/不合并，实现判共面/合并 → 顶点/索引不同（2×3 探针：上游 6 顶点/12 索引 vs 实现 4 顶点/6 索引；`fits_plane` 的预测/斜率判据同理）。其余所有 f32 值两侧一致（仅该值不重合）。README §2.1「阈值不落在相邻 f32 之间 ⇒ 与 f64 比较等价」的等价性不成立（错在阈值本身被转成 f32，而非相邻性）。影响仅当场景 hfield 数据出现该精确边界差值时几何差 ~1e-5·sz；S3 判据/参考不改 | **修**：实现改全 f32 比较（`let threshold = 1.0e-5f32;` 且 `diff.abs() < threshold` / `>= threshold`），或证据 §2.1 如实写明该边界差异并在 S3 前确认 fixture 不命中 |
| **O2** | 建议 | `None` 路径槽 64 上传值由 `u64(0)` 变为 `u64::MAX`（形状/类型不变：1 元素 Array(U64)）；设备侧仅 gtype==1 读取且"含 hfield geom 未注入"会被 host 前置拒绝 ⇒ 渲染行为零差异，但"上传内容不变"字面不成立 | **接受或补一句**：证据写明「值改变但不可达」 |
| **O3** | 记录 | used hfield 集合按 `geom_type==1 && geom_dataid>=0` 收集，未含上游的 `geom_enabled_mask` 条件；当前产品路径 `enabled_geom_ids` 要么空（全启用）要么全量（部分使能直接 `Unsupported`），实际集合与上游一致；未来支持部分使能需补该条件 | **接受**（留 S3/后续注意） |
| **O4** | 记录 | `fused_mul_add` 为 f64 计算 + 单次 f32 舍入：与 `f32::mul_add` 在极罕见双舍入边界理论上可能不同；S2a 既有约定（56 位姿冻结参考逐位一致），本片沿用；我用精确有理数复算两个 pinned 值一致 | **接受** |
| **O5** | 记录 | `SceneLeaf.size` 对 hfield 须由调用方传 `build_hfield_mesh` 的 `half`，实现**不校验/不自动推导**两者一致；用错（例如传 `geom_size`）会得到错误 bounds。证据 §5 已写用法 | **接受**；建议 S3 fixture 侧由同一 `half` 构造（或后续加 debug 断言） |
| **O6** | 记录 | BVH 策略差异（`leaf_size` 8 vs 上游 2、自研分箱 SAH）已如实记录；S3 若出现 tie-break 差异按卡面停手报 Arch，不改容差/参考 | **接受** |
| **O7** | 记录 | 三角化非边界行为与上游逐位一致（14/16 例，含 `half` 位型、索引顺序、贪心停位、缩放/非零 `size[2]`）；`nrow<2` 等退化给可读错误而非 panic（上游在 `np.min` 抛错），已记录 | **接受** |

## 10. 边界与未覆盖

- **零渲染数值结论**：RGB/depth/seg 判据、阴影、纹理采样正确性留 R05-S3 验证窗（本片不开窗、不跑 GPU）。
- 未在 S2 前树重跑内核编译门（PTX 值由我自跑 after 树取得并与证据对照；before 值引用实现证据记录，`MEGAKERNEL_SOURCE` 的 CUDA 源改动即预期变化来源）。
- 探针为临时测试文件（`tests/_qa_s2_hfield_probe.rs`，运行后删除、未入库）；上游复刻脚本与原始桩输出入 `logs/` 供复核。
- 未做 hfield 大网格性能/每帧重建开销测量；未覆盖 flex/splat/skybox/SDF 求交、hfield 多 world 复制。

## 11. 收口状态

- `normify gate 2026-10-28-render-bvh-r05-s2-hfield-intersect-check` → **exit 1「任务没有引用任何门禁」属预期**（QA 复核卡无门禁；receipt `gate_note` 已记）。
- `normify check 2026-10-28-render-bvh-r05-s2-hfield-intersect-r2` → exit 0；`validate --fingerprints` 错误 0。
- 两段提交：证据（本目录 + 任务卡）→ `normify done` → verified 记录单独提交；显式路径，无 `git add -A`。
