# G10@P5 G1（模型字段，含 native 导出扩展）独立复核（QA）

- 任务：`2026-10-25-g10-p5-model-fields-independent-check`（依赖 `2026-10-25-g10-p5-model-fields-r2`）
- 复核对象：**`ef47d450`**（G1 交付，父 `fec926f5`）；复核基线 HEAD `6cc3fe81`
- 环境：Windows / MSVC；MuJoCo **3.12.0**（`target/toolchains/mujoco-3.12.0/package`）；Rust 1.99.0（unit）；pinned venv Python 3.12.13；**CPU-only、未占 GPU**
- 结论：**0 阻断，G1 位级 parity、native 只增不改、f64→f32 单点转换、world 维口径均成立**。QA 用自己的 C++ ABI probe（对 G1 前/后两个提交各编译一次）+ 自己写的 3.12 MJB dump 独立复算：2 模型 12 个 volume 字段与 G0b 夹具 sha 全等（`noct`=63,497 / 0）、45 项既有 G01 字段前后位级不回归、ABI 结构不变、kind 仅新增 4/5 且负向预检会拒绝错误 kind/count/null。
- 口径差说明：任务文本与 Arch 消息曾写「现有 **44** 项 G01 字段」，实测 **45** 项（`src/io/g01.rs::FIELDS` 长度 45、native 既有 ID 0..44）；Arch 已确认 44 为笔误。本复核按 **45** 项执行。

## 1. QA 独立口径（不复用交付的 Rust reader 逻辑）

| 项 | QA 自写 harness | 独立性 |
| --- | --- | --- |
| 3.12 参考 dump | QA 自己的 `target/qa-g0b/qa_mjb_dump`（G0b 复核时写的 MuJoCo 3.12 C++ 工具）从 `fixtures/g10-p5/mjb-312/*.mjb` dump 原始小端字节 | 不读 G1 的 `src/io/volume.rs`，不读交付脚本 |
| 产品 ABI 侧导出 | `qa_abi_probe.cpp`：`#include "native/model_probe_sparse.cpp"`（来自提交快照）+ 按 `include/mjwarp_native_probe.h` 契约直接调 `mjwarp_native_g01_field_info`/`mjwarp_native_copy_g01_field`，自己写原始字节 | 不经 Rust reader/DTO；对 `fec926f5`（前）与 `ef47d450`（后）**各编译一次**做 before/after |
| 比较 | `qa_compare.py`：逐字段 sha/count/首差异；pre/post 逐字节/kind/count 比对；struct 常量比对 | 独立比较器 |
| 证据/口径 | `qa_evidence_check.py`：36 项（scope、ABI diff、转换点、world 措辞、计数、禁词） | — |

产物：`parity_report.json`、`evidence_check_report.json`、`abi/{pre,post}-{hfield-box,sdf-cylinder}/{manifest.csv,abi.txt}`、`unit-full.log`、`parity-test.log`、`build.log`、`qa_abi_probe.cpp`、`build_qa_g1.ps1`、`qa_compare.py`、`qa_evidence_check.py`、`qa_count_tests.py`。

## 2. 位级 parity（核心结论，独立复算）

方法：QA 自己的 dump 与 ABI 导出都只做「原始小端字节 + SHA-256」，与 `fixtures/g10-p5/octree-parity.json` 的 `sha256_312`（= `sha256_315`）比对；**不读夹具的 equal 结论**。

| 模型 | noct（QA 实测/夹具） | 12 个 volume 字段 | 夹具全部 18 字段（含 geom_*） |
| --- | --- | --- | --- |
| `hfield-box` | **0 / 0** | 12/12 字节+sha 全等（去重后逐字段见下） | QA dump 18/18 字段 sha 全等 |
| `sdf-cylinder` | **63,497 / 63,497** | 12/12 字节+sha 全等 | QA dump 18/18 字段 sha 全等 |

- 12 字段：`hfield_nrow/ncol/size/adr/data`、`oct_depth/oct_child/oct_aabb/oct_coeff`、`mesh_octadr/octnum`、`opt_sdf`（= `sdf_iterations`,`sdf_initpoints` 拼接）。
- **ABI 导出 vs 夹具 vs QA dump 三方一致**：每个字段 `sha256(ABI bytes) == sha256_312 == sha256(QA dump bytes)`，且 byte 数 = 夹具 `bytes`；`oct_aabb` 3,047,856 B / `oct_coeff` 4,063,808 B / `oct_child` 2,031,904 B / `oct_depth` 253,988 B（sdf）；`hfield_data` 36 B / `hfield_size` 32 B（hfield-box）。
- 值级锚点（QA 从原始字节解出）：`hfield_nrow=[3]`、`hfield_ncol=[3]`、`hfield_adr=[0]`、`hfield_size=[1.0,1.0,0.1,0.1]`、`hfield_data=[0,0.5,0,0.5,1,0.5,0,0.5,0]`、`opt_sdf=[10,40]`、`mesh_octadr=[0]`、`mesh_octnum=[63,497]`、`noct=63,497`。
- 推导的标量宽度（bytes / (count×kind 字节)）与 README §3 表一致：`hfield_size` 4、`oct_child` 8、`oct_aabb` 6、`oct_coeff` 8。
- 交付 parity 测试重跑（回归）：`cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features native-model-probe --test g10_p5_model_fields -- --ignored --nocapture` → **1 passed / 0 failed / 0 ignored**（0.22 s）。

## 3. native 只增不改 / ABI 不变

- `include/mjwarp_native_probe.h` 的 diff **只有注释**（kind 4/5 与 ID 45..58 说明），结构体未动；QA probe 在前后两个提交上编译并输出：`mjwarp_g01_field_info` size=**16**、align=**8**、offset(count)=**8**；`mjwarp_native_info` size=**96**、align=**8**；**前后完全相同**。
- 既有 45 项 G01 字段（ID 0..44）用同两个提交的 ABI probe 对同一 3.12 MJB 做 before/after：**45/45 id 的原始字节全等、kind 全等、count 全等**（0 mismatch）。
- kind 集合：pre = `{1,2,3}`；post = `{1,2,3,4,5}`；新字段 kind = `{1,2,4,5}`，**没有多余 kind**。kind 4 只用于 `hfield_data`（float32），kind 5 只用于 `noct`（int64）。
- kind 更正核实为真：G1 前头文件注释即 `1=int32, 2=double, 3=uint8`，表内 double 字段（如 `geom_pos`）已用 kind 2；本片缺的确实是 **float32（kind 4）** 与 **int64（kind 5，`mjtSize noct`）**，交付未新增 f64 kind。
- 负向预检：对 id 57 传 `kind=2`、`count+1`、`target=null`，`mjwarp_native_copy_g01_field` 均返回非零（**3/3 被拒绝**），符合「先校验后写、不静默」契约。
- 源码层面有 2 行「宽度表达式 → `g01_scalar_bytes()`」的等价重构（kind 1/2/3 语义不变，45 项前后位比已证明）；字段与 kind 均为只增。

## 4. f64→f32 单点转换

- 代码唯一转换点：`src/model/volume.rs::num_to_f32`（`values.iter().map(|&value| value as f32)`）；`src/model/volume.rs` 中 `as f32` 出现 **1 次**，`src/model/volume_input.rs`、`src/io/volume.rs` 为 **0 次**；三文件均无 `as f64`（无往返/二次量化）。
- 入口只有三个：`oct_aabb_f32()` / `oct_coeff_f32()` / `hfield_size_f32()`；`hfield_data` 原生 f32 不转换、整数字段不转换、`noct_i32()` 溢出报 `Overflow`。
- QA 从自己 dump 的原始 f64 字节独立复算位型示例：`hfield_size[2]` f64 `0.1` = `0x3FB999999999999A` → f32 **`0x3DCCCCCD`**；`1.0` → **`0x3F800000`**；`hfield_data` 的 `0.5` 原生即 f32 **`0x3F000000`**（不转换）。与 README §5 一致。
- 交付集成测试（重跑通过）逐元素断言：`converted.to_bits() == (raw as f32).to_bits()` 且 `f64::from(converted) as f32 == converted`（一次舍入、无二次量化）。

## 5. world 维上传口径

- receipt `world_dim_contract.status = "spec_only_not_implemented"`；规格为**模型静态数据共享/广播一份**（hfield、octree、`opt.sdf_*`），理由（与 mjwarp `put_model` 语义一致、逐 world 复制无物理收益）与 `naconmax` 预留（每 world 分段、总行数 `per_world*nworld`、kernel 侧算 world 偏移）都写明，可供 G4/G5 引用。
- 证据中**没有**「多世界已验证/多世界打包已通过」类正向表述；unmet 明确列出「nworld>1 未实装未实测」「GPU 内核未实装」「Linux/T4 未验证」「T4 78 项输入面未扩展」。

## 6. 范围 / 证据 / 计数

- `git diff ef47d450^..ef47d450` 共 23 文件，全部落在声明路径（`native/`、`include/`、`src/io/`、`src/model/`、`tests/g10_p5_model_fields.rs`、G1 证据、`.normify`）；**未改** `Cargo.toml/Cargo.lock/rust-toolchain.toml`、`fixtures/**`、`docs/evidence/**`、S2–S8 渲染证据；`src/io/g01.rs` 未动（FIELDS 仍 **45** 项）、`build.rs` 未动。
- 证据文件（README.md / receipt.json / parity-run.log）与 `ef47d450` 的 git blob 逐字节相同、且自该提交后未再变；sha256：README `c11f47be…`、receipt `abf90192…`、parity-run.log `86747e52…`。
- 计数一致性（QA 实测）：**unit 565 passed / 0 failed / 1 ignored**（唯一 ignored = G2 的 `native_bitwise_exploration`），与 receipt/README 一致；delivered parity 测试 **1 passed/0 failed**；**14** 个新字段（ID 45..58）、**45** 项既有字段；新文件行数 `volume.rs` 253、`volume_input.rs` 310、`io/volume.rs` 213、integration test 279（均 <500）。
- G1 证据未命中任务禁词表中的任何一项（6 项逐条扫过，命中 0）。

## 7. 问题清单（非阻断）

**口径更正（非交付缺陷，已由 Arch 确认）**
0. 任务文本/Arch 消息曾写「现有 44 项」；实测 **45** 项（`src/io/g01.rs::FIELDS` 长度 45、native 既有 ID 0..44），G1 README §1 也写 45。本复核按 45 项执行。

**建议**
1. **「只增不改」的字面表述**：`native/g01_fields.inc` 有 2 行宽度表达式被等价重构进 `g01_scalar_bytes()`（kind 1/2/3 行为不变，45 项前后位比已证明）。建议 README 把该处描述为「字段/kind 只增；宽度判定抽成 helper（等价重构）」。
2. **receipt 锚点命名**：`anchors.hfield_data_bits` 的值实际是数值数组 `[0,0.5,…]`（不是 bits），易误读；建议改名 `hfield_data_values`（README §4 的值级锚点本身正确）。
3. **证据自身 sha256/产物清单**：G1 证据未声明自身文件哈希/产物 manifest（任务模板中的「证据 sha256 与 README 声明一致」无声明可核）；建议后续证据补一个 `artifacts sha256` 清单（本复核证据已提供可直接对照的 sha）。

## 8. 限制（如实）

- 本复核的**独立导出**在 native ABI/bytes 层；交付的 Rust reader/DTO 校验路径未由 QA 重写，只作为黑盒由交付 parity 测试重跑确认（1 passed）——故「Rust reader 与 ABI 导出一致」的强度是「ABI 独立 + 测试重跑」，不是「两套 Rust 实现互证」。
- 只覆盖 Windows + MuJoCo 3.12.0；Linux/T4 未验证。
- 只验证静态模型字段读取与 parity；GPU 内核/设备上传、`nworld>1` 实装、hfield/SDF 查询均属 G4/G5，未覆盖。
- 未跑全量 G01；unit 为默认特性套件（native-model-probe 特性下只跑了 G1 的 ignored parity 测试）。
- 夹具仅存每字段 sha，无逐元素期望值；QA 用自己 dump 的原始字节给出逐字段 sha/字节数，若未来出现差异仍需重采 3.15 值定位首个元素。

## 9. 复现

```powershell
# 1) QA ABI probe：对 G1 前/后快照各编译一次，导出 45/59 个字段原始字节
& target\qa-g1\build_qa_g1.ps1
# 2) 比较：QA 3.12 dump vs 夹具 sha；ABI vs 夹具/参考；pre/post 不回归；struct/kind
target\mjwarp-parallel\venv\Scripts\python.exe -B target\qa-g1\qa_compare.py
# 3) 证据/范围/转换点/world 口径检查
target\mjwarp-parallel\venv\Scripts\python.exe -B target\qa-g1\qa_evidence_check.py
# 4) 交付测试重跑（回归）
$env:MJWARP_MUJOCO_ROOT='target\toolchains\mujoco-3.12.0\package'
$env:MJWARP_MUJOCO_DLL="$env:MJWARP_MUJOCO_ROOT\bin\mujoco.dll"
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features native-model-probe --test g10_p5_model_fields -- --ignored --nocapture
# 5) unit 计数
cargo test --locked    # 565 passed / 0 failed / 1 ignored
```

> 备注：QA probe 用 `git worktree add --detach` 在 `ef47d450^`（`fec926f5`）与 `ef47d450` 各建一个快照（`target/qa-g1/{pre,post}`），运行后已移除；产品 `Cargo.*` 未改动。所有 QA 产物在 `target/qa-g1/`，本证据目录只存脚本/日志/摘要，不存 MB 级原始 bin。
