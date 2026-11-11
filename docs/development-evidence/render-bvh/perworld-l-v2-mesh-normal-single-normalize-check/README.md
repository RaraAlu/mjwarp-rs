# QA 复核：mesh 法线链单次归一化修复卡（`f728b2d7`）

- 复核卡：`2026-11-11-render-perworld-l-v2-mesh-normal-single-normalize-check`（QA）
- 被复核：`2026-11-11-render-perworld-l-v2-mesh-normal-single-normalize`（Backend，`f728b2d7`）
- 性质：只读复核，零 GPU；未跑诊断轮；只写本卡与本卡证据。
- 结论：**PASS（0 阻断，3 条信息级观察）→ 可发诊断轮 token（验证修复）**。不宣称修复已生效；G-L2 判定维持未通过；不改判据/阈值/参考。

## 1. 最小改动（逐行 diff）

| 文件 | 变化 | 结论 |
| --- | --- | --- |
| `src/render/kernel/cuda/render_math.cu` | **-1 / +3** | `- *out_normal = normalize3_mesh(min_normal);` → `+ 2 行注释 + *out_normal = min_normal;` |
| `render_common.cu` / `render.cu` / `render_shadow.cu` / 其它 `.cu` | **0 diff** | 消费端归一化保留 |
| `.normify/modules/mjwarp/render/kernel.md` | 1/1 | 仅 fingerprint 刷新（源码变更派生登记） |

- 文件 sha256：`2c39c481…`/12,325 B → `4f103495…`/12,566 B（我从父提交 blob 独立复算，一致）。
- `intersect_tri` 仍 `*normal = cross(ab, ac);`（未归一化）；`mesh_query_ray` 的 `t/u/v/sign/face` 未出现在 diff。

## 2. 调用链（独立对源码）

- `mesh_query_ray` 全仓调用方**恰 2 处**：`render_common.cu:194`（megakernel `ray_mesh_leaf`，命中后 `:200 *normal = normalize3_mesh(mat_vec(mat, n_local))`）与 `render.cu:225`（depth/seg，`:231` 同式）——**每个消费端各一次归一化**。
- 背面剔除两处均 `dot(lvec, n_local) < 0`：**只用符号**，normal 是否归一化不改变符号。
- shadow：`render_common.cu:461 hit.normal = n`（已归一化）→ `hitpoint + normal*1e-4`，语义不变。
- hfield(gtype=1)/SDF(gtype=8)：`render_common.cu:417` 与 mesh(gtype=7) 同分支 → `ray_mesh_leaf`，三路一致。
- `render_shadow.cu` 零改动；证据 `logs/call-chain-audit.txt` 与源码逐条对上。

## 3. 内核身份新基线（独立复算）

按 `mod.rs` include_str 顺序做 LF 字节串联复算 source-sha：

| artifact | source old → new | PTX old → new |
| --- | --- | --- |
| megakernel | `796bade6…` → **`4d932c90…`**（我复算一致） | `aa51258b…`/345,514 B → **`13ac8a2e…`/344,186 B** |
| aa_resolve | `0bf8af5a…` → 同（未变，我复算一致） | `b28e155c…`/27,536 B 两侧同 |
| depth_seg | `562113a2…` → **`b546ef08…`**（我复算一致） | `d1e3e576…`/138,857 B → `e8d84963…`/138,540 B |

- 属**实现修复的新基线**，非判据/阈值/参考变更；证据 `logs/kernel-identity-old-new.txt` 与四个 identity 日志逐项一致。

## 4. 门禁 / 边界 / 卫生

- build（dev 0.13 s / test 50.93 s 均 Finished）+ `cargo test --no-run` 成功 ⇒ exit 0；四门禁 runs 全 `passed=true`、unit **1387/0/402**。
- 两个内核身份用例 `flag_and_aa_sources_compile_under_render_gate`、`depth_seg_source_compiles_under_render_gate` 均 `... ok`。
- 边界：`tests/`/`fixtures/`/`scripts/` 0 diff；G-L2/RCA/诊断卡证据 0；`guards-perworld-l2-light.json` 0（`fad2ed37…`/43,729 B 未变）；**冻结 stash 12/12 仍 == guards**（逐文件重算）。
- manifest **13/13 重算全对**；证据文件编码卫生（UTF-8/LF/无 BOM）。

## 5. 观察项（信息级，不阻断）

1. **O1**：修复后 **12 个 live exe 全部 ≠ guards**（源码变更触发重建，属预期）——**下一次正式窗口必须重新冻结并建立新基线**；README 已在「影响面」写明。
2. **O2**：同构站点 `src/render/bvh/cuda/mesh.cu`（`mesh_bvh_ray` L290 `normalize3(min_normal)` + L357 `normalize3(mat_vec(...))`）**未在本卡修改**；它不在 G-L2 判据 A 渲染链上（本卡范围正确），但其公开 API 与回归 `render_mesh_bvh`（G-L2 exit 0）存在，建议登记后续 H2-class 复查。
3. **O3**：本卡为源码修复，未跑 GPU、未验证 5 px 归零——结论只能到「可发诊断轮 token（验证修复）」。

## 6. 非声明

不宣称判据 A 已修复、不宣称 G-L2 通过/恢复、不宣称上游等价、不宣称任何渲染数值结论；不改判据/阈值/参考/相机/recipe；G-L2 判定维持未通过。

## Addenda（2026-11-11，P10 errata）

- 本卡复核的 H2 表述（「上游只归一化一次」）已按 append-only 记录作废；更正见 `perworld-l-v2-mesh-normal-single-normalize/README.md` §6、`perworld-l-v2-gl2-criterion-a-rca/README.md` §9、P10 卡 `perworld-l-v2-mesh-normal-chain-pin/README.md` §8。原复核文字保留不改。
