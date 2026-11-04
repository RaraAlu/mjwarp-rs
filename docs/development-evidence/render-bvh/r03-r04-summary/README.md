# R03/R04 输出族收口（深度输出 + 分割输出）

任务：`2026-11-04-render-r03-r04-output-closeout`（owner Arch）。日期：2026-11-04（空闲段）。
**边界：家族证据汇总 + 记账，零 GPU、零 `src/`/`tests/`/`fixtures/`、不改判据/容差/参考、不回改已 verified 证据、不替其他家族背书。**

## 1. 一句话结论

R03@P8（深度输出）与 R04@P8（分割输出）的交付面在 **S7b1 + S8** 已 verified，并由 **R08-S1**（逐相机视图/clamp 顺序）与 **R10-S2**（多 world 输出分区）扩展；两族判据维持**不改**。
本卡只做家族索引与记账，**不新增判定、不重复声明其他家族**。

## 2. R03 深度输出——逐片证据

| 片 / 卡 | 内容 | 提交 | 证据 |
| --- | --- | --- | --- |
| S7b1 `2026-10-25-render-bvh-s7b1-kernel` | 设备深度路径：120,000/120,000 像素**位级相等**、max **0 ULP**；动态==预计算；无命中 0 | `3e43278` | `……/s7b1-depth-seg/` |
| S7b1 QA `…-s7b1-independent-check` | 独立解码复算 120,000/120,000；新位姿泛化；算术序归因（非夹具拟合） | `51ee2d0`、`7df5ad6` | `……/s7b1-independent-check/` |
| S8 `2026-10-25-render-bvh-s8-e2e` | 端到端同配方深度位等 0 ULP；位姿链位级一致 | `d8bc9d8` | `……/s8-e2e/` |
| S8 QA `…-s8-independent-check` | 独立 harness 复算 0 位差；反例控制 | `20c728c` | `……/s8-independent-check/` |
| R08-S1 `2026-10-28-render-bvh-r08-camera-output` | `get_depth` 语义：`clamp(value/depth_scale, 0, 1)` 含除法/夹取顺序，逐相机切片不越界 | `a6425a5d`、`f305842e` | `……/r08-camera-output/` |
| R10-S2 `2026-10-28-render-bvh-r10-s2-multi-world` | nworld=2 下逐 world 输出分区与隔离（8/8 实体 exit 0、guards 46/46；QA `a3ce62e1`/`5cc6a3d5`） | `48ea01a2` | `……/r10-s2-multi-world-e2e/` |

判据（不重议）：**深度 ≤4 ULP 且 beyond4=0，位级优先**；实测位级 0 ULP。无命中返回 0；`get_depth` 缩放口径保持上游顺序。

## 3. R04 分割输出——逐片证据

| 片 / 卡 | 内容 | 提交 | 证据 |
| --- | --- | --- | --- |
| S7b1（同上） | 两相机 seg **0 mismatch**；`(id,type)` 对语义；动态==预计算 | `3e43278` | `……/s7b1-depth-seg/` |
| S7b1 QA（同上） | 独立复算 seg 0 mismatch | `51ee2d0` | `……/s7b1-independent-check/` |
| S8（同上） | 端到端 seg 精确相等 | `d8bc9d8` | `……/s8-e2e/` |
| R08-S1 | 逐相机 seg 视图、禁用通道拒绝、`camera_index` 语义（活动相机序位非模型 id） | `a6425a5d` | `……/r08-camera-output/` |
| R10-S2 | 多 world segmentation 分区 | `504ded3c` | `……/r10-s2-multi-world-e2e/` |

判据（不重议）：**分割精确相等**，背景 `(-1,-1)`。各族自有 seg 编码（hfield / flex `vec2i(flex_id, FLEX)` / skybox）在各自家族证据内，本卡不重复声明。

## 4. 已知限制（逐条）

1. **AA（`nsamples>1`）**：深度与分割只取第一趟（S7b3 记录的语义升级），不是逐样本合成。
2. **更高 spp**：仅覆盖既有样本级参考，更高 spp 未采样。
3. **上游 nworld>1 无数值参考**：多 world 只做内部一致性与隔离，不作上游对照。
4. **无性能声明**：本族不做时延/带宽判据（性能在 `perf/` 独立证据，相对基准）。
5. **家族边界**：skybox/hfield/flex/SDF 的深度/分割属各自家族，本卡不背书。

## 5. 非声明

- 不声明完整 R03/R04 之外的渲染能力；
- 不声明与上游完全一致；
- 不声明对外发布依据；
- 不替其他家族（R02/R05/R06/R07/R09/R10）背书。

## 6. plan.json 记账

- `delivery_tracking.rows` 追加 **R03** 与 **R04** 两行（task_id=本卡、evidence=本目录、note 含关键数字与实现卡）；
- `scheduling_notes` 追加 **N16**（说明 R03/R04 未建独立 slices 的原因：随 S7b1/S8 交付）；
- `roadmap-r03-p8` / `roadmap-r04-p8` **不 start/done**（roadmap 节点不承载实现，C 全绿后统一收口）。
