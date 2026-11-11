# RE-0 QA 跨片复核修正 addendum（卡 QA-ADD，2026-11-11；append-only，不改历史字节）

## ④ `residual_diff_types` 6 → 5 统一
`receipt.json` 的 `residual_diff_types` 有 6 条，最后两条是同一项的两种措辞：
[5] `no arithmetic difference` 与 [6] `no arithmetic difference (per-entry arithmetic streams equal)` → **同一项**。
统一 5 类（README 口径）：① 内核发射顺序（G2 多 entry 顺序不同、逐 entry 内容相等；G1 单 entry n/a）；
② mangled 内部符号名含源文件名/哈希；③ `__local_depotN` 编号不同；④ 行尾 CRLF vs LF + 尾部 NUL；
⑤ 无算术差异（逐 entry 算术流相等）。历史 receipt 字节不改，以本文件 5 类口径为准。

## ⑤ 脚本路径提示（§5 补充）
`scripts/re0_g2_codegen.py`、`scripts/a1_unique_codegen.py`、`scripts/a1_tile_codegen.py` 含**本机绝对路径**
（开发用，如 `D:\Rust\mjwarp-rs\...`、pinned venv/toolchain 路径）；移植到其他机器需改路径与 pinned 环境版本。

## ⑥ a1_* 脚本用途
`a1_unique_codegen.py` / `a1_tile_codegen.py` 是 **A1 离线 codegen 路线**：从 warp 工厂内核实例化
`@wp.kernel(module="unique")` 内层内核并生成参考侧 .cu/PTX（对应 A1-final 的 6 族/12 变体）；
与 RE-0 的 G1/G2 判级脚本（`re0_compile.py` / `re0_compare.py` / `re0_g2_codegen.py`）用途不同。

## ⑦ 措辞
统一「**归一化指令流等价**」；不写字节相等/位级等价。
