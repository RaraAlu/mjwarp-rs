# R06-S3b⑦ attempt6 统计打印（test-only 只读增强）

任务：`2026-11-04-render-r06-s3b-flex-attempt6-stats-print`（owner Backend）。
目的：attempt6 判据 PASS 但成功路径不打印逐相机/allowlist 数值明细（attempt6 README §6 已如实记录缺项）；
本卡只加**只读打印**，不改变任何断言、阈值、判据文本、容差、参考或 `fixtures/`，零 GPU、不重跑 attempt6、不改 attempt6 已收口结论。

## 1) 实现（两文件）

- `tests/support/render_bvh/allowlist.rs`（156→196 行）：新增
  - `format_depth_stats(camera, width, rendered, expected_bits) -> (String, Option<String>)`：纯函数，逐相机算
    `(mismatched, beyond4, max_ulp)`；cam2 额外生成 allowlist 像素明细行（bits/expected/ULP/beyond4/flexid）。
  - `print_depth_stats(...)`：GPU 成功路径调用，逐行 `println!`。
  - CPU 自证 `depth_stats_lines_format_cpu`：用 `cpu_depth_case` 合成输入断言两行格式（含 `ulp=3`、`beyond4=0`、
    `flexid=0`，并断言 cam0 不打印 allowlist 行）。
- `tests/render_flex_validate.rs`（495→498 行，≤500 棘轮）：`compare()` 内 3 行调用/打印（参考 flex 覆盖数、
  逐相机 depth 统计、逐相机 seg 结论）；**未改任何断言**。

## 2) 打印格式（逐字；`--nocapture` 下可见）

```text
R06-S3b reference flex px per cam: [0, 0, 654]
R06-S3b depth cam0: mismatched=0 beyond4=0 max_ulp=0
R06-S3b depth cam1: mismatched=0 beyond4=0 max_ulp=0
R06-S3b depth cam2: mismatched=1 beyond4=0 max_ulp=3
R06-S3b allowlist cam2 (134,128): bits=0x… expected=0x… ulp=3 beyond4=0 flexid=0
R06-S3b seg cam2: mismatch=0
r06-s3b③[flex] two-run sha256 identical rgb=… depth=… seg=…     ← 既有行，未改
```

注：`mismatched` 按**全像素**统计（含 allowlist 像素）；`beyond4` 仍必须为 0。上面的数值为 attempt6 已知结果
（depth cam2 `(1,0)`/max_ulp 3，allowlist 像素即 `(134,128)/flexid0`）的**预期示意**；实际值由下一次窗口运行的
`--nocapture` 输出给出，本卡不重跑、不产生新判据结论。

## 3) CPU 自证（零 GPU）

```text
cargo test --locked --features cuda-probe --test render_flex_validate -- --test-threads=1
→ running 19 tests: 18 passed, 0 failed, 1 ignored（ignored=GPU 用例，未执行）
```

## 4) 边界与纪律

- 不改判据/容差/参考/guards/`fixtures/`；未开 GPU 窗口、未重冻、未重跑 attempt6；attempt6 收口结论（`db082561` 等）不受影响。
- 门禁：fmt / clippy / clippy-cuda / unit + `unchanged fixtures/`。
