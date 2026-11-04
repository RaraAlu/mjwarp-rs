# R06-S3b attempt5 窗口记录（FAILED_ROUND：判据实现缺口，非数值红）

任务：`2026-11-04-render-r06-s3b-flex-validate-attempt5`（owner Arch）；窗口规则 v2（Chief 2026-11-04 批准）+ 用户签名 allowlist（`3406f1ca`，2026-11-04 22:14）。
唯一执行会话 = arch-chat；Chief GO 已发、群留痕已贴。**本文件不宣称 flex 渲染通过，也不宣称数值失败**——失败原因是**判据实现缺口**，数值状态恰好等于签名 allowlist 预期。

## 1. 开窗前（三条前提）

| 项 | 值 |
| --- | --- |
| 三点基线（22:29:59，间隔 2.5 s） | `free = 14610 / 14610 / 14610 MiB`、`util 2%`、`temp 27°C` |
| floor | `max(min_free−3072 = 11538, 已签 11688)` = **11,688 MiB**（取严）；三点均 ≥ floor |
| build 进程 | 0；worktree 0 dirty；HEAD `591a49d3`；锁 FREE |

## 2. 窗口规则 v2 执行

- **一次 canonical rebuild**（`cargo clean -p mjwarp-rs`（23,714 文件 / 14.0 GiB）+ `cargo test --locked --features cuda-probe --no-run --test render_flex_validate`）：**14.1 s**，构建进程归零；
- **记录该次产物**：`render_flex_validate-30bd8d889931ab2f.exe`，**107,661,795 B**，sha256 **`b92e1bed…`**；产品 bin `mjwarp-rs.exe` **85,041,741 B**，sha256 **`6dafc24c…`**（`attempt/canonical-rebuild.log`）；
- **`freeze -Attempt 1`**：**`GUARDS_OK=16/16 exes=2 inputs=14`**，aggregate_all **`8091ab5a…`**、aggregate_inputs **`ec6fa222…`**、guards_sha **`9b6a3535…`**（7,508 B）→ `attempt/freeze.log` + `attempt/guards-frozen.json`（冻结快照）；
- **等价核验**：live==guard **2/2**、stash==guard **2/2**、guard_inputs 逐条 sha **14/14**；exe 条目 `pending=false`、`status=frozen`；
- **冻结后零编译**：实体体运行时 `cargo` 输出 `Finished … in 0.16s`（无重编/relink）。

## 3. monitor（通过，含外部负载插曲，如实记录）

- 采样 105 点（22:31:08–22:39:57）：**最终 60/60 连续干净**（22:34:59–22:39:57），util 1–6%、free 14,468 MiB ≥ floor 11,688；
- **外部 GPU 负载插曲**：22:31:59–约 22:35 出现非我方负载（util 最高 **98%**、free 最低 **4,419 MiB**、`compute_filtered=0` 且 `build=0`，不是构建/我方任务），使 clean streak 归零；负载结束后重新累计 60/60 达标。原始 CSV 完整保留（含脏点）→ `attempt/monitor-pre-window.csv`。

## 4. 实体体（FAILED，判据实现缺口）

命令（PATH 前置 `target/toolchains/cuda-12.8.1/bin`）：
`cargo test --locked --features cuda-probe --test render_flex_validate -- --ignored --test-threads=1 --nocapture`
日志含 **`running 1 test`** → 1.44 s，`test result: FAILED`。

**已经过的断言（按用例断言顺序）**：
- D5 位姿自证打印：cam0 0.025°、cam1 **85.517°**、cam2 **75.562°**（D5 修复后用例改读参考 data 位姿，且断言已通过）；
- **rgb（三相机）全部通过**：`max_byte_diff ≤ 1`、`beyond_1_lsb = 0`；
- **depth cam0 / cam1 通过**：`(mismatched, beyond4) = (0, 0)`，且其 **seg 全等**。

**失败点**：`tests/render_flex_validate.rs:347` → `cam2 depth 位级/超 4 ULP: left (1, 0) vs right (0, 0)`。
即 cam2 depth：`mismatched = 1`、`beyond4ULP = 0`（该 1 像素 ULP ≤ 4），**恰好等于用户签名 allowlist 允许的状态**（`cam2/(134,128)/flexid0/ULP≤4 且 beyond4=0`），但**冻结的用例判据仍要求 `(0,0)`** → 断言先于 allowlist 生效而失败。
**未评估**：cam2 seg（断言在 depth 之后）、两跑 sha256。

## 5. 分类与结论

- **FAILED_ROUND（判据实现缺口）**：冻结用例的断言（严格 `mismatched=0`）**严于**本次用户签名 allowlist；数值观测 `(1,0)` 正好落在签名允许范围内 → **不是数值红、不是回归**。
- 依窗口纪律：**失败即停、不重试、不放宽、不做二次微跑**；现场/日志/冻结指纹全部保留（本轮 5 个原始文件在 `attempt/`）。
- **不采用**本轮结果作任何验收结论（rgb/cam0/cam1 的通过也仅为部分观测，不构成 attempt5 通过）。

## 6. 下一轮（attempt6）所需修复（不属于本轮）

1. 用例断言实现 allowlist：`cam2/(134,128)/flexid0` 允许 `ULP ≤ 4 且 beyond4 = 0`，**其余全部维持严格**（`mismatched=0`）；不得把 allowlist 扩大为相机级/区域级；
2. 端口 allowlist 逻辑要有 **CPU 自证**（普通 `cargo test`：给定 `(1,0)` 时必须通过、`(0,1)` 或第二像素不等时必须失败）；
3. 修复后需**重新 canonical rebuild + 重新 freeze 16/16**，并重新申请 **Chief GO**（新一次 ≤3 分钟窗口即可，因为只跑一个 ignored 用例）；
4. 判据文本（签名 allowlist）**不变**；只改判据的**实现**。

## 7. 边界与非声明

- 不改判据/容差/参考/`fixtures/`；不碰 r2 证据；窗口内未发生冻结后编译。
- 不宣称上游等价、parity pass、flex 渲染通过或 R06-S3b 通过。
