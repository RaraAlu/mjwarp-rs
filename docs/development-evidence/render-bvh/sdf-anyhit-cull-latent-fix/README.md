# SDF any-hit 背面剔除 latent fix（1 行判据 + 注释契约 + host 拒绝回归测试；不解除 host 拒绝）

任务：`2026-10-30-render-sdf-anyhit-cull-latent-fix`（proposed → in_progress → verified）
承接：gap 卡 `2026-10-28-render-shadow-sdf-gtype-cull-gap`（Chief 2026-10-30 批准方案 A：先独立小片修 latent，解锁片在其上另立）
前置：`2026-10-29-render-r08-rgb-hetero-validate`（verified；本片改 `render_common.cu` 会改内核身份，必须在 R08 收口之后）
**本片不开 GPU 窗口、不跑设备执行、不宣称任何 SDF 渲染能力或数值结论。**

## 0. 一句话

上游 any-hit 阴影射线对 MESH/SDF 都不填 normal ⇒ 不应剔除背面；我们原来的 cull 排除表只有 `gtype==7|1`，SDF（8）缺失。今天 host 仍硬拒绝 `gtype==8`，所以该缺口不可达；本片把判据补齐（latent 对齐）、在 SDF 占位分支写下解锁契约、并用 **host-only 单测**钉住「host 仍拒绝」这一不可达前提。**host 拒绝未解除，SDF 求交未实现。**

## 1. 改动（`git diff --stat -- src` 只两文件，+43 / −8）

| 文件 | 改动 | 行数 |
| --- | --- | --- |
| `src/render/kernel/cuda/render_common.cu` | ① `cull_this` 的 any-hit 排除表由 `gtype == 7 \|\| gtype == 1` 扩为 **`\|\| gtype == 8`**（上游 any-hit 对 MESH/SDF 不填 normal ⇒ 不剔背）；② `gtype==8` 占位分支注释写解锁契约：「命中 `d=0`、未命中 `d=-1`、**不填 normal**、不参与背面剔除」，并指向 gap 卡 | 410 → **414** |
| `src/render/kernel/megakernel.rs` | ① 把 host 对 `geom_type==8` 的拒绝抽成 `fn reject_sdf_geometry(geom_type: &[i32]) -> Result<(), LauncherError>`（**错误串与行为不变**：`Unsupported("S7b3 不含 SDF 求交")`）；② 新增 `#[cfg(test)] mod sdf_geometry_guard_tests`：`sdf_geometry_stays_unsupported`（`[0,2,8]` → `Unsupported`，断言错误串）、`non_sdf_geometry_passes_the_guard`（`[]`/`[0,1,2,7]` → `Ok`） | 409 → **440** |

**不可达论证（为什么今天改设备 cull 安全）**：`megakernel.rs` 在发射前对含 8 的模型返回 `Unsupported`，任何含 SDF 的模型到不了设备；单测 `sdf_geometry_stays_unsupported` 把这条前提钉在 `cargo test` 里（host-only，不需要 GPU 会话）。解锁片必须同时改这个守卫与 SDF 求交，否则本片改动永远不可达。

## 2. 内核身份（旧 → 新；`MEGAKERNEL_SOURCE` 6 个 `.cu` 拼接）

| 项 | 旧 | 新 |
| --- | --- | --- |
| `MEGAKERNEL_SOURCE` sha256 | `e68e3412…` | **`be4ef01b1ae45188ac512c91cfef60244a1f8b6670307b245ae1719a34cf5f5e`** |
| megakernel PTX sha256 | `0b947492…` | **`4dae03ffb3fe82214da930714c7fd1ccb6a2c8d5c5eb7cc8e489624b4e7dd085`** |
| megakernel PTX bytes | 268,275 | **268,305**（+30，来自注释与判据） |
| `AA_RESOLVE_SOURCE` sha256 | — | `0bf8af5a68f9cae50d0cb9cc4da2a01da2607066cf1fe3072c8345e6e104bc76`（**未变**，`aa_resolve.cu` 未动） |
| aa_resolve PTX sha256 / bytes | — | `b28e155c6e8b0f1c8c05d36054036764d2c437ae3c58e09c8d10360666ec8a70` / **27,536**（未变） |

实测来源：`logs/kernel-identity.txt`（`cargo test --locked --features cuda-probe --test render_kernel_flags flag_and_aa_sources_compile_under_render_gate -- --ignored --nocapture`，NVRTC 编译 + f32-render 门禁，1 passed / 0 failed）。

## 3. 内核身份引用清单（历史 vs 现行；**不回改历史证据**）

可复现口径（pin 在 fix 提交 **`f63834e0`**，任何人在任何时间可复算）：

```text
git grep -I -n -E "e68e3412|0b947492|268275|268,275" f63834e0 -- .
```

- **pin 口径全仓：139 行 / 42 文件**（本片 `materials/kernel-identity-refs.txt` 即这 139 行，长行截断到 500 字符并标 `[truncated]`）。
  - `src/` / `tests/` / `scripts/`：**0 行（活体断言 0）**——本片改动不会让任何活体测试红灯。
  - `fixtures/**`：4 行（`g01/reference.json`、mass-solve 的 ptx json）——**无关 hex 载荷，不是渲染内核身份断言**。
  - `docs/development-evidence/**`：130 行 / 33 文件（R02 skybox S1/S3/check/summary、R08-RGB 离线/复核/窗口、154 收口、precheck plan 等历史记录）——**保留当时身份，不改**。
  - `.normify/changes/**`：5 行 / 5 文件（卡面文字引用）。
- **QA 独立复核在其工作树测得 79 行 / 41 文件**（更窄的包含集合 + 当时时点，不含部分 logs/JSON 与后来新增的复核证据）。两个数字不冲突：以 pin + 上述 `git grep` 命令为权威可复现口径；差异全部来自「包含哪些文件集合」与「扫描时点」，不是清单过期。
- **清单本身 0 过期**（生成时 pin 全量检索、无遗漏行）、**活体断言 0**（code 分区 0 行）这两条成立；证据随新证据/新卡增长时引用数会自然增加（自引用），不影响结论。
- 清单文件：`materials/kernel-identity-refs.txt`（逐行 `path:line:content`，长行 `[truncated]`，38 KB）。

## 4. 门禁与架构校验

| 项 | 结果 |
| --- | --- |
| `normify gate`（fmt / clippy / clippy-cuda / unit） | 全过；unit **1108 passed / 0 failed**（ignored 392）；`logs/gate-*.txt` 是**某一次**门禁运行的转写快照（**与当前 `.normify/runs/2026-10-30-render-sdf-anyhit-cull-latent-fix/*.json` 4 个文件不是同一次 run**：`at` 时间与 `scope_hash` 都不同）；**权威门禁结果以已提交的 `.normify/runs/` 为准**。证据目录在最终 gate 之后冻结，避免 scope 反复变化 |
| `cargo test --locked --no-run` | **`EXIT=0`** + 含 `Finished` → `logs/no-run.txt`（首行 `Finished`、末行 `EXIT=0`；重跑后追加 exit 码） |
| `normify validate --fingerprints` | **错误 0**、警告 6：既有 5 条（`api/leaf-empty`×2、`structure/leaf-too-coarse` 39、`layout/order-incomplete`×2）+ `evidence/fingerprint-drift 1（mjwarp.render.kernel）`——预期内，由 `done` 刷新指纹 → `materials/validate.txt` |
| `git diff --stat -- src` | 只 `render_common.cu`（12+）与 `megakernel.rs`（39+/8−）；**+43 / −8** |
| 边界 | 不改 81 槽 ABI / 判据 / 容差 / 参考；不改 host 拒绝语义（错误串不变）；不改 R08 / R02 证据目录（`unchanged` 检查）；无 GPU、无窗口 |

## 5. 不宣称

- 不宣称 SDF 求交已实现、不宣称 SDF 渲染能力已具备、不宣称任何渲染数值结论；本片只是 latent 判据对齐 + 契约注释 + 回归钉子。
- 不宣称内核身份「不变」：megakernel 三个值已按上表更新；历史证据保留当时值。
- 不处理 depth/seg 路径的 SDF 静默 miss（gap 卡新发现），留解锁片一并解决。

## 6. 文件

- `README.md`（本文）、`receipt.json`、`manifest.txt`
- `logs/`：`gate-{fmt,clippy,clippy-cuda,unit}.txt`、`no-run.txt`、`kernel-identity.txt`
- `materials/`：`kernel-identity-refs.txt`、`validate.txt`
