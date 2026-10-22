# G01 rustdoc 断链修复（W-D3）

任务：`2026-10-22-g01-wd3-rustdoc-links`
基线：`a8a4eec`
范围：三处文档文字/转义；不改代码语义、不放宽门禁、不新增 allow 豁免。

## 根因

三处中文量纲记号写在方括号中，被 rustdoc 当作 intra-doc link：

| 文件 | 原文 | 修复后 |
| --- | --- | --- |
| `src/physics/kinetic_energy.rs` | `返回[势能,动能]；` | ``返回`[势能,动能]`；`` |
| `src/physics/mass_matrix/multiply/borrowed/device_energy/plan.rs` | `返回历史[势能,动能]，` | ``返回历史`[势能,动能]`，`` |
| `src/physics/mass_solve/sparse/resident/solve.rs` | `行优先[RHS,nv]，` | ``行优先`[RHS,nv]`，`` |

## 验证

| 命令 | 结果 |
| --- | --- |
| `RUSTDOCFLAGS=-D warnings cargo +1.99.0-x86_64-pc-windows-msvc doc --locked --all-features --no-deps` | 0 警告，退出码 0 |
| `RUSTDOCFLAGS=-D warnings cargo +1.99.0-x86_64-pc-windows-msvc doc --locked --features cuda-probe --no-deps` | 0 警告，退出码 0 |
| 门禁 fmt/clippy/clippy-cuda/unit | 见 `normify gate` 记录 |

## 边界

- 只改三行文档文字为代码文本（反引号），无逻辑、无断言、无容差、无 fixtures 变更。
- 未新增 `allow(rustdoc::broken_intra_doc_links)` 或任何豁免。
- 与 W-D5 历史基线 3 项、W-D4 重力位证书无关，不顺带处理。
