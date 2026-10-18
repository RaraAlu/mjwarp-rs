# 默认库测试修复

日期：2026-10-18。
任务：`2026-10-18-default-lib-test-fix`。
基线提交：`330132cbb2509e19977a2451c08f31ffe4232507`。
本项不需要GPU。
本项不覆盖NP01至NP04。

## 根因

默认配置下库测试无法编译。
`src/physics/fluid_force.rs`定义常量`CUDA`。
该常量只在`cuda-probe`下可见。
宿主测试`ellipsoid_constants_and_projection_order_match_frozen_source`
在两行读取它的源码文本。
默认配置因此报两处`E0425`。

## 修改

可见条件改为`cfg(any(test, feature = "cuda-probe"))`。
未屏蔽任何测试。
内核字符串、浮点常量、配方与ABI均未改动。
完整差异见`development-evidence/default-lib-test-fix/diff.log`。

## 验证

| 命令 | 工具链 | 结果 |
| --- | --- | --- |
| `cargo fmt --check` | stable-x86_64-pc-windows-gnu，rustc 1.99.0 | 通过 |
| `cargo test --locked --lib` | stable-x86_64-pc-windows-gnu，rustc 1.99.0 | 232通过，0失败，0忽略 |
| `cargo +stable-x86_64-pc-windows-msvc test --locked --features cuda-probe --lib` | stable-x86_64-pc-windows-msvc，rustc 1.99.0 | 262通过，0失败，83忽略 |

默认配置中4项流体宿主测试全部运行并通过。
`cuda-probe`配置的83项忽略为既有GPU测试。
本项未运行GPU测试。

## 证据

| 文件 | 内容 |
| --- | --- |
| `toolchain.log` | 活动工具链与版本 |
| `fmt.log` | 格式检查输出 |
| `diff.log` | 修改范围 |
| `lib-default.log` | 默认配置库测试完整输出 |
| `lib-cuda-probe.log` | `cuda-probe`配置库测试完整输出 |

以上文件位于`docs/development-evidence/default-lib-test-fix/`。

## 未覆盖项

- 默认配置的clippy诊断仍未处理。
- 集成测试未在本项重跑。
- Linux未验证。
