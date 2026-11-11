# NP5-01b：已实现子系统证据锚点登记（3 子系统 + 23 未登记单元；不建阻塞卡、不宣称门禁通过）

- 任务：`2026-11-11-np5-01b-implemented-subsystem-anchors`（owner Backend；p90；P8-渲染与BVH）。
  依据：Chief 2026-11-11 指令 + Arch `np5-01a-exec-spec.md`（`5321f312…`/12,402 B，**工作材料，非证据**）§9（NP5-01b）
  + `np-gate-ledger.json`（`6d6ff7d9…`/116,688 B，190 条）附录 A。
- 性质：**只登记锚点与未登记单元清单**——不建阻塞卡（Chief 裁定 1：渲染/BVH、地形/SDF、GJK/EPA 已实现，走本片登记；
  真未实现的 4 个已由卡 H 登记为阻塞卡）、**不宣称门禁通过/NP01 通过/上游等价/双平台**；零 GPU、零窗；
  不改 `src/`/`tests/`/`fixtures/`/`scripts/`、不动 guards/freeze script、不改判据/参考/容差。

## 1. 三个已实现子系统（锚点五段：实现面 / 入口 / NP02 回执 / verified 卡与身份 / 仍缺）

| 子系统 | 实现面 | 内核入口 | NP02 回执 | verified 卡 / 发布证据 | 身份基线 | 仍缺 |
| --- | --- | --- | --- | --- | --- | --- |
| 渲染/BVH | 12 个 `.cu`（bvh/camera/kernel/texture）+ 7 个 host 链 `.rs` | 8 个（render_megakernel、render_depth_seg、aa_resolve、build_rays、mesh_bvh_ray、scene_bvh_ray、sample_texture2d、s7a_abi_probe） | **6 行 verified**（build_rays/mesh_bvh_ray/scene_bvh_ray/aa_resolve/sample_texture2d/s7a_abi_probe）；**缺 2 行** render_megakernel/render_depth_seg → NP5-03 | `2026-11-04-render-r06-flex-closeout`、`2026-11-11-render-perworld-l-v2-validate-window-g-l2-attempt4` | megakernel source `cf356b72…` / PTX `b63a51dd…` / 348,459 B；depth_seg `1e3f273c…`/`31912377…`/138,857 B；aa_resolve `0bf8af5a…` | NP01 逐字段表（8 入口 + 4 宿主面）；NP02 2 缺行（NP5-03）；E 指标未定稿 |
| 地形/SDF 碰撞 | 4 个 `.cu`（sdf_math/sdf_narrowphase/sdf_octree/hfield）+ 装配 `product.cu`；host 链 sdf/hfield `.rs` | mjwarp_collision_hfield、mjwarp_collision_sdf（product.cu 6 个 `__global__` 面内） | **2 行 verified**（hfield/sdf，source `cad08d3e…`、PTX `54ab59e3…`） | `sdf-release-v2.2`（2026-11-02 发布结论）、`sdf-validate`、`sdf-criterion-v2.2`、`sdf-freeze-runner-v2.1` | product.cu source `cad08d3e…` / PTX `54ab59e3…`（3,883,769 B） | NP01 行与逐字段表（碰撞单元聚合项）；g10-p5 E 指标对 SDF 分支的口径（用户级） |
| 凸体/网格 GJK、EPA、convex | 8 个 `.cu`（gjk_core/epa/polytope/polytope_build/support/simplex/contact/common）+ gjk host `.rs` | mjwarp_collision_boxbox、contact、contact_multi、primitive | **4 行 verified**（同 product.cu 装配/同 PTX） | `2026-11-04-g10-p5-family-closeout`（+ `g10-p5-gjk-epa`、`g10-p5-faithful-fma-fix-independent-check` 证据） | product.cu source `cad08d3e…` / PTX `54ab59e3…` | NP01 逐字段表（碰撞入口 + 设备库聚合）；G08/G09 边界口径显式引用（不改口径） |

> 机器可读版：`implemented-subsystems.json`（每条锚点带 `present` 存在性标记、NP02 行含 evidence_state/source/PTX sha；`subsystem_count=3`）。
> 对应的 `np02.subsystem_recheck` 三子系统均 `now_present=true`（checker 断言）；真未实现 4 个保持 `false`。

## 2. 23 个未登记产品单元（附录 A 登记表）

机器可读版：`unregistered-units.json`（`unit_count=23`；`source_ledger_sha256=6d6ff7d9…`；与账本逐 id/kind 一致）。

| kind | 数量 | id |
| --- | --- | --- |
| render-kernel-entry | 8 | render_megakernel、render_depth_seg、aa_resolve、build_rays、mesh_bvh_ray、scene_bvh_ray、sample_texture2d、s7a_abi_probe |
| render-host-chain | 3 | render_per_world_host、world_light_host、megakernel_arrays_host |
| render-host-diagnostic | 1 | render_diag_optin |
| collision-kernel-entry | 6 | mjwarp_collision_contact、contact_multi、primitive、boxbox、hfield、sdf |
| collision-device-lib-aggregate | 1 | collision_device_libs |
| np01-host-site-uncovered | 2 | host_site_render_light_dir_normalize、host_site_relmat_relative_matrix |
| np01-product-helper | 2 | math_fma_exact_f32_helper、render_quat_cast_host |

- 每单元登记字段：id / kind / product_files（存在性标记）/ np01_status / owner（author=backend、verify=qa）/ next_action。
- **只登记对象与 owner，不写逐字段表内容**（逐表由 NP5-01a 方法后续片产出）；不建阻塞卡。

## 3. 一致性 / 存在性校验（`logs/existence-check.txt`，12 PASS / RESULT: PASS）

- 账本 sha/bytes = `6d6ff7d9…`/116,688；规格 sha = `5321f312…`/12,402；证据 JSON 内 `source_ledger_sha256`/`source_spec_sha256` 与实物逐字节一致。
- 所有子系统锚点（实现面 24 个 `.cu`/`.rs` 文件 + 发布证据目录）在盘、`present=true`；12 个 NP02 回执行在 inventory 中 `verified`。
- 23 单元与账本 id 集合、kind 直方图逐项一致；23 单元全部 `product_files` 在盘。
- 账本 `subsystem_recheck`：3 个已实现 `now_present=true`、4 个真未实现 `now_present=false`；证据 JSON 无未加否定的门禁通过声明。
- 校验器为一次性只读脚本，留在 `target/render-bvh/gl2-ptx-diff/_check_j.py`（工作材料非证据，sha 记于日志头）。

## 4. 顺带更正（W4 计划文档 §9，append-only）

`docs/development-evidence/plan/W4-dependency-governance.md` 追加 §9：§3 标题「24 片 = B0 4 + 实体 20」为算术笔误，
实体片 = 7 卡 × 3 = 21，总数应为 **25 片 = B0 4 + 实体 21**；GPU 窗数不变（3–4 窗）。原文不改。

## 5. 边界与非声明

- 只登记锚点/清单：**不建阻塞卡**（4 个真未实现已由卡 H 登记）、**不宣称门禁通过/NP01 通过/上游等价/双平台**。
- 仍缺项如实写：NP01 逐字段表（23 未登记单元）、NP02 2 缺行并入 NP5-03、E 指标未定稿（用户 2026-10-27）等。
- 零 GPU、零窗；不改 `src/`/`tests/`/`fixtures/`/`scripts/`、不动 guards/freeze script、不改判据/参考/容差；历史快照不改写。