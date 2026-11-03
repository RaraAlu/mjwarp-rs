# NP01 B2 字段级位差分账 QA 独立复核（B4）

任务：`2026-11-04-precision-np01-host-device-fields-check`（QA）。对象：`2026-11-04-precision-np01-host-device-fields`（verified，提交 `bc5e3682`：README + `host_device_fields.json`(555 行) + `scripts/authoring/np01-host-device-fields.py`(194 行)）；引用链 `2026-11-04-precision-np01-host-device-expr`（`487a7105`，12 条 PTX 定序表）与 A0 `np01-e-recompute`。复核时点 2026-11-04，HEAD `12bbb4b5`。零 GPU；只写本复核记录。

## 结论

**PASS-with-observations**：四类差异分开算、分开报，**未发现「把 recipe 差写成类型偏离」**；两脚本独立复跑成功且输出与入库 JSON **逐字节一致**。1 条需修-低为文字计数（8→9 条带乘加），数据正确；不改判据、不宣称 NP01/NP03 通过。

## 一、独立复跑（A0）

- 命令（README 给定）：`target\mjwarp-parallel\venv\Scripts\python.exe scripts\authoring\np01-host-device-fields.py`；expr 脚本同路径。
- 结果：fields `exit=0`（输出 5 recipe cases：cylinder-box/ellipsoid-box/mesh-mesh/convex-mesh/degenerate-mesh；4 residual cases：ellipsoid-box/hfield-sphere/hfield-capsule/sdf-box；known deviations=1）；expr `exit=0`（12 expressions，NVRTC 12.8 编译 `--fmad=true/false`，零 GPU）。
- **可复现性**：复跑后 `git status --short docs/development-evidence/np01-host-device-fields docs/development-evidence/np01-host-device-expr` 为空 ⇒ 两个 JSON 与入库版本逐字节一致（脚本无时间戳/环境依赖）。

## 二、抽 3 处数字对账（A0）

1. **recipe 5 case（`compare-cpu-vs-gpu.json`）**：与 README §2 表逐格一致，例：cylinder-box dist 8/8、pos 19/24（max_abs **1.49e-8**）、frame 72/72；ellipsoid-box dist 0/2（**1.86e-9**）、frame 8/18（**8.19e-7**）；mesh-mesh frame 2/18（5.36e-7）；degenerate-mesh frame 16/18（2.65e-11）。`max_abs` 均在 1e-9–8e-7 量级。
2. **同 recipe 残差（`g10-p5-gpu-kernels/gpu-window/result.json`，`failed_cases=[]`）**：hfield-sphere 8/8、24/24、72/72 **逐位全等**；hfield-capsule 7/7、21/21、63/63 全等；sdf-box dist 80/80、pos 203/240（7.45e-9，1.0 ULP@scale）、frame 675/720（2.38e-7）；ellipsoid-box dist 1/2（max_ulp 2）。
3. **witness known-deviation**：JSON 记录 `dist_bad 0 / x1_bad 1 / x2_bad 3`、ncontact 与 overflow 相等，来源 `g10-p5-gpu-kernels-independent-check/qa-compare-gpu.json`，登记卡 `2026-10-25-g10-p5-deep-epa-witness-sampling`（verified），归类「同 recipe 实现残差（witness 分量表达式顺序敏感）」，**未写成类型偏离、未写成 recipe 差**。

## 三、归因审计（A1，重点）

- `type_deviation = {count: 0, basis: [36 条逐字段表（30 产品 + 3 host，NP01 B1–B3）、CUDA token 门禁、host source token 门禁], explicitly_not_inferred_from: "数值位级相等/ULP 计数……不能反推类型"}` —— **类型 0 项只由表 + 两道 token 门禁支撑**，没有用数值反推。
- `verdict` 明确 `type_deviation_is_zero: true` 与 `does_not_mean_bitwise_equal_across_recipes: true` 分开，并写「不宣称任何位级等价或 NP01/NP03 通过」。
- recipe 段标题/定义均为「同码不同后端（CPU 两次舍入 vs CUDA fma.rn）」，机制引 B1 的 `--fmad=true` 融合；**无一处把 recipe 差计入类型**。
- 同 recipe 残差段明确「两侧同 fma.rn recipe ⇒ 既不能归 recipe 也不能归类型，只能归实现/算法顺序差」，并以 hfield 两例逐位全等作强证据。
- 引擎语义段只引用 A0（`e_recompute.json`），写明「本片只引用、不做新判定、不参与类型结论」；`native_unpairable_cases` 单列。
- 全文检索「反推/推断类型/因此类型/说明类型」类表述：命中处均为**否定句**（明确不这么做），未发现以数值大小倒推类型的表述。
- A2 口径：`max_ulp_nonzero` 近零失真说明在（README §2 读法：frame 出现 4294967295 是跨 0 号比较，判读用 `max_abs`；JSON 中 mesh-mesh frame 该值确为 4294967295）；witness known-deviation 引用 verified 卡（未冒充新发现）；类型 0 与位级等价区分在 `verdict` 与 README §6.5 均有。

## 四、expr 表（B1 引用链，`487a7105`）

- 12 条表达式计数与 README §2 表一致：`dot3` fma2/mul0/add0（fmad=true）vs fma0/mul3/add2（fmad=false）；`face_norm2_head_tail` **fma2**（源码写 head→tail→norm2 仍被 device 收缩，结论「源代码写法不能表达禁止融合，只有 `--fmad=false` 或 `__fmul_rn/__fadd_rn` 才能」）；`pos_midpoint` fma1；`project_plane_scl` 两侧无 fma（div 独立）；`contact_dist_margin` add1；`frame_dir_sub` 0/0。`fmad=true` 带乘加表达式全部出现 `fma.rn` 且 `mul/add` 归零的判定成立。
- 需修-低：README §3 第 1 条与 fields README §2 写「12 条里 **8 条**带乘加」；JSON 实测 `fma>0` 的表达式为 **9 条**（8 条 fma=2 + `pos_midpoint` fma=1，另有 3 条无可融合乘加）。**数据正确、文字少计 1**，建议改为「9/12 条带乘加（其中 8 条 2×fma）」。

## 五、结论与边界

- **PASS-with-observations**；需修-低 1 条（8→9 文字）；观察 1 条（expr 复跑依赖仓内 `target/toolchains/cuda-12.8.1/bin` 的 NVRTC DLL，README 已写前提）。
- 未改判据/容差/参考/fixtures、零 GPU、零窗口；不宣称 NP01/NP03 通过、不宣称位级等价。

## 六、需修项闭合（2026-11-04 增量复核）

- 需修-低（README 8/12→9/12）已由 `8c22ffcc`（B1 expr）与 `6b906daa`（B2 fields）订正：两 README 现写「**9/12** 条带乘加（8 条 2×fma + `pos_midpoint` 1×fma）」，两张被复核卡已各补 `normify fix` addenda；数据未动。QA 增量复核通过，本复核结论闭合。
