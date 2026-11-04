# R02 输出族收口（RGB 输出 + 天空盒）

任务：`2026-11-04-render-r02-rgb-closeout`（owner Arch）。日期：2026-11-04（空闲段）。
**边界：家族证据汇总 + 记账，零 GPU、零 `src/`/`tests/`/`fixtures/`、不改判据/容差/参考、不回改已 verified 证据、不替其他家族背书。**
天空盒族已有 `r02-skybox-summary/` 与收口卡 `2026-10-29-render-bvh-r02-skybox-closeout`，本卡**引用不复述**其数字。

## 1. 一句话结论

R02@P8（RGB 输出 + 天空盒背景通道）交付面全部 verified：RGB 由 **S7b2 / S7b3 / S7b-followups / S8 / R08-S3** 覆盖，
天空盒由 **R02-S1 / S1b / S2 / S2b / S3（attempt=2）/ S3a** 覆盖。判据不改，本卡只做索引与记账。

## 2. RGB 段——逐片证据

| 片 / 卡 | 内容 | 提交 | 证据 |
| --- | --- | --- | --- |
| S7b2 `2026-10-25-render-bvh-s7b2-shading` | 着色全链（headlight/背景/基色/纹理/法线/自发光/镜面/背面剔除），打包 u32 ABGR；**120,000/120,000 像素零差** | `f228099` | `……/s7b2-shading/` |
| S7b2 QA | 独立重采同配方参考：RGB beyond_1_lsb=0、depth 0 ULP、光源位姿四路一致、`mat_shin_exp` 修正被独立确认 | `38298e5` | `……/s7b2-independent-check/` |
| S7b3 `2026-10-25-render-bvh-s7b3-flags` | 5 个 flag 变体 + AA（spp=2）各自零差；判别力实测 | `52c22b7` | `……/s7b3-flags/` |
| S7b3 QA | 自采 7 份参考，逐变体零差；AA 语义反解 | `7e5affd` | `……/s7b3-independent-check/` |
| S7b-followups `2026-10-25-render-bvh-s7b-followups` | 法线开/关的可判别夹具（覆盖既有证据缺口） | `62c58a5`、`33303a7`、`1b2f79c` | `……/s7b-followups/` |
| S8 `2026-10-25-render-bvh-s8-e2e` | 端到端同配方 RGB 零差；反例控制证明输入来自产品 | `d8bc9d8` | `……/s8-e2e/` |
| R08-S3 `2026-10-29-render-r08-rgb-hetero-validate` | 逐相机 RGB 异构（cam0_on/cam1_on/all_off）beyond_1_lsb=0；9/9 实体、guards 77/77；QA `337abc14`、收口 `9b1c4d81` | `c5592d65` | `……/r08-rgb-hetero-validate/` |

判据（不重议）：**RGB ≤1 LSB 且 beyond_1_lsb=0，位级优先**；实测零差；两跑 sha 相同。

**已知偏差（只登记，不作等价声明）**：S0b 快数学（`use_fast_math=true`）夹具与产品配方的跨 recipe 差异（S7b2 记录的 332 px 类）只作跨配方偏差留档。

## 3. 天空盒段——逐片证据（引用既有收口）

| 片 / 卡 | 内容 | 提交 | 证据 |
| --- | --- | --- | --- |
| R02-S1 `2026-10-29-render-bvh-r02-skybox-s1` | 离线内核 + 注册表 | `8aa33e00` 等 | `……/r02-skybox-s1/` |
| R02-S1b `…-s1b-negative-controls` | 陈旧负控制修复 | `f2ee32d0` 等 | `……/r02-skybox-s1b-negative-controls/` |
| R02-S2 `…-s2-capture-authoring` | 采集 authoring + 窗口期脚本修复 | `3e3665f0` 等 | `……/r02-skybox-s2-capture-authoring/` |
| R02-S2b `…-s2b-reference-test` | 参考测试目标 | `9ea9fa59` 等 | `……/r02-skybox-s2b-reference-test/` |
| R02-S3 `…-s3-validate` | 验证窗 **attempt=2** 全绿：9/9 exit 0、sha_guards 77/77、post 9/9、零编译、floor 9,786 未破、samples 166；天空盒 2 相机 RGB/depth/seg **全零偏差**、两跑 sha 相同；attempt1 INVALID_ROUND 原样保留 | `34117187` 等 | `……/r02-skybox-s3-validate/` |
| R02-S3 QA | `cecbe729`/`a61dcd4c`：needs-fix 仅 O1 措辞、0 阻断、7 观察 | — | 同上 |
| R02-S3a `…-s3-artifacts-commit` | 签入冻结参考 4 文件 + guards attempt=2（blob==disk，local `-text` 留痕） | `f0213748` 等 | `……/r02-skybox-s3-artifacts-commit/` |
| 家族收口 | `2026-10-29-render-bvh-r02-skybox-closeout` + `……/r02-skybox-summary/` | — | 既有收口证据 |

判据（不重议）：天空盒 2 相机 RGB/depth/seg 全零偏差；两跑 sha 相同；窗口规则按当次材料。
**例外留痕**：该窗口使用用户预批的 **validation-only util<60 例外**，不作为常规阈值通过；attempt1 失败原样保留。

## 4. 已知限制（逐条）

1. **跨 recipe**：S0b 快数学夹具差异只作偏差记录，不作一致声明。
2. **`vertex_normals_off`**：在 scene-a 无判别力（S7b-followups 用合成夹具补齐）。
3. **AA**：仅 `samples_per_pixel=2` 有图像级参考。
4. **R02 窗口例外**：validation-only util<60（用户预批）+ attempt1 INVALID_ROUND 保留。
5. **未覆盖**：更高 spp、Linux/T4、上游完全一致；无性能声明。

## 5. 非声明

- 不声明完整 R02 之外的渲染能力；
- 不声明与上游完全一致；
- 不声明对外发布依据；
- 不替其他家族（R03/R04/R05/R06/R07/R09/R10）背书。

## 6. plan.json 记账

- `delivery_tracking.rows` 追加 **R02** 与 **R02-skybox** 两行；
- `scheduling_notes` 追加 **N17**；
- `roadmap-r02-p8` **不 start/done**（roadmap 节点不承载实现，C 全绿后统一收口）。
