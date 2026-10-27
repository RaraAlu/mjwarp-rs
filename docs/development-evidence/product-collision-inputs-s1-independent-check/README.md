# 产品碰撞输入接线 S1 独立复核（QA）

- 任务：`2026-10-27-product-collision-inputs-s1-independent-check`（复核对象 `2026-10-27-product-collision-inputs-s1-data-snapshot`）
- 复核提交：`feb66f4222b49c03e23a980ca766d20631557b62`（33 文件；父 `1f478513`）
- 复核时 HEAD：`feb66f42`；工作区仅本复核任务的 `.normify` 任务记录被 start 修改，其余干净
- 环境：Windows / 仓库 `D:\Rust\mjwarp-rs`；QA 侧另跑定向测试与门禁等价命令（见 §9）
- 结论：**pass / 阻断 0 / 观察 9（1 一般 + 8 建议）**；未改产品代码、未改证据、未动判据/参考/容差
- 边界：无 GPU、无窗口、无设备数值重跑；本片 S1 宿主侧，本复核不做产品接线，也不作等价/精度类结论

## 1. 总体判定

S1 的三条硬契约（只新增文件、禁改 18 个 pinned 源、既有 ABI 不动）我逐条独立核过，均成立；
新增探针与 Rust 侧快照的行为可复现（4 条定向测试我独立重跑通过），oracle 独立复算 **576 项检查 0 问题**；
门禁数字我独立复跑一致（unit 994 / 0 / 376）。发现的问题全部落在「记录口径/防御性/门禁覆盖」层面，
**没有阻断**。唯一一条「一般」观察是对既有债务定性的更正：所谓两个 pinned 文件的身份差异是
工作树 CRLF 造成的假象，不是内容漂移（§8）。

## 2. pinned 18 源零改动与 ABI 不动

`git show --name-status feb66f42` 共 33 文件：10 个 `.normify/*`（任务记录、5 个模块指纹、4 个 gate runs）、
`build.rs`、`native/model_probe_sparse.cpp`、`src/io/mod.rs`、`src/model/mod.rs` 为修改；其余为新增。
逐条核：

1. **18 pinned 路径改动数 = 0**：`git diff 1f478513 feb66f42 --name-only -- <18 paths>` 为空。
2. **身份一致性**：18/18 与 `fixtures/precision/g01-inputs.json` 记录一致——16 个按原始字节 sha256 直接相等，
   2 个（`include/mjwarp_native_probe.h`、`native/model_probe.cpp`）工作树是 CRLF，按文本 CRLF→LF 归一化后
   sha256 精确相等（`logs/pinned-sources.txt`）。四代提交（`d8024c73`/`1f478513`/`feb66f42`/`HEAD`）
   这 18 个路径的 git blob 全部相同。
3. **ABI 不动**：`feb66f42` 没有任何 `src/runtime/` 路径；`abi_tests.rs:41` 的
   `size_of::<CollisionArgs>() == 504`、`abi_tests.rs:143` 的 `entries.len() == 6`、
   `multicontact_tests.rs:14` 的 `DEVICE_SOURCES.len() == 28` 都随 unit 门禁跑过并通过。
   新 ABI 声明只在新头 `include/mjwarp_collision_inputs_probe.h`（7 个入口，符号 `mjwarp_native_collision_inputs_*`），
   既有 `mjwarp_native_*` 头未被改。
4. **新 cpp 的 include 路径**：`native/model_probe_sparse.cpp` 只 +1 行
   `#include "model_probe_collision_inputs.cpp"`（放在 `model_probe_collision.cpp` 之后）；新 cpp 不再 include
   `model_probe.cpp`（避免同 TU 重复定义）——我逐行看过该文件，确实只 include 新头。
5. **build.rs**：只 +2 个 `rerun-if-changed`（新 cpp、新头），无 `.file()`、无新 feature；
   `Cargo.toml`/`Cargo.lock`/`rust-toolchain.toml` 在 `1f478513..feb66f42` 零 diff。
6. **无产品接线**：新增模块只有 `io::NativeModelProbe` 的两个方法，没有接到任何渲染/碰撞管线入口。

## 3. 4 条定向测试独立重跑

我按 S1 记录的命令自行重跑（两次，结果相同）：

```
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features native-model-probe \
  --test g10_p5_collision_inputs_snapshot -- --ignored --test-threads=1
```

`4 passed / 0 failed / 0 ignored / 0 filtered out`，0.20 s；4 个名字与记录一致
（`logs/targeted-test-rerun.txt`）。既有 `native_g01` 的
`converts_complete_native_g01_fields_after_owner_drop` 我也独立重跑：`1 passed / 0 failed`，0.03 s
（`logs/native-g01-rerun.txt`）。

测试覆盖我逐段核过：32 项计数与逐元素值（`Num` 比 f64 位型、`Float` 比 f32 位型）、
两组 qpos 的 `geom_xpos`/`geom_xmat` 位型、mjData 复用后同帧、owner drop 后快照仍可读、
oct_* 与同模型 G01 体积桥逐元素相等、以及原始 ABI 负例。测试代码本身没有「预置期望值放宽」：
期望值全部来自 `tests/support/collision_inputs_oracle.json`，比较是 `assert_eq!` 逐元素/逐位型。

## 4. oracle 独立性与复算

我没有复用作者的生成脚本，而是自写 `verify_s1.py`（本目录）用 pinned **mujoco 3.12.0 / numpy 2.5.4 /
python 3.12.13**（`target/render-bvh-precheck/venv312`，环境与记录一致）从三份冻结 MJB 重算：

- 3 份 MJB 的 bytes 与 sha256：3/3 一致；
- 11 项计数 × 3 模型：一致；
- 30 项模型字段（id 2..31）× 3 模型：`i32` 列表、`f64` 位型、`f32` 位型逐元素一致
  （sdf-cylinder 的 oct_* 只存计数，我核到 `count == noct × width` 且 `noct=63497`）；
- 2 组 qpos × 3 模型的 `geom_xpos`(ngeom×3) / `geom_xmat`(ngeom×9) f32 位型：一致；
- 三份重复的字段表交叉一致：Rust `COLLISION_INPUTS_FIELDS` 32 行、C++ switch 32 个 case、
  `CollisionGpuInputs` 32 个字段顺序 —— 名称/kind/unit/width/source 全对（Rust `Data` 仅 id 0/1）。

合计 **576 项检查、0 问题**（`logs/oracle-verification.txt`、`verify_s1.json`）。
oracle 文件本身：12545 B、sha256 `b5a3fd29b30686b398f659a94cc74a7d6b930dc714a6b21db13b6eb7671fe17e`，
与 README/receipt/交付记录三处一致。

**「f64 用 u64 位型存储」的理由我独立复现了**：用仓库同款 `serde_json 1.0.151`（未开 `float_roundtrip`）
把 oracle 里 166 个 f64 值的十进制文本重新解析，**恰好 1 个**（scene-312 `geom_size[10]` =
`0.11801833394695993`）落到低 1 ULP（`3fbe367314a16ef7` vs 期望 `3fbe367314a16ef8`），
与作者记录的例子完全对上（`logs/serde-json-1ulp.txt`，探针源码 `sjparse.rs`）。

## 5. 生命周期与健壮性（代码 + 负例测试）

- **句柄晚于 owner**：快照持有 `Vec` 拷贝；测试先 `drop(probe)` 再读 `counts()/field()` 并 clone
  （测试文件 176–183 行）；`CollisionInputsData<'a>` 借用 `NativeModelProbe`，`Drop` 用同一 owner 指针销毁，
  另用 `PhantomData<Rc<()>>` 禁止 Send/Sync（单线程约定成立）。
- **mjData 缓存复用**：`create` 只 `mj_makeData` 一次，`set_qpos` 只 memcpy + `mj_kinematics`
  （cpp 190–201）；测试重设第一组 qpos 后要求帧完全相等。
- **非法参数先 preflight 后写、不 panic**：copy 入口先校验 reserved/kind/count/目标再 memcpy；
  负例测试用哨兵缓冲断言 id 0/1、count+1、kind 错、reserved 非 0、空目标五种拒绝都不写一个字节
  （`collision_inputs_preflight.rs:176-215`）；未知 id（32/33/`u32::MAX`）、空 owner、空 DTO 均返回 8；
  `xpos/xmat` 入口先校验 ngeom 与两个指针再转换。
- **外部 owner destroy 拒绝**：cpp 227–236 行校验 `data->owner == owner`，不匹配返回 8 且不 delete，
  负例测试断言（preflight.rs:305）。
- **无 panic 源**：新产品代码（`src/io/collision_inputs.rs`、`src/model/collision_inputs.rs`、新 cpp/新头）
  没有 `unwrap/expect/panic!/todo!`；分配走 `try_reserve_exact` + 溢出检查。

## 6. W4 / 门禁 / 范围

- **W4**：`build.rs:42` `warnings_into_errors(true)`（cc 1.6 对 MSVC 映射 `/W4 /WX`）；`logs/build-w4.log`
  里新 TU 参与构建且**无任何 warning 行**、`Finished`。属**间接证据**（cc 的 cl.exe 命令行未归档，
  但 `/WX` 下任何警告都会让构建失败，构建成功即等价于零警告）。
- **门禁**：我独立复跑了 `.normify/gates.toml` 的三条命令（`cargo fmt --check`、`cargo clippy --all-targets
  --locked -- -D warnings`、`--features cuda` 版本）全部 exit 0；`cargo test --locked` 独立复跑
  **994 passed / 0 failed / 376 ignored**，与 `.normify/runs/.../unit.json` 完全一致；
  `t4_input_precision` 默认 **36 passed / 0 failed / 12 ignored** 也在其中复现（`logs/gate-equivalent.txt`、
  `logs/unit-gate-equivalent.txt`）。
  额外我跑了唯一能 lint 到新代码的那条：`clippy --all-targets --locked --features native-model-probe
  -- -D warnings` → exit 0（`logs/gate-equivalent.txt`）。**没有重跑 `normify gate`**：那会覆盖已 verified
  任务的 runs 目录产物；我改为直接跑同样命令并对照已归档 runs。
- **范围**：`git show --name-only feb66f42` 的 33 个路径逐条对任务声明范围 = **33 在范围内 / 0 越界**
  （`.normify/*` 属治理产物）；当前 `scope-selfcheck` 报「改动 0 / 越界 0」。

## 7. 行数棘轮

按字节数 LF 计（权威口径）：`src/io/collision_inputs.rs` 474、`src/model/collision_inputs.rs` 211、
`tests/g10_p5_collision_inputs_snapshot.rs` 234、`tests/support/collision_inputs_oracle_support.rs` 105、
`tests/support/collision_inputs_preflight.rs` 313、`native/model_probe_collision_inputs.cpp` 236、
`include/mjwarp_collision_inputs_probe.h` 69、`src/io/mod.rs` 462、`src/model/mod.rs` 345；
`src/model/render.rs` 仍 500 且未被改。全部 ≤500（`logs/line-counts.txt`）。
证据 README 175 行、receipt 297 行；`tests/support/collision_inputs_oracle.json` 为单行机器可读 JSON（例外）。

## 8. 一般观察 O1：「既有 pinned sha 漂移」不成立（CRLF 假象）

S1 记录（README §6、receipt `pinned_sources.drift_note`、交付记录 `notes`）称
`include/mjwarp_native_probe.h` 与 `native/model_probe.cpp`「已被既有 render/volume 片更新、与当前文件不一致」，
并称「G01 全量 `--include-ignored` 才会暴露」。我独立核后认为**这个定性不成立**：

1. 两个文件的**内容没有漂移**：raw sha256 不等于 pinned（因为工作树是 CRLF），但按文本 CRLF→LF 归一化后
   sha256 **精确等于** pinned（`03151bf5…` / `6a01eb01…`）。
2. `fixtures/precision/g01-inputs.json` 最近一次更新是 `61fb6baf`（2026-10-26 01:14，提交信息即
   「重钉 T4 当前源码身份——身份漂移 18→0」），晚于 render 片（`d642c039`，10-24）与模型字段片
   （`ef47d450`，10-25），所以 pinned 值本来就是更新后的身份。
3. 仓库自有的 T4 审计本来就按 `digest(text.replace("\r\n", "\n"))` 比对
   （`tests/support/t4_evidence/sources.rs:13`），CRLF 不会被当成漂移。
4. 我实测 `cargo test --locked --test t4_input_precision -- --include-ignored` 的 12 个失败**不是** hash 差异：
   失败原因是这些审计需要环境变量 `MJWARP_FROZEN_UPSTREAM_ROOT` / `MJWARP_FROZEN_WARP_ROOT`，
   本机未设置时 panic 在 `tests/t4_input_precision/tests.rs:139:71`「set upstream root」
   （`logs/t4-include-ignored.txt`）。

=> 结论：18/18 pinned 源身份一致；「S1 未加剧」成立但原因不是「本来就有漂移」，而是**根本没有漂移**。
建议把 §6 改成「两文件工作树为 CRLF、pinned sha 为 LF 归一化值；git blob 与本片前一致；T4 审计自带 LF 归一化，
因此不构成身份漂移」，并删除「已被 render/volume 片更新」的表述；receipt 的 `pinned_sources.verification`
建议注明比对口径（原始字节 or LF 归一化）。**不影响代码、判据、参考、容差与 S1 的结论。**

## 9. 其余观察（建议级）

- **O2 gate 覆盖**：`gates.toml` 的 clippy/unit 用默认 features，不含 `native-model-probe`；新代码全部
  `#[cfg(feature = "native-model-probe")]`、新测试 `#[ignore]`，所以**门禁本身不 lint/不跑新增代码**。
  本片靠 `logs/fmt-clippy.log` 里显式 `--features native-model-probe` 的 clippy 和定向测试兜底，建议把这条
  纳入门禁（或登记为「定向门禁项」），否则后续改动没有自动保护。
- **O3 数据帧收窄口径不一致**：`copy_xpos_xmat` 用裸 `static_cast<float>` 逐元素收窄，不做有限性/溢出检查，
  而模型 `Num` 字段走唯一转换入口 `convert_f64_to_f32_into`（拒非有限与 f32 溢出）。极端但有限的 qpos
  （如 1e300 平移）会静默得到 inf 帧。建议数据帧也走同一入口，或在头文件/receipt 注明该口径差异。
- **O4 qpos 数值未校验**：`set_qpos` 只校验长度，不校验 qpos 有限性（NaN/Inf 会静默进 `geom_xpos/xmat`）；
  另 `nq == 0` 的模型未覆盖——cpp 在 `d->qpos` 为空指针时返回 8，即使空 qpos 合法。建议注明「调用方负责 qpos 合法」
  并补一个 nq=0 负例（不阻断 S1，三份 fixture 的 nq 均 ≥7）。
- **O5「gate 跑两次」无两次留档**：`runs` 目录里的 4 个 json 是覆盖写，只有最后一次（15:46）。
  建议保留两次 run 目录或 receipt 写明「仅最终树留档」。
- **O6 错误类型语义**：`checked_count` 把 kind/宽度不匹配映射成 `InputError::InvalidDimension { field }`，
  与「长度不符」场景混用；排障时容易误导，建议用 ABI 类错误或独立错误变体。
- **O7 计数一致性未交叉校验**：`NativeCollisionInputsSnapshot::capture` 没有把 ABI `info` 的 `nq/ngeom/nmesh…`
  与 `collision_inputs_info` 的计数互相校验（测试是靠 oracle 双比间接覆盖）。建议加一个防御性
  「两组计数一致」检查，避免将来 DLL/模型错配时静默通过。
- **O8 摘要行措辞**：README §6 摘要「18 个 pinned 源与 HEAD 逐字节一致（零改动）」容易被读成「与 pinned sha 一致」，
  建议写成「相对本片 HEAD 零改动」。
- **O9 行数口径**：任务消息里给的 445/189/230/310 与文件 LF 行数 474/211/234/313 不符（用 `Get-Content`
  计行会少计）；README/receipt 里的数字是对的。后续证据建议统一注明「按 LF 计数」。

## 10. 证据完整性

- 交付记录 `docs/development-evidence/changes/2026-10-27-product-collision-inputs-s1-data-snapshot.json`
  的 **21 个文件**逐条复验：blob、sha256、bytes、LF 行数 **0 不符**，记录文件自身 blob 与 HEAD 一致
  （`logs/delivery-record-verification.txt`、`verify_delivery.json`）。
- S1 证据目录 9 个文件（README 175 行、receipt 297 行、6 份日志、生成脚本）：UTF-8 无 BOM、LF；
  receipt `/task_id` = S1 任务 id；日志里的数字与 runs/交付记录一致。
- 禁词扫描（6 个规定词条）：S1 证据、新代码、交付记录 **0 命中**；「上游等价」只在否定句里出现一次
  （README §6「不宣称与上游等价」）。
- 本复核目录的 `manifest.json` 记录本目录所有文件 sha256/bytes 与本复核引用的关键提交/文件身份。
  本目录文件统一 LF；manifest 的 sha256 按 LF 字节计（= git blob），三个脚本已固定 LF 输出。

## 11. 复现命令（QA 侧，只读）

```
# 定向测试（需 MJWARP_MUJOCO_ROOT / MJWARP_MUJOCO_DLL 指向 target/toolchains/mujoco-3.12.0/package）
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features native-model-probe \
  --test g10_p5_collision_inputs_snapshot -- --ignored --test-threads=1
# oracle / 字段表独立复算
target/render-bvh-precheck/venv312/Scripts/python.exe verify_s1.py
# 交付记录身份复验
target/render-bvh-precheck/venv312/Scripts/python.exe verify_delivery.py
# 18 pinned 源身份表
target/render-bvh-precheck/venv312/Scripts/python.exe gen_pinned.py
# 门禁等价命令（不写 normify run 文件）
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --all-targets --locked --features cuda -- -D warnings
cargo test --locked
```

## 12. 边界与不宣称

本复核只做宿主侧只读核对：未开 GPU、未开窗口、未重跑设备数值、未改产品代码/证据/判据/参考/容差。
S1 不做产品接线；本复核不对 host↔device 行为一致性、精度、性能、跨平台或任何验收结论作判断；
`oct_*` 的交叉核对对象是同 pinned 模型的 G01 体积桥（同仓库另一条读取路径），不是第二份独立实现参考。
U2 类「设备侧防御分支」不在本片范围。
