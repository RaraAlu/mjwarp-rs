# W2/A2-1：mesh multicontact 容量 32/32（设备宏/scratch + 宿主镜像/常量 + 边界单测）

- 任务：`2026-11-12-w2-a2-1-mesh-capacity-32`（owner Backend；W2-A2 第 1 片，0 窗、CPU-only、零 GPU）。
- 日期：2026-11-12（空闲段）。实现提交：`5689e0ef`（父 `984cb630`）。
- 依据：用户 2026-11-12 已签「U7/U8 提升为 pass/fail；A2 容量 20/10 → 32/32」（签名级）；Arch
  `docs/development-evidence/plan/u7-u8-promotion/a2-slices.json` A2-1 范围 + `u7-u8-criteria.json`。
- 边界：不改判据/容差/参考/夹具；零 ABI；不宣称 U8 已通过（设备验证在 A2-3 窗）、不宣称
  「mesh 容量缺口已清 / P5 通过 / 与上游一致（over-32）」。

## 1. 一句话结论

mesh multicontact 固定容量已由 **20/10 提升到 32/32**：设备宏与 `contact.cu` scratch、宿主镜像
`clip.rs`、宿主常量 `types.rs`、容量门措辞全部同步；新增 33 顶点容量前置检查（确定性空结果、不 panic）。
A2 边界 CPU 单测 + A1 既有 20 顶点测试全部通过；四门禁通过（unit **1409 过 / 0 败**）；**零 ABI 改动**。

## 2. 改动清单（12 文件）

| # | 文件:行 | 改前 | 改后 |
| --- | --- | --- | --- |
| 1 | `src/physics/collision/cuda/multicontact_geom.cu:381-385` | `G5_MC_MESH_DEG_MAX 10` / `FACE_MAX 20` | **32 / 32**（A2 注释：U7/U8 已提升，over-32 为限制） |
| 2 | `src/physics/collision/cuda/contact.cu:216-228` | `polygon/clipped[40]`、`pnormal/pdist[20]`、`idx1/idx2/n1/n2/endvert[10]`、`face1/face2[20]` | `polygon/clipped[64]`、`pnormal/pdist[32]`、`idx1/idx2/n1/n2/endvert[32]`、`face1/face2[32]` |
| 3 | `src/physics/collision/cuda/multicontact.cu:13-15` | 调用方容量注释 40/20/10 | 64/32/32（A2） |
| 4 | `src/physics/collision/cuda/multicontact_clip.cu:10-12` | 容量契约注释 20/40 | 32/64（A2） |
| 5 | `src/physics/collision/cuda/multicontact_mesh.cu:10-16` | 容量注释 10/20；guard 保留 | 32/32；guard（L120/L148/L180）**保留**为 defense-in-depth |
| 6 | `src/physics/collision/gjk/multicontact/clip.rs` | `pn/pd[20]`、`vec![..;40]` | `pn/pd[32]`、`vec![..;64]`；**新增** `nface1 > 32 || nface2 > 32 → 空结果`（不 panic） |
| 7 | `src/physics/collision/gjk/multicontact/mod.rs` | A1 两条 20 顶点测试 | 新增 3 条（32 face1 / 32 face2 / 33 拒绝），A1 两条保留 |
| 8 | `src/runtime/collision_gpu/types.rs:139-143` | `FACE_MAX=20` / `DEGREE_MAX=10` | **32 / 32**（`host_limits_match_device_macros` 守护） |
| 9 | `src/runtime/collision_gpu/error.rs:26-29` | 「超过设备固定容量」 | 加「A2 起 32/32」口径 |
| 10 | `src/runtime/collision_gpu/inputs_from_snapshot.rs` | 测试模块内联（498 行） | 注释 32/32；测试模块迁出（主文件 **233 行**） |
| 11 | `src/runtime/collision_gpu/inputs_from_snapshot_tests.rs`（新） | — | 258 行；边界用例：24/24 接受、32/32 边界接受、face=33/degree=33 拒绝（`limit=32`）、无 sidecar、宏同步 |
| 12 | `.normify/modules/mjwarp/runtime/collision-gpu.md` | 模块源 20 | **21**（登记新测试文件） |

## 3. 测试与门禁

| 项 | 命令 | 结果 |
| --- | --- | --- |
| multicontact clip（A1 2 + A2 3） | `cargo test --locked --lib physics::collision::gjk::multicontact` | **5 passed / 0 failed** |
| 宿主容量门（6 条） | `cargo test --locked --lib runtime::collision_gpu::inputs_from_snapshot` | **6 passed / 0 failed** |
| fmt / clippy / clippy-cuda | `normify gate` | 通过 |
| unit | `normify gate` | **1409 过 / 0 败**（基线 1406 + 新增 3 条 clip 测试） |

A2-1 边界单测（对应 Arch 清单 1–7）：
1. `polygon_clip_accepts_thirty_two_vertex_face1`（`pn/pd[31]` 边界，`ncontact ≤ 4`）；
2. `polygon_clip_accepts_thirty_two_vertex_face2`（64 格中间缓冲）；
3. `polygon_clip_rejects_thirty_three_vertex_face`（face1/face2=33 → 空结果、不 panic）；
4. `product_entry_accepts_face_32_degree_32_boundary`（模型级 32/32 接受）；
5. `product_entry_rejects_face_33` / `product_entry_rejects_degree_33`（各自维度容量错误，Display `limit=32`）；
6. `host_limits_match_device_macros`（沿用，自动核对宏与常量）；
7. A1 `polygon_clip_accepts_twenty_vertex_face1/face2` 保留且 PASS（不回退）。

## 4. 500 行棘轮处置

`inputs_from_snapshot.rs` 改前 **498 行（LF）**，已贴 500 上限；A2 边界用例改写会超限 →
把 `#[cfg(test)] mod tests`（原 :231-498，268 行）整体迁到新文件
`inputs_from_snapshot_tests.rs`，主文件以 `#[path]` 声明；迁后主文件 **233 行**、新文件 **258 行**，均 <500。
其余改动文件行数：`multicontact_geom.cu` 412、`contact.cu` 271、`multicontact.cu` 235、
`multicontact_clip.cu` 257、`multicontact_mesh.cu` 191、`clip.rs` 180、`mod.rs` **408**（329→408，<500）、
`types.rs` 187、`error.rs` 72（见 `logs/line-counts.txt`）。

## 5. 不变面 / ABI

- 81 槽、`CollisionArgs` 504 B、launch 签名、产品 JSON 字段：**零改动**（`abi.rs`/`product.cu` 未改，unit 内
  ABI 测试原样通过）。
- 不新增 `.cu`；`DEVICE_SOURCES` 仍 **28** 段、产品入口仍 6 个（`source.rs` 未改）。
- `fixtures/`、`scripts/`、`Cargo.toml`、`Cargo.lock` 零改动（片内 `unchanged` 机检）。
- 模块源：`mjwarp.runtime.collision-gpu` 20 → 21（新测试文件）；`mjwarp.physics.collision` 源清单不变。

## 6. 边界与非声明

- 本片只做 A2-1：**A2-2（内核身份重采/重冻清单）与 A2-3（M2/M3/S4 验证窗）不在本提交**，由后续卡执行。
- **不宣称**：U8 设备 count=4（须 A2-3 窗实测）、U7 通过、mesh 容量缺口已清、G10@P5 整族通过、
  与上游一致（over-32）、不作位级等价声明、不提供位级等价保证。
- over-32 行为：产品路径由宿主容量门返回确定性 `MeshMulticontactCapacity` 错误；设备直调回退为
  record-only（A2-3 窗口口径），不进 pass/fail 分母。

## 7. 证据文件与 manifest

- `README.md`（本文件）
- `receipt.json`（机器可读：容量/改动/测试/门禁/ABI/模块源/非声明）
- `logs/gate.txt`（四门禁输出）、`logs/targeted-multicontact.log`、`logs/targeted-capacity-gate.log`
- `logs/line-counts.txt`（11 个改动文件 LF 行数）
- `materials/src.diff`、`materials/numstat.txt`、`materials/changed-files.txt`、`materials/commit-stat.txt`
- `manifest.txt`（逐文件 sha256+bytes 自核，不列自身）

## 8. 过程事件（记录，不声称）

- 2026-11-12 01:33 提交窗口内发生 QA 并发提交（我持锁期间），其提交说明一度覆盖我的首提交 `d0e2b236`；
  已本地 reset/重提交为 `5689e0ef`：本提交只含本片 12 个文件，**不含** QA/外部 Codex 的任何文件；
  QA 的复核卡工作树内容未被改动（已留快照，不在本提交）。
