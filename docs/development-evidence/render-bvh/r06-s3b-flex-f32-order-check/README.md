# R06-S3b f32-order 修复复核（残留 1 px 结论）——**pass**

- 复核卡：`2026-11-04-render-r06-s3b-flex-f32-order-check`（owner QA）
- 复核对象：`2026-11-04-render-r06-s3b-flex-f32-order-fix-r2` 的**未提交证据**（`docs/development-evidence/render-bvh/r06-s3b-flex-f32-order/`，r2 在 `cb815c3f` 起 start，复核期间锁在 Backend、仍在收口）。**对象不是 committed blob**：本卡钉住复核时快照，逐文件 sha256+bytes 与对象 `manifest.txt` 的 sha 见 `logs/qa_f32_verify.log` §1；r2 正式提交后若字节变化，应在 r2 复核/attempt5 材料里对最终 blob 再核一次。
- 纪律：**只读、零 GPU、零写产品代码**；复核时写锁在 Backend（`pid=7156`，19:24:50 acquire），本卡不抢锁做只读核，落库阶段才单独取锁。
- **结论：pass（0 阻断）。** 残留 1 px 的定性（flex-0 候选顺序 tie、非 f32 收缩）与关键数字**全部独立复现**；前一版证据里我报的 2 条需修（product 列来源未标注）已在对象 19:25/19:27 更新中收口，复核确认。另有 **1 条需修-低**（对象 `manifest.txt` 缺 `#` 表头，触发 BOM 审计告警）+ 2 条**观察**（不阻塞）与对象「仍在收口」的时点说明。
- 判据面：**本卡不改任何判据/容差/参考/fixtures**；「放行该 px」属**判据放宽（签名级）**而非 B 类前提自然适用，见 `DECISION-SUPPORT-user-signature.md`（供 Chief 引用）。

## 1. 独立复算（脚本 `scripts/qa_f32_verify.py`，日志 `logs/qa_f32_verify.log`）

输入：本卡入库的产品 cam2 位副本 `artifacts/cam2-depth-seg-fixed.npz`（由复核时产品 dump 派生，17,005 B）+ 仓内参考 `reference/base/{depth,seg}.npz` + 对象 oracle npz + D8 基线 artifact。不复用对象/D8 的分析脚本。

| 指标 | 复算值 | 对象声明 | 判定 |
| --- | --- | --- | --- |
| cam2 depth `mismatched / beyond4 / max_ulp` | **1 / 0 / 3** | 1 / 0 / 3 | 一致 |
| 唯一失配像素 | `(x=134, y=128)` flat **41094**、`flexid=0`、ULP=3、`0x400015ba` vs 参考 `0x400015b7` | 同 | 一致 |
| cam2 seg | **0 / 76,800** 失配 | 0 | 一致 |
| 失配属地 | flex0 **1** / flex1 0 / 非 flex 0 | 同 | 一致 |
| D8 基线 321 px | 修复后 **321→1**；**修掉 320**；新失配 **是 D8 子集** | 320/321 修掉 | 一致 |
| oracle flex0 vs 参考 | **0 / 403** 位级 | 0/403 | 一致 |
| oracle flex0 vs 产品 | **1 / 403** | 1/403 | 一致 |
| oracle flex1 vs 参考 | **0 / 251** 位级 | 0/251 | 一致 |
| oracle 列绑定 | flex0 的 product 列=**修复后**产品、flex1 的=**D8 修复前**产品；ref 列均绑参考位序 | 同（`oracle-product-sources.json`） | 一致 |

## 2. tie 判定（复算成立）

- 取自 `artifacts/flex0-oracle.npz` 该 px 的三边距离：`d3 = [2.009920835494995, 2.0099215507507324, -1]`（第三边无命中）。
- `Δd = 7.152557373046875e-07`；`tol = RAY_TOL_REL * max(d,1) = 1e-6 * 2.009920835494995 = 2.009920835494995e-06` → **Δd < tol**，两候选在容差内平局。
- 参考胜出 **edge0**、产品胜出 **edge1**；`depth = d * factor`（factor `0.995723482`）**位级同时复现两侧 bits**（`0x400015b7`/`0x400015ba`）→ 3 ULP 就是两条边自身深度差。
- 规则源码一致：`RAY_TOL_REL=1.0e-6f`（`src/render/kernel/cuda/render_math.cu:28`）；`candidate_id = (leaf_geom < 0) ? -2 : geom`（`render_common.cu:377`）；`closer = (d < dist - tol) || (fabsf(d-dist) <= tol && (hit.geom_id < 0 || candidate_id > hit.geom_id))`（`render_common.cu:455-460`）→ flex 首命中后 `geom_id<0` ⇒ 容差内**后访问者胜**。
- 结论：该 px 是**候选访问顺序**问题，不是 f32 运算次序/FMA 收缩点；f32 卡 scope 内不可修（oracle 对参考 0/403 已证链算术一致）。

## 3. rigid 零漂移（从断言顺序推得，成立）

`tests/render_flex_validate.rs:332-353` 的 `compare()`：先对三相机 rgb 断言，再按参考相机顺序逐相机断言 depth（`:347`）→ `max_ulp`（`:348`）→ seg（`:351`）。
验证短跑日志停在 **cam2 depth**（`left=(1,0)`），说明 cam0/cam1 的 depth `(0,0)` 与 seg 断言、以及三相机 rgb 断言**都已通过**；cam2 的 seg 断言在 depth 之后未执行，但由产品 dump 复算给出 **0/76,800**（§1）。

## 4. 指纹 / PTX（独立重算与口径）

- 产品 PTX（离线 NVRTC，我本地复跑 `nvrtc_precheck.py`）：`source_concat_sha256=545c8fbe…`、**345,514 B**、`ptx_sha256=aa51258b…`；旧值 `d48fc74e…`/343,993 B/`9118d1ea…`。复跑输出见 `logs/ptx-precheck.log`。
- test exe：debug test exe 的 sha **随构建实例变化**（同源同大小不同 hash）——对象已记录 canonical rebuild `3A096B52…`、determinism probe `3CF1EA02…`、被 superseded 的 gate-build 值 `29184181…`，并在 19:27 又记 `2331BA26…`。**正确口径（对象 README §5 窗口规则 v2，Chief 19:26 批）**：attempt5 开窗前先做一次 canonical rebuild → 立即记录 exe sha+bytes → 用该次产物 `freeze -Attempt 1`（16/16、exes=2、inputs=14）→ 之后零编译跑实体；**预录 sha 只作参考、不作冻结卡口**。我确认该口径与对象证据一致，且 r2 后续材料应照此写。
- 对象 `verification-summary.json` 的 `fingerprints.test_exe_sha256` 已自带「deprecated as a freeze target」注释 —— 与我上轮意见一致。

## 5. 需修（前轮 2 条已收口 + 新增 1 条）与观察（2 条，不阻塞）

**已收口（我复核确认）**
1. `verification-summary.json.oracle`(flex1) 的 product 列来源已标注 `product_source = "D8 pre-fix product dump (a1cc22ae)"`，并新增 `post_fix_product` 数值（flex1 0、flex0 1、全 flex 1、cam2 1/0）；新增 `artifacts/oracle-product-sources.json`；README §3 明写「flex1 的 `prod_depth` 是修复前 D8」。
2. r2 的四门禁已在 **r2 卡 id** 下重跑（`.normify/runs/…-f32-order-fix-r2/` 四件 pass，19:25–19:26），不再挂在 abandoned 卡下。

**需修-低（r2 收口时）**

3. 对象 `docs/development-evidence/render-bvh/r06-s3b-flex-f32-order/manifest.txt` **首字节不是 `#`**（直接以第一条 sha 行开头）→ `scripts/validate/check-evidence-bom.ps1` 记为 `WARN manifest`，使全仓 manifest 表头违规 **4→5**（我实测：`RESULT: WARN (128 non-.ps1 BOM files, 5 manifest header violations)`）。修法：文件头加一行 `# …` 注释即可（manifest 不自列，不影响已列条目的 sha/bytes）。

**观察（建议 r2 收口时顺手）**
1. 产品两套 dump 由**临时 WIP 的 `F32DUMP` harness** 产生（已 `git checkout` 还原、未入库），`scripts/residual_pixel.py` 只消费 dump；对象 README §8「复现」只列 oracle/枚举/短跑，**没有产生 dump 的步骤**。目前 dump 只以 target 路径 + sha 绑定（本卡另存了派生位副本，见 §7）。建议 r2 证据补「dump 由 WIP harness 产生、已还原」一句，或附 harness 补丁/片段。
2. 「两跑 sha 相等」只有 harness 打印值（`logs/residual-run.log`），单份 dump 无法离线重算；attempt5 窗口内应按现场两跑重新记录。

## 6. 「B 类前提」还是「判据放宽」（供 Chief 引用，详见 `DECISION-SUPPORT-user-signature.md`）

- 修复后唯一未过项是 **bit-first `mismatched=1`**（`beyond4` 已满足）。放行它＝覆盖冻结判据（`tests/render_flex_validate.rs:344/:347`）→ **判据放宽（签名级），必须用户签**；不能写成「B 类前提自然适用」。
- 该 px 根因是 **tie 顺序**，与 D8 的 **321 px codegen ULP 尾差（B 类、已降级登记）分开**。
- 建议签文本（最小范围）：`cam2 / (134,128) / flexid0 / ULP≤4 且 beyond4=0`；attempt5 判据同步为「除该子集外 bit-first」；并行立「1D flex 候选顺序/tie 语义对齐」根因卡（**Arch 已登记 `2026-11-04-render-r06-s3b-flex-tie-order`，提交 `3e3f363d`，与 attempt5 解耦**）；不签则 attempt5 前置不满足。**本卡不改判据本身。**

## 7. 可复现性与产物

- `artifacts/cam2-depth-seg-fixed.npz`（17,005 B）：复核时产品 cam2 的 `depth_bits`(76800, uint32) + `seg`(76800,2, int32) 压缩副本，**由复核时 target dump 派生**，使「1/76800、ULP=3、seg 0/76,800」可在无 GPU、无 target 残留时用本卡证据复算。
- 派生一致性：若 target 上两份 dump 仍在，脚本 §3 会重算其 sha 并验证派生副本逐位相等；target 被清理时以 `artifacts` 的 sha 为绑定记录。
- 复核时仓状态（HEAD、锁、对象 manifest sha、r2 gate 状态）见 `logs/reviewed-state.txt`。

## 8. 非声明与边界

- 不宣称 r2 修复 verified、不宣称 attempt5 通过、不宣称该 px「已放行」；放行需用户签（§6）。
- 不改判据/容差/参考/guards/fixtures；不重冻；不改对象证据（只读，pin 快照）。
- 本卡只覆盖「残留 1 px 结论 + 证据可复核性 + 放行口径建议」；根因修复（tie/BVH 遍历）另立卡。

## 9. 复现

```powershell
# 仓库根执行；CPU-only，不触 GPU。--out 以 LF 写日志（字节确定）。
$ev = "docs\development-evidence\render-bvh\r06-s3b-flex-f32-order-check"
target\mjwarp-parallel\venv\Scripts\python.exe -B "$ev\scripts\qa_f32_verify.py" --out "$ev\logs\qa_f32_verify.log"

# 产品 PTX 指纹（离线 NVRTC；重跑后与 logs/ptx-precheck.log 比对）
target\mjwarp-parallel\venv\Scripts\python.exe -B `
  docs\development-evidence\render-bvh\r06-s3c-flex-mesh-device\tools\nvrtc_precheck.py `
  --repo . --cuda target/toolchains/cuda-12.8.1 --out target/qa-ptx-rerun
```

## 10. 产物清单

- `README.md`（本文）、`DECISION-SUPPORT-user-signature.md`（供 Chief 引用的一页口径）、`facts.json`、`manifest.txt`
- `artifacts/cam2-depth-seg-fixed.npz`（产品 cam2 位副本，派生）
- `scripts/qa_f32_verify.py`、`logs/qa_f32_verify.log`、`logs/ptx-precheck.log`、`logs/reviewed-state.txt`
