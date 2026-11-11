# NP5-01a RE-0 复核（QA）：参考侧 PTX 离线重生成证据入库

- 复核对象：卡 `2026-11-11-np5-01a-re0-evidence`；内容提交 **`d8f5402d`**（docs-only）。
- 性质：只读复核（零 GPU、零窗、不改判据/参考/容差）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 2 低 + 3 观察）**；判级 **`G1-partial PASS`（双门 `NORMALIZED_EQUAL`）**成立，措辞口径正确。

## 1. 判级与两门结果（独立复现，非引用自述）

- **我独立重跑入库的 `scripts/re0_compare.py`** 对 G1/G2 各一次（CPU-only，无 GPU）：
  - G1：`grade=NORMALIZED_EQUAL`、bytes 618,096 vs 637,336、entries 1 vs 1、streams/multisets/counts 全 `True`；
  - G2：`grade=NORMALIZED_EQUAL`、bytes 965,105 vs 993,601、entries 48 vs 48、streams/multisets/counts 全 `True`。
  与提交的 `g1-compare.json`/`g2-compare.json` 与 receipt 完全一致。
- **计数两种口径我独立复算**（自写 30 算子词表 + wide 口径）：G1 arith **13,036 = 13,036**、wide **15,915 = 15,915**；G2 arith **18,883 = 18,883**、wide **24,735 = 24,735**——与 README §2/receipt 一致；interim 15,001/23,888 的**更正记录在位**（`counts_correction`）。
- **`bytes_equal=false` 已如实记录**（G1/G2 皆 false），未出现「字节相等」正面结论。
- **5 类残余差异**（发射顺序 / mangled 符号 / `__local_depotN` / 行尾 CRLF vs LF+NUL / 无算术差异）为非算术类，与 `diff_types`、per-entry 差异为空（`per_entry_diff:{}`）互相印证。
- **G3 确定性**：`target/` 内两次运行产物我实测 sha 相同（`g1_render.ptx`==`g1_render_b.ptx` = `0e8cc11c…`；`g2_smooth.ptx`==`g2_smooth_b.ptx` = `dbef2d41…`），与 receipt `g1_sha_runs`/`g2_sha_runs` 一致。

## 2. `fast_math=False` 前提修正的证据链（独立核）

- 冻结参考件我实测：`reference_module.sm89.ptx` = **637,336 B / `ad21b93f…`**、`smooth.ptx` = **993,601 B / `4179acd0…`**，与 README/receipt/provenance 一致。
- 前提修正证据我独立复核：两份冻结参考 PTX **`ex2`=0、`lg2`=0**，且含 `rcp`（96/11）与 `rsqrt`（3/11）——与「fast_math=True 时 `powf→ex2/lg2` 会多出 7+7 条」的对照论证一致；「去掉 `--use_fast_math` 后 G1 立即 NORMALIZED_EQUAL」与 `provenance.config.fast_math=false` + `compile_flags`（无 `--use_fast_math`）自洽。
- provenance 齐全且固定：warp 1.15.0、mujoco_warp 3.15.0、upstream pin `71da24d9…`、NVRTC 12.9（CL-36037853/V12.9.86，dll `0cc63349…`）、sm_89、release、`lineinfo=false`、`enable_backward=false`、`gpu_used=false`。

## 3. 证据/manifest/可复现性/边界

- `manifest.txt` **10/10** 全对（我独立重算 sha256+bytes；不自列；集合与目录 10 个非 manifest 文件完全相等）；README §6 的 8 个自列哈希我逐个复核全对，G2 `gen_cu` sha `66fe2c8d…` 亦对。
- **两份生成 PTX 未入仓**：只记 `sha256`+bytes（`0e8cc11c…`/618,096、`dbef2d41…`/965,105）+ 重生成命令 +「target 工作材料、可由入库脚本+pinned 环境确定性重生成」；本轮我以 target 现存产物重跑 compare 成功，可复现性说明与实现一致。
- **脚本入仓可读**：5 个脚本均在 `docs/development-evidence/np-gate/re0/scripts/`（**不是** 仓库 `scripts/` 写域），`re0_compare.py` 归一化逻辑（去注释/谓词、寄存器与符号归一、entry 集 + 有序流 + 多重集 + 计数）与 grade 分支实现完整。
- 门禁：fmt/clippy/clippy-cuda exit 0、unit **1,406 / 0 / 402**；边界仅 docs evidence + `.normify`，无 `src/tests/fixtures/scripts` 改动；全部文件 UTF-8/LF、无 BOM/CR/TAB/NUL。
- **措辞**：README/provenance/receipt 的「等价」只出现在「归一化指令流等价」与「不写字节相等/不写位级等价/不宣称上游等价」等否定句，无正面字节/位级结论。

## 4. 发现

### 低
- **L1 残余差异计数口径**：`receipt.residual_diff_types` 列 **6 条**，而 README/提交说明写「**5 类**」——其中 `[4] no arithmetic difference` 与 `[5] no arithmetic difference (per-entry arithmetic streams equal)` 是同一类的两种措辞；建议合并为 5 条或统一口径。
- **L2 脚本硬编码本机路径**：`scripts/re0_g2_codegen.py`（含 `D:\Rust\mjwarp-rs\target\mjwarp-parallel\venv\Scripts\python.exe`）、`scripts/a1_unique_codegen.py`、`scripts/a1_tile_codegen.py` 均硬编码 `D:\Rust\mjwarp-rs…`；README §5 的重生成命令写的是相对路径、未说明该硬编码，换路径/换机不能直接重跑。建议参数化根路径或在 README 注明「脚本按本机路径固定」。

### 观察
- **O1**：`a1_unique_codegen.py`/`a1_tile_codegen.py` 在 RE-0 目录入库，但 RE-0 README 未说明其用途（属 A-1a addenda R3 引用的「A1 codegen」路线，服务于 RE-1 的 device 级升级）；建议在 README 注明产物与状态（是否已成功生成 tile/unique PTX），或随 RE-1 一并说明。
- **O2**：README §6 自列行 sha 写「(computed at write)…」占位（manifest 内有真值）；可接受，但建议改为自引用说明。
- **O3**：README §1 表述「两侧算术指令流等价」与裁定固定措辞「归一化指令流等价」并用；含义相同、但建议统一为固定措辞，减少被误读为「算术内容位级相等」的空间。

## 4b. 处置（Chief 2026-11-11 裁定）

- 低项/观察项（由 RE-1 卡 append-only 处置，本卡按记录保留）：
  - L1 residual_diff_types 5/6 口径统一（由 RE-1 卡修正）
  - L2 脚本绝对路径说明（由 RE-1 卡修正）
  - O1 a1_* 脚本用途说明（由 RE-1 卡修正）
  - O3 措辞统一为「归一化指令流等价」（由 RE-1 卡修正）

## 4b. 处置（Chief 2026-11-11 裁定）

- 低项/观察项（由 RE-1 卡 append-only 处置，本卡按记录保留）：
  - L1 residual_diff_types 5/6 口径统一（由 RE-1 卡修正）
  - L2 脚本绝对路径说明（由 RE-1 卡修正）
  - O1 a1_* 脚本用途说明（由 RE-1 卡修正）
  - O3 措辞统一为「归一化指令流等价」（由 RE-1 卡修正）

## 5. 非声明

本复核确认 RE-0 判级与计数可独立复现、provenance 与可复现性说明成立；**不宣称**字节/位级等价、NP01 通过、上游等价或双平台，不改判据/参考/容差。
