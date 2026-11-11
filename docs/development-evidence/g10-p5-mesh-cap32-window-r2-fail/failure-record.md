# r2 最小窗失败记录（fail-stop 第 6 条命中）——U7/U8 comparator failures≠0

- 任务：`2026-11-12-w2-u7u8-full-condition-assert`（卡保持 **in_progress**，未动卡状态）
- 日期：2026-11-12；GO 来源：Chief（第二次 monitor 通过后）；窗口：单窗、单实体、仅执行 **1 次**，未重试
- 产物目录（target，未入库）：`target/a2-window-r2/`（facts.txt/sha-guards.txt/results.json/window-samples.csv/logs/*）
- 结论：**窗口有效执行但 U7/U8 pass_rule 全条件判据失败（compare failures=21）→ fail-stop，按纪律停止，不重试、不放宽**。

## 1. 执行事实（纯记录）

- 实体：`target/a2-freeze-r2/g10_p5_branch_closure_cuda-1b98cdc445ce5abc.exe`
  （sha `6d2b39fcad182e43c85a9faf64ab48a1369614889a60d00befc723efe0373677` / 41,771,008 B），参数
  `--ignored --nocapture --test-threads=1`；耗时 40.05 s。
- sha 守卫：**18/18 SHA_OK**（1 exe + 15 fixture + 2 参考）。
- floor：pre free 13,440 MiB / post free 13,440 MiB，硬门 10,240 → 通过。
- monitor（Chief 开窗前，CSV `monitor-pre-window-r2b.csv`）：60/60 clean、util_max=4、free_min=13,443。
- 窗内 0.25 s 采样：**134 samples**，实测间隔 min 288.7 / median 295.7 / max 361.2 ms，util_max=42、
  free_min=13,439（CSV `window-samples.csv`）。
- 零 Rust 编译：precheck 无 cargo/rustc/cl/nvcc；POST zero-rust-compile PASS；无残留 compute 进程。
- 测试结果（`logs/g10_p5_branch_closure_cuda.log`）：
  `test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 39.74s`
  - `branch_closure_acceptance_group_matches_s3_upstream_gpu`：通过（S4 branch 9/9 failures=0）
  - `mesh_limits_group_matches_s3_upstream_gpu`：**失败**（panic：U7/U8 pass_rule 全条件不一致 21 条）

## 2. 失败明细（comparator FAIL 行，原文）

- U7 `mesh-limits-cone-box`：4 条
  - `frame[0] dev=-7.450576e-8 ref=7.450576e-8`（符号相反）
  - `geom[0][0] dev=0 ref=1`、`geom[1][0] dev=1 ref=0` → 设备 `contact.geom=[0,1]`，参考 `[1,0]`
  - `contact.geom 顺序应为 pinned 类型序 [1,0]，实际 [0,1]`
  - 可观测子集仍成立：count=1、dist 4 ULP、pos x/y 0 ULP、z 1 ULP。
- U8 `mesh-limits-prism-box`：17 条
  - `contact.geom=[0,1]` vs 参考 `[1,0]`（同 U7）
  - `frame[2] dev=1e0 ref=-1e0`（法向 z 符号翻转，逐行重复）
  - `pos[0]` 与参考逐行配对不一致（±0.05 符号交换，4 行），而 A2-3 已证 pos 四角为集合相同、行序轮换
  - dist 4/4 仍 0 ULP；count=4 与参考一致。
- 汇总：21 条（U7 4 + U8 17）。

## 3. 窗口工具自身缺陷（如实记录，不改判据）

- `results.json` 的 `exit_code` 为 **null**：本 runner 为做 0.25 s 采样未用 `Start-Process -Wait`，
  而本机 PowerShell 下 `-PassThru` 无 `-Wait` 时 `ExitCode` 读回为空（用 `cmd /c exit 3` 复现：空；加
  `-Wait` 为 3）。**失败事实由实体日志 `test result: FAILED` + `.err` panic 行机证**，不依赖该空值。
- `SUMMARY entities=1 failed=4` 计数错误：`$results` 元素是 `[ordered]@{}`（OrderedDictionary），
  `Where-Object | .Count` 取到字典键数 4。实际失败实体数 = 1（日志可证）。
- 处置：**不重跑窗口**（fail-stop 不重试）；runner 修复（.NET Process 直读 ExitCode + 数组强转计数）
  归入下一次 0 窗修正片，与新 exe 重冻一起做。

## 4. 根因初判（只读代码 + 参考事实；待 Arch/Chief 确认）

- S4 `LIMIT_CASES` 的 meta 固定 `g1=0, g2=1`、`type1=geom_type[0]`、`type2=geom_type[1]`
  （`tests/support/branch_closure_case.rs::build_meta`）；U7/U8 的 `geom_type=[7(mesh),6(box)]`，
  世界 geom 顺序为 0=mesh、1=box（fixture XML 实查）→ 设备按调用方顺序写 `contact.geom=[0,1]`。
- 上游参考（S3 `mjw.collision` 捕获）是**类型序**配对：`contact.geom=[1,0]`（box 6 在前、mesh 7 在后）；
  M2 的目标正是「调用方按 pinned 类型序传 g1=箱、g2=mesh，`type1=6, type2=7` → `contact.geom=[1,0]`」
  且 M2 全字段 comparator 通过。
- 因此本窗失败的最可能原因是**用例输入配对顺序与上游捕获顺序不一致**（test-case 侧），不是产品内核
  新差异：S4 record-only 只比 count/dist/pos 时该差异不可见，全条件 comparator 把 `geom`/`frame`/pos 配对
  暴露出来。**未验证前不作结论**；需在 0 窗修正片改 case 配对顺序并重新窗口验证。
- 若改 case 顺序后仍失败（如 frame/零容差字段），则需另立案做产品侧对齐；届时按新事实重报。

## 5. 建议的下一步（待批准，未执行）

1. **0 窗修正片**（新卡）：`tests/support/branch_closure_case.rs` 增加逐 case 配对顺序（U7/U8 改为
   `g1=1(box), g2=0(mesh)`、`type1=6, type2=7`，与 M2 同约定；`contact.geom` 期望 `[1,0]`），
   保留 `RECORD-ONLY` 打印与 comparator 断言；不改 comparator/判据/容差/参考。
2. 四门禁 + 重建 branch exe（**新 exe 身份 → 必须重冻**，r3 冻结轮，旧 r2 与 A2-3 冻结保留）+
   修复 runner 的 ExitCode/计数缺陷（只改 target 工作材料，不入库）。
3. 重报窗口就绪（新 exe sha/bytes、18 守卫、预计时长、fail-stop 第 6 条），由 Chief 按四前提开新窗。
4. 本失败窗证据作为恢复记录 append-only 落库（在修正片或收口时），**不宣称 U7/U8 通过、不宣称产品分歧**。

## 6. 非声明

- 本记录不是通过声明；不宣称 U7/U8 pass_rule 全条件通过、U8 已验收、mesh 容量缺口已清、与上游等价
  （over-32）、G10@P5 整族通过、NP01/NP02 门禁通过。
- 失败已按 fail-stop 停：未重试、未放宽容差、未改参考/判据、未动卡状态、未写仓。
