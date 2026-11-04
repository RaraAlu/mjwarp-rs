# R05/R09/R10 家族收口（高度场地形 + 光照与阴影 + 多世界）

任务：`2026-11-04-render-r05-r09-r10-family-closeout`（owner Arch）。日期：2026-11-04（空闲段）。
**边界：家族证据汇总 + 记账，零 GPU、零 `src/`/`tests/`/`fixtures/`、不改判据/容差/参考、不回改已 verified 证据、不替其他家族背书。**
三族历史证据含**窗口例外与失败轮**，本卡只归纳，不重判、不抹除。

## 1. 一句话结论

R05（高度场地形）、R09（光照与阴影）、R10（多世界）三族全部 verified，且各有独立 QA 复核。
判据维持各族已签口径；**例外轮与 INVALID_ROUND 原样保留，一律不当作常规阈值通过**。

## 2. R05 高度场地形——逐片证据

| 片 / 卡 | 内容 | 提交 | 证据 |
| --- | --- | --- | --- |
| R05-S1 `2026-10-28-render-bvh-r05-s1-hfield-params` | 主机参数校验/解析 + `VolumeModelInput` 薄适配器（含 hardening 与 channel 两片） | `c2a098aa`、`6f9f82e8`、`9a6fd66e` 等 | `……/r05-s1-hfield-params/` |
| R05-S2 `2026-10-28-render-bvh-r05-s2-hfield-intersect`（r2） | 两次零行为拆分 → 三角化移植 + 场景/网格 BVH + 设备派发；f32 阈值修 `1b876391` | `5d4d23d0`、`72a7de66`、`0656bd6e`、`1b876391` | `……/r05-s2-hfield-intersect/` |
| R05-S3a `2026-10-28-render-bvh-r05-s3a-hfield-capture` | 采集 authoring（转置可变夹具）+ 采集窗口 | `18c17d3a`、`e5f498d4`、`8741e8e4` | `……/r05-s3a-hfield-capture/` |
| R05-S3b `2026-10-28-render-bvh-r05-s3b-hfield-validate` | 验证窗 **attempt=2**：9/9 实体 exit 0、sha_guards 73/73、post 9/9、零编译、floor 10,233 未破、samples 171、两组 hfield 全零偏差、两跑 sha 相同 | `2c56cc1a` 等（见 plan rows） | `……/r05-s3b-hfield-validate/` |
| 产品修复 | 零 mesh 家族槽占位 `dce78b1f`；主机相机位姿 FMA 收缩序 `6f246b46` | — | 同上 |

**例外留痕**：S3a 使用用户预批的 **authoring-only util<60** 例外；S3b 使用 Chief 预批的 **validation-only util<60** 例外；**attempt1 INVALID_ROUND 原样保留**。均不写成常规阈值通过。

## 3. R09 光照与阴影——逐片证据

| 片 / 卡 | 内容 | 提交 | 证据 |
| --- | --- | --- | --- |
| R09-S1 `2026-10-28-render-bvh-r09-s1-lighting-params` | 主机逐光源参数校验（QA `df804751`/`354c7808`） | `ef557f6e` 等 | `……/r09-s1-lighting-params/` |
| R09-S2 `2026-10-28-render-bvh-r09-s2-lighting-kernel` | 设备光照 + 阴影分支（复用 first-hit `cast_ray`，路径 A）；**当时主机仍拒绝 `use_shadows=true`**（披露未抹） | `6b5d6dc6` | `……/r09-s2-lighting-kernel/` |
| R09-S2b `2026-10-28-render-bvh-r09-s2-host-guard-unlock` | 主机守卫解锁：移除前置拒绝、两条 `#[ignore]` 断言翻转、槽 40 契约单测 | 见卡 | `……/r09-s2b-host-guard-unlock/` |
| R09-S3a `2026-10-28-render-bvh-r09-s3a-capture-exact` | 精确采集（阴影/二次射线参考） | 见卡 | `……/r09-s3a-*` |
| R09-S3b `2026-10-28-render-bvh-r09-s3b-shadow-validation` | 阴影验证窗 + 二次射线修复 + 证据勘误 `…-s3b-evidence-erratum` | 见卡 | `……/r09-s3b-*` |

判据（不重议）：按 R09-S3b 当次窗口材料（RGB/depth/seg 同一族口径、两跑 sha 相同）。
**未覆盖**：阴影质量/性能不作声明；更高 spp/Linux/T4 未覆盖。

## 4. R10 多世界——逐片证据

| 片 / 卡 | 内容 | 提交 | 证据 |
| --- | --- | --- | --- |
| R10-S1 `2026-10-28-render-bvh-r10-s1-world-state` | 逐 world 状态/布局；QA 抓出 3 个 panic 并修；后续 `world/` 四文件拆分 | `4cf1159c`、`0398c23c`、`ac1787ab` 等 | `……/r10-s1-world-state/` |
| R10-S2 `2026-10-28-render-bvh-r10-s2-multi-world` | nworld=2 端到端窗 **attempt2-run**：8/8 实体 exit 0、guards 46/46、post 8/8、零 Rust 重编（deps 1823→1823）、floor 9,708 未破、两跑 sha 相同；QA `a3ce62e1`/`5cc6a3d5`（0 阻断、4 观察） | `504ded3c`、`40e9238f`、`48ea01a2` | `……/r10-s2-multi-world-e2e/` |
| L-S1 `2026-10-30-render-perworld-impl-s1` | 逐 world 场景位姿（后续补充，verified） | 见 plan | `……/perworld-impl-s1/` |

**限界（写死）**：**上游 nworld>1 无数值参考** → R10 只做内部一致性与 world 隔离，不作上游对照；per-world 灯位姿、AA+perworld、上游式单发射留在 backlog `2026-11-03-render-perworld-l-v2-k-backlog`（只跟踪、不执行）。

## 5. 已知限制（逐条）

1. R05/R09 窗口使用 **authoring-only / validation-only util<60 例外**（各自预批），非常规阈值通过；失败轮与 INVALID_ROUND 保留。
2. R09 产品阴影路径在 **R09-S2b 之后**才可用；解锁前的拒绝契约保留在历史证据中。
3. R10 **无上游 nworld>1 数值参考**；per-world 灯位姿 / AA+perworld / 单发射未做（backlog 跟踪）。
4. 更高 spp、Linux/T4、与上游完全一致均未覆盖；三族均无性能声明。

## 6. 非声明

- 不声明三族之外的渲染能力；
- 不声明与上游完全一致；
- 不声明对外发布依据；
- 不替其他家族（R02/R03/R04/R06/R07）背书。

## 7. plan.json 记账

- `delivery_tracking.rows` 追加 **R05 / R09 / R10** 三行；
- `scheduling_notes` 追加 **N18**；
- `roadmap-r05-p8` / `roadmap-r09-p8` / `roadmap-r10-p8` **不 start/done**（roadmap 节点不承载实现，C 全绿后统一收口）。
