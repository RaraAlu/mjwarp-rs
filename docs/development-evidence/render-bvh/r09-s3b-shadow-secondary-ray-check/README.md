# R09-S3b secondary-ray 修复独立复核（mesh 遮挡 cull 语义对齐上游 any-hit；无 GPU 验证窗）

任务：`2026-10-28-render-bvh-r09-s3b-shadow-secondary-ray-check`（P8 渲染与 BVH；scope 仅本目录）
复核对象：实现 `e17da2b7`（3 个 `.cu`）、其后 attempt1 红项 b 的测试修正 `d501118b`；当前 HEAD `d501118b`
被复核证据：`docs/development-evidence/render-bvh/r09-s3b-shadow-secondary-ray/`
上游锚点：冻结 MJWarp 3.15 `render.py` / `ray.py`（`71da24d`）

## 0. 结论

**verdict：pass；阻断 0；观察 6 条（全部建议/记录级，无必须修项才可接受）。**

本片为 attempt1 红项 a（shadow-on cam1 漏判）的根因修复复核。我**未开 GPU 验证窗、未跑 runner/采样器**；
除四门禁与「内核编译门」外，只做了 Arch 授权的**少量 dev 单渲染诊断**（用已提交的验收测试本体 + 一次
上游 cull-off 单渲染），全部标注为诊断、不作窗口结论。数值验收仍在 **attempt2 验证窗**。

所有数字均为我自己重算/重跑（`recompute.py` → `recompute.json`，`logs/` 8 份），未引用 Backend/Arch 报告值；
attempt1 原始窗口产物与 dev 诊断 dump 在 `target/`（未入库，只读引用并在 JSON 标注 provenance）。

## 1. 独立复跑

| 项 | 命令 | 退出码 | 结果 |
| --- | --- | --- | --- |
| fmt | `cargo fmt --check` | 0 | 无输出（通过） |
| clippy | `cargo clippy --all-targets --locked -- -D warnings` | 0 | 通过 |
| clippy-cuda | `... --features cuda -- -D warnings` | 0 | 通过 |
| unit | `cargo test --locked` | 0 | **1050 passed / 0 failed / 387 ignored**（130 条 result 行求和） |
| 内核编译门 | `--test render_kernel_rgb -- --ignored megakernel_source_compiles_under_render_gate` | 0 | 1 passed；`source_sha256=2d2c27df…`、`ptx_sha256=e955bb4b…`、`ptx_bytes=258660` |
| 架构树 | `normify validate --fingerprints` | 0 | 错误 0（警告 5，既有项） |

内核编译门需 pin 的 `target/toolchains/cuda-12.8.1`（等价 `scripts/g01-fast.ps1` 的 `CUDA_PATH`），**纯 NVRTC 编译、不发射**；
我另算 `MEGAKERNEL_SOURCE` 六段拼接 sha256 = `2d2c27df…`（80988 B）与门禁一致，且与实现证据里 `render_kernel_rgb` /
`render_kernel_flags` 回归日志记录的 source/PTX sha **一致** → 回归日志确实是当前树的产物。

## 2. dev 单渲染诊断（非验证窗；仅 3 条已提交 GPU 用例 + 1 次上游单渲染）

**命令**（可复现）：
`cargo test --locked --features cuda-probe --test render_shadow_reference -- --ignored --test-threads=1 --nocapture`
（`logs/dev-acceptance.log`；GPU 现采 2555 MiB 桌面基线、无计算进程，单次运行 ~4 s。）

- **shadow-on**（cam0/cam1）：rgb `mismatched=0 / max_byte_diff=0 / beyond_1_lsb=0`；depth `mismatch=0 max_ulp=0`；seg `mismatch=0`；两次运行 rgb/depth/seg sha256 相同（用例内断言通过）。
- **shadow-off**：对 `capture-exact/shadow-off` 同判据全 0；对解锁前同配方参考（`s7b1::RECIPE_REFERENCE` / `s7b2::parse_rgb_fast_math_false_reference`）depth/seg `mismatch=0`、rgb `max_byte_diff=0`。
- **可见性二值对照**：product `rgb_diff=4745`，reference `rgb_diff=4745`（attempt1 时为 3537 vs 4745，缺 1208）；depth/seg diff 0。
- 结果：**3 passed / 0 failed**；sha256：shadow-on rgb `f044f3fa…`、depth `de732643…`、seg `065caf08…`。

**上游 cull 开关差异复算**（`logs/upstream-cull-probe-repro.log`）：把 S3a 冻结脚本复制到 `target/`、仅改
`enable_backface_culling` 与 ROOT、用 pinned venv 单次渲染 `--use-shadows true`，与 `capture-exact/shadow-on`
比较：cam1 `beyond_1_lsb=38`（与实现证据 `upstream-cull-probe.log` 的 38 一致）、cam0 `beyond_1_lsb=0`。
`legacy_reference_untouched=true`，fixtures 零写入。

## 3. 根因链独立核对（不接受「调参凑数」式解释）

1. **上游语义（pinned 源码原文）**：阴影走 `cast_ray_first_hit = _make_cast_ray(..., first_hit=True)`（any-hit 变体）；
   mesh 分支 `ray_mesh_with_bvh_anyhit(...) → d = 0.0 if hit else -1.0`，**不填 normal**（`render.py` L740-750）；
   随后统一 cull `if cull_backfaces and d >= 0 and dot(ray_dir_world, n) > 0`（L802-806），注释原文
   "skips branches with a zero-vector normal (any-hit)"；`ray.py` L739-756 的 any-hit 助手只返回 `bool`、
   无 cull 参数 → **mesh 遮挡不剔背、primitive 遮挡仍剔背**。
2. **修复前产品路径**：`cast_ray` 对 mesh 走 `ray_mesh_leaf(..., cull_backfaces)` 的背面 march（`CULL_MARCHES`）且外层 cull 对所有类型生效（`git show e17da2b7^` 逐行核对）→ 阴影射线对 mesh 遮挡多剔除。
3. **观测与像素级反例**（attempt1 原始日志 + dump，只读复算）：cam1 `mismatched=1451 / beyond_1_lsb=1436 / max_byte_diff=77`、depth/seg 两相机 0；dump 1436 行**全部落在参考阴影像素集（4745）内**、**所有通道只偏亮**（0 多判）、其中 **1195 像素与 shadow-off 完全一致**（即完全漏阴影）、max diff 77；产品阴影集 3537 vs 参考 4745（缺 1208）。→ 与「mesh 遮挡被 cull 掉」机制自洽。
4. **修复**：`cast_ray` 增 `bool shadow_ray`；`shadow_ray` 时 mesh 内部 cull 关闭、外层 cull 仅非 mesh；
   primary 传 `false`、shadow 传 `true`；全部 `.cu` 调用点仅两处（经 grep 核对）。与上游 any-hit 语义对齐。

## 4. 判据 / 参考 / 红线（零改）

- `git diff cbee1127..HEAD` 对 `fixtures/**`、`tests/support/render_bvh/**`、`tests/render_shadow_reference.rs`、
  `render.cu`/`launch.rs`/`cast.rs`/`model/render.rs`/`camera_output.rs`、`src/runtime/**`、`Cargo.*` **为空**。
- 冻结参考：`capture-exact/{shadow-on,shadow-off}/{rgb,depth,seg}.npz` 6 个 sha256 与 `shadow.rs` 冻结常量逐条相符；
  on/off 的 depth、seg 逐字节相同（阴影只影响光照）。
- 行数：`render_common.cu` 393、`render_megakernel.cu` 340、`render_shadow.cu` 30、`render.cu` 499、`launch.rs` 500、
  `cast.rs` 494、`model/render.rs` 500、`camera_output.rs` 499、验收测试 307、`shadow.rs` 416、`shadow_scene.rs` 238——全部 ≤500。
- `DEVICE_SOURCES=28` / 入口 6 / `CollisionArgs` 504 B 所在路径零改动；本片无新增文件。
- 证据：11 个文件（README/receipt/recompute.py + 8 份 logs；不含产物 `recompute.json`）CR=0、无 BOM；
  manifest 逐条对 committed blob 相符、无自引用；禁词 0。

## 5. 观察（全部建议/记录级；处置建议）

**Arch 2026-10-28 20:4x 裁决：verdict 保持 pass / 阻断 0 / 观察 6；O1/O2 由 S3b 收口证据补记、O3 已登记
backlog `2026-10-28-render-shadow-sdf-gtype-cull-gap`（不 start）、O4 记录接受并写入冻结协议、O5 措辞由
S3b 收口证据按本文口径统一、O6 记录接受。本目录证据不改判、不补库。**

- **O1（建议｜修）诊断数据的 provenance 未入库**：像素 dump（1436 行）、per-light 覆盖、产品 cull 开关实验
  （1436→38）、产品 cull=false 时的 38 depth/29 seg 差异，都只在 `target/`（`dev-diag/`、`attempt1/`），
  证据目录只留摘要；`target/` 一旦清理即不可复现。建议把关键诊断摘要（脱敏）入库，或在 receipt 里写明
  provenance（生成方式/路径/是否 gitignore）。我已按「未入库 provenance」只读引用，并复算了可复算部分。
- **O2（建议｜修）`logs/upstream-cull-probe.log` 缺口径说明**：只有两行计数、无脚本/配方。我独立复算得
  cam1 `beyond_1_lsb=38`（一致）、cam0 `any_diff=2` 但 `beyond_1_lsb=0`（max 1）——日志写 cam0=0，说明其
  「diff_pixels」实为「超 1 LSB」口径。结论不受影响，建议日志字段/命名写明口径。
- **O3（建议｜记录）`gtype == 8`（SDF/HFIELD）分支**：shadow_ray 下仍走外层 cull（`!(shadow_ray && gtype == 7)`），
  上游 any-hit 对 MESH 与 SDF 同一分支（都不填 normal → 不 cull）。当前 host 对 `geom_type 1|8` 写前拒绝
  （`megakernel.rs:106-111`）→ **不可达**；若未来解锁 SDF 需同步。建议记 backlog 或加注释。
- **O4（记录｜接受）dev 复核对 `target/debug/deps` 的副作用**：我跑四门禁 + 内核编译门 + dev 验收复跑，
  重写了 2 个 exe（`render_kernel_rgb-…` 20:26:19、`render_shadow_reference-…` 20:26:42），使 attempt2 早前
  冻结材料失效。我已停止一切构建，当前无 cargo/rustc 在途；9 exe 现态（mtime+sha256）见
  `logs/target-exe-state.txt`，供 Backend 重新冻结。
- **O5（建议｜修措辞）「38 = 上游 cull 开关自身差异」的证据强度**：我的复算支持 cull 开关本身在 cam1 改变
  38 个「超 1 LSB」像素（38=38 计数一致），但**未做像素集合相等证明**；产品 cull=false 的 38 残余也未独立
  复现（需 pre-fix 构建，按协调不再构建）。**「非产品缺陷」的直接依据是修复后判据全 0**（我 dev 复跑 3 用例全过）。
  建议证据把 38 表述为「诊断计数一致，非集合相等；结论依据修复后 0」。
- **O6（记录｜接受）attempt1 的 `shadows_acceptance_*` 失败是 runner 单行正则误报**（见
  `attempt1/instrument-note.txt`；runner 已改为多行匹配并离线复验）——与本片修复无关，引用以免误读。
  另：Backend 的 dev 诊断 harness（`tests/dev_shadow_diag.rs`）未入库，`dev-verify.log` 不可从仓库重放；
  我用已提交验收测试替代复跑，命令见 §2。

## 6. 边界（我没有验证的）

- **未开 attempt2 验证窗**：窗口判定、runner/采样/守卫、9 实体全窗结果均不在本复核范围；数值验收待 attempt2。
- dev 3 用例复跑是**诊断**（与 attempt2 将跑的同一测试本体），不构成窗口验收，不写窗口结论。
- 未独立重跑 primary-ray 回归 4 实体（会再次改写 `target/debug/deps`，按 Arch 协调不再构建）；改为只读核对
  其日志内容（6/3/6/5 passed、0 failed）并用当前树内核编译门核对 PTX source sha 与日志一致。
- flex / SDF 遮挡路径不在覆盖面（产品 kernel 的 `flex_bvh_ngeom=0`、host 拒绝 `geom_type 1|8`）。
- 不分场景外推：本结论只对 `scene-shadow` + `capture-exact` 单场景参考成立；容差级路径差异（closest-hit 复用于阴影射线）仍存在，跨场景风险留 S3b/后续。

## 7. 复算入口

```text
<repo>/target/render-bvh-precheck/venv312/Scripts/python.exe docs/development-evidence/render-bvh/r09-s3b-shadow-secondary-ray-check/recompute.py
```

`recompute.json` 含：证据 EOL/BOM、manifest 逐条、行数、冻结参考 sha、上游锚点原文命中、产品静态核对、
修复前代码事实、attempt1 红项与 dump 复算、上游 cull-off 复算、dev 验收 0 判据、禁词、红线 diff、problems（当前 **0**）。
