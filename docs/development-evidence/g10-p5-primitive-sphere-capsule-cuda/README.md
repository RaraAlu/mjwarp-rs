# G10@P5 B2：sphere/capsule 族 primitive 设备实现（4 组合）

任务：`2026-10-26-g10-p5-primitive-sphere-capsule-cuda`（in_progress，owner Backend）。
只报「4 个组合在给定 fixture case 上对照 B0 上游 GPU 参考的结果」，不宣称 primitive 全组合覆盖 /
跨环境等价；判据/容差/参考均为 B1 同口径，未改动。

## 1. 范围与语义来源

pinned 上游 revision `71da24d956378a87a703b6e1442b13aec0c4ac29`
（`docs/evidence/d04-complex-physics/authoring-runs/run-5/installed-source/_src/`）：

| 组合（pinned key） | wrapper（collision_primitive.py） | 数值核心（collision_primitive_core.py） |
| --- | --- | --- |
| SPHERE-SPHERE | `sphere_sphere_wrapper` L357-429 | `sphere_sphere` L58-87 |
| SPHERE-CAPSULE | `sphere_capsule_wrapper` L432-507 | `sphere_capsule` L90-125 |
| SPHERE-CYLINDER | `sphere_cylinder_wrapper` L915-1000 | `sphere_cylinder` L393-462 |
| CAPSULE-CAPSULE | `capsule_capsule_wrapper` L510-597 | `capsule_capsule` L128-255 |

pinned dispatch 表：`_PRIMITIVE_COLLISIONS` L1342-1346（key 顺序即 wrapper 入参顺序）；
上游 pair 顺序按类型序规范化，故设备分派在几何顺序互换时交换 wrapper 入参，`d_contact_params.geoms`
仍取调用方传入顺序（与 B1 plane 分派同口径）。逐式转写保留 pinned 算式形态：`sphere_sphere` 的
`dir / dist` 逐分量除法与零距离 `n=(1,0,0)` fallback、`capsule_capsule` 的 det 分支/平行轴 4 端点分支
与 `dist <= margin` 过滤、`sphere_cylinder` 的侧面/端面/角点分支（端面分支调 `plane_sphere` 后反转法线）。
未改结合顺序、未加豁免、未加特判。

## 2. 文件与行数

| 文件 | 行数 | 说明 |
| --- | --- | --- |
| `src/physics/collision/cuda/primitive_sphere_capsule_shapes.cu` | 223 | 新增：4 个数值核心（复用 `g5_primitive_contact`/`g5_primitive_pair`/`g5_plane_sphere`） |
| `src/physics/collision/cuda/primitive_sphere_capsule_wrapper.cu` | 71 | 新增：4 个 wrapper（`g5_write_contact` + `make_frame`，单/双接触顺序按 pinned） |
| `src/physics/collision/cuda/product.cu` | 394 | 改：primitive 入口追加 4 组合分派（入口名不变；<450 未拆） |
| `src/runtime/collision_gpu/source.rs` | 133 | 改：`DEVICE_SOURCES` 25→27 段（本地 `include_str!`） |
| `src/runtime/collision_gpu/validate.rs` | 56 | 改：CPU 侧组合校验放行 4 组合（无序对） |
| `src/runtime/collision_gpu/primitive_tests.rs` | 174 | 改：27 段/顺序/符号断言 + B2 正负例 |
| `src/runtime/collision_gpu/multicontact_tests.rs` | 109 | 改：27 段与顺序断言 |
| `tests/g10_p5_primitive_sphere_capsule_cuda.rs` | 465 | 新增：4 case 对照测试（单 test，一次 NVRTC + 一次 context） |

新文件均 ≤500 行；`product.cu` 394 < 450，未拆分。

## 3. 拼装顺序（`DEVICE_SOURCES` 27 段）

`…→primitive→primitive_wrapper→boxbox→boxbox_wrapper→primitive_plane_common→
primitive_plane_convex→primitive_plane_shapes→primitive_plane_wrapper→
primitive_sphere_capsule_shapes→primitive_sphere_capsule_wrapper`，最后追加 `product.cu`。
B1 的 25 段相对顺序未变，B2 两段全部追加在 `primitive_plane_wrapper` 之后；
旧 25 段指纹 `source_bytes=222849 / sha256=59631d83…`（B1 README §6）→ 新 27 段见 §4，变化为预期。

## 4. CPU NVRTC 冒烟（窗口外；只拼装源 + NVRTC，不建 CUDA context）

命令：

```text
$env:PATH="D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1\bin;$env:PATH"
cargo test --locked --features cuda-probe --lib runtime::collision_gpu::nvrtc_smoke -- --include-ignored --nocapture
```

结果（1 passed / 0 failed，21.47 s）：

```text
nvrtc_smoke_ok ptx_bytes=3786439 entries=6 source_bytes=237212 source_sha256=054aaa40e27a73b054609d5046c4de885ff99cd75c6b4a88de86f0a9a681c618
```

- 入口数 **6 → 6 不变**（`mjwarp_collision_primitive` 内部 type-pair 分派；4 组合不是新 PTX 入口）。
- 拼装源 222849→237212 字节、sha 变化 = 追加 2 段设备源（预期）；`make_frame` 仍单一定义。
- 段数/顺序断言在 `primitive_tests.rs`、`multicontact_tests.rs` 中同步为 27。

## 5. 测试与门禁（窗口外）

| 命令 | 结果 |
| --- | --- |
| `cargo test --locked`（默认特性） | **974 passed / 0 failed / 371 ignored**（片前 B1 收口态 972/0/370：+1 新增组合校验单测、+1 ignored 窗口测试、+1 launcher 行序归一化纯 CPU 单测（§11）） |
| `cargo test --locked --features cuda-probe --lib runtime::collision_gpu -- --include-ignored` | **26 passed / 0 failed**（含 `nvrtc_smoke`；B1 为 24，+2 = 新组合校验单测 + B2 正负例） |
| `cargo test --locked --features cuda-probe --test g10_p5_primitive_sphere_capsule_cuda --no-run` | 编译通过（Executable 生成；窗口内不编译） |
| `cargo test --locked --no-default-features --lib` | 455 passed / 0 failed / 5 ignored |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --locked --all-targets --features cuda-probe -- -D warnings` | exit 0 |
| `cargo clippy --locked --all-targets -- -D warnings` | exit 0 |

设备对照测试（ignored）**尚未运行**：需 GPU 窗口 + Arch/Chief 双签，见 §6。


### 5.1 源指纹的 EOL 口径（两套都记）

- **工作树（CRLF，NVRTC 实际读到）**：27 段 `237212 / 054aaa40…`（本次冒烟实测）；旧 21 段 `199869 / 67fb8445…`。
- **git blob（LF 归一化，可从提交复现）**：27 段 **`236732 / 9807a539256cf4dff8c633d5d3d70a5653ad74cd585bb246f0c6eee056485e8f`**；
  旧 21 段同 `199869 / 67fb8445…`。
- 原因：新增 `.cu` 在作者工作树为 CRLF、提交为 LF；`include_str!` 读工作树字节。设备源码语义/判据不受影响。

## 6. 窗口计划与判据

- 对照参考：B0 上游 GPU 参考 `docs/development-evidence/g10-p5-primitive-capture/upstream-primitive-gpu.json`
  （sha256 `3a6475acf362795ab98e8ee39309b09102c2dc76f7a57c6c6dbc95a069a98734`，device=cuda）。
- 4 case：`sphere-sphere` / `sphere-capsule` / `sphere-cylinder` / `capsule-capsule`，GPU 参考 `nacon` 均 = 1。
- 判据（与 B1 同口径，未改）：**位级优先**；非位级 `dist` ≤4 ULP@自身量级（ref=0 用 4 ULP@1.0）、
  `pos` 4·ULP(参考三轴最大绝对值)、`frame` 4 ULP@1.0；`count`/结构（worldid/type/dim/geom/
  geomcollisionid/includemargin/adhesion/friction/solref/solreffriction/solimp）**零容差**；
  `efc_address`/`overflow` 参考未采集 → 只做设备侧不变量（efc 全 -1、overflow 全 0）；
  行序不作判据，按 `geomcollisionid` 结构键排序配对。
- `capsule-capsule` 的 CPU 预检 `nacon=2` 与 GPU 参考 `nacon=1` 属后端差异（CPU 后端与 GPU 后端接触条数
  可不同），预检只证 fixture 有效；**判据参考是 GPU 参考，不按 CPU 预检凑 count**。
- 窗口内同时回归：**B1 plane 6 case**（`g10_p5_primitive_plane_cuda`）、既有 primitive 3 case
  （`g10_p5_primitive_cuda`）、launcher 3 case（`g10_p5_nvrtc_launcher`），用预构建二进制并行跑，
  窗口内零 Rust 编译。
- 窗口纪律：开窗前重采基线（free ≥10240 MiB、无本任务 CUDA 计算进程、驱动/温度正常）；
  0.25 s 采样 + guard v2；post 快照 + 结束基线 + `query-compute-apps` 无残留；单窗口短跑。

## 7. ABI / T4 身份声明

- **0 个新 ABI 字段**：`CollisionArgs` / `CollisionContactOut` / `collision_gpu/abi.rs` 未改；
  **0 个新入口**（PTX 入口 6 → 6）。
- **T4 `physics_core` 组合身份未变**：未改 `src/physics/mod.rs`、未另立 repin、`fixtures/` 零改动；
  新增 `.cu` 片段经 `src/runtime/collision_gpu/source.rs` 本地 `include_str!` 引入（B1 同法）。

## 8. 输入来源

4 个 case 的 fixture：`fixtures/g10-p5-primitive/{sphere-sphere,sphere-capsule,sphere-cylinder,capsule-capsule}.xml`，
sha256 与 B0 `docs/development-evidence/g10-p5-primitive-capture/inputs-sha256.json` 逐条一致
（`716c4165…` / `e381ae66…` / `26b0d31d…` / `05f6937e…`，本次实测复算相符）。
参考 JSON 只存 `geom_xpos` / `geom_xmat`；两个 geom 的 `geom_size` / `geom_type` 按 fixture 在测试内
硬编码（与 B1 plane 测试同口径；B1 T-5 已记录该回退方式）。

## 9. 未覆盖项 / 状态

- 设备对照窗口已跑（attempt1 有效非收口 / attempt2 收口），逐 case N/N 见 §10。
- `capsule_capsule` 平行轴分支、`sphere_cylinder` 端面/角点分支是否被 4 个 fixture 覆盖，按窗口结果
  如实记录；未覆盖分支不宣称验证。
- 未宣称 primitive 全部组合覆盖、未宣称跨平台/跨环境等价。

## 10. GPU 窗口结果

**attempt1（2026-10-26 21:07:51–21:08:31，有效轮、非收口轮）**：B2 4/4 通过；B1 plane 6/6；primitive 3/3
bit-exact；launcher `ellipsoid-box` 因**既有测试原始行序硬比**报 14 条（两行 worldid 0/1 整体互换；按 worldid
配对后所有字段位级相同、0 ULP）→ 触发行序归一化修复（§11），本窗口不计收口。窗口事实：body 35.1 s；
baseline `3407/12655`；end `3408/12654`；samples 134；peak 4186 MiB；free_min 11876 MiB；util_max 59%；
residual 0；聚合守卫通过（delta_peak 774 MiB < 3072）。

**attempt2（2026-10-26 21:13:40–21:14:10，收口轮，4 二进制并行，body 25.5 s）**

| 对象 | 结果 |
| --- | --- |
| B2 4 case | **4/4 通过**：sphere-sphere 1/1、sphere-capsule 1/1、sphere-cylinder 1/1、capsule-capsule 1/1（nacon 与 GPU 参考一致，failures=0） |
| B1 plane 回归 | **6/6 通过**（nacon 1/1/1/1/3/3，failures=0） |
| primitive 回归 | **3/3 bit-exact**（80/80，max ULP 0） |
| launcher 回归 | **3/3 bit-exact**：hfield-sphere 314/314、sdf-box 3122/3122、ellipsoid-box 80/80，max ULP 0（行序归一化后） |

窗口事实：baseline `3408/12654`（3 点）；post/结束基线 `3408/12654`（3 点）；采样 **100** 点；
`used_peak=4784 MiB`、`free_min=11278 MiB`、`util_max=41%`；`residual=0`；聚合守卫通过
（`delta_peak=1370 MiB < 3072`、util 连续段 0）；post compute-apps 无本任务进程；窗口内零 Rust 编译。
签名二进制 sha：B2 `8359f17c…`、plane `43a6f32b…`、primitive `3ac9d52c…`、launcher `23ae4d82…`
（launcher 因 §11 修复重建，其余未变）。

原始日志：`window-attempt1/`、`window-attempt2/`（baseline / facts / guard.log / samples.csv / 各 run 日志）。

## 11. 回归测试行序归一化（既有测法脆弱性修复；只改 `tests/`）

- 现象：attempt1 launcher `ellipsoid-box` 14 条红，实为**两行整体互换**（dev worldid [0,1] vs ref [1,0]；
  dist/pos/frame 对应交换；按 worldid 配对后 0 ULP）。
- 根因：`tests/g10_p5_nvrtc_launcher.rs` 按原始行序硬比（`chunks_exact` + `check_exact(worldid)`），
  而多 world 的接触写入依赖共享 `atomicAdd`，跨 world 行序本无保证；B2 新增 2 段设备源改变模块布局/调度
  触发该脆弱性，**非数值语义回退**。
- 修法（复用 G5 既有排序语义，判据不放宽）：`g5_contact_order` 先按 `worldid` 分组、组内按
  `sort_contacts` 键 `dist → pos[0..2] → geomcollisionid` 稳定排序（与
  `tests/support/stage2_equivalence_compare.rs::deterministic` 同键），再逐字段走原检查器；
  判据/字段集/期望/参考一律不动。
- 证明（纯 CPU；用仓库内 `ellipsoid-box/out1` dump 构造 attempt1 同型交换）：
  `raw_order_failures=14 sorted_failures=0`，旧口径 ULP 指纹 `dist 76 / pos 2676526 / frame 28621803`
  与 attempt1 日志完全一致 → 证明是测法脆弱、非数值回退。
- 同类脆弱测试审计：全仓唯一 `chunks_exact(width)` + 原始行序硬比即本文件（已修）；
  `stage2_equivalence_compare::deterministic`、`multicontact_compare_support`、`primitive_compare_support`
  已是排序口径；`g10_p5_sdf_host.rs`、`g10_p5_sdf_relmat.rs`、`gjk_epa.rs`、`stage2_equivalence_window.rs`
  有 worldid 相关比对但非该模式 → 记**观察/债务**，本片不改。
- 旧证据有效性：launcher 在 raw-order 下通过的旧记录仍有效（旧口径更严）；新比较器从本片起生效。
- **launcher 二进制 sha 作废链（留痕）**：d08491c6…（B1 期旧）→ e330e9d1…（删 compare_exact! 宏后）→
  1af86cba…（proof 测试首版重写前）→ 6e4fc62a…（proof 迭代）→ 6000a50c…（proof 定稿前）→
  **23ae4d82…（attempt2 唯一实际窗口集合，Chief 已重签确认）** →（500 行 size gate 拆分：纯搬移）→
  5c191046…（提交源构建）。除 23ae4d82… 外的 launcher sha 全部作废、不得用于任何结果引用。

- **500 行上限拆分（commit 前）**：加完归一化后 	ests/g10_p5_nvrtc_launcher.rs 达 830 行、超 500 上限，
  normify size gate 拒提交；故把比对机制抽到 	ests/support/launcher_compare_support.rs（417 行），主测试文件
  419 行；为纯搬移（无逻辑改动，proof 单测输出逐字相同）。因此：**attempt2 窗口实际用的是拆分前二进制
  23ae4d82…**，仓库提交源构建出的对应二进制为 35ba212d…——两者仅源代码布局不同、语义与判据不变，
  QA 可按 diff（纯移动 + pub(crate) 可见性）复核。
- **修复落位（Governance）**：受 normify 文件范围规则限制（B2 已 in_progress 且 files 含 `tests/`），
  独立修复片无法与本片并行 in_progress（`normify start` 报 conflict:本任务）；故修复并入本片提交，
  **改动严格限于 `tests/g10_p5_nvrtc_launcher.rs`**（未触 `src/`、未动判据/参考/期望），
  证明与审计即本 §11。

## 12. QA 观察收口（B1/B2 共同项，2026-10-26）

- O2（EOL 指纹）：见 §5.1（工作树 CRLF 与 git blob LF 并列）；B2 交付 smokes 用工作树口径。
- O5（证据卫生）：本目录日志已统一 UTF-8 无 BOM；`console.log` 由 UTF-16LE 转 UTF-8。
- O1/TETRA 类问题不适用 B2（B2 无 mesh 输入）。
- `capsule_capsule` 平行轴分支/`sphere_cylinder` 端面角点分支的 fixture 覆盖情况按 §9 如实记录，不宣称未覆盖分支已验证。
