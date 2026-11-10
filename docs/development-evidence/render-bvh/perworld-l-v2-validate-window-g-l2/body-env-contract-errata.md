# G-L2 后置 errata：body 环境契约、无效轮次归因与 guards `body_env` 缺字段

- 记录：2026-11-11（G-L2 收口后 errata 批；任务 `2026-11-11-render-perworld-l-v2-gl2-post-errata`，owner Backend）。
- 性质：**文档补记**。不改本窗任何运行记录/结果/判定；不重跑、不重冻；不改判据/参考/容差。本窗判定维持 **未通过（判据 A FAIL）**。
- 关联记录：`window/body-env.txt`（run3 实际 env）、`window/canonical-env-checklist.txt`（env 自检 + pre-body outline 校验）、`window/window-incidents.txt`（INVALID_ROUND #1）、`window/invalid-rounds/run1-rootcause-false-red-outline-env.txt`（INVALID_ROUND #2）、`window/material-errata.txt`（R5：S-3 签名材料 body 命令缺 env 导出的 backlog）。

## 1. body 环境契约（验证窗运行前置；两条都必须满足）

1. **pinned CUDA bin 前置 PATH**：

   ```text
   $env:PATH = "<repo>\target\toolchains\cuda-12.8.1\bin;" + $env:PATH
   ```

   与 `scripts/validate/render-bvh/run-window.ps1:307-308`、`scripts/g01-fast.ps1:40-41` 同口径。缺失后果：cudarc 无法加载 NVRTC，exe 在**测试体开始前**即 panic（`Owner(Transfer(Backend(NvrtcUnavailable)))`）。
2. **criterion v2.2 outline 环境必须成对导出**：

   ```text
   $env:MJWARP_RENDER_OUTLINE_LIST_DIR = "<repo>\docs\development-evidence\render-bvh\sdf-validate-window\attempts\attempt5\outline"
   $env:MJWARP_REQUIRE_OUTLINE_CRITERION = "1"
   ```

   与 `run-window.ps1:291-292` 同口径。语义（`tests/support/render_bvh/outline.rs` 模块文档 1-16 行、`enabled()` 50-68 行、`diff_depth_with_outline()` 359-391 行）：

   - **两者全缺**：判据不启用，调用方退回 **raw 位级比较**（不报错，属静默放松，不是验证窗语义）；
   - **只设一个**：`panic!` 拒绝（"set both ... or neither"），不静默放宽；
   - **两者都在**：启用冻结 outline 列表——列表外像素必须位级一致，列表内像素允许 `|ULP| <= bound_ulp`（本窗 128），任何违规 **fail-closed panic**，不静默通过。
3. **pre/post body 两次 outline 列表校验**（`run-window.ps1:280-298` 语义）：

   - recipe：sha `cfd1038a…24ee` / 10,099 B / rows 183 / first_pixel 28,969 / exact_max_ulp 73 / sidecar `8a80fb0a…7948`（5,027 B）；
   - aa2：sha `14893fc6…8eab` / 9,509 B / rows 172 / first_pixel 29,448 / exact_max_ulp 91 / sidecar `f23ca697…c303`（4,734 B）；
   - `outline-lists.json` sha `c2dd876d…d79e`（2,808 B）、`criterion_version=render-sdf-criterion-v2.2`、`bound_ulp=128`；
   - 本窗 pre/post 均 OK（README §6.2），canary `violations=0`（`window/run/exe-logs/render_e2e.stdout.log`）。
4. 本窗实际执行值：`window/body-env.txt`（PATH 注入 + 两个 outline env + cuda bin/清单指纹）；自检清单 `window/canonical-env-checklist.txt`（`ENV_SELFCHECK_PASS failures=0`）。
5. 12 个冻结 exe 的环境需求（同自检清单 §5）：6 个 scene-a 回归需 PATH + outline 两项；`render_per_world` / `render_context_lifecycle` / `render_mesh_bvh` / `render_hfield_reference` / `render_shadow_reference` / `render_per_world_light` 只需 PATH；12 个 exe 不读取其它 `MJWARP_*` env。

## 2. 两次 INVALID_ROUND 的归因（与 §1 契约逐条对应）

| 轮次 | 现象 | 根因 | 结论/处置 |
| --- | --- | --- | --- |
| INVALID_ROUND #1（run-0，04:16:39） | 第 1 个 exe `render_per_world` exit 101、0.209 s，测试体未开始 | 直接启动冻结 exe 未注入 pinned CUDA bin → `NvrtcUnavailable` | 执行面环境缺失；0 轮消耗；`window/window-incidents.txt` |
| INVALID_ROUND #2（run-1，04:20:16/51） | `render_e2e` cam1 报 183/73/28969，与冻结 outline 列表逐项一致 → 假红 | 漏两个 criterion v2.2 outline env → 判据未启用、回落 raw 比较 | 非产品回归；0 轮消耗；`window/invalid-rounds/run1-rootcause-false-red-outline-env.txt` |

修正后 run3 为 canonical 轮：11/11 回归 exe PASS、实体 `render_per_world_light` 判据 A FAIL（world0 cam1 2 px/1 byte）；本窗判定不变。

## 3. guards `body_env` 缺字段（backlog；本批不改代码/冻结器）

- 已核 `scripts/validate/render-bvh/guards-perworld-l2-light.json` 顶层键**无** `body_env`（keys：schema / suite / task_id / generated_at / repo / attempt / criterion_version / bound_ulp / counts / aggregate_algorithm / aggregate_sha256 / frozen_exe_stash_dir / entity_order / frozen_exes / guard_inputs / outline_lists / old_suite / old_stash_drift / optional_inputs）。
- 冻结器（`scripts/validate/render-bvh/freeze-guards-perworld-l2-light.ps1`）也不记录 body 环境；验证窗所需 PATH 前缀与 outline 成对 env 目前只存在于窗口证据（`window/body-env.txt`、`window/canonical-env-checklist.txt`）。
- 风险：guards 自描述不足以复现 body 环境；将来重跑/复核须回到窗口证据人工复刻环境。
- 处置：登记 **backlog**（冻结器/guards schema 增加 `body_env` 字段：pinned CUDA bin PATH 前缀、两个 outline env 及取值、pre-body outline 校验结果/指纹、12 exe 逐项 env 需求）；本批不改冻结器、不改 guards JSON（冻结产物 sha `fad2ed37…` / 43,729 B 不动）。

## 4. 签名材料相关（只记 backlog，不改正文）

- S-3 签名材料 `materials/command.txt` §5 body 命令骨架缺 env 导出（PATH + 两个 outline env + pre-body 校验），已在 `window/material-errata.txt` 记录（R5）。修改签名材料属改签事项，**本批不动**，留 Chief 裁决。
- 本窗记录的实际 env 以 `window/body-env.txt` 为准。

## 5. 边界

- 只新增/修改 G-L2 证据目录内文档与 `manifest.txt`；`window/**` 运行记录字节不动；不改 guards/冻结件；不改 `.cu`/`.rs`/`tests/`/`fixtures/`/`scripts/`；不改已签 spec 与 S-2/S-3 材料正文。
- 本文件已加入本目录 `manifest.txt`；本目录 README 追加 §11 指针。
