# 工具链锁定与静态诊断治理

日期：2026-10-18。
任务：`2026-10-18-toolchain-clippy-governance`。
基线提交：`775f63c57996e13c0a2ecbeb8a7928f9434827cb`。
平台：Windows，RTX 4070 Ti SUPER，驱动596.36。
本项不覆盖NP01至NP04。
治理完成不等于全产品验收。

## 结论

工具链锁定为Rust 1.99.0。
71项诊断逐项处置。
39项修复，22项窄范围豁免。
6项因冻结证据暂缓。
4项位于范围外的集成测试。
GPU测试结果与内核摘要前后一致。
约50条证据记录的源码身份转为历史。

| 门禁 | 基线 | 当前 |
| --- | ---: | ---: |
| 默认配置`clippy -D warnings` | 23处（短格式24行） | 4处，均为暂缓项 |
| `cuda-probe`配置`clippy -D warnings` | 52处 | 3处，均为暂缓项 |
| `cuda-probe`全目标清单（警告级） | 未覆盖集成测试 | 7处：暂缓3、范围外4 |

两项门禁仍未转绿。
剩余项全部列在下文。

## 工具链

### 基线

| 项目 | 值 |
| --- | --- |
| 活动工具链 | `stable-x86_64-pc-windows-gnu`（rustup默认） |
| MSVC工具链 | `stable-x86_64-pc-windows-msvc` |
| rustc | 1.99.0（b940084d7 2026-09-28） |
| clippy | 0.1.99 |
| 锁定文件 | 无 |
| `rust-version` | 未声明 |

见`development-evidence/toolchain-clippy-governance/baseline-toolchain.log`。

### 锁定

| 文件 | 内容 |
| --- | --- |
| `rust-toolchain.toml` | `channel = "1.99.0"`，组件`rustfmt`、`clippy`，`profile = "minimal"` |
| `Cargo.toml` | `rust-version = "1.99"`，只声明已验证最低版本 |
| `scripts/*.ps1` | 32处`+stable-*`改为`+1.99.0-*`；1处报告字段同步 |

锁定版本等于原`stable`的实际版本。
未降级，未改宿主目标与平台边界。
`stable`不是固定版本，不能用作锁定。

Rara新增两套工具链：

- `1.99.0-x86_64-pc-windows-gnu`
- `1.99.0-x86_64-pc-windows-msvc`

原`stable`工具链保持不变。
MSVC首次安装中断，已卸载残缺目录后重装。

### 选用核验

仓库内实际选用如下：

```text
1.99.0-x86_64-pc-windows-gnu (overridden by 'D:\Rust\mjwarp-rs\rust-toolchain.toml')
rustc 1.99.0 (b940084d7 2026-09-28)
```

`scripts/test-windows-transfers.ps1`已实际运行。
它以`+1.99.0-x86_64-pc-windows-msvc`构建。
10项测试通过；报告字段记为`1.99.0-x86_64-pc-windows-msvc`。
见`script-transfers.log`。

Linux脚本直接调用`cargo`。
它们将遵循`rust-toolchain.toml`。
Linux未安装与验证1.99.0，属未覆盖项。

## 诊断处置

逐项清单见`diagnostics.json`。
每项记录文件、行号、规则、配置与处置理由。

| 处置 | 数量 | 说明 |
| --- | ---: | --- |
| 修复 | 39 | 语义等价改写 |
| 窄范围豁免 | 22 | 单项、单循环或单测试函数 |
| 暂缓（冻结证据） | 6 | 文件恢复为HEAD |
| 范围外 | 4 | 需先扩展任务范围 |

### 修复

| 规则 | 数量 | 做法 |
| --- | ---: | --- |
| `drop_non_drop` | 26 | `drop(x)`改为`let _ = x`；调用副作用保留 |
| `needless_range_loop` | 3 | 改为`enumerate`或切片`zip`；顺序不变 |
| `useless_vec` | 2 | 改为同值数组 |
| `type_complexity` | 3 | 同条件编译的类型别名 |
| `collapsible_if` | 2 | 合并为let链；条件与错误不变 |
| `unnecessary_cast` | 1 | 删除i32到i32转换 |
| `manual_is_multiple_of` | 1 | 改用`is_multiple_of` |
| `chunks_exact_to_as_chunks` | 1 | 改用`as_chunks::<3>().0`；余数仍忽略 |

`drop_non_drop`只替换被标记的位置。
`device_energy.rs`另有3处`drop`。
它们丢弃实现`Drop`的值，含`drop(session)`。
这3处保持原样。

### 窄范围豁免

| 规则 | 数量 | 范围 |
| --- | ---: | --- |
| `dead_code`（默认配置） | 12 | 逐项`cfg_attr(not(feature = "cuda-probe"), allow(dead_code))` |
| `excessive_precision` | 8 | 1个测试函数与4个常量条目 |
| `needless_range_loop` | 2 | 同一下标读取多组数组的单个循环 |

默认配置没有设备路径。
相关条目只在`cuda-probe`下读取。
豁免不改变`cuda-probe`构建。

`fluid_force.rs`的冻结常量共8处：

- 测试`ellipsoid_constants_and_projection_order_match_frozen_source`中4处。
- 宿主参考实现的4个常量条目中4处。

交接记录写为4处。
`cuda-probe`配置另有4处，值相同。
全部字面量与位型断言未改。
豁免不作用于整个文件。

### 暂缓：冻结证据保护

| 文件 | 诊断 |
| --- | --- |
| `src/physics/gravity_compensation.rs` | `needless_range_loop`、`unused_imports`、`dead_code`、`useless_vec` |
| `src/physics/resident.rs` | `dead_code`、`needless_range_loop` |

两文件的整文件摘要受测试强制。
测试为`frozen_nonzero_product_chain_contract_is_self_consistent`。
证据包为`fixtures/gravity-upstream-abi/nonzero-product-chain-package.json`。
HEAD上该测试通过；修改两文件后失败。
因此两文件恢复为HEAD。
处理须另立证据重绑任务。

### 范围外

| 文件 | 诊断 |
| --- | --- |
| `tests/mixed_mass_solve.rs` | `manual_is_multiple_of`、`needless_range_loop`×2 |
| `tests/mixed_mass_factor.rs` | `needless_range_loop` |

基线时库目标先报错。
cargo因此未检查集成测试目标。
库目标清零后，这4项首次出现。
暂缓项存在时，`-D warnings`仍会遮蔽它们。
完整清单改用警告级、`--keep-going`收集。

## 验证

### GPU测试与内核摘要

改动前后各运行全部GPU测试。
范围为68个测试程序的`--ignored`用例。
记录器捕获每次NVRTC编译的源码与PTX摘要。

| 对比项 | 改动前 | 改动后 |
| --- | ---: | ---: |
| 测试程序 | 68 | 68 |
| 计数或失败名单不同的程序 | — | 0 |
| 内核摘要（入口、源码、PTX） | 279 | 279，逐项相同 |
| 既有失败 | 27项，10个程序 | 相同27项 |

比较不含耗时。
改动后运行时，两暂缓文件仍为修改状态。
两文件随后恢复为HEAD，即改动前状态。
见`gpu-compare.json`与两份`gpu-*-digests.txt`。

### 宿主测试

| 配置 | 结果 |
| --- | --- |
| 默认`cargo test --locked --no-fail-fast` | 303通过，0失败 |
| `cuda-probe`同上 | 437通过，19失败，237忽略 |

19项失败在HEAD上同样失败。
它们来自既有源码漂移：

- `resident.rs`：HEAD摘要`3bf476…`，多项T4测试固定`b2d63a…`。
- `src/io/inertial.rs`：本项未改，HEAD即已漂移。

无新增失败。
见`final-host-tests-*.log`。

## 证据身份失效登记

修复涉及14个物理源文件。
50条证据记录绑定这些文件的HEAD整文件摘要。
修改后这些记录转为历史。
没有测试强制这些记录。
内核源码与PTX摘要不变。
GPU与宿主测试结果不变。

涉及10组证据：

```text
applied-force
borrowed-mass-multiply
device-energy
fluid-force
joint-passive
joint-passive-rotation
kinetic-energy
passive-force-sleep
resident-mixed-mass
tendon-velocity
```

逐条记录及新旧摘要见`evidence-pin-impact.json`。
原证据文件未修改。
按新源码重新登记须另立任务。

## 过程纠错

以下问题已在提交前纠正：

1. 批量替换多改了3处`drop`。
   它们丢弃实现`Drop`的值。
   已按行号恢复原样。
2. 类型别名插入位置错误。
   参考函数一度失去条件编译。
   已恢复`cfg(feature = "cuda-probe")`。
3. 宏调用语句上的`allow`无效。
   已改为测试函数级豁免。
4. 首次HEAD基线未切换目录，结果作废。
   原因是worktree路径过长。
5. 第二次HEAD基线混入换行差异。
   新检出受`core.autocrlf=true`影响。
   改用HEAD对象摘要直接核对固定值。

## 新发现

- 集成测试从未经过clippy检查。
- 新检出工作区按CRLF落盘。
  摘要固定类测试在新检出上额外失败。
  清洁部署复现可能受影响。
- T4多项源码固定已在HEAD上失效。

## 未覆盖项

- Linux未安装与验证1.99.0。
- release配置未运行。
- 6项暂缓诊断待证据重绑。
- 4项范围外诊断待扩展范围。
- 50条证据记录待重新登记。
- NP01至NP04未验收。

## 证据文件

位于`docs/development-evidence/toolchain-clippy-governance/`：

| 文件 | 内容 |
| --- | --- |
| `baseline-toolchain.log`、`final-toolchain.log` | 前后工具链 |
| `baseline-clippy-*.json` | 基线逐项诊断 |
| `final-clippy-*.json` | 当前`-D warnings`门禁 |
| `final-inventory-*.json` | 当前全目标警告级清单 |
| `diagnostics.json` | 71项处置 |
| `gpu-before-*`、`gpu-after-*`、`gpu-compare.json` | GPU结果与内核摘要 |
| `final-host-tests-*.log` | 宿主测试输出 |
| `script-transfers.log` | 锁版脚本实测 |
| `evidence-pin-impact.json` | 证据身份失效清单 |
