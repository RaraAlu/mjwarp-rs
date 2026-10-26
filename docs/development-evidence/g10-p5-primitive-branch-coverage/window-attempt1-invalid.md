# B3 验证窗 attempt1：aborted（测试 support 层缺陷，非数值回退）

记录时间：2026-10-26 21:55–21:56（+08:00）。本文件只记录作废轮事实，不与 attempt2 拼接。

## 1. 结论

- attempt1 **aborted**：`g10_p5_primitive_branches_cuda` exit 101（panic），`plane` / `sphere_capsule` /
  `primitive` / `launcher` 4 个回归二进制 exit 0。
- panic 根因：`tests/g10_p5_primitive_branches_cuda.rs`（当时版本）把参考里的
  `geom_xpos`/`geom_xmat`（`[nworld][ngeom][3/9]` 三层数组）喂给只摊平两层的
  `flatten_f32`（`fields` 的 `[count][width]` 用）→ 第三层仍是数组 → `.expect("f32")` panic。
- **没有任何 per-case 数值比较执行**：panic 发生在第一个 case 的 device 运行之前；日志无
  `case: nacon=` 行。因此 attempt1 不产生任何分支/数值结论，也没有既有 case 位级回退证据。
- 修复只碰 `tests/`（加三层摊平 + 非 ignored 的 CPU 支持层自检测试）；判据/期望/参考/fixtures/
  `src/` 零改动。

## 2. 窗口事实

- 开窗前基线：`3530, 12532, 30~32, 31, 596.36`（free ≥ 10240、`compute-apps` 无本任务进程）。
- 签名集合（窗口内逐字节核对一致，窗口内零编译）：
  branches `1e3c2adf`、plane `115b5c99`、B2 `280f2e47`、primitive `ec56ff74`、launcher `84ee6074`。
- `window_start=2026-10-26T21:55:42.076+08:00`，`window_end=21:56:23.829+08:00`，body wall 37.1 s。
- 采样（0.25 s，141 samples）：`used_peak=4672`、`free_min=11390`、`util_max=68`、
  `delta_peak=1203 MiB`（limit 3072）、`util_streak_max=0`、`residual=0`、`process_exited=True`；
  guard post / 聚合断言无 abort。
- post 快照 `3657,12405,27,31`；结束基线 3 点 `3657,12405,27 / 3655,12407,34 / 3659,12403,30`；
  post `query-compute-apps` 无本任务进程。

## 3. 同窗回归（有效证据，但 attempt2 会按重签集合复跑）

- B1 plane 6/6：`failures=0`（plane-sphere/capsule/ellipsoid/cylinder nacon=1、plane-box nacon=3、
  plane-mesh nacon=3）。
- B2 sphere/capsule 4/4：`failures=0`（nacon 全 1）。
- primitive 3/3：`bit_equal=80/80, max_dist_ulp=0, max_pos_ulp=0, max_frame_ulp=0, failures=0`。
- launcher 3/3：`max_*_ulp=0, failures=0`（bit_equal 80/80、3122/3122）。
- 结论：既有 13 组合在该窗口内**位级无回退**；但该窗口因 branches panic 判为 aborted，
  attempt2 会用重签后的 5 二进制复跑全部回归，最终证据以 attempt2 为准。

## 4. 复现与修复

- panic：`tests/support/primitive_compare.rs:48:35`，`flatten_f32` 的 `.expect("f32")`。
- 修复：新增 `flatten_f32_nested`（三层）供 `geom_xpos/geom_xmat` 使用；原 `flatten_f32` 仍只
  用于 `fields` 两层。新增 `reference_support_parses_without_gpu`（non-ignored，纯 CPU）锁定
  参考 sha、3 层长度、`fields` 行数/行宽、`reference_rows` 逐行解析、6 case 分支谓词。
- 原始日志：`target/g10-p5-primitive-branches-verify/attempt1/`（baseline/samples/facts/
  run-*.log/run-*.err.log）。
