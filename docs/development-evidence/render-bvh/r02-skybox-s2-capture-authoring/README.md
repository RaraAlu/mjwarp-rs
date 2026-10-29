# R02-S2：天空盒 authoring capture 变体 + 窗口计划（离线，不跑 GPU）

任务：`2026-10-29-render-bvh-r02-skybox-s2-capture-authoring`（P8-渲染与BVH，owner Backend，Arch 登记）
本片性质：**离线 authoring 材料片**。不开 GPU、不跑 capture、不冻结、不跑窗口、不动 `guards.json`；
产物是「可给用户双签的窗口材料」+ 参数化 capture 脚本 + 新 suite 的 guard 冻结器与 runner 参数。

## 1. 交付物

| 交付物 | 路径 | 说明 |
| --- | --- | --- |
| capture 变体 | `scripts/authoring/render-bvh-r02/capture_skybox.py` | 只在 S0b 参数上把 `render_skybox=True`、输出到独立目录 `fixtures/render-bvh/scene-a/reference-skybox/`（既有 S0b 参考不重采、保持逐字节不变）；`--dry-run` 纯离线自检（不 import warp/mujoco_warp、不需 GPU） |
| guard 冻结器（新 suite） | `scripts/validate/render-bvh/freeze-guards-r02.ps1` | suite `r02-skybox-validate`、attempt 参数、9 exe + 68 只读输入 = **77 条目**、exe stash；不动 R05 的 `guards.json` |
| runner 参数化 | `scripts/validate/render-bvh/run-window.ps1` | 新增可选 `-GuardsFile`；缺省行为不变（仍读 `<SuiteDir>/guards.json`） |
| 窗口计划（一页） | `materials/window-plan.txt` | 两阶段：A capture（GPU）→ 冻结 → B 验证窗（GPU，9 实体）；基线/floor 公式；monitor 常规 40、无例外；逐字命令；stop conditions |
| 实体表骨架 | `materials/entity-table.json` | 9 实体 = 1 skybox 新目标 + 8 回归；判据沿用既有位级口径 |
| 脚本身份与 ASCII 自检 | `materials/script-identity.txt` | 三份脚本 sha256/bytes/lines + ASCII 三项（nonascii=0 / cr=0 / bom=false）+ PS 语法解析 0 错 |

## 2. 关键决定

1. **不重采 S0b 参考**：新脚本独立输出 `reference-skybox/`，S0b reference 与 provenance 不动；
   参数仅 `render_skybox` 一项差异，`PARAMS_28` 其余保持 S0b 原值（`splat_*` 全 None、nworld=1、spp=1）。
2. **新 suite 而非改 R05 签字物**：`guards-r02-skybox.json`（suite `r02-skybox-validate`，attempt=1）；
   R05 `guards.json`（suite `r05-s3b-hfield-validate`、`e2444a96…`、attempt=2）按历史记录保持不动、
   在新 suite 里标 superseded。理由：S1 重建已使 R05 的 9 个 exe live sha 全变（Chief 核出 0/9），
   旧 guards 不能也不该被恢复。
3. **冻结器自带总量断言**：9 exe + 68 输入 = 77（68 = 8 hfield capture + 2 hfield xml + 42 legacy
   [38 + 4 天空盒参考] + 16 scene-shadow）；`reference-skybox/` 必须恰好 4 文件，否则抛错 —— 防止
   stage A 缺产物就冻结。
4. **monitor 先常规**：`monitor-until-clean.ps1 -Required 60 -MaxMinutes 30`（默认 `-MaxUtil 40`），
   **不预批任何例外**；常规不达标 → 停手报 Chief。窗口命令不带 `-OpenUtilLimit`。
5. **判据零改动**：RGB `mismatched=0/max_byte_diff=0/beyond_1_lsb=0`、depth `mismatched=0/max_ulp=0/
   beyond_ulp=0`、seg `mismatched=0`、两跑 sha 相同 —— 与 R05 完全一致，未新增档位、未替换参考。
6. **8 回归实体覆盖零泄漏**：`render_camera_rays / context_lifecycle / e2e / kernel_depth_seg /
   kernel_flags / kernel_rgb / r08_cameras / r10_multi_world` 在本片改动后的内核上重跑，任何设备侧
   回归都会红。

## 3. 前置依赖（窗口前必须完成，本片不解除）

* `2026-10-29-render-bvh-r02-skybox-s2b-reference-test`（proposed）：新增 ignored GPU 测试目标
  `tests/render_skybox_reference.rs`（实体 `render_skybox_reference`），对 `reference-skybox/` 做
  位级比较 —— **没有它冻结器取不到第 9 个 exe，窗口不能开**。
* stage A capture 产物（4 文件）必须先生成，冻结器才能在 68 输入里包含它们。

## 4. 未执行 / 非声明

* 本片**零 GPU 执行**：未跑 capture、未开窗口、未生成任何参考产物、未做任何渲染数值结论。
* 未改判据/容差/参考/fixture；`fixtures/` 仅将在 stage A 由脚本写入（本片不写）。
* `-GuardsOnly` 未运行（R05 旧 guards 已历史化，跑它按预期会失败，不属漂移）。

## 5. 窗口期修复（2026-10-29，Normify-Fixes；本片后补）

R02 窗口 attempt1 暴露 capture 脚本两处问题，已修复并重采（只改脚本，判据/容差/输出格式/命令不变）：

1. **确定性自检实现 bug**（stage A attempt1 失败根因）：原实现把「内存中拼接 buffer 的 sha」与
   「写盘 `.npz` zip 的 sha」比较，两种编码不可能相等；现改为**同一 `write_npz` 写两跑到临时目录后
   比较实际文件 sha256**，相等才清空参考目录、拷入 run1 并把三 sha 写进 `capture.json`。
2. **`PARAMS_28.use_fast_math` True→False**（attempt1 天空盒 depth 1–4 ULP 根因）：产品内核配方
   `WARP_1_15_RENDER` 恒为 `fast_math=false`（`src/runtime/launcher/recipe.rs`），参考必须同配方。
   其余 27 个参数逐项对照产品解析值**未改**，对照表见 `materials/params28-audit.txt`。

* 新脚本身份：`d6f9864cc29fb452e1914f135fe10d55c65751a89ad9a84244cfa362fabcc16d` / **16 495 B** / **428 LF**
  （ASCII=0/CR=0）；旧 `812a2f77…`/16 084/425 在 `materials/script-identity.txt` 标 **superseded**。
* 新 `params28_sha256 = e2bae1991f39e8a8ad995a2ee651cd1dd307b07b617eb1a7ad4b20f50aa9c3cb`（dry-run 打印）。
* stage A attempt1 的失败留档：`target/render-bvh/r02-skybox-window/stage-a-attempt1-failure.txt`
  + `capture-skybox-fix.diff`（原错误、旧/新 sha/bytes/lines、3 个部分产物 sha 与 `capture.json` 缺失）。
* 重采后 `capture.json` 必须 `use_fast_math=false`、两跑三 sha 相等、恰 4 文件（attempt2 的实测数字
  记在窗口证据目录，不在本片）。
