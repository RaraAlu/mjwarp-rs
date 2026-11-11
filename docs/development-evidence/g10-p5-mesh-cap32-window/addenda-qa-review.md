# A2-3 复核发现收口（append-only；历史窗口/回执字节不改）

- 任务：`2026-11-12-w2-u7u8-full-condition-assert` 第一节；对 QA 复核（`docs/development-evidence/np-gate/w2-a2-3-window-check/`，`7ff2c29b`）
  「0 阻断 / 0 严重 / 0 一般 + 3 低 + 5 观察」的 A2-3 窗口证据做 append-only 收口。
- 处置原则：只追加说明、补规范化副本、更新 derives（manifest）；**不修改 A2-3 窗口证据已有 20 个文件的任何字节**，
  不修改 A2-2 身份证据的历史窗口/回执字节（identity `addenda.md` 仅末尾 append，manifest 为簿记更新）。
- 条数口径（更正旧 `addenda.md` 措辞，QA O3）：manifest **不列自身**，记录条数 = 文件数 − 1。
  本次更新前：20 条记录 / 21 个文件；更新后：**24 条记录 / 25 个文件**（新增本文件 + 3 个 logs）。

## L1 `window-runner-console.log` 二次编码损坏

- 事实：该文件是 **UTF-16LE（含 BOM）字节流被二次转码**后的产物——头 2 字节 BOM 变为两个 `U+FFFD`
  （`ef bf bd ef bf bd`），其余为 ASCII + `00` 间隔，逐行行尾的 CR 字节（`0d`）被规范化成 LF 字节（`0a`，
  长度不变），文件含 5,467 个 NUL，不能作为 UTF-8 文本审阅；与 manifest 头注「UTF-8/LF 无 BOM」不符（QA L1）。
- 处置：从 target 原件（`target/render-bvh/gl2-ptx-diff/w2-a2-precheck/window-runner-console.log`，
  UTF-16LE/BOM，10,936 B）解码生成 **`logs/window-runner-console.utf8.log`**（UTF-8/LF、无 BOM，43 行）；
  新增 **`logs/console-vs-facts.diff.log`** 记录与 facts 的逐行差异。
- 原损坏文件 **字节不动**（manifest sha `7e46a6bc…` 不变）；manifest 头注改为准确表述。
- 差异披露：规范化副本与 `facts.txt` 仅差 2 行——facts 多 `GPU pre` / `GPU post` 两行原始采样
  （console 捕获未含），其余 43 行逐行相同；`facts.txt` 为权威超集（与 QA 复核结论一致）。

## L2 readiness §8「窗内 0.25 s 采样」未实现

- 事实：A2-3 runner 只在窗口前后各采样一次 GPU（pre/post）；`a2-3-window-readiness.md` §8 的
  「窗口期间 0.25 s 采样同样监控」是计划承诺，**未实现、也未在证据中披露**（QA L2）。
- 处置（二选一取「实装」）：本片下一最小窗（r2 runner）**实装窗内 0.25 s GPU 采样循环**——只读
  `nvidia-smi` 查询，窗口内不再只有两次采样；采样 CSV（`logs/window-samples.csv`）随 r2 窗证据入库，
  README 记录样本数、实测间隔（目标 250 ms，实测以时间戳为准）、util_max、free_min。
- A2-3 readiness 历史字节不改；本节即为其未落地事实的披露记录；A2-3 窗口结论不因此变化。

## L3 identity addenda「不改 A2-2 任何历史字节」表述不准确

- 事实：A2-3 §1 **同时修改了** `g10-p5-mesh-cap32-identity/receipt.json`（新增 `a2_3_section1_errata` 块）
  并重算了 identity manifest（11→12）；旧 addenda 只写「日志仅末尾 append」，未披露 receipt 改动（QA L3）。
- 精确表述（取代旧措辞）：**log 为末尾 append；receipt/manifest 为簿记更新；其余 A2-2 历史窗口/回执字节未改。**
- 已在 `g10-p5-mesh-cap32-identity/addenda.md` 末尾 append 补正说明（历史字节不改），
  identity manifest 的 `addenda.md` 条目按新 sha 重算（记录数仍 12）。

## 观察处置

- **O1 floor 公式写法**：硬门为 **free ≥ 10,240 MiB**；公式值为 `min(10240, baseline_free−3072)` =
  `min(10240, 12,981−3,072=9,909)` = 9,909；实际生效取两者**更严者（max）= 10,240**。
  旧文「floor=min(...)…取严后生效 10,240」把 min/max 混写，此处更正；A2-3 数值与结论不变
  （pre free 12,954 > 10,240 > 9,909）。
- **O2 三点基线 raw log**：见 `logs/three-point-baseline-recalled.txt`（按 README §2 回填，**非原始机器日志**，
  原始日志未落盘）。后续 r2 窗把 runner pre/post 原始采样写入 facts，并要求开窗基线 raw 落盘（若提供则入证据）。
- **O3 manifest 条数措辞**：旧 `addenda.md` 的「20/20 自核；追加后 21 条」已按上式更正为
  「19→20 条记录 / 20→21 个文件；manifest 不列自身」；不影响自核（QA 复算 20/20 全对、无自列）。
- **O4 卡 addenda/gate run 时序**：A2-3 卡 4 个 gate run `at=02:20:29–47` **晚于**卡 `closed_at=02:18:59`，
  来自「为 O3 补 tests 写入范围并重跑门禁」的尝试（后因 hook 拒绝改为独立片）；两次运行均 PASS、
  树差异仅文档，**不影响判定**；已在 A2-3 相关提交说明与本任务 note 点明。
- **O5 monitor 脚本路径笔误**：README §2 写 `scripts/validate/render-hvh/monitor-until-clean.ps1`，
  实际仓内为 **`scripts/validate/render-bvh/monitor-until-clean.ps1`**；历史字节不改，此处更正。

## 非声明

- 本节只做证据卫生收口，不改变 A2-3 窗口结论（10/10 exit 0、sha 27/27、monitor 60/60、零 Rust 编译、
  U7/U8 可观测子集满足、pass_rule 全条件未覆盖）。
- 不宣称 U7/U8 pass_rule 全条件通过、U8 已验收、mesh 容量缺口已清、与上游等价（over-32）、G10@P5 整族通过、
  NP01/NP02 门禁通过；不作位级等价声明、不提供位级等价保证。
