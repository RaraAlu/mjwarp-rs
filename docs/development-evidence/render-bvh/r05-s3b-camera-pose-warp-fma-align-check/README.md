# R05-S3b 相机世界位姿 FMA 收缩对齐 · QA 独立复核

任务卡：`2026-10-28-render-bvh-camera-pose-warp-fma-align-check`（priority 61，Owner QA）
被核对象：`2026-10-28-render-bvh-camera-pose-warp-fma-align`（**verified**）
被核提交：`6f246b46`（代码 `src/render/kernel/cast.rs`）→ `d42a465a`（证据）→ `f98d74a5`（收口）
复核基线 HEAD：`f98d74a5`；复核期未重冻结 guards、未开 GPU 窗口、未改任何生产代码。

---

## 1. 结论

**verdict：pass / 阻断 0 / 观察 9。**

被核实现的「host 派生相机世界位姿 = 上游 GPU 链的逐元素唯一 FMA 收缩序」这一主张，我用**四条彼此独立**的路径自己重算/重跑后成立：

1. **PTX 依据自核**：pinned `sm89.ptx`（993 976 B / 28 448 行 / `5ff8714d…`）中 `mul_quat` 切片（L14719–14733）有 **6** 条 `fma.rn.f32`，`quat_to_mat` 矩阵切片（L14734–14782）**0** 条、只有 30 条 `mul.f32/add.f32/sub.f32`；已提交 `upstream-camera-kernel-ptx-excerpt.txt` 的每一条指令行都能在 PTX 原文中**逐字命中**。
2. **收缩序独立重推**：我自己按 PTX 源序（不可重结合，`--fmad=true` 只允许收缩，不允许重排/结合）构建候选族 = 对角元各 **12** 候选、非对角元各 **3** 候选（全局 **1 259 712 = 12³×3⁶**），用**精确有理数 f32 fma**（Fraction + RNE，非 numpy 近似）在 1003 条参考四元数上求解：**9 个元素各自恰好 1 个候选全中**，收敛到唯一解最多需 **16** 条四元数（我的逐元素值为 7/16/4/5/4/6/2/3/7，与证据一致）。**不是「照着 cam1 拟合」**：唯一解在全 1003 条上成立。
3. **全量与对照**：实现公式（我按 `cast.rs` 源码手工转写，独立于证据脚本）对 pinned GPU 链参考 **1003×9 = 9027 项 0 差异**；同一批上未收缩代数展开版 **2614 项 / 947 条**差 —— 两个数字都复现。
4. **Rust 侧独立复现**：（a）我 `touch` 后强制重编并跑 `cargo test --locked` → 单元 `render::kernel::cast::tests::derived_contraction_matches_upstream_gpu_bits` **ok**；（b）另写**硬件 FMA 探针**（`_mm_fmadd_ss` = `vfmadd231ss`，与生产 `crate::math::fma::fma_f32` 硬件主路同一指令）对全 1003 条计算 9 元素：**9027 项 0 差异**，且 opt-level=0 与 3 输出**逐位相同**。

**三条硬门我全部独立复核通过**（含对真实采集 `d.cam_xmat` 的溯源，见 §4）；**零外溢**、内核三 sha 逐位不变、`cast.rs` 481 行 ≤500；门禁自跑全 exit 0、unit **1092 / 0 / 389**。

**观察 9 条均不影响验收**（O1/O3/O4 建议改措辞/编码，O2 建议修影响面清单，O5/O6 建议补留档，O7/O8/O9 记录）。

---

## 2. 我做了什么（自算清单，不采信实现声明）

| 复核点 | 我的方法 | 结果 |
| --- | --- | --- |
| PTX 无 fma 反证 | 自己打开 pinned `sm89.ptx` 按行号切 A/B 两片，数 `fma.rn` | A=6、B=0、B 的 `mul/add/sub`=30 |
| 证据 PTX 片段可信 | 逐条指令与 PTX 原文比对（不是看证据自述） | 60/60 指令行命中；仅 2 条我自己比对的「段标题」注释行不属于 PTX |
| 候选空间 | 自己按源序枚举（12/3/12/3/12/3/12/3/12） | 全局 1 259 712 |
| 唯一性 | 逐元素独立求解 + 最小前缀条数 | 9 元素各 1 个候选全中；最多 16 条 |
| 实现 vs GPU | 手工转写实现公式 + 精确 f32 运算 | 9027 项 0 差 |
| 反例对照 | 未收缩展开版 | 2614 项 / 947 条 |
| 参考链口径 | 机检 `mul_quat_bits == quat_bits` 行数 | 1003/1003 |
| 采集溯源 | 与 RCA 证据 `upstream-camworld.json` 的 pinned-GPU `d.cam_xmat` 位型比对 | cam0/cam1 逐位相等 |
| scene-a cam-persp | 与已 verified 参考 `camera_data_check[0].xmat_bits` 比对 | 9/9 |
| Rust 实现 | 硬件 FMA 探针全 1003 条 + crate 单测 | 0 差；opt0==opt3 |
| 内核门 | 自己重跑 NVRTC 纯编译门 | `72bc364c…`/`97467d83…`/264 015 |
| 门禁 | 自己跑 fmt/clippy/clippy-cuda/unit/no-run/validate | 全 exit 0；unit 1092 |
| 零外溢 | `git diff a7beafae..HEAD` 全量路径 + 受保护目录 | 只有 `cast.rs` + 证据/卡 |

脚本：`recompute.py`（自校清单 `problems` = 空，见 `recompute.json`）；日志：`logs/`。

---

## 3. 收缩序为什么必须「反推」而不是从 PTX 读（我核的口径）

PTX 切片 B 的指令序列（我摘录并在 PTX 原文核对）为：

```text
mul.f32 %f106,%f90,%f90      # q0*q0        mul.f32 %f110,%f95,%f95    # q1*q1
mul.f32 %f113,%f100,%f100    # q2*q2        mul.f32 %f115,%f105,%f105  # q3*q3
add.f32 %f116,%f106,%f110    sub.f32 %f117,%f116,%f113   sub.f32 %f118,%f117,%f115
...（非对角 2*(ab±cd)）...   st.global.f32 ×9
```

矩阵部分在 PTX 层**没有任何 fma**，因此单次舍入的收缩只能发生在驱动 JIT（ptxas→SASS）；
这解释了「必须用 pinned GPU 输出反推、且必须逐元素唯一解」这条方法。我的候选族正是按
「源序固定 + 每步可被驱动收缩」构造的：对角元 = 3（首个乘积三选：不收缩 / 收缩左 / 收缩右）
× 2（第三个平方项收缩与否）× 2（第四个平方项收缩与否）= 12；非对角元 = {不收缩, 收缩左乘积,
收缩右乘积} = 3。

---

## 4. 三条硬门（含对真实采集位姿的溯源）

| 门 | 内容 | 我的复核 |
| --- | --- | --- |
| 硬门 1 | hfield cam0/cam1 `cam_xmat` 9/9、9/9 位等 | 参考链 digest 第 0/1 行 = RCA 证据 `upstream-camworld.json` 的 pinned-GPU `d.cam_xmat` 位型（cam0 恒等；cam1 `0x3f1b8312 … 0xb31f2429 …`）；实现公式对这两条 0 差 |
| 硬门 2 | 1003 条随机四元数 × 9 = 9027 项 0 差异（seed 20261029） | 精确 f32 重算 0 差；另用 Rust 硬件 FMA 探针 0 差；同一批 plain 版 2614/947 |
| 硬门 3 | scene-a `cam-persp` 9/9 | 对已 verified 参考 `fixtures/render-bvh/scene-a/s7b/depth_seg.json::camera_data_check[0].xmat_bits` 9/9；digest 第 2 行同参考亦 0 差 |

附带事实（我自算）：同一 cam1 上 mujoco 主机侧 CPU（`mjd`）位型与 pinned-GPU 采集位型**不同**
（9 元素中多处差 1–2 ULP），与「收缩发生在 GPU JIT」这条因果相吻合 —— 也就是说本片对齐的是
**产品采集时实际使用的那份 GPU 位姿**，不是主机侧 mujoco 的展开值。

---

## 5. 零外溢与不变量（我自跑/自读）

- `git diff --name-status a7beafae..f98d74a5`：**只有** `.normify/changes/…-fma-align.json`（M）、
  `src/render/kernel/cast.rs`（M）、证据目录 22 个新增文件。
- `git diff --stat a7beafae..HEAD -- src/math src/render/kernel/cuda tests fixtures scripts src/render/bvh src/model src/render/camera.rs`：**空**。
  ⇒ `src/math/mod.rs`（含 `fma.rs`）、`cuda/**.cu`、`tests/`、`fixtures/`、`hfield_scene.rs` 零改动。
- `cast.rs`：committed blob **481 行**（LF 481 / CR 0 / 19 561 B），≤500；`git show --numstat` = **+72/−2**。
- 内核编译门（我自己重跑）：`source_sha256=72bc364c…`、`ptx_sha256=97467d83…`、`ptx_bytes=264015`，与改动前逐位相同 ⇒ 设备源零改动。
- 不变量：`DEVICE_SOURCES: [(&str,&str); 28]`（`collision_gpu/source.rs:51`）、`MEGAKERNEL_SLOTS.len()==81`（单元断言）、`size_of::<CollisionArgs>()==504`（`abi_tests.rs:41` 断言，均随 unit 门禁通过）。

---

## 6. 门禁（我自己跑，`logs/gates.txt`）

| 门禁 | 我的结果 |
| --- | --- |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0（我 touch `cast.rs` 后强制 `Checking mjwarp-rs` 重编再判） |
| `cargo clippy --all-targets --locked --features cuda -- -D warnings` | exit 0（`cuda` 为默认特性 ⇒ 与上条同一编译单元，缓存命中） |
| `cargo test --locked` | **1092 passed / 0 failed / 389 ignored**（134 个 `test result:` 行） |
| `cargo test --locked --no-run` | exit 0（132 Executable） |
| 内核编译门 | 1 passed；三 sha 见 §5 |
| `normify validate --fingerprints` | 错误 0、警告 5（既有架构提示） |
| `normify impact mjwarp.render`（模块级） | exit 0（源码 39、API 11） |

文件级 `normify impact src/render/kernel/cast.rs` = **exit 1「没有模块对应」**：我复核了被核证据的处置 ——
`gates.txt` 与 `impact.md` 都**显式**标 `N/A（未登记，非门禁）`，并给了替代口径（模块级 impact exit 0 +
任务 scope + `validate --fingerprints` 错误 0），未把它混进「全绿」清单，披露成立（观察 O7）。

被核对象 `cargo test` 里与本片路径相关的实体（我自己跑到的结果行）：

```text
unittests src\lib.rs            -> ok. 730 passed; 0 failed; 137 ignored
tests\render_e2e.rs             -> ok. 1 passed; 0 failed; 5 ignored
tests\render_kernel_depth_seg.rs-> ok. 2 passed; 0 failed; 6 ignored
tests\render_hfield_reference.rs-> ok. 3 passed; 0 failed; 2 ignored
test render::kernel::cast::tests::derived_contraction_matches_upstream_gpu_bits ... ok
```

---

## 7. 两条 need-to-know 机器事实（我独立复测）

### 7.1 `f32::mul_add` 不可用作单次舍入（结论成立，但证据的机制表述需修正 → O3）

我写了独立探针（`materials/muladd_steps.rs`），用 `black_box` 阻止常量折叠，取被核单测里同一组
cam1 四元数，逐步与精确 f32 fma 对比：

| 步 | 精确单次舍入 | `f32::mul_add` 实测 |
| --- | --- | --- |
| `q11 = q1*q1` | `0x3e83bec4` | `0x3e83bec4` |
| `s1 = fma(q0,q0,q11)` | `0x3f4dc189` | `0x3f4dc189` |
| `s2 = fma(-q2,q2,s1)` | **`0x3f3daba1`** | **`0x3f3daba0`**（低 1 ULP） |
| `s3 = fma(-q3,q3,s2)` | `0x3f1b8312` | `0x3f1b8311` |

关键实测：`opt-level=0` 与 `opt-level=3`（同样 `black_box`）**位型完全相同**，即运行时它
**两个 profile 都不是正确舍入**；差异只来自「常量折叠 vs 运行时」。⇒ 改用项目精确
`crate::math::fma::fma_f32` 的决定成立（且比证据所述更强）；但 `fma-toolchain-note.md` 里
「debug 与 release 会给出不同位型」的归因不准确（观察 O3）。我另确认 `fma.rs` 本次零改动，
生产路径走 `is_x86_feature_detected!("fma")` → 内联 `vfmadd231ss`（本机为真），与我的探针同指令。

### 7.2 `render_megakernel.cu` 的 CRLF 事件

- 现状可核：工作树 `src/render/kernel/cuda/render_megakernel.cu` 与 committed blob **逐字节相同**
  （17 814 B / LF 347 / CR 0 / sha256 `62f0aa8a…`），`git status` 对该路径**无输出**，
  我自己重跑内核门得 `source_sha256=72bc364c…`。
- 「一度 `f3e639ac` → 归一化回 `72bc364c`」这段**过程**在已提交物里没有机器证据：
  `f3e639ac` 在证据目录零出现，且 `zero-change-paths.md` 写「详见 kernel-gate-after.log」，
  但该日志只含最终成功那一次运行（观察 O4）。

---

## 8. 影响面清单（我核的结果与证据表述有差异 → O2）

`CameraPose::from_fields`（本片改动的那条路径）的**实际引用点**（`git grep` 全仓，我逐条看过上下文）：

| 位置 | 是否引用被改路径 | 断言性质 |
| --- | --- | --- |
| `tests/render_e2e.rs:160` | 是 | **容差**（`camera0 vs host 派生 max_ulp <= 1`，注释明确「辅助口径」）；位等断言全部在参考 `camera_data` 直传路径上 |
| `tests/render_kernel_depth_seg.rs:144` + `:137`（经 `s7b1::check_camera_data`） | 是 | **容差**（`fixed_max <= 1.0e-6`）；位等断言在 `camera_data_poses` 直传路径（L131–133） |
| `tests/support/render_bvh/hfield_scene.rs:220` → `tests/render_hfield_reference.rs` | 是 | 位级（GPU `#[ignore]` 实体，depth `max_ulp<=4`）；本次 unit 门禁下 3 passed / 2 ignored |
| `src/render/kernel/renderer.rs:49` | 否（仅文档注释） | — |
| `tests/support/render_bvh/s7b1.rs:100` | 是（helper，仅被 `render_kernel_depth_seg` 调用） | 容差 |

`src/` 内无生产调用点；`render_camera_rays` / `render_context_lifecycle` / `render_kernel_flags` /
`render_kernel_rgb` / `render_r08_cameras` / `render_r10_multi_world` 走 `s7b1::camera_data_poses`
（参考位姿直传）或 `CameraModel::from_fields`（**不同**类型），**不经过**被改路径。
⇒ 被核证据 §8.2 把「9 个 render exe」都列为「实际引用该路径」不准确：行为面是 **3 个 exe**；
另外 6 个的 exe sha 变化来自重链（lib 重编），与行为路径无关。这一点是措辞/清单问题，不影响改动正确性
（实际行为面比声明**更小**）。

---

## 9. 观察（8 条，均不阻断；建议见「建议」列）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 建议 | **4 个证据文件是 UTF-16LE（无 BOM）+ CRLF**：`materials/hfield-final.log`、`kernel-gate-after.log`、`reference-chain-identity.txt`、`verify-contraction.log`（我按 NUL 字节/UTF-8 解码判定；内容可正常解码且我已解码核对）。与 `manifest.txt` 头「all files LF, no BOM」及卡面「LF 无 BOM」不符；这些文件无法直接 grep/diff | 修：用 UTF-8 LF 重存（或改 manifest 口径并写明是 PowerShell 原始重定向产物） |
| O2 | 建议 | **影响面清单过宽**：README §8.2 称 9 个 render exe「实际引用该路径」，实测只有 3 个（`render_e2e` / `render_kernel_depth_seg` / `render_hfield_reference`）；其余 6 个走参考位姿直传或别的 `*::from_fields`。9 条 exe 的 sha 变化来自重链 | 修：把「引用该路径的 exe」与「sha 会变的 exe」分成两句写；按断言性质（位级/容差/不经过）列 |
| O3 | 建议 | **`fma-toolchain-note.md` 机制表述不准**：实测 `f32::mul_add` 在 opt0/opt3（black_box）位型相同且都不是正确舍入（单步反例 `fma(-q2,q2,s1)`：应 `0x3f3daba1`、实得 `0x3f3daba0`）；差异源是「常量折叠 vs 运行时」，不是 debug/release | 修：表格改成「常量折叠 / 运行时」两行，并附单步反例（结论与选型不变） |
| O4 | 建议 | **CRLF 事件的过程不可核**：`f3e639ac` 中间值在已提交物中零出现；`zero-change-paths.md` 的「详见 kernel-gate-after.log」指向的日志只有最终 `72bc364c` 那次成功运行 | 修：把中间那次的日志（或删掉中间值的表述、只留「按 blob 归一化」事实）改准 |
| O5 | 建议 | **本地调试的原始日志未入证据**：8 条回归 8/8 exit 0、world dir `42449→0`、scene-a 重放 9/9 只有文字/计数（`world-dir-counts.json`）；原始日志留在 `target/render-bvh/r05-s3b-window/fix-local-logs/reg-*.log`（瞬态）。hfield 两组判据有 `hfield-final.log` 可核 | 补：把 8 条回归日志（或其 sha256 + 行数）纳入证据，或写明瞬态路径 |
| O6 | 记录 | 新口径的**位级**回归网 = `cast.rs` 里 1 条 4 案例单测；`render_kernel_depth_seg` 里 fixed-quat 路径仍为容差断言（`≤1e-6`，本片要求 `tests/` 零改动，未收紧） | 接受；后续可把该断言收紧到位等（非本卡范围） |
| O7 | 记录 | `normify impact src/render/kernel/cast.rs` = exit 1「没有模块对应」；证据已在 `gates.txt`/`impact.md` 显式标 `N/A（未登记，非门禁）` 并给替代口径，未混入全绿清单 | 接受（既有治理缺口，backlog `arch-volume-module-sources` 覆盖） |
| O8 | 记录 | 证据的候选枚举用 float64 近似 fma（`(a64*b64+c64).astype(f32)`）做初筛；我用 Fraction 精确单次舍入重跑，唯一解与 9027 项 0 差一致 ⇒ 该近似未改变结论 | 接受（若将来扩候选族，建议改精确 fma 初筛） |
| O9 | 记录 | **复核期 HEAD 漂移**：派工 HEAD `f98d74a5` → 复核期间新增 `5dd881d0`（工程(渲染): R05-S3b 冻结脚本加 `-Attempt` 参数），只改 `scripts/validate/render-bvh/freeze-guards.ps1`；我核过 `git diff f98d74a5..5dd881d0` **未触**被核的 `cast.rs`/证据/判据/参考/fixture | 接受（我的证据提交落在 `5dd881d0` 之上；若需严格基线请指定提交号） |

---

## 10. 边界（我没做/不能做的）

1. **未开 GPU 窗口、未重跑渲染判据**：`depth mismatch=0 / max_ulp=0 / beyond=0`、RGB/seg 0 来自被核证据的
   **本地调试留档**（`materials/hfield-final.log`，我解码后核对了 4 个相机 8 组数字与
   `local-debug-summary.md` 一致、`depth_zero_pixels` 29776/37050 与 S3b 参考规模一致），**不等于**我独立渲染验证。
2. **world dir 位差 `42449→0` 未做像素级重跑**：它由「`cam_xmat` 9/9 位等 + 同一 local dirs + 同一 plain 点积序」
   逻辑蕴含（等价输入必得等价输出）；`42449` 的前值取自 RCA 已 verified 证据。
3. **未直接调用 crate 私有 `warp_quat_to_mat`**：Rust 侧 9027 项由「硬件 FMA 探针（与 `fma_f32` 硬件主路同指令）
   + crate 单测 4 案例」覆盖，不是对私有函数逐项签名。
4. **未重跑 attempt2 窗口材料**（不属本卡）；本卡不产生任何渲染数值验收结论。
5. 未 re-freeze guards、未改任何生产代码/判据/参考/fixture；复核期 HEAD 由派工基线 `f98d74a5` 漂到 `5dd881d0`
   （只改 `scripts/validate/render-bvh/freeze-guards.ps1`，未触被核路径，见 O9），本卡证据提交在 `5dd881d0` 之上。

---

## 11. 复现命令（仓库根目录）

```text
$env:PATH = 'D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1\bin;' + $env:PATH
target\render-bvh-precheck\venv312\Scripts\python.exe -B docs\development-evidence\render-bvh\r05-s3b-camera-pose-warp-fma-align-check\recompute.py
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --all-targets --locked --features cuda -- -D warnings
cargo test --locked
cargo test --locked --no-run
cargo test --locked --features cuda-probe --test render_kernel_rgb -- --ignored megakernel_source_compiles_under_render_gate --test-threads=1 --nocapture
normify validate --fingerprints
normify impact mjwarp.render
# Rust 侧硬件 FMA 探针（opt0 与 opt3 各一次；输入 = 证据 gpu-reference-digest.json 的 1003 条 quat_bits）
rustc -C opt-level=0 materials\hw_fma_probe.rs -o target\qa-hw0.exe
```

---

## 12. 收口

- 本卡证据提交：本目录（README/receipt/manifest/recompute.py+json/logs/materials）。
- 被核对象两段提交 `6f246b46` + `d42a465a` + 收口 `f98d74a5` 保持不动（本卡只读）。
- 结论仅供 Arch/Chief 决定 attempt2 窗口与 guards 重冻结；本卡不出窗口结论。
