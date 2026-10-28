# `f32::mul_add` 在本工具链 debug 构建中不是单次舍入（改用项目精确 FMA 的机器证据）

## 现象

收缩序初版用 `f32::mul_add` 实现，`cargo test --locked --lib` 单元测试失败：

```text
left : [1058767633, 3197258443, 1061010872, ...]   # 0x3f1b8311 / 0x3f3dbdb8 ...
right: [1058767634, 3197258443, 1061010871, ...]   # 0x3f1b8312 / 0x3f3dbdb7 ...
```

同一表达式在独立 `rustc` 小程序（2024 edition，默认 x86_64-pc-windows-msvc 工具链）中：

| 构建 | `e0` bits | `e2` bits | 结论 |
| --- | --- | --- | --- |
| `rustc -O`（常量折叠） | `0x3f1b8312`（正确） | `0x3f3dbdb7`（正确） | 与 GPU 位等 |
| `rustc`（debug，`black_box` 阻止常量折叠） | `0x3f1b8311`（-1 ULP） | `0x3f3dbdb8`（+1 ULP） | **非单次舍入** |

即 debug 构建在无 FMA 指令的软件路径上，`f32::mul_add` 的实现等价于「先乘后加两次舍入」，
debug 与 release 会给出不同位型；若用它实现收缩序，位级门禁将随构建 profile 漂移。

## 解决

改用项目既有精确 FMA：`crate::math::fma::fma_f32`（`src/math/fma.rs`）
- 主路：CPU 支持 FMA 时内联汇编 `vfmadd231ss`（本机支持；单指令 = IEEE 单次舍入）；
- 兜底：整数 576 位精确模拟 + RNE（无 FMA 机器上仍单次舍入）；
- debug/release 同一路径；`src/math/mod.rs` 未改动（该模块只做 `pub(crate) mod fma;`，本次零 diff）。

改后 `cargo test --locked --lib derived_contraction...` 通过，`cargo test --locked` 全量
**1092 passed / 0 failed / 389 ignored**（父提交 1091 passed，本片 +1 条单测）。

## 对本片结论的影响

- 本片所有位级对照（`materials/cam-xmat-bits.json`、`materials/sampling-20.tsv`、
  `materials/verify-contraction.log`）由独立 Python 脚本用 Fraction 精确 FMA 复算，与 Rust 实现同序；
- Rust 单测用 `fma_f32` 固定 4 条案例（恒等 / hfield cam-side / scene-a cam-persp / 随机反例），
  保证 debug 与 release 位型一致。
