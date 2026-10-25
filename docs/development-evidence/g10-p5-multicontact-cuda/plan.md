# multicontact 片实现计划（只读准备稿；Arch 2026-10-25 23:07 拍板范围后）

任务：`2026-10-25-g10-p5-multicontact-cuda`。上游 pinned `71da24d`，冻结源
`docs/evidence/d04-complex-physics/authoring-runs/run-5/frozen-source/_src/`。

## 1. 已拍板的范围

| 项 | 决定 |
| --- | --- |
| ccd multicontact（≤4） | 实现（`cylinder-box` 覆盖，4 接触/world） |
| `box_box` 核（primitive family） | 一并移植（为覆盖 `box-box-margin`；证据里**与 multicontact 分开记归属**） |
| `mesh-mesh` | 上游搬 fixture 未启用 multiccd → 1 接触/world；按既有单接触路径对照 |
| mesh multicontact | **不实现**（需 mesh polygon ABI 输入），记未覆盖 |
| 参考 | 复用 QA `qa-upstream-gpu.json`（不新捕获；运行时 23 项输入 sha256 守卫） |
| ABI | 不改 `CollisionArgs` |
| PTX 入口 | 4 → 6（新增 `mjwarp_collision_contact_multi`、`mjwarp_collision_boxbox`） |
| `nvrtc_smoke.rs` | `entries=4` 改实测计数（test-only，证据声明） |
| 任务 files | 补 `src/runtime/` |

## 2. 设备文件与行数预算（全部 ≤500）

| 新/改文件 | 内容（上游映射） | 预计行数 |
| --- | --- | --- |
| `cuda/multicontact_geom.cu`（新） | `_feature_dim` L1533、`_aligned_faces` L1559、`_aligned_face_edge` L1573、`_box_normals` L1772、`_box_edge_normals` L1846、`_box_face` L1884、`_cylinder_normals` L1953、`_cylinder_edge_normals` L1972、`_cylinder_face` L1994 | ~300 |
| `cuda/multicontact_clip.cu`（新） | `_polygon_clip` L2061、`_set_edge` L2199 | ~150 |
| `cuda/multicontact.cu`（新） | `multicontact` L2216-2451（box/cylinder 分支；mesh 分支不实现并注明） | ~220 |
| `cuda/boxbox.cu`（新） | `box_box` 核心 `collision_primitive_core.py` L595-1049 + `_compute_rotmore` L563 | ~420（超 500 时拆 `boxbox_face.cu`） |
| `cuda/boxbox_wrapper.cu`（新） | `box_box_wrapper`（`collision_primitive.py` L1250-1352）：8 候选 `g5_write_contact(id_=i)` | ~35 |
| `cuda/contact.cu`（改） | `g5_ccd_pair_write` 末端加 multicontact 分支（L146 注释位）；单接触回退不变 | 221→~290 |
| `cuda/product.cu`（改） | 新增 2 个 `extern "C"` 入口（末尾追加，既有 4 个不动） | 318→~375 |
| `cuda/mod.rs`（改） | 5 个新 `include_str!` 常量 | 55→~62 |
| `src/physics/mod.rs`（改） | 5 个 `*_CUDA` 重导出 | 421→~427 |
| `src/runtime/collision_gpu/source.rs`（改） | `DEVICE_SOURCES` 16→21（顺序：`multicontact_geom→multicontact_clip→multicontact→contact→…→boxbox→boxbox_wrapper→product`） | 87→~95 |
| `src/runtime/collision_gpu.rs`（改） | `CollisionKernel` +2、入口常量 +2、Program 字段/加载/分派 +2 | 445→~478 |
| `src/runtime/collision_gpu/nvrtc_smoke.rs`（改） | 入口计数改实测；断言 6 入口 | 34→~38 |
| `src/runtime/collision_gpu/multicontact_tests.rs`（新） | CPU 单测：源片段顺序/入口唯一性/boxbox 与 multicontact 函数存在/`CollisionArgs` 尺寸不变 | ~120 |
| `tests/g10_p5_multicontact_cuda.rs`（新） | ignored GPU 测试：3 case + 参考 + 输入 sha 守卫 + 排序比较 | ~90 |
| `tests/support/multicontact_qa_support.rs`（新） | 参考加载/23 项 sha 守卫/meta 组装/case 运行 | ~260 |
| `tests/support/multicontact_compare_support.rs`（新） | 排序（`worldid`+全字段、重复点多重集）与 G5 判据比较 | ~380 |

## 3. 设备数值内核要点

- `g5_ccd_pair_write` 增加（编译期开关，对应上游 static `use_multiccd`）：
  1. EPA 后保留 `pt`（`has_polytope`）与 `multiccd_idx`；
  2. 条件：`(multi || (BOX,BOX))` 且两类型 ∈ {BOX,MESH,CYLINDER}；mesh 的 `polyadr` 检查本片不做
     （mesh multicontact 未实现，直接置 -1）；
  3. `g5_multicontact(...)` 用固定本地 scratch（`polygon[32]`/`clipped[32]`/`pnormal[16]`/`pdist[16]`/
     `idx1[10]`/`idx2[10]`/`n1[10]`/`n2[10]`/`endvert[10]`/`face1[16]`/`face2[16]`，单位置化；
     上游 `npolygonmax=16`（含 cylinder）、`nmeshdegmax=10`）；
  4. `ncollision<1` 回退单 contact；frame 仍由 EPA witness[0] 算一次；逐 i `g5_write_contact(..., id_=i,
      dists[i] if n>1 else dist, 0.5*(w1[i]+w2[i])+origin, frame)`。
- `g5_box_box_wrapper`：`g5_box_box(pos/rot/size/margin)` → 8 候选（未填充 `dist=MJ_MAXVAL`）→
  逐 i `g5_write_contact(..., id_=i, make_frame(normal[i]))`；`detected` 过滤沿用 `g5_write_contact`。
- 上游 `frame` 用 `make_frame`（`common.cu`）复用；不新写。

## 4. 测试与判据

- 参考：`docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-upstream-gpu.json`
  的 `cases.{cylinder-box,box-box-margin,mesh-mesh}`（`two_captures_raw_bitwise_identical=true`）。
- 输入：G5 冻结 case bins `docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases/<case>/`
  23 项；运行时重算 sha256/count 与 QA `input_bins` 全同才比较（同 primitive 片）。
- 判据：G5 既有口径（排序后逐字段；位级优先；`dist` 自身量级 4 ULP；`pos` 4·ULP(scale)；
  `frame` 4 ULP@1.0；`nacon`/`nccd`/`overflow`/参数/整数/`geomcollisionid` 位级；多接触行按
  `(worldid, dist, pos, frame)` 稳定排序、重复点按多重集）。**结构字段零容差。**
- 内核选择：`cylinder-box` → `ContactMulti`；`box-box-margin` → `BoxBox`；`mesh-mesh` → `Contact`
  （与上游 dispatch 表 + NATIVECCD 关闭规则一致）。

## 5. 风险与待观察

1. `box_box` 455 行 Python 移植量大，必须逐式对齐（含 `_compute_rotmore`、8 候选筛选顺序）；
   反向判别：把 5 接触的 case 打回单接触必然失败（预期 nacon 2 vs 10）。
2. multicontact 的 `_polygon_clip` 是核心（16 顶点、裁剪顺序、`dists` 赋值顺序）——先做 CPU 侧
   数值对照（可选 host 复算脚本），再上 GPU。
3. `mesh-mesh` 参考若是 full-model 捕获，需确认 1/world 单接触与设备逐字段一致；若不一致按
   未覆盖如实记（不伪造）。
4. PTX 入口 4→6 会改动已验证 NVRTC 证据里的入口计数描述；按 Arch 决定用 test-only 声明，
   不改已 verified 数字（只在 `nvrtc_smoke.rs` 里改打印/断言）。
5. 窗口只需验证测试（无 authoring 捕获）；基线申请等 Arch 放行后报。

## 6. 执行顺序（放行后）

1. 设备源 5 个新文件 + `contact.cu`/`product.cu`/`mod.rs`/`source.rs`/`collision_gpu.rs` 接线；
   `cargo check --features cuda-probe`。
2. CPU 单测（`multicontact_tests.rs`）+ NVRTC 冒烟（入口 6、拼装 len/sha 记录）。
3. 新增集成测试 3 case（参考复用 + sha 守卫）；窗口外预构建。
4. 报窗口基线 → Arch 审 → Chief 双签 → 跑。
5. 证据 README/receipt + `normify gate` + `check`；报 Arch 显式路径。

## 7. 放行前 CPU 复算结果（authoring，warp CPU；不占 GPU、不碰 src/tests）

脚本与输出（只读准备稿，放行后随证据一起整理）：

- `authoring/cpu_boxbox_recalc.py` → `authoring/cpu_boxbox_recalc.json`
  - 上游 `box_box` + `detected = dist < margin + gap` 复算 `box-box-margin`：
    **10/10 行、5+5/world、`geomcollisionid` 0..4**，`max_dist_ulp=1`、`max_pos_abs=1.86e-8`、
    `max_frame_abs=0.0`（frame 位级一致），无未匹配参考行。
  - 结论：`box_box_wrapper` 的 8 槽候选 → `detected` 筛 5 条 → `id_=i` 升序，与 QA 参考一一对应；
    设备移植的写入顺序/id 映射已验证。
- `authoring/cpu_multicontact_recalc.py` → `authoring/cpu_multicontact_recalc.json`
  - 上游 `geom_collision_pair → gjk_phase → epa_phase → multicontact` 复算 `cylinder-box`：
    **`multiccd_idx=8/8`、4 接触/world、id 0..3**，`max_dist_ulp=0`（位级）、`max_frame_abs=0.0`
    （位级）、`max_pos_abs=1.49e-8`，8/8 行匹配。
  - 结论：`multicontact()` 的 `dists`/witness 顺序、共享 frame、`0.5*(w1+w2)+origin` 位置公式
    与 QA 参考一致；`_polygon_clip` 的裁剪顺序假设已被参考数据验证。
- 两个复算都只启动 warp **cpu** 设备（未创建 CUDA 上下文；脚本仅读 G5 case bin + QA 参考 JSON）。
