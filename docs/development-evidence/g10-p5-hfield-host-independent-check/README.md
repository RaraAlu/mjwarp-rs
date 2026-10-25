# G4 hfield 宿主窄相：QA 独立复核（`2026-10-25-g10-p5-hfield-host-independent-check`）

复核对象：**`1f882361`**（G4 交付，父 `e3da9f3e`）。任务：`2026-10-25-g10-p5-hfield-host`（Backend）。
本复核**不改产品代码、不改交付证据、不改夹具、不跑 GPU**；只写本目录。

## 1. 结论

**0 阻断，可收口；2 条一般（均为证据/测试 helper 级，不影响 G4 交付判据） + 2 条建议。**

独立手段：QA 自写 3.12 pinned 上游重采脚本（33 个场景）、自写 hfield 宿主 **oracle**（filter / 子网格 / prism /
四接触选择 / 去重 / 溢出全部独立重写，只有窄相 `ccd()` 复用产品公开 API——该件 G2 已独立复核）、
临时 `git worktree` 内的产品探针（直接调产品 `hfield_pair_contacts` / `hfield_filter`），三方逐位比对。

关键结果：

- **交付三 case 的参考 provenance 成立**：QA 独立重采两次与交付 fixture **逐位相同 3/3**
  （box 8/8、sphere 8/8、capsule 7/7，`dist` 位等 8/8、8/8、7/7，`pos`/`frame` 全位等）；
  fixture 的 device=cpu、pinned 版本、`authoring_script_sha256`、`qpos_bits`、两次捕获位等全部核对一致。
- **产品 hfield 宿主导出 == QA 独立 oracle，逐位一致**：33 场景 / 37 world，
  接触（`dist`/`pos`/`frame`）**37/37 world 逐位相同**，filter bounds **37/37 位等**，
  `prism_count` **37/37 相等**，overflow **37/37 一致**。
- **产品 vs 同配方上游 f32**：33 场景中 30 个逐位一致；不一致的 3 个为窄相级（G2 继承）小差：
  `syn-rot-box` dist 1 ULP；`syn-class-cylinder` dist 0 ULP / pos 2 ULP；
  `syn-class-ellipsoid` dist 4 ULP / pos 绝对差 4.5e-9 / frame 9.9e-7。**这与 QA oracle 对上游的差相同**
  （oracle 也只用产品 `ccd()`），即差异归属 G2 窄相实现，不属 G4 逻辑。
- **`MJ_MAXCONPAIR=50` 溢出**：QA 自造 11×11 / 13×13 超容量场景，产品 `prism_count=50`、
  `overflow=true`（上游对应 `OverflowType.HFIELD`=32），4 接触与上游逐位一致，无越界写（写前判 `count>=50`）。
- **重复接触点与 `1e-3` 去重**：按 `worldid` 分组 + `(dist,pos,frame)` 位型多重集比较；
  重复行确实存在且被写出（`hfield-capsule` world0 四个接触同一点），产品与上游逐位一致，
  没有出现「比上游多丢或多留」的情况。上游 `geomcollisionid` 为每对接触序号（0..n-1），已核。
- **不回归**：在 `1f882361` 的临时 worktree 实跑 `cargo test --locked` = **572 passed / 0 failed / 1 ignored**，
  与交付门禁计数一致；G2 的 `case_evidence::upstream_f32_within_4ulp`、G3 的
  `sdf_box_host_matches_upstream_f32_within_4ulp`、G4 三个 hfield 测试全部通过。

## 2. 复核对象与范围

| 项 | 值 |
| --- | --- |
| 复核提交 | `1f882361`（`git show --name-only` 18 文件，全部在声明范围） |
| 产品改动 | `src/physics/collision/hfield/{mod,filter,driver,tests}.rs` 新增；`collision.rs` 注册 `pub mod hfield`；`gjk/mod.rs` 仅加 `pub(crate) use support::support`（`ccd`/`math`/`make_frame` 未改） |
| 声明范围外改动 | **无**（未改 `Cargo.toml`/`Cargo.lock`/`rust-toolchain.toml`、`fixtures/mjwarp-complex-physics/*.xml`、S2–S8 证据） |
| 复核目标 | 与登记条目一致：filter 候选集、prism 六顶点、CONVEX 路由、四接触+去重、MAXCONPAIR 溢出、参考 provenance、不回归、计数一致 |
| GPU | **未开 GPU 窗口**（CPU-only） |

单次 hfield 调用的产品入口为 `driver::hfield_pair_contacts`（`pub`，但 `physics::collision` 模块私有）。
为拿到**产品真实执行结果**而不改产品树，本复核在 `target/qa-g4/worktree`（git 忽略）建 **1f882361 的只读副本**，
仅在该副本 `hfield/mod.rs` 追加 `#[cfg(test)] mod qa_probe;` 并放入 QA 探针，跑完立即
`git worktree remove --force`。**产品工作树与产品文件全程未变**（见 §9 的 `git status` 证据）。

## 3. 独立方法（不复用交付脚本/断言）

1. `qa_capture_upstream.py`：QA 自写捕获脚本（不复用 `scripts/authoring/g10-p5/capture_hfield_f32.py`）。
   3 个交付 case 用**同配方输入**（仓库 XML + 参考 qpos，输入数据不可自造）重采；另造 **30 个合成场景**
   （见 §4.4）。每场景两次独立捕获（各自新建 `Data`），逐位比较后落盘 `qa-upstream-cases.json`。
2. `probe/qa_probe.rs`：产品探针（只在临时 worktree 编译）。逐 world 现场构造 `HfieldInput`
   （`geom2` 变换到 hfield 帧、`margin/gap` 取两侧之和，与上游 `ccd_hfield_kernel` 口径一致），
   调产品 `hfield_pair_contacts` 与 `hfield_filter`，导出接触 / bounds / `prism_count` / `overflow`。
3. `harness/src/main.rs`：**自写 oracle**。按 pinned 上游源码（`_hfield_filter`、`ccd_hfield_kernel`）独立重写
   box-sphere 粗筛、支撑紧 AABB、box-box、子网格 `cmin/cmax/rmin/rmax`、prism 六顶点链式构造与高度早退、
   四接触选择、`1e-3` 早退、50 上限；支撑函数按六类 geom 独立实现（MESH 走顶点穷举分支）。
   唯一的共享件是产品公开 `ccd()`（G2 已独立复核），故差异只会落在 G4 自己的逻辑上。
4. `qa_compare.py`：三方逐位比较（`worldid` 分组 + 位型多重集，不做容差近似；另给 ULP/绝对差读数），
   并核对 filter bounds、`prism_count`、overflow、重复点重数。
5. `qa_run_all.py`：一条命令复跑 1–4 并落日志（`qa-*.log`）。

## 4. 结果

### 4.1 交付参考 provenance（独立重采对照）

| case | QA count / 交付 count | dist 位等 | pos 位等 | frame 位等 | QA 两捕获 | 交付两捕获 |
| --- | --- | --- | --- | --- | --- | --- |
| `hfield-box` | 8/8 | 8/8 | 8/8 | 72/72 | 是 | 是 |
| `hfield-sphere` | 8/8 | 8/8 | 8/8 | 72/72 | 是 | 是 |
| `hfield-capsule` | 7/7 | 7/7 | 7/7 | 63/63 | 是 | 是 |

另核：三份 fixture 的 `provenance.device=cpu`、`mujoco 3.15.0 / warp 1.15.0 / mujoco_warp 3.15.0 /
numpy 2.5.4 / python 3.12.13`、`upstream_revision=71da24d9…`、`authoring_script_sha256` 与当前脚本一致、
`case.qpos_bits` 与 QA 重采输入逐位相同。

### 4.2 三方比对总览（33 场景 / 37 world / 112 接触）

| 比较 | 结果 |
| --- | --- |
| 产品 vs QA oracle（接触逐位） | **37/37 world 全等** |
| 产品 vs QA oracle（filter bounds 位等） | **37/37** |
| 产品 vs QA oracle（`prism_count`） | **37/37** |
| 产品 vs QA oracle（overflow） | **37/37** |
| 产品 vs 上游 f32（接触逐位） | 30/33 场景全等；3 例为窄相级小差（见 §4.3） |
| QA oracle vs 上游 f32（接触逐位） | 同上 30/33（同样 3 例，证明 oracle 与产品同源差异） |

### 4.3 非逐位的 3 例（全部为 G2 窄相级，非 G4 逻辑）

| 场景 | count | dist | pos | frame |
| --- | --- | --- | --- | --- |
| `syn-rot-box`（对侧 box 旋转 20/0/30） | 4/4 | max 1 ULP（3/4 位等） | 0 ULP（4/4 位等） | 36/36 位等 |
| `syn-class-cylinder` | 4/4 | 0 ULP（4/4 位等） | 2 ULP@尺度（3/4 位等） | max 1.86e-7 |
| `syn-class-ellipsoid` | 2/2 | 4 ULP（1/2 位等） | max 4.5e-9 绝对 / 318 ULP@该坐标尺度 | max 9.9e-7 |

判断依据：两例**产品与 QA oracle 逐位相同**，而 oracle 与产品共用同一 `ccd()`；差异只能来自窄相
f32 实现与 warp f32 的舍入（G2 已复核件），不是 G4 的 filter/prism/选择逻辑。
`ellipsoid`/`cylinder` 不在 G4 交付声明的三个样本内（交付 unmet 已注明），故列为**建议**补采样，不作阻断。

### 4.4 逐条 acceptance 核对

| # | 验收条目 | 结论与证据 |
| --- | --- | --- |
| 1 | `_hfield_filter` 候选集（数量+索引）+ 边界截断 | **通过**。产品 filter bounds 与独立 oracle **逐位相同 37/37**；中心/单 cell（4 candidate cells）/多 cell（8）/`+x`、`-x` 边界 clamp（`cmax=ncol-1`、`cmin=0`）均覆盖；边界场景接触与上游逐位；spike/plateau 起伏场景 `prism_count` 与 oracle 相等。限制：上游 warp 核不导出 `cmin/cmax`，故索引集用「bounds 位等 + 独立公式 + `prism_count`/接触端到端」三层间接核对（未为取证改上游核显）。 |
| 2 | prism 六顶点顺序/坐标、高度早退、六类全走 CONVEX | **通过**。oracle 按上游逐句重写（底三角 `z=-size[3]`、顶三角 `hfield_data[adr+(r+dr)*ncol+c]*size[2]+margin` 链式前移、`prism[3..5].z < zmin` 早退）；六类配对（box/sphere/capsule/ellipsoid/cylinder/mesh）逐 world 与上游/产品对照；源码审计：`driver.rs` 只有一个 `ccd()` 调用点（`geom_type::HFIELD` × `input.geomtype2`），**无任何按类型分支/解析捷径**；`geomtype2` 仅用于 filter 的 `support()`（与上游一致）。 |
| 3 | 接触装配 `pos=0.5*(w1+w2)`、`frame=make_frame(w1-w2)` | **通过**。代码逐行对齐（driver.rs L191–195 / 225–231 / 77–81）；产品与 oracle/上游逐位一致的接触已覆盖该语义。 |
| 4 | 四接触 + `1e-3` 去重、重复点不得静默丢弃 | **通过**。`worldid` 分组后按 `(dist,pos,frame)` 位型多重集比较；`geomcollisionid` 已核为每对序号 0..n-1（常量映射，故多重集等价）。重复行实测存在且被写出：`deliv-hfield-capsule` world0 四个写出接触为同一点，产品与上游逐位一致；`deliv-hfield-sphere` 8 行 6 个不同点（2 重复）；`syn-dedup-{small,mid,wide,capsule}` 覆盖 2/3/4 接触与去重边界。 |
| 5 | `MJ_MAXCONPAIR=50` 溢出（自造超容量场景） | **通过**。`syn-overflow-11`/`syn-overflow-13`：`prism_count=50`（恰好上限）、`overflow=true`、候选 51 次尝试后停止、上游 overflow 位=32；4 接触与上游逐位；代码审计写前判 `count >= MJ_MAXCONPAIR` 后 break，固定 50 容量数组不越界。 |
| 6 | 参考 provenance + QA 独立重采 | **通过**（§4.1）。 |
| 7 | 不回归（G2 5 case / G3 sdf-box 80/80 / 既有窄相） | **通过**（§6）。 |
| 8 | 证据计数与实测一致 / forbid 词 / 门禁 | **部分通过**：unit=572/0/1 与交付一致；`capture-cpu.log` 的 count 与 fixture 一致；README 重复点数字与行数有不准确（见 §7-一般#2）。交付证据与本次证据均未命中 6 个 forbid 词。 |

### 4.5 专项场景（全部含在上表的 33 场景内）

- **filter 拒绝**：`outside-x` / `above-top` / `below-base` / `gap-skip`（正间隙跳过）→ 产品与上游均 0 接触、
  bounds=None、`prism_count=0`。
- **边界 cell**：`flat-center`、`flat-plus-x`（`cmax` clamp）、`flat-minus-x`（`cmin` clamp）、`bump/plateau/spike` 起伏。
- **切向/斜接触**：`side-sphere`（约 45° 法向）、`side-box`（旋转 box 边缘接近）。
- **旋转 hfield**：hfield 40°（z）、倾斜 20°（y）；**旋转对侧 geom**：box 20/0/30、capsule 35/0/25。
- **六类对侧 geom**：box / sphere / capsule / ellipsoid / cylinder / mesh（4 顶点四面体）。
- **多 world**：`two-world`（2 world，6 接触，`worldid` 分组后逐位）。
- **深穿透**：`deep-sphere`（EPA 分支）。

## 5. 重复接触点 / 去重细节（实测）

| 场景 | 写出接触 | 位型不同点 | 重复行 | cached 数 | 说明 |
| --- | --- | --- | --- | --- | --- |
| `deliv-hfield-box` | 8（2 world） | 8 | 0 | 12 | 无重复 |
| `deliv-hfield-sphere` | 8 | **6** | 2 | 12 | 与 G0b「8 行 6 个不同点」一致 |
| `deliv-hfield-capsule` | 7 | **2** | 5 | 9 | world0 四个写出接触同一点；上游 f32 参考实际 2 个不同点（G0b 的「3」是 native 参考口径，见 §7-一般#2） |

「不得静默丢弃」的判据取**与上游逐位相同**：上游写出的重复行产品也写出、上游没写的产品也不写；
`1e-3` 早退只在「最远点距离 < 1e-3」时结束后续选择，与上游逐句一致。

## 6. 不回归（在 `1f882361` 的 worktree 实跑）

```
$ cargo test --locked        # worktree @ 1f882361，104 个 harness
合计：572 passed / 0 failed / 1 ignored
physics::collision::gjk::case_evidence::upstream_f32_within_4ulp ... ok      # G2 判据
physics::collision::sdf::tests::sdf_box_host_matches_upstream_f32_within_4ulp ... ok  # G3 sdf-box
physics::collision::hfield::tests::hfield_host_matches_upstream_f32_within_4ulp ... ok
physics::collision::hfield::tests::filter_rejects_far_geoms_and_accepts_overlap ... ok
physics::collision::hfield::tests::maxconpair_overflow_sets_flag_and_caps_scan ... ok
```

与交付 `unit.json`（572/0/1，`1f882361`）一致；`1f882361` 之后到当前 HEAD 的产品改动是 G5 的
`src/physics/collision/cuda/**` 与 `collision.rs`，不涉及本复核的 hfield 宿主文件。

## 7. 问题清单（按严重级别）

### 一般 #1：交付测试 helper `driver::relative_matrix` 返回的是转置

- **复现**：探针 `helper_rotation_check`：取 `a=Rz(0.7)`、`b=Rx(0.4)`，比较
  `relative_matrix(a,b)` 与上游 `geom2.rot = aᵀb`（行优先）。结果：`bit_equal=false`，
  `delivered_equals_transpose_of_expected=true`，`max_abs_diff≈1.2376`。
- **期望**：`out[i][j] = Σ_k a[k][i]·b[k][j]`（行优先）——上游 `mat1T @ mat2`。
- **实际**：实现为 `out[column*3+row] = …`，按行优先解释即 `(aᵀb)ᵀ`。
- **影响**：**不影响 G4 交付判据**——三个交付 case 的 `geom_xmat` 全为单位矩阵（已核 fixture 8/8/8/8/8/8/8），
  转置不可见；但 helper 只对单位旋转正确，任何「对侧 geom 带旋转」的后续用例（G5/G6 或新增 evidence）
  会被喂错旋转。**产品 driver 本身无此问题**：本复核用自己的正确 `aᵀb` 构造 `geom2` 后，
  `syn-rot-box`/`syn-rot-capsule`/`syn-class-mesh` 等旋转场景 product==oracle==上游（rot-box 仅 dist 1 ULP）。
- **建议**：修正片把该 helper 改为行优先 `aᵀb`（或改注释并显式转置使用），并在 hfield evidence 补一条
  「对侧 geom 旋转」case。注意 `sdf/descent.rs::relative_matrix` 与 `cuda/common.cu` 有同名同实现，
  本片不判其用途，建议 Arch 让 G3/G5 复核方各自确认调用点约定。

### 一般 #2：交付证据文字口径小偏差（不影响判据）

- README §1「hfield-capsule 7 行 3 个不同点」：本片**上游 f32 参考实测为 2 个不同点**
  （`(dist,pos,frame)` 位型；world0 4 行同点、world1 3 行同点）。「3」来自 G0b 的 native 参考口径，
  引用到本片 f32 参考不准确；sphere 的「8 行 6 个不同点」实测一致。
- README §2「`driver.rs` 约 330 行、`tests.rs` 约 340 行」：实测 **297 / 373 行**（仍满足 ≤500 行）。
- receipt `scope.files` 列了 `tests/g10_p5_hfield_host.rs`，该文件实际未落盘（数值判据在 lib 内
  `hfield/tests.rs`，与 README §4「公开 API 未暴露」一致）。
- 影响：仅文档一致性；判据、计数（572）、路径范围均与实测一致。建议随修正片一并对齐。

### 建议 #1：交付样本外的 `ellipsoid`/`cylinder` 配对建议补采样

`hfield×ellipsoid` 合成场景 dist 4 ULP（判据边缘）、pos 绝对差 4.5e-9、frame 9.9e-7；
`hfield×cylinder` pos 2 ULP。两例产品与 QA oracle 逐位一致 → 属 G2 窄相级差，不是 G4 逻辑；
交付未在声明样本内（unmet 已列）。建议 G5/G6 把这两类对侧 geom 补进 hfield 采样。

### 建议 #2：G5 实现世界→hfield 变换时不要复制交付测试 helper

G5 GPU 核会自己做 `geom2` 的 hfield 帧变换；应直接用上游 `mat1T @ mat2`（行优先）语义，
避免继承 §7-一般#1 的转置；`hf_mat`/`hf_pos` 回世界系的语义（`hf_mat@local+hf_pos`）本片已通过
旋转 hfield 场景与上游逐位验证。

## 8. 环境

| 项 | 值 |
| --- | --- |
| OS / host | Windows（DARKSEA） |
| 工具链 | `cargo 1.99.0 (5f94df478 2026-08-27)`、`rustc 1.99.0 (b940084d7 2026-09-28)`（默认 gnu 目标；门禁/单测口径） |
| 上游重采 | pinned venv：Python 3.12.13 / mujoco 3.15.0 / warp 1.15.0 / mujoco_warp 3.15.0 / numpy 2.5.4；**device=cpu** |
| 上游基线 | `71da24d956378a87a703b6e1442b13aec0c4ac29`（与交付声明一致） |
| GPU | **未使用**（未开 GPU 窗口，无显存占用） |

## 9. 复跑

```
# 1) 产品侧探针 + 整链（会在 target/qa-g4/worktree 建 1f882361 只读副本，跑完自动删除）
target/mjwarp-parallel/venv/Scripts/python.exe \
  docs/development-evidence/g10-p5-hfield-host-independent-check/qa_run_all.py

# 2) oracle 需先构建（独立 crate，自带 [workspace]，不碰产品 Cargo 文件）
cd docs/development-evidence/g10-p5-hfield-host-independent-check/harness
CARGO_TARGET_DIR=D:/Rust/mjwarp-rs/target/qa-g4/oracle-target cargo build --offline

# 3) 不回归（在 1f882361 的临时 worktree）
git worktree add --detach target/qa-g4/unit-wt 1f882361
cd target/qa-g4/unit-wt && cargo test --locked
```

证据文件：`qa-upstream-cases.json`（33 场景上游重采，含模型数组/f32 运动学/接触与两次捕获标记）、
`qa-product-output.json`（产品探针导出，含 filter bounds/prism/overflow/helper 旋转核对）、
`qa-oracle-output.json`（独立 oracle，含候选 cell 数/高度早退/缓存重数）、`qa-compare-report.json`
（三方比对 + summary 表）、`qa-*.log`（各步原始输出）、`qa-unit-regression.log`（572/0/1）。

## 10. 限制 / 未覆盖

- 未实装/未验证 **G5 GPU 内核**、`nacon` 截断与原子计数（本片 host-only）。
- 上游 warp 核不导出子网格 `cmin/cmax` 索引，候选 cell 集为间接三层核对（§4.4 #1）。
- 合成场景最大 hfield 为 13×13；未覆盖真实大尺寸地形网格、非均匀高程文件、flex/TRIANGLE。
- mesh 对侧仅 4 顶点四面体（走上游/产品顶点穷举支持分支；顶点数 ≥10 的 graph hillclimb 未覆盖）。
- 未验证 Linux/T4；本报告全部数字仅对本机 Windows + warp CPU 后端成立。
- 原始接触顺序只作观察项（未作判据）；比较统一按 `worldid` 分组 + 全字段位型多重集。
