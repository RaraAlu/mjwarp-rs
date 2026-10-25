# case_evidence frame 环境敏感性诊断

- 任务：`2026-10-25-case-evidence-frame-diagnosis`（Backend）
- HEAD：`47f1f7a3`（实验执行时 `eddad9c0`；其后 QA/治理提交仅 docs/.normify，`src/`、`tests/`、`fixtures/`
  相对 `eddad9c0` 无变化）
- 性质：**只诊断，不改判据/容差/排除清单/产品源码/门禁/G01 脚本**；不跑 GPU、不改已 verified 证据
- 环境：Windows 11 Pro 10.0.26200、AMD Ryzen 9 9950X3D、rustc 1.99.0 (b940084d7)、cargo 1.99.0 (5f94df478)
- 证据文件：本 README、`receipt.json`、8 份环境日志（`gnu-control.log` / `msvc-control.log` /
  `msvc-plus-ldc.log` / `gnu-fma.log` / `msvc-fma.log` / `gnu-release.log` / `msvc-release.log` /
  `gnu-no-config.log`）、`probe-fmaf.log`

## 0. 结论摘要（TL;DR）

1. **直接机制**：`windows-gnu`（仓库默认工具链）下 `f32::mul_add` 的库回退**不是 IEEE 单次舍入 FMA**。
   与 `-C target-feature=+fma`（硬件 FMA）构建对比，4096 组随机 f32 三元组中 **991 组差 1 ULP**；
   MSVC 默认构建与硬件 FMA 构建**完全一致（0/4096）**。即 GNU 默认构建的 FMA 结果非合规，
   MSVC 默认构建是 IEEE 正确的。
2. **受影响路径**：碰撞 host 路径里唯一的 `f32::mul_add` 使用点是
   `src/physics/collision/gjk/polytope.rs` 的 `face_norm2` 点积链（EPA 面法线，**对齐 pinned ccd PTX 的
   FMA 收缩**）。`ellipsoid-box` 的参考 `dist = -0.00999989…`（穿透）→ 走 EPA → 命中该链。
3. **结果翻转**：GNU 默认构建 frame = **`3.838112e-7`**（恰低于判据 `4.8e-7` → pass）；
   MSVC 默认构建 / GNU+`+fma` / MSVC+`+fma` 三者 frame **完全相同 = `6.351978e-7`**（→ fail）。
   即：**仓库默认环境（GNU）的 pass 是该环境库 `fmaf` 非合规造成的偶合**；faithful（硬件 `fma.rn`）
   行为是 `6.351978e-7` ≈ 5.3 ULP，**超过 4·ULP 判据**。
4. **排除项**：环境变量、`link-dead-code`、`--release/profile`、产物复用/指纹、fixture/输入差异
   均不是原因（见 §3.5）。`Arch` 侧此前报的「`--release` 1 passed」对应 GNU 默认工具链；
   本片实测 **MSVC release 同样红（`6.351978e-7`）**，release 不改变结论。
5. **影响**：r2 的「GNU 口径 gate/默认全绿」数值上是**偶合**，不具 faithful 数值可信度；
   G01 `host-msvc` 红（Arch 复现 `689 passed / 1 failed / 137 ignored`）是该 case 的 faithful 结果。
   处置候选见 §6（均**不实施**，判据/容差类需用户拍板）。

## 1. 环境矩阵（按「干净构建判定标准」，逐环境记录）

| # | 环境 | 工具链 / triple | 关键 flags | CARGO_*/RUST* | deps 文件数 | 该测试 | frame 实测 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ① | Arch `target/arch-verify-clean`（参考复核） | default → `1.99.0-x86_64-pc-windows-gnu` | gnu rustflags `-C link-dead-code=yes` | 未设 | 295（Arch 口径） | exit 0 / 1 passed | pass（未打印 frame） |
| ② | 本机 `target/verify-clean-001`（**非 clean 反例**） | `+1.99.0-x86_64-pc-windows-msvc` | 无 `link-dead-code` | 未设 | **1557**（多配置叠加） | 默认/cuda-probe/all-features 三配置均红 | **`6.351978e-7`** |
| ③ | 主 `target/`（**非 clean 反例**） | 混用（`.rustc_info.json` 最近 host=gnu） | 混用 | 未设 | **14534**（68 个 `mjwarp_rs-*.exe`） | 同测试多 exe pass/fail 混杂 | 不可信，不作证据源 |
| ④ | `target/diag-gnu-control`（本片 clean） | `1.99.0-x86_64-pc-windows-gnu` | gnu rustflags（`-C link-dead-code=yes`） | 无 | 62 | **exit 0 / 1 passed** | `3.838112e-7` |
| ⑤ | `diag-gnu-control` 的独立性复核（worktree 删除 `.cargo/config.toml`） | 同上 | **无** `link-dead-code` | 无 | 62 级（同构） | **exit 0 / 1 passed** | `3.838112e-7` |
| ⑥ | `target/diag-msvc-control`（本片 clean） | `1.99.0-x86_64-pc-windows-msvc` | 无 | 无 | 65 | **exit 101 / FAILED** | **`6.351978e-7`** |
| ⑦ | `target/diag-msvc-plus-ldc`（clean，显式补 flag） | MSVC | `RUSTFLAGS=-C link-dead-code=yes` | 仅 RUSTFLAGS | 67 | exit 101 / FAILED | **`6.351978e-7`** |
| ⑧ | `target/diag-gnu-fma`（clean，硬件 FMA） | GNU | `RUSTFLAGS=-C target-feature=+fma` | 仅 RUSTFLAGS | 62 | exit 101 / FAILED | **`6.351978e-7`** |
| ⑨ | `target/diag-msvc-fma`（clean，硬件 FMA） | MSVC | `RUSTFLAGS=-C target-feature=+fma` | 仅 RUSTFLAGS | 65 | exit 101 / FAILED | **`6.351978e-7`** |
| ⑩ | `target/diag-gnu-release`（clean，release） | GNU | release profile | 无 | 62 | exit 0 / 1 passed | `3.838112e-7` |
| ⑪ | `target/diag-msvc-release`（clean，release） | MSVC | release profile | 无 | 65 | **exit 101 / FAILED** | **`6.351978e-7`** |

补充（Arch 侧独立证据）：GNU 默认环境干净全量 `cargo test --locked --no-fail-fast` = exit 0；
干净 `--features cuda-probe --lib` = 678/0/137；非缓存 `normify gate` unit 595/0（22.5 s）。
MSVC 口径 G01 `host-msvc`（`--features native-model-probe,cuda-probe --lib`）= **689 passed / 1 failed /
137 ignored**，唯一失败即本测试。

## 2. 「干净构建」判定标准（本片作为模板写入）

一个构建环境只有**同时**满足以下各项才算 clean（本片所有 `diag-*` 环境均按此执行）：

1. **新建专用 `CARGO_TARGET_DIR`**，不与主 `target/` 或其它配置共用（本片：`target/diag-*`；
   不使用 `verify-clean-001/002` 作为 clean 证据源——二者是多配置叠加目录）。
2. **清空 `CARGO_*`/`RUST*` 相关环境变量**并记录清空前后：本机清空后为空；只在实验需要时显式设置
   `RUSTFLAGS`/`CARGO_TARGET_DIR`，并写进记录。
3. **记录该目录的 deps 文件数**：`(Get-ChildItem <target>\debug\deps -File).Count`。
   本机默认特性全新基线：**GNU = 62、MSVC = 65**；复用目录会到 1557 / 14534，不能算 clean。
4. **记录 `rustc -vV`（host triple）、`cargo -V`、`.rustc_info.json` 的 host、以及
   `cargo test --no-run -v` 中 `--crate-name mjwarp_rs` 行的工具链路径与 `-C` 参数**
   （本片实测的两类命令行差异见 §3.3）。
5. **判据只用行为（exit / 结果行），不用 exe sha**：Rust debug 构建不以位级可复现为目标；
   本片观察到同体积 `54,607,221 B` 不同 sha、同名 `-bbf617de7cd0b294` 不同 sha，
   所以「sha 一致」不能当确定性判据。
6. **复用带多套配置的目录不算 clean**：`verify-clean-001`（1557 deps）与主 `target/`
   （14534 deps / 68 个 mjwarp 测试 exe）只能作反例。

## 3. 归因

### 3.1 frame 的计算链

- `contact.rs`：`frame = make_frame(sub3(witness1, witness2))`（或反向），`make_frame` 在 `math.rs`：
  `normalize3`（`sqrt` + 除法）→ `b = normalize(selector - a*dot)` → `c = cross(a,b)`。
- `ellipsoid-box` 参考 `dist = -0.00999989…`（穿透）→ `ccd.rs::epa_phase` → `epa.rs` 用
  `polytope.rs` 的 `face_norm2` 选面/取法线；`polytope.rs:260` 还有 `face_norm2.sqrt()`。
- 因此 EPA 面法线点积链是本 case 的数值敏感点。

### 3.2 碰撞路径唯一的 `f32::mul_add` 点

`src/physics/collision/gjk/polytope.rs:67-70`（注释原文：「对齐 pinned ccd PTX 的 FMA 收缩：
dot(r,r) 为『首项乘积 + 逐项 mul_add』链」）：

```rust
let tail = r[1].mul_add(r[1], head);
pt.face_norm2[idx] = r[2].mul_add(r[2], tail);
```

`face_norm2` 是 EPA 面法线平方，直接影响法线/接触 frame。

### 3.3 微探针：两种工具链的 `f32::mul_add` 语义不同

探针源码/命令见 `probe-fmaf.log`（`rustc --edition 2024 -C opt-level=0 -C debug-assertions=on`，
与 `cargo test` dev profile 一致；同一 4096 组确定性 f32 输入）：

| 对比 | GNU 默认 (windows-gnu) | MSVC 默认 (windows-msvc) |
| --- | --- | --- |
| `mul_add` vs `-C target-feature=+fma`（硬件 `fma.rn`） | **991 / 4096 不同，全部 1 ULP** | **0 / 4096（完全一致）** |
| `mul_add` vs 非融合 `a*b + c` | 986 / 4096 | 55 / 4096 |
| `mul_add` vs f64 双舍入 `(f64 a*b + c) as f32` | 991 / 4096 | 0 / 4096 |
| `-C target-feature=+fma` 两工具链互相比较 | 0 / 4096（一致） | 0 / 4096 |

已知向量（如 `fma(1+2^-23, 1-2^-23, -1)`、`fma(3,7,-21)`、`fma(0.1,0.2,0.3)`）四个构建全部给出
正确融合结果——说明差异不是「完全不融合」，而是特定输入下 **1 ULP 的舍入差异**；
团队在 `relmat.rs` 已记录过同类现象（QA：`0x3f17e6af` vs 正确 `0x3f17e6b0`）。

两条 `--crate-name mjwarp_rs` 命令行（同 HEAD、同 features、同一 crate）差异只有：
- rustc 工具链路径：`...\1.99.0-x86_64-pc-windows-gnu\bin\rustc.exe` vs `...-msvc\bin\rustc.exe`；
- GNU 独有 `-C link-dead-code=yes`（来自仓库 `.cargo/config.toml` 的 gnu rustflags）；
- 其余相同：`--edition=2024`、`-C embed-bitcode=no`、`-C debuginfo=2`、`--test`、
  `--cfg "feature=\"default\""`、无 `--target`、无显式 `-C opt-level`。

### 3.4 产品级最小差分（同 HEAD、全新目录）

| 构建 | `case_evidence::upstream_f32_within_4ulp` | ellipsoid-box `[判据]` |
| --- | --- | --- |
| GNU 默认（④/⑤） | **pass** | count 2/2；dist 位等 2/2；pos ≤2.00 ULP@尺度；**frame 13/18 位等，max abs `3.838112e-7`** |
| MSVC 默认（⑥） | **fail** | 同上但 **frame max abs `6.351978e-7`** |
| GNU + `-C target-feature=+fma`（⑧） | **fail** | **`6.351978e-7`** |
| MSVC + `-C target-feature=+fma`（⑨） | **fail** | **`6.351978e-7`** |
| GNU release（⑩） | pass | `3.838112e-7` |
| MSVC release（⑪） | fail | `6.351978e-7` |

→ 打开硬件 FMA 后，GNU 与 MSVC 的结果**完全相同**（`6.351978e-7`），且与 MSVC 默认一致；
说明 **FMA 舍入语义是 pass/fail 翻转的唯一原因**，其余数值（dist/pos）在两环境一致。

### 3.5 已排除项

- **环境变量**：`CARGO*/RUST*` 清空后结果不变；Arch 侧查证唯一相关变量 `G2_DUMP_NATIVE_ERROR`
  （dump 开关，不参与断言），干净二进制 + G01 env 重跑仍 pass。
- **`-C link-dead-code=yes`**：MSVC 显式加上该 flag 仍红（⑦）；GNU 在 worktree 中删除
  `.cargo/config.toml`（去掉该 flag）仍绿（⑤）→ flag 不是原因。
- **`--release` / opt-level**：GNU release 绿、MSVC release 红（⑩/⑪），与 debug 结论一致；
  排除「优化导致 frame 超限」。
- **产物复用/指纹/时钟**：全部实验使用全新专用目录并复现两侧结果；`verify-clean-001` 的
  1557 deps、主 target 的 14534 deps / 68 个 exe 只是**多配置叠加**现象（会话中交替使用两种
  工具链写同一 `target/`），不是 pass/fail 差异的原因。
- **fixture/输入差异**：该测试 4 个关键输入与 HEAD 逐字节一致（r2 证据已记：
  `case_evidence.rs a4100273…`、`case_compare.rs 5b0cd05c…`、`case_data.rs b3b95951…`、
  `fixtures/g10-p5/references/ellipsoid-box.json a3223a77…`）。

### 3.6 与团队已有认知一致

- `src/physics/collision/relmat.rs:10-15` 注释：「刻意**不用** `f32::mul_add`：本机 dev 档
  （opt-level 0）实测未融合（QA：`0x3f17e6af` vs 正确 `0x3f17e6b0`）」，改用 f64 精确承载乘积的
  `fma_f32` 单次舍入 helper；
- `src/render/bvh/aabb.rs:126` 注释：「宿主侧不直接用 `f32::mul_add`：本目标 debug 构建会落到
  不保证单次舍入的回退」，改用 `fused_mul_add` helper；
- 而 `gjk/polytope.rs` 仍直接使用 `f32::mul_add` → 这是本 case 的遗漏点，也是环境敏感性的来源。

## 4. 后果与既有证据的关系

- r2 的「GNU 口径 gate unit 595/0、干净默认全量 exit 0、cuda-probe lib 678/0/137」**数值上成立**，
  但其绿色依赖 GNU 默认工具链的非合规 `fmaf`：同一棵树在 IEEE 正确 FMA 下为 `6.351978e-7`（红）。
  因此 r2 的「环境差异」之谜在本片得到解释：**不是 clean 与否/产物复用，而是工具链 target 的
  `fmaf` 语义差异**；r2 已入库证据中的「差异待诊断」结论由本片补齐，不改 r2 已 verified 数字/判据。
- G01 `host-msvc`（MSVC、IEEE 正确 FMA）红是本 case 的 faithful 结果，属既有数值问题，
  不因 GNU 口径 gate 绿而消失（与 r2 证据 §2 的 GNU 口径注记一致）。

## 5. 产物可信度：如何判定「结果对应当前源码」

1. 新建专用 `CARGO_TARGET_DIR`（名字带目的，如 `target/diag-*`），先删空；
2. 清空 `CARGO_*`/`RUST*`（记录清空前后的原始输出），只显式设置必要变量并写入记录；
3. 记录 `rustc -vV`/`cargo -V`/`.rustc_info.json` host/deps 文件数、`--crate-name mjwarp_rs`
   工具链路径与 `-C` 参数（见 §2 模板）；
4. 判定只看行为（exit / 结果行 / 打印的 frame 值），不看 exe sha；
5. 若必须复用目录，先在记录中标注「非 clean / 多配置叠加」，结果只能作参考。

## 6. 候选处置（**不实施**，供拍板）

| 选项 | 内容 | 影响面 | 决策方 |
| --- | --- | --- | --- |
| ① 统一数值验收工具链 | 显式要求 IEEE 正确 FMA 的构建（MSVC，或 GNU 加 `-C target-feature=+fma`） | gate/G01 环境定义、证据口径需重写；GNU 默认将不再「绿」 | Arch/Chief（改门禁属独立片） |
| ② 修实现（推荐方向） | 把 `gjk/polytope.rs` 的 `f32::mul_add` 换成与 `relmat.rs` 一致的「单次舍入」实现或显式硬件 FMA；再排查残余 5.3 ULP | 会使 GNU 默认暴露红（`6.351978e-7`）；需另立实现片并重跑 G01 | Backend/Arch 另立片 |
| ③ 判据/容差调整 | frame 上限 `4.8e-7`（4·ULP@1.0）是否应放宽/改口径 | 属精度契约变更，**需用户拍板** | 用户 |
| ④ 双 target 必过 / CI 策略 | G01 `host-msvc` 红是否阻断、是否要求 GNU/MSVC 双绿 | 流程与门禁策略，需 Chief/用户拍板 | Chief/用户 |

本片**未**改任何判据/容差/排除清单/产品源码/`gates.toml`/G01 脚本；不宣称物理整体通过或精度达标。

## 7. 未覆盖与边界

- 未在本片修改 mingw-w64 `fmaf` 的具体实现（lib 名/算法未定；只给行为证据：与硬件 FMA 的 1 ULP 差异
  及分布）；如需可另立片查 mingw-w64 源码/反汇编。
- 未跑 GPU；G01 `host-msvc` 的数字引用 Arch 复现（689/1/137），本片用 MSVC 默认/`+fma` 最小实验
  复现了同一失败值 `6.351978e-7`。
- 未测 Linux（按团队约定暂缓）；未测其它 CPU/编译器版本。
- 未覆盖 `render/` 路径的 `mul_add`（`megakernel_arrays.rs:189`）与测试侧 `runtime/transfer` 的
  `mul_add`：不在本 case 碰撞路径上。

## 8. 复现命令（要点）

```powershell
# 干净环境：新建专用目录、清空 CARGO_*/RUST*（原样记录）
$env:CARGO_TARGET_DIR = 'target\diag-gnu-control'   # 每次新建/删空
cargo test --locked --lib physics::collision::gjk::case_evidence::upstream_f32_within_4ulp -- --exact --nocapture

# MSVC 对照
$env:CARGO_TARGET_DIR = 'target\diag-msvc-control'
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --lib `
  physics::collision::gjk::case_evidence::upstream_f32_within_4ulp -- --exact --nocapture

# 硬件 FMA 对照（任一工具链）
$env:RUSTFLAGS = '-C target-feature=+fma'
# 其余同上；测完 Remove-Item Env:RUSTFLAGS

# 观察 frame 值（不 panic 的打印测试）
cargo test --locked --lib `
  physics::collision::gjk::case_evidence::our_contacts_match_upstream_f32_within_4ulp -- --exact --nocapture
```
