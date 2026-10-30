# SDF（gtype==8）设备侧 mesh 路径对齐（U1；host 拒绝保留，能力不暴露）

任务：`2026-10-30-render-sdf-mesh-path`（SDF 解锁片 U1；预核 `target/sdf-unlock-precheck.md` §8）
前置：`2026-10-30-render-sdf-anyhit-cull-latent-fix`（latent cull 对齐，verified）
关键上游事实：**渲染器对 SDF 没有距离场求交**——`render.py:738-761` 把 MESH|SDF 放同一分支，用 `geom_dataid`
指向的底层 mesh 走 `ray_mesh_with_bvh(_anyhit)`；`bvh.py:211-220` 的 bounds 用 `mesh_bounds_size[did]`；
纹理/顶点法线同 mesh 分支（`render.py:466-481/1211-1216`）；`render_test.py:1251` 原文「Mesh and SDF geoms
render using the underlying mesh geometry」。真正的 SDF 距离场只在物理碰撞（`collision_sdf.py`）。

**本片只做设备 dispatch 对齐；`megakernel.rs` 的 `reject_sdf_geometry` 原样保留**（未验证能力不暴露，
同 R09-S2 先例）⇒ 含 SDF 的模型今天仍到不了设备，改动**不可达**；host 解锁 = U1b（U3 verified 后、U4 前）。

## 1. 改动（`git diff --stat -- src` 只两文件：+9 / −11）

| 文件 | 行数 | 改动 |
| --- | --- | --- |
| `src/render/kernel/cuda/render_common.cu` | 414 → **415** | ① mesh 分支条件 `gtype == 7 \|\| gtype == 1` → **`\|\| gtype == 8`**（8 与 7 同用 `scene.leaf_mesh[leaf]`，即 `geom_dataid` 指向的底层 mesh）；② `hit.mesh_id = gtype == 7 ? ... : -1` → **`(gtype == 7 \|\| gtype == 8) ? mesh_index : -1`**（否则 `render_megakernel.cu:238` 的顶点法线分支因 `mesh_id<0` 跳过，与上游不一致）；③ 占位分支注释改为「8 已进上一分支、此处条件保留为防御」 |
| `src/render/kernel/cuda/render.cu` | 499 → **496（净负）** | DepthSeg 分支 `if (gtype == 7 \|\| gtype == 8)` 里 8 也调 `ray_mesh_leaf(..., scene.leaf_mesh[leaf], ...)`；删两行过时注释（原「SDF 按 miss 处理」） |

shadow（any-hit）路径无需单独改：`render_shadow.cu:28` 复用 `cast_ray`（即 `render_common.cu` 的同一函数），主命中对齐即覆盖；`cull_this` 已排除 8（latent fix）。

## 2. host 拒绝保留（能力不暴露）

- `src/render/kernel/megakernel.rs` **零改动**（卡面 `unchanged` 检查钉住）：`reject_sdf_geometry` 与调用原样，含 `geom_type==8` 仍返回 `LauncherError::Unsupported("S7b3 不含 SDF 求交")`。
- latent fix 的两条 host-only 单测仍通过（unit 1108 passed / 0 failed）。
- 结论：本片改动**今天不可达**；证据不宣称任何 SDF 渲染能力或数值结论。

## 3. 内核身份（旧 → 新）

| 项 | 旧（latent fix 后） | 新 |
| --- | --- | --- |
| `MEGAKERNEL_SOURCE` sha256 | `be4ef01b…` | **`778216b23e64e1211ad5c6e9087550851c4ee79a6d8385c5d3f0fd05016f40f5`** |
| megakernel PTX sha256 | `4dae03ff…` | **`34f91147f8861601f4d96602223d282b570be38a305b83511e43fa4fa76b408d`** |
| megakernel PTX bytes | 268,305 | **268,589** |
| `AA_RESOLVE_SOURCE`/PTX/bytes | `0bf8af5a…`/`b28e155c…`/27,536 | **未变**（`aa_resolve.cu` 未动） |

实测：`logs/kernel-identity.txt`（`cargo test --locked --features cuda-probe --test render_kernel_flags flag_and_aa_sources_compile_under_render_gate -- --ignored --nocapture`，NVRTC 编译，1 passed / 0 failed）。

**旧身份引用清单（pin = `02b9c629`，历史不回改）**：`git grep -I -n -E "be4ef01b|4dae03ff|268305|268,305" 02b9c629 -- .`
⇒ **28 行 / 14 文件**：`docs/development-evidence` 8、`fixtures` 4（无关 hex 载荷）、`.normify/changes` 2（卡面文字）、
**`src`/`tests`/`scripts` 活体断言 0**。逐行清单：`materials/kernel-identity-refs.txt`（长行截断 `[truncated]`）。

## 4. 门禁与校验

| 项 | 结果 |
| --- | --- |
| `normify gate`（fmt/clippy/clippy-cuda/unit） | 全过；unit **1108 passed / 0 failed**（ignored 392）；`logs/gate-*.txt` 为某次门禁的转写快照，权威见已提交 `.normify/runs/2026-10-30-render-sdf-mesh-path/` |
| `cargo test --locked --no-run` | **`EXIT=0`** + 含 `Finished` → `logs/no-run.txt`（末行 `EXIT=0`） |
| `normify validate --fingerprints` | **错误 0**、警告 6：既有 5 条 + `evidence/fingerprint-drift mjwarp.render.kernel`（预期，`done` 刷新）→ `materials/validate.txt` |
| `git diff --stat -- src` | 只 `render_common.cu` 与 `render.cu`，+9 / −11 |
| 行数 | `render_common.cu` **415**（≤417）、`render.cu` **496**（≤498，净负）；`megakernel.rs` 440 / `cast.rs` 481 不动 |

## 5. 边界与不宣称

- 不改 81 槽 ABI / 判据 / 容差 / 参考 / 其它 `.cu`；R08·R02 证据目录与 `megakernel.rs` 均未改（`unchanged` 检查）。
- **零 GPU、零窗口、零设备执行**；不实现 SDF 求交。
- 不宣称 SDF 渲染能力、不宣称任何渲染数值结论（host 仍拒绝 8，能力未暴露）。
- 后续：U2 采集 authoring（离线）→ U3 采集窗（**需用户双签**）→ U1b host 解锁（U3 verified 后、U4 前；U4 未过则回滚 U1b）→ U4 验证窗（**需用户双签**，新冻结 guards、不复用旧 exe 哈希）。
