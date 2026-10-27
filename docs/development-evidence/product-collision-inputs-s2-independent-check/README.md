# 产品碰撞输入接线 S2 独立复核（QA）

- 任务：`2026-10-27-product-collision-inputs-s2-independent-check`（复核对象 `2026-10-27-product-collision-inputs-s2-adapter`）
- 复核提交：`2910040c222184d0c5fbbfde8690f3db86176b26`（16 文件；父 `6a113daf`）
- 复核时 HEAD：`2910040c`；工作区仅本复核任务的 `.normify` 任务记录被 start 修改
- 环境：Windows / 仓库 `D:\Rust\mjwarp-rs`；定向测试与门禁等价命令由 QA 侧独立重跑（见 §7）
- 结论：**pass / 阻断 0 / 观察 7（1 一般 + 6 建议）**；未改产品代码/证据/判据/参考/容差
- 边界：无 GPU、无窗口、无设备数值重跑；S2 本身不做产品入口接线，本复核也不作等价/精度/性能/验收类结论

## 1. 总体判定

S2 的三条核心主张我都独立核过并成立：adapter 视图与 `CollisionGpuInputs` 的 32 项顺序/类型 1:1；
三 case 的 32 数组对账 **96/96 逐字节 + sha256 相等（0 不符）**（我自写 harness、自己解析 bins，未复用产品测试）；
旧 23 项输入的 9 项空 + sidecar null + 23 项透传语义成立。`run` 未改、ABI 未动、23 个 unchanged 路径 0 改动、
门禁 997/0/376 与证据数字一致。唯一一条「一般」观察是**记录里的第三 case fixture 归属写错**
（写成 `mesh-mesh-two-assets.xml`，实际是 `mesh-mesh.xml`），不影响 bins、测试或代码。

## 2. adapter 语义（逐项核）

1. **32 项视图顺序/类型 1:1**：脚本对比 `CollisionInputsSnapshot<'a>` 与 `types.rs::CollisionGpuInputs` 的
   32 个字段名与 `&'a [f32]/&'a [i32]` 类型，**32/32 完全一致**；`inputs_from_snapshot` 里 32 条
   `X: snapshot.X` 赋值对**名字 32/32 相同**（不靠人工看）。
2. **复用既有校验**：`inputs_from_snapshot` 组装后调用 `validate_inputs`（与 `run()` 首步同一个函数），
   没有第二套校验逻辑；`validate_collision_inputs` 也是 `validate_inputs` 的薄包装。
3. **空/部分空语义与 M0a 一致**：9 项 mesh_poly 全空 → `validate_mesh_poly` 直接返回 Ok，sidecar 不构造；
   部分非空 → `InvalidArgument`「mesh_poly 输入不完整…」。`mesh_poly_sidecar_present` 的判据
   `!mesh_polynum.is_empty()` 与 `run()` 内 `collision_gpu.rs:162` 的条件一致。
4. **不 panic**：adapter 代码无 `unwrap/expect/panic!/todo!`（只有 1 处 `unwrap_or`，安全兜底）；
   负例也确认返回 `InvalidArgument` 而非 panic。
5. **`run` 零改动**：`git diff 6a113daf..2910040c -- src/runtime/collision_gpu.rs` = **+4 行**
   （`mod inputs_from_snapshot;` + 3 行 `pub use`），diff 中没有 `run` 函数体的任何行。

## 3. 独立 96/96 对账（不复用产品测试）

我自写 `verify_s2.rs`（本目录）：自己按 32 个字段名读三 case 的 `.bin`，自己从 bin 长度反推计数
（ngeom/nmesh/nmeshvert/nmeshgraph/nmeshpoly/nmeshpolyvert/nmeshpolymap/nhfield/nhfielddata/noct），
自己构造 `CollisionInputsSnapshot`，经 `inputs_from_snapshot` 组装后逐字段比较 **LE 字节** 与 **sha256**，
不调用产品测试的 support 代码。结果：

- **3 case × 32 = 96/96 字节相等、96/96 sha256 相等、0 不符**；
- 每项长度 == 我独立推导的 `unit × width × 4` 字节（host f32/i32 均 4 字节）；
- 三 case 的 `inputs_from_snapshot` 均 `Ok`，`mesh_poly_sidecar_present == true`；
- 三 case 的 `meta.txt` 自洽（`nworld=1`、`ngeom` 与 bins 一致、`naconmax>0`、`efc_width>0`）；
- 独立计数：mesh-box `nmesh=1/nmeshvert=8/nmeshpoly=6`、mesh-cylinder `nmesh=1/8/6`、
  mesh-mesh `nmesh=2/nmeshvert=16/nmeshpoly=12`（两份 cube，见 §5 O1）。
  （完整 96 行 sha 表：`logs/independent-96-reconcile.txt`；原始 bins 表：`logs/case-bins-table.txt`）

**旧 23 项输入**：我对**三个 case 都**构造了「9 项 mesh_poly 为空」的视图（产品测试只覆盖 mesh-box），
结果一致：validate Ok、sidecar null、9 项空、其余 23 项逐字节原样透传。

**负例**（产品测试 + 我的 harness 双向确认）：部分 mesh_poly → `InvalidArgument`（含「不完整」）；
`geom_xpos` 空 → `InvalidArgument`（含 `geom_xpos`）；`geom_type` 少 1 → `InvalidArgument`（含 `geom_type`）；
`meta.nworld/naconmax/efc_width` 置 0 → 各自 `InvalidArgument`；全部不 panic、不产生半初始化视图。

**`from_model_input` 桥接**：`cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features native-model-probe
--test g10_p5_collision_inputs_adapter -- --ignored`，我自己重跑 **1 passed / 0 failed**（pinned DLL）；
32 项与 S1 拥有式快照逐字节一致、frame 长度按单 world 校验。

## 4. 不变式、门禁、范围、行数

- **23 个 unchanged 路径**：`git diff 6a113daf..2910040c -- <23 路径>` 为空（18 pinned 源、`abi.rs`、
  `src/physics/collision/cuda/`、两个 fixtures 目录、M0b 证据）；`logs/unchanged-paths.txt`。
- **M3 case 目录零改动**：提交文件列表无 `mesh-multicontact-m3` 路径；该目录 33 文件（32 bin + meta）
  与父提交一致（`logs/scope.txt`、`logs/case-bins-table.txt`）。
- **ABI 未动**：`CollisionArgs` 504 B、入口 6、`DEVICE_SOURCES` 28 的断言随 unit 门禁跑过并通过；
  `Cargo.toml/Cargo.lock/rust-toolchain.toml` 0 diff（无新 feature）。
- **门禁**：我独立复跑 `.normify/gates.toml` 命令：fmt / clippy / clippy-cuda exit 0；`cargo test --locked`
  **997 passed / 0 failed / 376 ignored**（与 `.normify/runs/.../unit.json` 一致；+3 即本片默认 feature 测试）；
  额外跑 `clippy --features native-model-probe`（唯一覆盖 `from_model_input` 的 lint）exit 0、0 warning。
  没有重跑 `normify gate`（会覆盖已 verified 任务的 runs 产物），改为直跑同命令并对照归档。
- **范围**：提交 16 路径逐条对声明范围 = **16 在范围内 / 0 越界**。
- **行数（LF 口径）**：adapter 215、`collision_gpu.rs` 462、test 233、support 319、README 253、
  receipt 424（机器可读 JSON 例外）；全部 ≤500。
- **交付记录**：10 个交付文件 blob/sha256/bytes/lines **0 不符**；记录自身 blob 与 HEAD 一致。
  receipt `s2` 段 4 个交付条目 sha **0 不符**；顶层 `task_id` 仍为 S1（未被 S2 改写）、`s2.task_id` 正确。
- **provenance 交叉**：M0b manifest 与 M3 manifest 的 `arrays_order` 都与 `types.rs` 的 32 项顺序一致；
  M0b manifest 记录的 64 个 bin sha、M3 manifest 记录的 32 个 bin sha 与磁盘文件逐一相符。

## 5. 观察

### O1（一般）｜第三 case 的 fixture 归属写错：不是 `mesh-mesh-two-assets.xml`

S2 的 README（212、215 行）、receipt（325 行）、交付记录（91 行）都写第三 case 的 fixture 是
`fixtures/g10-p5-mesh-multicontact/mesh-mesh-two-assets.xml`。实际：

- M3 `mesh-case-manifest.json` 记录该 case 的 xml = `fixtures/g10-p5-mesh-multicontact/mesh-mesh.xml`，
  `xml_sha256=bb9ffb2f…`，qpos z=0.18；
- 我实测 `mesh-mesh.xml` 的 sha256 = `bb9ffb2f…`（与 M3 manifest 一致）；
- M3 case bins 实测 `nmesh=2 / nmeshvert=16 / nmeshpoly=12`，`mesh_vertnum=[8,…,8]`、`mesh_polynum=[6,…,6]`
  → 两份 cube（8 顶点/6 面）→ 对应 `mesh-mesh.xml`（cube + cube_b 缩放立方体，qpos 0.18）；
- 而 `mesh-mesh-two-assets.xml`（sha `1d7115…`）是 cube + tetra（8+4=12 顶点、6+4=10 面、qpos 0.09），
  它是 S3/S4 U6 分支 case 的 fixture，与 M3 case 不是同一个。

**影响**：bins 与 S2 测试本身不受影响（测试只读 bins、不读 xml），但证据追溯会指到错的 fixture，
后续片（S3 接线、台账引用）容易把两个 case 混用。**建议**：把 README §8.2 的 fixture 名、
receipt `s2.parity` 附近的同名描述、交付记录 `bin_inputs.note` 改为 `mesh-mesh.xml` 并附
`xml_sha256=bb9ffb2f…`（出自 M3 manifest）作为 provenance；S2 任务卡 acceptance 里旧的
`cases/{mesh-box,mesh-cylinder,mesh-mesh-two-assets}` 路径口径建议用 `normify fix` addenda 同步。

### O2（建议）｜`from_model_input` 只支持单 world 帧，nworld>1 无入口

`from_model_input` 要求 frame 长度 == `ngeom×3 / ngeom×9`（单 world），而 `validate_inputs` 要求
`geom_xpos` 长度 == `nworld × ngeom × 3`。因此 meta.nworld>1 时 `from_model_input` + `inputs_from_snapshot`
**必然**返回 `InvalidArgument`；本片没有测 nworld>1，也没有表达「重复 frame」的入口（视图字段是 pub 的，
调用方可以手工构造，但那绕过了桥接校验）。文档只写「重复由更上层负责」。建议在 README/台账写明
「本片桥接仅 nworld==1，S3 需定义多 world 组装方式（复制 frame 或在新入口内重复）」，避免 S3 误用。

### O3（建议）｜sidecar 判据两处重复

`mesh_poly_sidecar_present` 复制了 `run()` 内联判据（`collision_gpu.rs:162` 的 `!inputs.mesh_polynum.is_empty()`）。
本片按契约不改 `run`，可以理解；建议下一片把 `run` 改成调用该 helper（零行为变更），消除两处判据漂移风险。

### O4（建议）｜README §8.2 对桥接测试 meta 的描述偏强

README 写 `from_model_input` 桥接测试「用从 30 项字段/帧导出的 meta 通过 validate」。实际测试里 meta 是
手工构造的，只有 `ngeom`、`noct` 取自快照计数，其余字段是固定字面量（`naconmax=64`、`efc_width=4` 等）。
建议改成「手工构造的 meta（ngeom/noct 取自快照计数）」，避免读者以为 meta 由快照推导。

### O5（建议）｜共享证据目录里有一处禁词原文（否定句），目录级扫描会误报

`docs/development-evidence/product-collision-inputs-wiring/s1-observation-closure-receipt.json:70`
（S1 收口片，不在 S2 diff 内）把本任务 forbid_claim 的 6 条规定词条原文整串写进否定句。
S2 提交自身改动的文件我扫过 **0 命中**；但若今后有按目录全量扫描的检查，会被这条误判。
建议改写为不带字面的表述（如「不宣称窗口类、覆盖类、精度类、等价类结论」）。

### O6（建议）｜native 路径不在默认门禁内

gate 的 clippy/unit 用默认 features，`from_model_input`（cfg `native-model-probe`）与 S1 桥接测试
（`#[ignore]`）都不在门禁里；本片默认 feature 的 3 个 host 测试确实进了 unit（997=994+3，这点很好）。
native 侧靠 `logs/s2-fmt-clippy.log` 的显式 clippy 与定向重跑兜底，建议把它登记为「定向门禁项」，
否则后续改动桥接/原生读取没有自动保护。

### O7（建议）｜「gate 跑两次结果相同」只留最终一次归档

`s2-gate.log` 注明 gate 跑过两次且结果/计数相同，但 `.normify/runs/<task>/` 的 4 个 json 是覆盖写，
只有最终一次（与 S1 复核的同类观察一致）。建议保留两次 run 目录，或在 receipt 写明只归档最终树。

## 6. 证据完整性

- 我自己产出的独立复算：`verify_s2.rs`（96/96）、`logs/case-bins-table.txt`（96 行 sha 表来源）、
  `logs/independent-96-reconcile.txt`（逐字段 字节/sha/相等）。
- 结构检查脚本 `verify_s2_checks.py`：**22 项检查 0 问题**（视图顺序/类型、构造配对、交付记录 10 文件、
  case 目录计数与不变、日志 BOM）。
- `verify_receipt.py`：**132 项检查 0 问题**（receipt s2 交付条目、顶层 task_id 未变、M0b 64 bin sha、
  gate run json、arrays_order）。
- S2 证据日志（s2-targeted-default/native、s2-fmt-clippy、s2-gate）均 UTF-8 无 BOM；本复核目录亦无 BOM、LF。
- 交付记录 10 文件与 receipt s2 交付条目的 blob/sha/bytes/lines 全对；S1 的顶层 receipt 字段未被 S2 改写。

## 7. 复现命令（QA 侧，只读）

```
# 默认 feature 的 3 个 S2 host 测试
cargo test --locked --test g10_p5_collision_inputs_adapter
# from_model_input 桥接（需 MJWARP_MUJOCO_DLL 指向 target/toolchains/mujoco-3.12.0/package/bin/mujoco.dll）
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features native-model-probe \
  --test g10_p5_collision_inputs_adapter -- --ignored --test-threads=1
# 独立 96/96：把 verify_s2.rs 复制到 examples/qa_s2_verify.rs 后
cargo run --locked --example qa_s2_verify
# 结构/交付/收据检查
target/render-bvh-precheck/venv312/Scripts/python.exe verify_s2_checks.py
target/render-bvh-precheck/venv312/Scripts/python.exe verify_receipt.py
target/render-bvh-precheck/venv312/Scripts/python.exe bin_table.py
target/render-bvh-precheck/venv312/Scripts/python.exe attribution.py
# 门禁等价命令（不写 normify run 文件）
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --all-targets --locked --features cuda -- -D warnings
cargo test --locked
```

说明：为跑独立 harness，我在仓库临时建过 `examples/qa_s2_verify.rs` 并在跑完后删除（工作区已干净），
复现时同样复制/删除即可；harness 输出原始日志在 `logs/independent-96-reconcile.txt`。

## 8. 边界与不宣称

本复核只做宿主侧只读核对：未开 GPU、未开窗口、未重跑设备数值、未改产品代码/证据/判据/参考/容差。
S2 不做产品入口接线；本复核不对 host↔device 行为一致性、精度目标、性能、跨平台或任何验收结论作判断；
对账口径是 bins 原始字节与 adapter 输出字节的逐一相等，不是任何数值近似判据。
