# R02 天空盒家族整片汇总（skybox：离线内核 → authoring → 验证窗 attempt=2 → 签字产物入库）

家族：R02@P8 天空盒（pinned 上游 `mujoco_warp 3.15.0` `render.py` L494-556 / L1259-1268 / L1017-1031）
状态：**六片全部 verified**；QA 复核 S1 pass/0/8、S1b pass/0/6、S3 needs-fix（仅 O1 措辞，已修）/0 阻断/7 观察。
本汇总只聚合已在各片证据中的数字；不新增判据、不重跑窗口。

## 1. 卡片清单（逐张）

| slice | task id | 状态 | 证据 | QA verdict |
| --- | --- | --- | --- | --- |
| R02-S1 离线内核/注册表 | `2026-10-29-render-bvh-r02-skybox-s1` | verified | `render-bvh/r02-skybox-s1/` | pass/0/8（O1 已修 58dd417f/6319500e） |
| R02-S1b 负控修正 | `2026-10-29-render-bvh-r02-skybox-s1b-negative-controls` | verified | `render-bvh/r02-skybox-s1b-negative-controls/` | pass/0/6（6517615c/9705eb21） |
| R02-S2 authoring 材料 | `2026-10-29-render-bvh-r02-skybox-s2-capture-authoring` | verified（含窗口期 fix `b3a47aca`） | `render-bvh/r02-skybox-s2-capture-authoring/` | —（材料片，S3 审计覆盖） |
| R02-S2b 参考测试实体 | `2026-10-29-render-bvh-r02-skybox-s2b-reference-test` | verified | `render-bvh/r02-skybox-s2b-reference-test/` | —（实体在 S3 窗内执行） |
| R02-S3 验证窗 attempt=2 | `2026-10-29-render-bvh-r02-skybox-s3-validate` | verified | `render-bvh/r02-skybox-s3-validate/` | 需修 1（O1 措辞）/0 阻断/7 观察（cecbe729/a61dcd4c；O1 已修 82a025c2） |
| R02-S3a 签字产物入库 | `2026-10-29-render-bvh-r02-skybox-s3-artifacts-commit` | verified | `render-bvh/r02-skybox-s3-artifacts-commit/` | —（S3 证据引用） |

QA 复核卡：`…-s1b-negative-controls-check`（verified）、`…-s3-validate-check`（verified）。
roadmap 卡 `2026-10-11-roadmap-r02-p8` 保持 `proposed`（roadmap 跟踪占位，不自动关闭，与 R05 处置一致）。

## 2. 交付数字（按片）

**R02-S1（离线实现，无窗口）**
- 内核：`sample_skybox`（render_shade.cu）+ `geom_id==-1` 背景分支（`MK_FLAG_SKYBOX` bit9）；非天空盒路径逐字节不变（静态论证）
- 内核基线：source `72bc364c…` → **`e68e3412…`**；ptx `97467d83…` → **`0b947492…`**；ptx_bytes 264 015 → **268 275**
- 纹理注册表子集：`use_textures || skybox`，非请求项 1×1 占位（下标=模型纹理 id）；模块源 **39→40**
- 行数：render_shade.cu 383→457、render_megakernel.cu 347→368、megakernel.rs 412→410、arrays 326→332、uploads 446→465、lifecycle 496→498、texture_inputs 157、tests/render_skybox 260；红线零改动
- unit **1103 / 0 / 389**；提交 `8aa33e00`/`b6b2a679`/`299459c0`；QA O1 修正 `58dd417f`/`6319500e`

**R02-S1b（仅测试）**
- 3 个 stale 负控更新：`render_e2e.rs` 500→**496**（净 −4，红线 ≤500）、flags 495、rgb 309；动态射线/splat 负控保留
- unit **1104 / 0 / 390**；提交 `f2ee32d0`/`1d3c7159`/`df24283c`

**R02-S2（authoring 材料 + 窗口期 fix）**
- `capture_skybox.py` sha 链：`812a2f77…`（S2 原）→ `d17d2418…`（确定性自检修复）→ **`d6f9864c…`**/16 495 B/428 LF（`use_fast_math` False 对齐产品 recipe）；params28 28 项对照（唯一改动 use_fast_math）
- `freeze-guards-r02.ps1` `a8a3f71c…`/8 578 B；`run-window.ps1` R02 修订 `b1ff24f8…`/18 279 B（仅 `-GuardsFile`，默认不变）
- 提交 `3e3665f0`/`f714af77`/`b9e38461` + fix `b3a47aca`

**R02-S2b（测试实体）**
- `tests/render_skybox_reference.rs`（138 LF）+ `tests/support/render_bvh/skybox.rs`（479 LF）；实体 `render_skybox_reference`；unit 1104
- 提交 `9ea9fa59`/`7d873a64`/`bd9bc386`

**R02-S3（验证窗 attempt=2，validation-only 例外 util<60）**
- **9/9 实体 exit 0**；`sha_guards=77/77`（9 exe + 68 输入）；`post_window_exe_recheck=9/9`；`zero_rust_compile=True`（deps 1839→1839，max write 18:12:48）
- samples 166、util 18–40、free_min **12 587 MiB**、baseline_free 12 858、**floor 9 786 未破**、used 增量 271 ≤3 072、residual 0、body_wall 48.7 s
- 天空盒 `render_skybox_reference`：cam0（76 800）/cam1（43 200）RGB **mismatched=0 / max_byte_diff=0 / beyond_1_lsb=0**、depth **mismatch=0 / max_ulp=0 / beyond_ulp=0**、seg **0**、depth_zero 16 995/27 498；两跑 sha `92ac188d…`/`de732643…`/`065caf08…`
- 8 回归实体全部 exit 0、零漂移
- 冻结 attempt=2：guards `2d55c0b3…`/28 931 B、77(9+68)、aggregate `ca1fcaa5…`；`-GuardsOnly` 77/77
- 提交 `34117187`/`699e4365`/`d7f705d0` + 修正 `8fecc3e4`/`fa5cfcfd`/`82a025c2`

**R02-S3a（签字产物入库）**
- 参考 4 文件：capture `47f704de…`/4 730 B、rgb `cf65a570…`/1 440 478 B、depth `28c1ea4b…`/480 486 B、seg `2153fa72…`/960 478 B
- guards attempt=2 committed **blob == 现盘 28 931 B**（本地 `.git/info/attributes -text`）
- 提交 `f0213748`/`99b88d1f`/`65022c3a`/`8a967468`

## 3. 例外、无效轮与修复链（历史，不可表述为常规轮）

- **attempt1 = INVALID_ROUND（exit 10）**：4 实体失败（`render_skybox_reference`、`render_e2e`、`render_kernel_flags`、`render_kernel_rgb`），两类根因：
  - A. 3 个 ignored GPU 测试仍断言「`render_skybox` 必须写前拒绝」（S1 scope 缺口）→ 修复 R02-S1b；
  - B. 参考采集用 `use_fast_math=true` 而产品 recipe 恒 `false` → depth 1–4 ULP → 修复 capture 脚本（`d6f9864c…`）。
  - 另：stage A attempt1 因脚本确定性自检实现 bug 中止（3 npz 已写、capture.json 缺失）→ 修复 `d17d2418…`。
- **attempt2 例外**：常规 `util<40` 在 12:00 时段失败（354 样本/longest 18/75 个 ≥40/exit 20）→ 用户签署 `validation_only_exception_util_lt_60`；attempt2 例外 monitor **60/60**、util 32–41、**12 个 util≥40 样本全 clean 且 streak 到 60**、free_min 12 751；窗口 `-OpenUtilLimit 60` 逐字留痕；`not_a_normal_threshold_pass=true`、`authoring_exception_used=false`；precheck 不作主证。
- 旧参考 4 文件、attempt1 guards、attempt1 原始日志与例外 CSV 均保留（路径/sha 在各片证据）。

## 4. 遗留与已知限制

- **下一家族待用户拍板**：R07 高斯泼溅（需先解「三处 splat 槽复用 → 编译期常量注入」架构选择）与 R06 柔性体（需 flex dumper/probe 工具片）；预核与推荐顺序 R02→R07→R06 已报。
- `run-window.ps1` 的 `guard_aggregate.suite` 字段仍写死 R05 字符串（QA B-O2，纯外观，哈希正确）。
- `-GuardsOnly` 干跑输出未归档到证据（窗口 `guard.log` 77/77 为证）。
- guards 签字字节依赖本地 `.git/info/attributes -text`（R05 QA O7 同类；fresh clone 复现需重新归一）。
- `src/render/kernel/` 多个 `.rs/.cu` 仍未登记模块源（backlog `2026-10-28-arch-volume-module-sources`）。
- 未覆盖面：Linux/T4（按用户 2026-10-24 裁决暂停）；R02 只覆盖 scene-a 单世界两相机；splat/flex/SDF 仍不支持。

## 5. 非声明

本汇总不写窗口级「通过」结论；例外不是常规阈值轮；attempt1 为非法轮；不写精度、等价或验收类断言；判据/容差/参考未因本家族放宽（唯一参考变更是 capture 配方对齐产品 `fast_math=false`，见 R02-S2 修复与 attempt2 数据）。
