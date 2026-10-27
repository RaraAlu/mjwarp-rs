# R08-S2 观察收口独立复核（QA）：O1–O4 措辞/旁注/定义 + 零数值改动

- 任务：`2026-10-28-render-bvh-r08-e2e-obs-closure-check`（复核对象 `2026-10-28-render-bvh-r08-e2e-obs-closure`）
- 提交链：`df7f583d`（收口证据 6 文件，docs-only）→ `254a49c4`（verified 记录）；复核基线 = 我的 pass 版 `10301dc6`；复核时 HEAD `254a49c4`
- 结论：**pass / 阻断 0 / 观察 0**；只读复核，未重开门、未重跑、未开 GPU、未改任何文件

## 1. 总体判定

4 条观察全部按 Arch 裁决落地，**且窗口数值零改动**：收口提交只改证据文本（README + 3 个旁注 +
收口 receipt + 任务卡 = 6 文件），对 `src/ tests/ fixtures/ Cargo*` 为空；`facts / baseline / samples /
sha-guards / guard_aggregate / 7 entity 日志` 与 pass 版逐字节相同；runner 与 7 个冻结 exe sha 未变。
47 项独立检查 0 问题。

## 2. O1（修）：空选择 no-op 按轮次分列

- README §3.1 已改为：**attempt1 已通过** = `nrender()==0` / `total_rays==0`、渲染前
  `is_enabled(Rgb/Depth/Segmentation)==false`（三通道缓冲未分配）、`megakernel.render ×2` + `depth_seg.render ×1`
  共 **3 次** `Ok`（不是"各两次"）；**attempt1 在失败点终止**（`memory_after_construct == memory_after`
  失败，2 097 152 B 计入，该轮未取得"显存无增量"结论）；**attempt1 未执行**失败断言之后的
  `outputs().read(...).is_err()` 三通道"无可读输出"断言，这两类结论**只在 attempt2** 取得。
- 旧过强表述（「各两次」「no-op 语义断言全部成立」「attempt1 起即成立」）已从断言性文本删除；
  §11 仅以"已删除…类表述"形式引用这些字面（元描述，不是断言）。
- §6 定向用例段同步改为"attempt2 完整执行并通过；attempt1 只执行到失败点之前"。

## 3. O2（修）：attempt1 sha-guards 历史旁注

- 新增 `logs/attempt1/sha-guards-note.txt`：记录旧 target sha
  `6a73c51a…065d` / **94 027 937 B**（A2 前、仅历史留档、不改写原文件）；A2 后新 target
  `918b5851…2775` / **94 029 705 B**；"用当前 guards.json 重放 attempt1 清单 = **44/45**，属预期"。
- README §3.1/§3.2 同步该旁注；`logs/attempt1/sha-guards.txt` 本身字节未动（45 行/<45 未改）。

## 4. O3（修）：sustained_delta_mib 定义

- README §3 表后与 §11 写明 `sustained_delta_mib = used_peak − baseline.used`：
  attempt1 **275 = 4560 − 4285**、attempt2 **270 = 4593 − 4323**；并注明与 S3 窗口
  "排除尖峰后 max−min"的 sustained 口径**不同**、不可直接互比；数值不改。
- 两个旁注文件（attempt1/attempt2 `facts-note.txt`）写同一口径与算式，且与各自 `facts.txt` 一致。

## 5. O4（接受并注明）：guards-check 与 guards-check-a2 关系

- README 按**事实**写：两者都是 A2 之后的 guards-only 自检产物、静态对盘同 45/45；
  两份 `sha-guards.txt` 与 `guard.log` **逐字节相同**；`baseline.txt` / `facts.txt` /
  `machine-load.txt` 是不同时刻实时采样、**数值不同属正常**；两者均为 `target/` 历史产物不入库。
- 我独立验证（`target/r08-s2-window/`）：相同集合 = {`sha-guards.txt`, `guard.log`}，
  不同集合 = {`baseline.txt`, `facts.txt`, `machine-load.txt`}；README **没有**写"全部内容相同/完全相同"。

## 6. 零数值改动（对照 pass 版 `10301dc6`）

`10301dc6..254a49c4` 仅 6 个文件变更（README、3 个 note、`obs-closure-receipt.json`、
`.normify` 收口记录），无代码路径；以下关键证据**逐字节未变**：两轮 `samples.csv`、`facts.txt`、
`guard_aggregate.json`、`entity-results.csv`、`sha-guards.txt`、两轮 `render_r08_cameras.log`。
逐项数字仍为：7 exits（101 + 6×0 / 全 0）、attempt2 6 passed、判据全 0（120000）、
145 样本、4593 / 11469 / 36、delta 270·248·271、floor 8667、attempt1 floor 8705、
baseline 4285/11777/35 与 4323/11739/36–38。

## 7. 其他核对

- 收口 receipt `/task_id` 正确、`gate.not_applicable=true` 且写原因（docs-only，不重跑门禁）；
  `dispositions` 四项 = O1/O2/O3 **fix**、O4 **accept**，与 Arch 裁决一致；`zero_numeric_change` 声明可复算。
- runner `c8786951…` / 14976 B 未变；冻结 7 exe 逐条重哈希全 match；证据文本 LF 无 BOM；
  3 条禁词组 0 命中；四措辞仅否定/约束句。
- 本复核任务无门禁项：`normify gate` 返回「任务没有引用任何门禁」exit 1 属预期（receipt 记录）。

## 8. 边界与不宣称

只读复核，只核这 4 条措辞/旁注/定义与"窗口数值零改动"，不重算窗口数字、不重开门/重跑/开 GPU，
不替代用户/Chief 的窗口签字，也不对后续方向作验收宣称。
