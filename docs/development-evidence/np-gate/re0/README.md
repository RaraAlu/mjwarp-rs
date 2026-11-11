# NP5-01a RE-0 证据入库：参考侧 PTX 离线重生成（G1-partial PASS；docs-only、零 GPU）

- 任务：`2026-11-11-np5-01a-re0-evidence`（owner Backend；p95；P8-渲染与BVH）。
- 来源：Arch `target/render-bvh/gl2-ptx-diff/refgen/re0-final/`（target 工作材料）+ 判级依据 Chief 2026-11-11
  三级裁定（`BYTES` / `NORMALIZED_EQUAL`（G1-partial PASS，只写「归一化指令流等价」）/ `FAIL`）。
- 边界：**docs-only、零 GPU、零窗**；本目录不改 `src/`/`tests/`/`fixtures/`/`scripts/`/判据/参考/容差。
- 结论口径：**G1-partial PASS**——两次门均 `NORMALIZED_EQUAL`（归一化指令流等价），
  **不写字节相等、不写位级等价**。

## 1. 三级判级与两门结果

| 门 | 生成 | 参考 | 判级 |
| --- | --- | --- | --- |
| **G1**（render 模块；NVRTC 12.9、sm_89、fast_math=False） | `g1_render.ptx` `0e8cc11c…`/618,096 B（**未入仓**） | `reference_cache/reference_module.sm89.ptx` `ad21b93f…`/637,336 B（既有冻结件） | **NORMALIZED_EQUAL** |
| **G2**（smooth 模块；enable_backward=False、block_dim=256） | `g2_smooth.ptx` `dbef2d41…`/965,105 B（**未入仓**） | `fixtures/gravity-upstream-abi/independent-inputs/smooth.ptx` `4179acd0…`/993,601 B（既有冻结件） | **NORMALIZED_EQUAL** |
| **G3**（确定性） | G1/G2 各跑两次 | 两次 sha 相同（G1 `0e8cc11c…`×2、G2 `dbef2d41…`×2） | 确定性成立 |

- 归一化比较口径：entry 集 + 逐 entry 算术指令流（序列、多重集、计数）；两侧 stream/count/multiset 全等，`bytes_equal=false`。
- 5 类残余差异（已定位、均为非算术差异）：① 内核发射顺序（G2 多 entry 顺序不同、逐 entry 内容相等）；
  ② mangled 内部符号名含源文件名/哈希；③ `__local_depotN` 编号不同；④ 行尾（参考 CRLF vs 生成 LF）+ 尾部 NUL；
  ⑤ **无算术差异**（两侧算术指令流等价）。

## 2. 计数口径与 interim 更正

| 口径 | G1 | G2 |
| --- | --- | --- |
| `arith_instr`（30 算子词表过滤，mov/cvt/setp 计入） | **13,036 = 13,036** | **18,883 = 18,883** |
| `instr_lines_wide`（去注释/谓词后的全部指令行） | **15,915 = 15,915** | **24,735 = 24,735** |

- **interim 计数更正**：早期 interim 报告引用的 G1 15,001 / G2 23,888 来自更松的过滤口径（非正式）；
  权威计数为 `arith` 13,036 / 18,883（另有 wide 口径 15,915 / 24,735）。以本表为准。
- `entries`：G1 1 = 1；G2 48 = 48。

## 3. fast_math 前提修正（重要）

- **问题**：参考侧 render 模块此前从 recipe 参数 `use_fast_math=True` 推断为 fast math ON（**错误**）。
- **证据**：`fast_math=True` 时 `powf` 走 `ex2/lg2`（`g1_t5_mathdx1`/T3 生成物有 7+7 条）；参考 PTX 中
  **0 条 `ex2`/`lg2`**，且含 `rcp`/`rsqrt`（非 fast 的 libdevice 内联形态）⇒ 参考编译为 **fast_math=False**。
- **动作**：去掉 `--use_fast_math` 后 G1 立即达到 `NORMALIZED_EQUAL`。RE-0 结论以修正后配置为准。

## 4. provenance（固定）

- warp **1.15.0**、mujoco_warp **3.15.0**、upstream pin **`71da24d956378a87a703b6e1442b13aec0c4ac29`**。
- NVRTC **12.9**（build CL-36037853 / V12.9.86；dll `0cc63349…`），arch **sm_89**，release、`lineinfo=False`、
  `enable_backward=False`、`optimization_level=None`（`--Ofast-compile=0`）、**fast_math=False**、`gpu_used=false`。
- 编译 flags：`--gpu-architecture=sm_89 --std=c++17 --Ofast-compile=0 --define-macro=NDEBUG --fmad=true
  --device-as-default-execution-space --extra-device-vectorization --restrict --diag-suppress=177,550
  --include-path=include/warp-headers`（无 `--use_fast_math`）。

## 5. 生成 PTX 未入仓（target 工作材料）

- `g1_render.ptx`：sha256 `0e8cc11cba79facbf3ecb770028d5ff398f99a60f4131b22c89e3becebdedb5d` / 618,096 B
- `g2_smooth.ptx`：sha256 `dbef2d411eb82edce1a078c2847b4426eb530e9d078477598ddf2d0db789465f` / 965,105 B
- 说明：两份 PTX 是 `target/` 工作材料，**可由本目录入库脚本 + pinned 环境确定性重生成**（G3 已验证两次 sha 一致）；
  证据只存 sha/bytes/命令，不把生成物入库。
- 重生成命令（示例，pinned venv + CUDA 12.9 nvrtc）：
  `python scripts/re0_compile.py --src <reference_module.cu|smooth_eb0_bd256.cu> --out <ptx> --arch sm_89 --include include/warp-headers`
  （G1 与 G2 的输入源与 flags 见 `re0-provenance.json` 与各 compare JSON；G2 的 .cu 由 `scripts/re0_g2_codegen.py` 生成）。

## 6. 本次入库文件与哈希（逐文件 sha 见 manifest.txt）

| 文件 | sha256 | bytes |
| --- | --- | --- |
| `re0-provenance.json` | `cd36a91ca21c0b83…` | 5096 |
| `g1-compare.json` | `dbab0ed3bcd1af9b…` | 697 |
| `g2-compare.json` | `6763b6259962482e…` | 686 |
| `scripts/re0_compile.py` | `8132e048db328316…` | 5250 |
| `scripts/re0_compare.py` | `5ffe5ec6363d4bde…` | 5784 |
| `scripts/re0_g2_codegen.py` | `1c04bfce54e04dd3…` | 4436 |
| `scripts/a1_unique_codegen.py` | `24e294e7b51832c3…` | 2975 |
| `scripts/a1_tile_codegen.py` | `9135759571296a76…` | 3484 |
| `README.md` | `(computed at write)… | — |

## 7. 非声明

- G1-partial PASS 仅表示**归一化指令流等价**；不写字节相等、不写位级等价、不宣称 NP01 通过或上游等价。
- 不改判据/参考/容差；零 GPU、零窗；生成 PTX 不入仓。
