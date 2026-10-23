# A10 measure release 计时断言修复

任务：`2026-10-23-a10-measure-release-timing-fix`
发现：QA 在 `2026-10-23-g01-final-acceptance-2` 的 release host-msvc 阶段发现（同轮 debug 14 阶段全绿）；来源批次 2 A10@P10 交付 `3757567`。
基线：`bb621d9`（任务登记提交 `8ad2ff9`）
范围：只改 `src/runtime/tooling/tests.rs` 一个测试的负载；不改产品路径、断言口径、排除清单、G01 脚本。

## 根因

`measure_is_headless_and_rejects_zero_iterations` 用 `measure(3, || counter += 1)` 断言 `nanos > 0`。
release 优化下 3 次自增的总耗时落在 `Instant` 计时粒度内（实测 0 ns），断言失败；debug 下耗时非 0，所以一直通过。
该断言意在验证「无头计时确有推进」，但对亚计时粒度的负载过于敏感。

## 修复

保留 `nanos > 0` 严格断言、迭代计数断言与零迭代拒绝意图，只把被测负载改为可测：

- 每次迭代执行 `10_000` 次 `std::hint::black_box(counter + 1)`，3 次迭代共 30,000 次自增；
- 断言 `counter == 30_000` 与 `nanos > 0`。

未采用 `nanos >= 0`、删除断言等放宽写法；未改 `measure` 产品实现。

## 验证（本机实测）

| 命令 | 修复前 | 修复后 |
| --- | --- | --- |
| release 定向：`cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --release --lib measure_is_headless_and_rejects_zero_iterations` | 退出 101，0 过 1 败（`assertion failed: nanos > 0`，`src/runtime/tooling/tests.rs:39`，见 `pre-fix-release-failure.log`） | 退出 0，1 过 0 败（`post-fix-release-pass.log`） |
| debug 定向：`cargo +1.99.0-x86_64-pc-windows-msvc test --locked --lib measure_is_headless_and_rejects_zero_iterations` | 1 过（未复现，历史上一直通过） | 退出 0，1 过 0 败（`post-fix-debug-pass.log`） |
| release 全量预检：`cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --release`（G01 host-msvc 阶段同命令） | — | 退出 0，全部 target 0 失败；lib 537 过 0 败 129 ignored；墙钟 43.9 s（`release-host-msvc-precheck.log`） |
| 门禁 fmt/clippy/clippy-cuda/unit | — | 全过；unit 480 过 0 败（见 `.normify/runs/2026-10-23-a10-measure-release-timing-fix/`） |

环境：`MJWARP_MUJOCO_ROOT`/`MJWARP_MUJOCO_DLL` 指向 `target/toolchains/mujoco-3.12.0/package`；`CUDA_PATH=target/toolchains/cuda-12.8.1`（仅构建需要）。

## 边界

- 只改该测试的负载与等价断言表达；未改 `measure` 产品实现、其它测试、容差、排除清单、G01 脚本。
- 未跑 G01 debug/release 全脚本；G01 双模式是否全绿由 QA 在修复后 HEAD 复跑终验判定，不在此宣称。
- 观察：`measure` 对亚计时粒度的极短负载返回 0 ns 属时钟粒度限制（本测试用可测负载规避），未改产品契约。
- 平台边界：仅 Windows；release/Linux 的其它阶段未在此任务内复跑。
