# QA 复核：诊断轮 #7（产品侧扩展 dump）运行与 dump 合规

- 任务：`2026-11-11-render-perworld-l-v2-gl2-diag7-check`（owner QA；零 GPU、只读复核）。
- 对象：诊断轮 #7 现场 `target/render-bvh/perworld-l-v2-gl2/diag7/`（dump `pixel-dump.json` 55,485 B / sha `8cadf764…`；run-summary/pre-state/body-env/preflight/test7 日志/analysis/diagnostics-decoded）。token 基准 `a3dc5e7b`，复核 HEAD `a7841834`。
- **结论：PASS（0 阻断，3 条低残留 + 1 条观察）。** 单次/无重试、锁与现场记录成立（锁无审计日志，按记录采信）；dump 顶层 `schema==2`、4 px 全部带 18 键 `diagnostics`、偏移/位型/编码与 dump-ext README §2 一致，且我做了三项**绑定校验**（`ray_index = cam*76800+pixel`、`view_dir = -ray_dir` 精确符号位翻转、`trunc(color*255) == product bytes` 12/12）全部通过；C/A 数字与历轮一致、depth/seg 0；与 diag6 的「像素集/packed 字节相同、差异仅新增字段」可逐字段复现。低残留均为现场记录生成瑕疵（`$gpu` 未展开、body-env 路径转义污染、GPU 数字与派工信息不一致），不影响运行与 dump 数据。**不判定修复效果**（4 px 仍在，A 仍 FAIL）。

## 1. ① 单次/无重试、锁纪律、现场记录 ✅（附记录瑕疵）

- **HEAD/基线**：HEAD `a7841834`；`git diff a3dc5e7b..HEAD -- src/ tests/ fixtures/ scripts/` **空**（仅治理/证据前移）；树 clean；diag7 全部文件未入库（`git ls-files target/.../diag7` 空）。
- **单次/无重试**：diag7 只有一对 `test7.stdout/stderr.log`，无 retry/attempt/rerun 文件；时间线单调：pre-state 07:22:07 → dump/logs 07:22:18 → analysis 07:22:27/35 → run-summary 07:22:43；prebuild `--no-run` 0.13s cache hit（无重编译）。
- **运行结果**：实体 exit **101**（预期，A 仍 FAIL）、wall 2.73 s；`gpu_per_world_light_matches_reference` FAILED 于 world0 cam1 断言（1≠0），`pilot_c1_crosscheck_matches_readout` **ok**（1 passed / 1 failed / 18 filtered）。
- **exe/身份**：`render_per_world_light-8f8e60a3…exe` 实测 **92,588,042 B / `5ca3619c…`**、mtime 2026-11-11T07:03:02（修卡后构建——dump schema==2 即证测试侧 `SCHEMA=2` 生效）；内核身份 stitched source **`624ab298…`/109,384 B**（我独立重算）、identity-check.log 记 PTX `1fd53131…`/347,070（离线 NVRTC）——与修复前一致、无 `.cu` 改动。
- **锁**：pre-state/run-summary 记 Backend acquire→release；仓内 `repo-write-lock.ps1` 只有当前状态、**无审计日志** ⇒ 按记录+披露采信（当前 FREE）。
- **记录瑕疵（低）**：`run-summary.txt` 的 `time=…07:2x…` 与 `gpu_after=$gpu` 未展开；`body-env.txt` 路径被 PowerShell 转义污染（2 个 TAB + 1 个退格：`\target`→TAB、“\bin”→退格，`where nvrtc64_120_0.dll` 行路径不可读）；`preflight.txt` §6 标题有 `({ts})` 占位符。运行本身成功（pilot 通过 + dump 生成），属记录生成问题。

## 2. ② dump 顶层 schema / diagnostics 字段 / 偏移编码 ✅

- **dump 身份**：55,485 B / sha `8cadf76492e150da5e193b701d808666b197a3cea4ddde1669616605d0e584d7`（与派工一致）。
- **schema**：顶层 `schema=2`、`kind=g-l2-entity-pixel-diagnostic`、`env=MJWARP_DIAG_PIXEL_DUMP`；`rgb_mismatches=4`（全部带 `diagnostics`）、`depth_mismatches=0`、`seg_mismatches=0`。
- **字段**：每个 diagnostics **18 键**（bary/base_color/color/encoding/face/hit_point/light_slots/lights/mesh_id/normal/ray/ray_dir/stride/t/tex_color/tex_uv/view_dir/view_dir_derivation），`stride=64`、`light_slots=4`、每 px **4 灯槽**；逐字段数量与 dump-ext README §2 偏移表逐项一致（color 3、base 3、tex_uv 2、tex_color 3、bary 2、t 1、face 1、mesh_id 1、hit_point 3、normal 3、ray_dir 3、lights 4×8）。
- **编码抽查**：f32 以 u32 位型（`encoding` 字段自述），我反解得到 color `[1.0, 0.110776544, 0.11372561]` 等合理值；`face=2`、`mesh_id=0`（与 seg `[3,5]`/MESH 命中一致）；`lights` 每槽 weight/visible/ndotl/ndoth/half.xyz/pow 八值齐全。

## 3. ③ ray_index 与像素槽映射自洽 ✅

- 公式核验（per-world 单发射，相机像素 320×240=76,800）：`ray_index = cam*76800 + pixel`，且 `y*320 + x == pixel`：
  - w0c1 px 13268 (148,41) → **90,068** ✅
  - w1c1 px 10744 (184,33) → **87,544** ✅
  - w1c1 px 22903 (183,71) → **99,703** ✅
  - w1c1 px 39492 (132,123) → **116,292** ✅
- 四处 `ray.ray_index` 与 `diagnostics.ray` 相等、`sample=0`；4 px 与派工给的 90068/87544/99703/116292 完全一致。

## 4. ④ A/B/C 数字与历轮一致 ✅

- stdout：`c1_rgb_changed=[38285, 40644]`、`c2_depth_changed=[35828, 33883]`、`c2_seg_changed=[16041, 16754]`、`verdict=PassMargin`、`capture.json C1 交叉核对 = Compared`、`diag campersp depth=[0,0] seg=[0,0]`、`camhinge rgb=[50363, 52329]` —— 与历轮逐数一致。
- A 逐帧（dump frames）：`0/0, 0/1, 0/0, 0/3, 0/0, 0/0`（rgb 失配；max byte diff 1），depth/seg 全 0 —— 与派工一致（4 px：w0c1 1 个、w1c1 3 个）。

## 5. ⑤ dump 与 diag6 一致性声明可复现 ✅

- diag6（`44977 B / 521718e5…`，schema=1、无 diagnostics）vs diag7（55,485 B、schema=2）：
  - 像素集完全相同 `[(0,1,13268),(1,1,10744),(1,1,22903),(1,1,39492)]` ✅；
  - `product.packed_u32` / `reference.packed_u32` 逐项相同 ✅；
  - 除 `diagnostics` 外**全部 mismatch 条目字段逐项相同**（depth/seg/camera/ray/lights/neighborhood_3x3/x/y 等）✅；
  - 顶层除 `schema`/`generated_at_unix`/`rgb_mismatches[].diagnostics` 外无差异 ✅。
- ⇒「差异仅新增字段、44,977→55,485 B」可复现。

## 6. 附加绑定校验（我加做，全部通过 ✅）

1. **color→packed 绑定**：内核 pack 为 `pack_rgba_to_uint32(hit_color.x*255,…)`（截断，非四舍五入）；用 diagnostics `color_bits` 反解 f32 后 `trunc(color*255)` 与 `product.bytes` 逐通道比对 **12/12 全等** ⇒ diagnostics 记录与像素/射线确实对应、color 字段偏移与编码正确。
2. **`view_dir` 派生**：4 px × 3 分量 `view_dir_bits == ray_dir_bits ^ 0x80000000` 全部成立（精确符号位翻转，含负零口径）。
3. **灯槽**：4 px 均为 4 槽、每槽 8 值；本场景 nlight=3 + headlight 落 slot3，无 `nlight>=4` 冲突。

## 7. 低残留 / 观察

- **R1（低）**：`run-summary.txt` 的 `gpu_after=$gpu` 未展开（**跑后 GPU 未记录**）、`time=2026-11-11T07:2x+08:00` 为占位式时间戳；建议以 pre/post 采样补记。
- **R2（低）**：`body-env.txt` 路径转义污染（TAB×2、退格×1；`where nvrtc64_120_0.dll` 行不可读、`PATH_PREPEND` 行文本缺 `target` 段）——属记录生成瑕疵，运行实际成功（pilot 通过 + dump 生成 + nvrtc 已加载）；建议用单引号/`WriteAllText` 重生成该记录。
- **R3（低）**：GPU 数字与派工信息不一致——现场 `pre-state.txt` 为 **2% / 3,924 MiB / 12,138 free / 26℃**，派工信息为 **pre 4% / 3,919 MiB / 26℃、post 2% / 3,914 MiB / 26℃**；且跑后值未落盘（R1）。建议核对采样时点、补记跑后值。
- **观察 O1**：`preflight.txt` §6 标题残留 `({ts})` 占位；§1 记录的 exe（92,558,136 / `fcfc9035…`）是修卡前旧的，文件内已自行注明 stale 并以 §6 说明 #7 会 relink，与 run-summary 的新 exe 不矛盾，但阅读时需注意。
- **观察 O2**：本轮锁纪律按记录+披露采信（仓内无锁审计日志，见既有 backlog）；未发现并发写或第二份运行证据。

## 8. 非声明

- 本复核**不判定修复效果**：4 px 仍在、判据 A 仍 FAIL，G-L2 判定维持未通过；dump 仅诊断用途、未入库；未改代码、未重冻/重跑/采集，零 GPU。
- 不改变任何判据/阈值/参考/容差/相机/recipe。

## Addenda（2026-11-11，dump 字段 errata B）

- 本卡引用的 `seg.attribution.geom_type = 5` 已确认是 **`seg[1]`（object type，`mjOBJ_GEOM = 5`）误标**，不是模型 geom 类型；真实 geom 3 = **7（MESH）**。旧 dump（diag7/diag8）字节不改、仍按旧语义读；新语义见 `perworld-l-v2-gl2-errata-batch/README.md` §B：`geom_type` 改为模型真实值、新增过渡键 `seg_objtype = seg[1]`，dump schema 保持 2。本卡原有数值结论不受影响（按 `seg.product[0]` + `mesh_id/face` 判几何）。
