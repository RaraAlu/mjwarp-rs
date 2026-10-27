# UNC-3 分支收口 S3 · QA 观察收口（O1–O10）

- 任务：`2026-10-27-unc3-branch-closure-s3-observation-closure`（Arch 登记，2026-10-27）
- 上游：S3 独立复核 `2026-10-27-unc3-branch-closure-s3-independent-check` → **pass / 阻断 0 / 观察 10**（未重跑窗口、未建 CUDA context）
- 被收口证据：`docs/development-evidence/g10-p5-branch-closure-capture/`（README + receipt + 交付记录 `docs/development-evidence/changes/2026-10-27-unc3-branch-closure-s3-capture.json`）
- 收口性质：**只改证据文字与数字标注**；fixture、两份上游参考、capture/precheck 脚本、判据、容差、产品代码、设备源 **零改动**；**未重跑捕获窗**。

## 1. 处置总表

| # | 级别 | QA 观察 | 处置 | 落点 |
| --- | --- | --- | --- | --- |
| O1 | 一般 | 「dist/pos 位级一致」case 数三处不一致（README 6 / receipt 5 / 实际 4） | **更正**：dist+pos 同时位级一致 = **4**（`plane-cylinder-degenerate` / `plane-box-corner-face-down` / `-face-up` / `plane-capsule-lying`）；`corner-single` **仅 dist 位级**（pos x `1.1678e-8` GPU vs `1.4901e-8` CPU，绝对差 3.2e-9）；mesh 三例 CPU precheck 不含 dist/pos | S3 README §7.5 / receipt `cpu_precheck_vs_gpu_reference` |
| O2 | 一般 | U8「4 接触来自 ≤16 顶点侧面」推断与 fixture 几何相反 | **推断作废**：接触面含 **20-gon 顶帽**（棱柱顶帽 z=+0.05、box 底面 z=0.04、穿透 0.01，参考 4 条 pos `(±0.05,±0.05,0.045)` 为盒 4 角与顶帽中点）；上游 `npolygonmax` 取模型值 20（动态容量）故不整体回退；**polygon 号未打印**，「接触面含顶帽」为几何位置佐证，不写「已证实」 | S3 README §7.3 / receipt `u7_u8.U8` |
| O3 | 建议 | `SHA_OK fixtures=13 …` 行未归档（runner stdout 未落盘） | **接受**：QA 已 16/16 独立重算一致，守卫失败会 exit 4 而参考/事实已产出 ⇒ 实质通过；**纪律**：今后捕获 runner stdout 随窗口日志归档 | S3 README §11 |
| O4 | 建议 | 4 个 PS 日志带 UTF-8 BOM（`Out-File -Encoding utf8`） | **接受**：窗口原始日志按原样留档不改（保真优先）；文档/脚本仍守 UTF-8 无 BOM | S3 README §11 |
| O5 | 建议 | `pre` 与 `post` 完全相同，疑 post 复制到 pre | **更正**：`pre` = `2849,13213,39`（`guard.log` 的 `guard_baseline` / `phase=preconditions`），`post` = `2844,13218,23,31`；温度未记已注明 | S3 README §4 / receipt `capture_window.pre` |
| O6 | 建议 | compute-apps 实际 29 条（README/receipt 写 28） | **更正**：原文 **29 条**（baseline 与 post 各 6 条 `[Insufficient Permissions]`），过滤后 0 的结论不变 | S3 README §4 / receipt `capture_window.compute_apps` |
| O7 | 建议 | U6 的 `geom_dataid` 被列在 GPU 参考「关键数值」，实来自 precheck | **更正标注**：GPU 参考 `output` 用 `geom`；`geom_dataid`/nmesh 统计来源 = CPU precheck | S3 README §5.1 U6 行 / receipt |
| O8 | 建议 | S3 acceptance「U7/U8 超界进入回退」与证据不符（设备回退是 S4） | **接受**：S3 只证明模型级超界 + 上游行为；**S5 汇总不使用「进入回退」措辞** | S3 README §11 / 本回执 |
| O9 | 建议 | U7 上游 count=1 与设备单接触回退 count=1 巧合相同，只比 count 无法区分 | **接受**：**S4 对 U7/U8 同时记 dist/pos**（不只比 count） | S3 README §11 / S4 计划 |
| O10 | 建议 | `receipt.json`/`precheck-cpu.json` 行数超 500 | **接受**：500 行棘轮只约束新增 **target 测试文件**；机器可读 JSON 证据豁免（已注明） | S3 README §11 |

## 2. 更正后的关键数字（与 QA 独立复算一致）

- 窗口：有效轮 / guard 无 abort / A、B 两进程 exit 0（29.4 s / 7.4 s）/ 两文件 `RAW BYTE-IDENTICAL: PASS` / 11-11 两次捕获一致
- 采样：`samples=134`、`baseline_used=2849`、`used_peak=3132`、`delta_peak=283`（三口径 283 / 227 / 283）、`free_min=12930` > floor 10085、`util_max=94` 且 80 以上连续 2 样本、`residual=0`
- compute-apps：原文 **29** 条（6 条无权限噪声），过滤后 **0**；`pre=2849,13213,39` / `post=2844,13218,23,31`
- CPU precheck vs GPU：count + `qpos_bits` **11/11**；dist+pos 同置位级一致 **4** 例；`corner-single` 仅 dist；3 个 tilt pose dist 8–16 ULP / pos 8–48 ULP
- U8：上游 count=4、接触面含 **20-gon 顶帽**（polygon 号未打印）；U7 count=1 与 degree>10 限制语义一致

结论方向不变：**S3 = 有效捕获轮（不写「窗口通过」）**；U7/U8 仍 record-only 不进分母；数值验收在 S4。

## 3. 改动集（本片）

- `docs/development-evidence/g10-p5-branch-closure-capture/README.md`：§4 pre/compute-apps、§5.1 U6 来源、§7.3 U8 方向、§7.5 位级 case 数、新增 §11 观察收口表
- `docs/development-evidence/g10-p5-branch-closure-capture/receipt.json`：`capture_window.pre` / `compute_apps.raw_lines` / `cpu_precheck_vs_gpu_reference` / `u7_u8.U8` / 新增 `qa_independent_check`
- `docs/development-evidence/changes/2026-10-27-unc3-branch-closure-s3-capture.json`：更正 `cpu_vs_gpu` / `u7_u8` / `window` 文字，新增 `qa_check` 与里程碑，**27 个文件 blob+sha256 刷新**
- 本目录（README + receipt + manifest）

不改：`fixtures/`、`scripts/`、两份上游参考、判据/容差/期望值、产品代码、设备源；不重跑捕获窗。

## 4. 边界（不宣称项）

- 不宣称任何分支「已覆盖」；U7/U8 保持「仅限制存在」，U2 保持「不可达/防御」。
- 不写「窗口通过」；数值验收仍在 S4 验证窗。
- O1/O2 是证据文字错误更正，不改变参考值、不改判据、不触发重采。
