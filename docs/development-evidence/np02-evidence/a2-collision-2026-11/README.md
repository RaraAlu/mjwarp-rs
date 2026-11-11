# NP02 collision 回执重采（A2 容量 32/32 后；CPU-only，零 GPU）

- 任务：`2026-11-12-w2-a2-2-identity-refreeze`（A2-2；归 NP5-03/NP02 证据面）。
- 触发：A2-1 把 collision kernel 容量 20/10 → 32/32（`5689e0ef`），设备源字节变化 ⇒ NP02 `mjwarp_collision_*`
  6 行原回执（PTX `54ab59e3…`）身份失效。
- 工具：`scripts/np02-evidence/np02-capture-new-kernels.py capture --only collision`（NVRTC 12.8.93、
  **不创建 CUDA 设备上下文**），物理配方 `--fmad=true --use_fast_math=false --std=c++17
  --gpu-architecture=compute_70`；`index` 幂等追加补采索引。
- 采集时间：2026-11-12（0 窗）。

## 1. 结果（6/6 门禁 pass）

| 项 | 值 |
| --- | --- |
| 编译单元 | 28 片段装配源（`DEVICE_SOURCES` 顺序，剥本地 `.cu` include）+ `product.cu` |
| source sha256 | `fb03c5b10956685408ef720769e69f14560db472aa3516c98b30e59dc8956d81` |
| source bytes | **254,434**（A1 为 254,263） |
| PTX sha256 | `78da9deafb9c8b787120d1cd1ca7d2c560271d2e67b39b45f21852cc6d1ef59d` |
| PTX bytes | **3,883,643**（A1 为 3,883,769） |
| 入口 | 6（boxbox/contact/contact_multi/hfield/primitive/sdf 同一 PTX 单元） |
| 精度门禁 | 6/6 `f32-physics` pass |

回执文件命名 `<entry>-<ptxSha>.<cu|ptx|json>`（本目录 `receipts/`），共 6×3 个实体文件。

## 2. 与旧回执的关系（历史不改写）

- 旧回执（A1 身份 `54ab59e3…`）仍在 `np02-evidence/new-kernels-2026-11/receipts/`，**保留为历史**、不删除不改写。
- 本目录是 A2 身份的**新增补采波次**；`np02-evidence/receipt.json` 补采索引追加 6 条（索引总计 25 → **33**），
  逐条 (entry, ptxSha) 幂等。
- `inventory.json` 6 行 collision 的 `artifact.ptx_sha256` 与 `artifact.evidence_paths` **改指新回执**
  （identity-only swap）：`counts.receipts_total=715` / `receipt_ptx_entities_verified=711` 等计数**不变**，
  非 collision 行零改动；inventory 全量重算与计数口径更新归 NP5-03 面（本片不重算，原因见 A2-2 evidence README §4）。
- 旧→新映射：`docs/development-evidence/g10-p5-mesh-cap32-identity/np02-identity-mapping.json`。

## 3. 边界与非声明

- 不改任何产品源码/判据/容差/参考；本波次只重采回执身份。
- 不宣称 U7/U8 通过、mesh 容量缺口已清、与上游等价（over-32）、NP01/NP02 门禁通过。
- 不建 CUDA 上下文、零 GPU、零窗口。
