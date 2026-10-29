# 架构树剩余 154 个未登记源收口（方案 B）

任务：`2026-10-29-arch-unregistered-sources-remaining`（Arch）
范围：只改 `.normify/`（modules / layouts / changes / runs）与本文证据目录；`src/**`、`tests/**`、`fixtures/**`、`Cargo.*` 零改动。

## 1. 目标与口径

P1 卡 `2026-10-28-arch-volume-module-sources` 只登记 24 个文件；全量比对后剩 154 个未登记。本卡让文件级 `normify impact` 对全部**自家代码**可用，并给 vendored 上游头一个成文政策（不入架构树）。

154 的构成（口径与 P1 一致）：

| 分类 | 文件数 | 处理 |
| --- | --- | --- |
| physics collision | 72 | 新叶 `mjwarp.physics.collision` |
| physics 其余 | 19 | 并入既有叶/既有族（core 17、mass-matrix 1、tendon-velocity 1） |
| runtime collision_gpu | 20（19 + `collision_upload.rs`） | 新叶 `mjwarp.runtime.collision-gpu` |
| runtime 其余 | 7 | 并入既有 `mjwarp.runtime` 容器 5 + `mjwarp.runtime.external` 2 |
| include/warp-headers | 36 | **不入架构树**（vendored 政策） |
| native | 0 | P1 已随 probe 归属 |

## 2. 方案 B（Chief 2026-10-30 批准）

新增 2 个叶子，其余并入既有叶/容器（合计 118 个自家文件）：

| 模块 | 父 | 源数（src 计） | 行数 | API |
| --- | --- | --- | --- | --- |
| `mjwarp.physics.collision` | `mjwarp.physics` | 72 | 17,535 | 5（从 `physics.core` 原文迁入） |
| `mjwarp.runtime.collision-gpu` | `mjwarp.runtime` | 20 | 4,004 | 1（真实入口 `run_product_collision`） |
| `mjwarp.physics.core` | — | 104 → **121** | — | 88 → 83（迁出 5） |
| `mjwarp.physics.mass-matrix` | — | 45 → **46** | — | 16（不变） |
| `mjwarp.physics.tendon-velocity` | — | 16 → **17** | — | 4（不变） |
| `mjwarp.runtime`（容器） | — | 53 → **58** | — | 0（不变） |
| `mjwarp.runtime.external` | — | 1 → **3** | — | 2（不变） |

未采用的更细切法：collision 再拆 `cuda`/`gjk`/`sdf`/`hfield` 子叶。原因是这 4 个新叶没有可用 API 契约，`module add` 默认 `apis=[]`，每个都会独立产出一条 `api/leaf-empty` 告警（5 行 → 9 行），收益只是 impact 粒度更细。等确实需要时另立细化片。

## 3. API 迁移（5 条原文搬迁 + 1 条真实入口）

`mjwarp.physics.core` → `mjwarp.physics.collision`，键与描述**原文搬迁、不新增不改语义**：

- `rpc:mjwarp_rs::physics::collision::collision`（U006）
- `rpc:mjwarp_rs::physics::collision::nxn_broadphase`（U007）
- `rpc:mjwarp_rs::physics::collision::sap_broadphase`（U008）
- `rpc:mjwarp_rs::physics::collision::primitive_narrowphase`（U009）
- `rpc:mjwarp_rs::physics::collision::sdf_narrowphase`（U010）

`mjwarp.runtime.collision-gpu` 新增 1 条**真实实现入口** `rpc:mjwarp_rs::runtime::collision_gpu::run_product_collision`（已核 `pub mod collision_gpu` 与 `pub use product_entry::{..., run_product_collision}`）。

键唯一性：全项目 398 个 API 键、**重复键 0**（见 `logs/module-counts-after.txt` 与本次只读核对脚本输出）。`mjwarp.io.core` 对 `physics::history::reset_history` 的既有依赖未受影响（history API 未搬迁，本卡不建 history 叶）。

## 4. vendored 头政策（不入架构树）

`include/warp-headers/` 36 个文件（NVIDIA/warp `v1.15.0` @ `ffa038b1adf3927a8e893d4536f1a7562f17d749`）**不登记**，理由：

1. 身份已在别处冻结：`docs/development-evidence/w15-verbatim-headers/headers.json`（`header_count=36`、逐文件 sha256），并由 `tests/launcher-warp-headers.rs::copied_headers_match_the_frozen_manifest` 断言。登记会引入第二事实源。
2. 规模噪声：36 文件 **43,321 行**，是本次自家登记量（27,432 行）的 1.58 倍，会把模块规模/行数语义整体带偏。
3. 图语义无效：它们是 C++/CUDA 第三方头，经 NVRTC include 路径消费，不参与 Rust 模块 `apis`/`deps` 图；没有本地文件级改动场景，只会随 W15 整树升级（按 manifest 差集处理）。
4. 反例代价：若建 `mjwarp.vendored.warp-headers` 单叶（36 源），会让 `leaf-too-coarse` 计数再 +1、且因无 API 契约再产出一条 `api/leaf-empty`，只是为了把 4.3 万行第三方代码拉进度量。

政策落点：写进 `mjwarp.runtime.compiler` 描述（zh/en），并把 impact 抽样「`include/warp-headers/mat.h` 仍返回『没有模块对应』」留证为**政策断言**（`logs/impact-vendored-header-mat.txt`，exit=1 属预期）。

## 5. 计数（前后）

| 指标 | before | after |
| --- | --- | --- |
| 跟踪文件（`src`+`include`+`native`） | 822 | 822 |
| 已登记条目（同上三棵树） | 680 | **798**（+118） |
| 已登记不同路径 | 668 | **786**（+118） |
| 未登记 | **154** | **36**（全部 vendored） |
| 模块数 | 66 | **68** |
| 叶子数 | 59 | **61** |

明细：`logs/counts-before.txt`、`logs/counts-after.txt`、`logs/module-counts-after.txt`。

**822 vs P1 记录 823**：在 `f86b8575`（P1 之前的基线）与 `645befa7`（本卡前 HEAD）两个提交点用 `git ls-tree -r --name-only <sha> -- src include native` 现算，均为 **822**；`git diff --name-status f86b8575..HEAD -- src include native` 只有两个**修改**文件（`src/render/kernel/megakernel.rs`、`megakernel_arrays.rs`），无增删。因此差异来自 P1 当时的计数口径（多计了 1 条路径），不是文件增减；本卡以现算 822 为准，未登记 154 与 `178-24=154` 自洽。

## 6. validate 结果

- 结构（`normify validate`）：**错误 0**；警告 5 行（`verified` 后终态）：
  - `api/leaf-empty` ×2（`mjwarp.io.render`、`mjwarp.model.render-model`，均为既有）
  - `structure/leaf-too-coarse` 计数 **34 → 37**：+2（两个新叶各持 ≥2 源）+1（`mjwarp.runtime.external` 由 1 源变 3 源而翻越 coarse 口径）
  - `layout/order-incomplete` ×2（既有 io/model 两处；本卡的 physics/runtime 两处 layout 已补 order 与 group，未新增告警）
- 中间态：`in_progress` 期间 `validate` 会额外报 5 条 `change/api-already-removed`（迁移已完成、任务尚未 verified 的正常现象），`done` 后消失（见 `logs/validate-intermediate.txt` 与 `logs/validate-final.txt`）。
- `--fingerprints`：`done` 后 0 错误、无指纹漂移。

## 7. impact 抽样（6 处）

| 路径 | 结果 |
| --- | --- |
| `src/physics/collision/cuda/contact.cu` | `mjwarp.physics.collision`（72 源/5 API） |
| `src/physics/collision/gjk/epa.rs` | `mjwarp.physics.collision` |
| `src/runtime/collision_gpu/product_entry.rs` | `mjwarp.runtime.collision-gpu`（20 源/1 API） |
| `src/physics/sensor.rs` | `mjwarp.physics.core`（121 源/83 API） |
| `src/runtime/presence.rs` | `mjwarp.runtime`（容器 80 源） |
| `include/warp-headers/mat.h` | 仍「没有模块对应」= vendored 政策断言 |

原文见 `logs/impact-*.txt`。

## 8. 零代码改动与门禁

- `git diff 645befa7..` 收口提交只含 `.normify/` 与 `docs/development-evidence/`；`src/**`、`tests/**`、`fixtures/**`、`Cargo.*` 零改动。
- 门禁：fmt / clippy / clippy-cuda / unit（结果写入 `.normify/runs/`）；`cargo test --locked --no-run` **exit=0** 且日志含 `Finished`（`logs/no-run.txt`，按卡面 O2 规范）。

## 9. 未做与遗留

- 不宣称「全部源已登记」或「架构树完整」：vendored 36 个文件按政策**刻意**不入树。
- render 拆分（`mjwarp.render.kernel` 等）另立卡 `2026-10-30-render-module-split`（案 2 分两步），不在本卡范围。
- `structure/leaf-too-coarse` 仍为聚合告警（37 个叶子多文件），属既有结构现状，非本卡引入的缺陷。
