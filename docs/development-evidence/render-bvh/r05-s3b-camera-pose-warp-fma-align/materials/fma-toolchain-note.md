# `f32::mul_add` 在本工具链**不是正确舍入**（改用项目精确 FMA 的机器证据）

## 结论（先给）

工具链：`rustc 1.99.0 (b940084d7 2026-09-28)`、host `x86_64-pc-windows-gnu`、`cargo 1.99.0`。
本工具链上 `f32::mul_add` **在 opt0 与 opt3 运行时都不是 IEEE 单次正确舍入**（QA 观察 O3 独立复核；
本仓最小复现实测见下）。因此本片收缩序**不能用 `f32::mul_add`**。

## 最小复现（无外部依赖）

源码（与 `target/render-bvh/r05-s3b-window/ptx-order/dbg2.rs` 同型，`black_box` 阻止常量折叠）：

```rust
fn main() {
    let inp = std::hint::black_box([0x3f3d3c57u32, 0x3f01dbed, 0x3e805783, 0x3ebb0653]);
    let (q0, q1, q2, q3) = (
        f32::from_bits(inp[0]), f32::from_bits(inp[1]),
        f32::from_bits(inp[2]), f32::from_bits(inp[3]),
    );
    let q11 = q1 * q1;                                   // 0x3e83bec4
    let s0 = q0.mul_add(q0, q11);                        // 0x3f4dc189
    let s1 = (-q2).mul_add(q2, s0);                      // 正确：0x3f3daba1
    let e0 = (-q3).mul_add(q3, s1);                      // 正确：0x3f1b8312
    let e2 = 2.0 * q1.mul_add(q3, q0 * q2);              // 正确：0x3f3dbdb7
    println!("q11={:08x} s0={:08x} s1={:08x} e0={:08x} e2={:08x}",
             q11.to_bits(), s0.to_bits(), s1.to_bits(), e0.to_bits(), e2.to_bits());
}
```

## 实测输出（同一源码、同一输入）

| 构建 | `s1` | `e0` | `e2` | 与正确值 |
| --- | --- | --- | --- | --- |
| `rustc`（opt0，`black_box` 阻止折叠，运行时） | `0x3f3daba0` | `0x3f1b8311` | `0x3f3dbdb8` | **差 1 ULP（错误舍入）** |
| `rustc -O`（opt3，常量可折叠） | `0x3f3daba1` | `0x3f1b8312` | `0x3f3dbdb7` | 正确（**常量折叠**掩盖） |

**单步反例（QA O3 口径）**：`fma(-q2, q2, s1)` 的正确单次舍入应为 **`0x3f3daba1`**，运行时实得
**`0x3f3daba0`**（1 ULP）。

## 归因更正（QA 观察 O3）

- 旧说法「debug 构建差、release 对」**不准确**；
- 实际是 **`f32::mul_add` 在本工具链运行时（opt0/opt3 皆然）非正确舍入**；
  `-O` 下看到的「正确」来自**常量折叠**（编译器用精确值算出了正确结果），不是运行时行为正确；
  opt0 无常量折叠，因此暴露差异。差异源 = **常量折叠 vs 运行时**，不是 debug/release 模式。
- 对生产代码的含义更强：任何依赖 `f32::mul_add` 的位级门禁都可能在「不折叠」的输入/构建下漂移，
  必须显式使用精确 FMA。

## 解决

改用项目既有精确 FMA：`crate::math::fma::fma_f32`（`src/math/fma.rs`，本片未改该文件）
- 主路：CPU 支持 FMA 时内联汇编 `vfmadd231ss`（本机支持；单指令 = IEEE 单次舍入）；
- 兜底：整数 576 位精确模拟 + RNE（无 FMA 机器仍单次舍入）；
- opt0/opt3 同一路径（不依赖编译器折叠）；`src/math/mod.rs` 零改动。

改后 `cargo test --locked --lib derived_contraction…` 通过；`cargo test --locked` 全量
**1092 passed / 0 failed / 389 ignored**（父提交 1091，本片 +1 条单测）。

## 对本片结论的影响

- 本片所有位级对照（`materials/cam-xmat-bits.json`、`materials/sampling-20.tsv`、
  `materials/verify-contraction.log`）由独立 Python 脚本用 Fraction 精确 FMA 复算，与 Rust 实现同序；
- Rust 单测用 `fma_f32` 固定 4 条案例（恒等 / hfield cam-side / scene-a cam-persp / 随机反例），
  不依赖构建 profile。
