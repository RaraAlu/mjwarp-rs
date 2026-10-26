# G10@P5 M2（convex-vs-mesh multicontact）QA 独立复核

- 任务：`2026-10-26-mesh-multicontact-m2-independent-check`（owner QA；CPU-only，未开窗）。
- 被复核片：`2026-10-26-mesh-multicontact-m2-convex-vs-mesh`（verified）。
- 提交链：`85cce21b`（`#[doc(hidden)] validate_collision_inputs` 单独提交，父文件 scope 补救）、
  `16214a22`（M2 主体：device 实现 + tests + 证据 + 门禁 runs）。
- 复核时主树 HEAD：`1fb2a695`（M3 启动的治理提交）；`git diff 16214a22..HEAD -- src tests fixtures
  Cargo.toml Cargo.lock` 为空 → 代码树 = M2 提交树。
- 结论：**pass（阻断 0 / 观察 8）**；未 commit、未 done。device 数字未由我重跑（未开窗），
  按纪律只核窗口日志与冻结实体（见 §4/§8）。

## 0. 结论

1. **移植正确**：pinned `collision_gjk.py`（rev `71da24d9`）的 `_intersect1`/`_intersect2`
   （L1591-1616）、`_mesh_normals`（L1620-1686）、`_mesh_edge_normals`（L1690-1734）、
   `_mesh_face`（L1926-1948）与 `multicontact_mesh.cu`（191 行）逐式对照：循环边界、最多 2 个值、
   `mat @ polynormal` 的行主序约定（`mul_mat3_vec(m, v)=m0x+m1y+m2z`，box/cylinder 路径同 helper、
   已被 G5 位级验证）、`normalize`、边法线前驱顶点 `k=(j==0?nvert-1:j-1)`、面顶点**逆序**写出
   全部一致；三处固定容量 clamp（degree≤10 / face≤16）是**新增的回退保护**（超界返回 0 → 单接触），
   不改变 pinned 公式（详见 `qa-port-audit.json`）。任务书写的 `collision_convex.py` 确实是笔误，
   函数在 `collision_gjk.py`（行号与任务书一致）。
2. **既有 13 组合不回退**：`multicontact_geom.cu`/`product.cu` 无任何**代码行删除**；
   `multicontact.cu` 唯一删除的代码行是旧 mesh 早退 `return out;`（mesh 分支重写，符合本片目的）；
   box/cylinder 各分支只作为上下文出现、逐字未动；新增 `nface==0` 防御对 box/cylinder 恒不触发
   （box face 恒 4、cylinder face 恒 16、edge 恢复恒 ≥1）。
3. **窗口口径正确**：attempt4 =「数值全绿 + guard 因 free floor 环境边界 abort，经 Chief 裁定接受为
   有效轮」；证据中**没有** window-pass 式结论（我逐文件扫过，见 qa-window-attempts.json 的 wording_check）。attempt1（runner PATH 缺 CUDA DLL、
   8 个加载期 panic、无 device 工作）/ attempt2（8 并行 free_min 8587）/ attempt3（3/3/2 分批
   free_min 9824）作废原因与原始 guard.log 一致。
4. **冻结与范围**：8 个窗口二进制 sha256/大小/字节数与证据逐项相同，mtime **00:01:20–00:02:41**
   早于窗口开始 00:10:14；attempt4 全部日志无 `Compiling`/`rustc`（0 命中）；`DEVICE_SOURCES` 27→28
   （`multicontact_mesh.cu` 紧随 `multicontact_geom.cu`、`clip` 之前，拼装顺序断言同步）、入口仍 6、
   `src/physics/mod.rs` 零 diff、旧 G5 bins/参考零 diff、既有判据 support 零 diff。
5. **常驻真实校验（Chief 追加点）**：`85cce21b` 只加一个 3 行转调函数（`validate_collision_inputs`
   → `abi::validate_inputs`），`run()` 仍直接调 `validate_inputs`（无新发射路径、无 env/feature 开关，
   不是间接假验证）；`real_cpu_validation_accepts_m0b_and_old_cases` 是 `#[test]` 且无 `#[ignore]`，
   用 M0b **真实 bins**（`load_case` 读 32 个 bin，不是合成数组）与旧 G5 **全部 10 个 case**（空
   sidecar）调真实 `validate_collision_inputs`；默认 `cargo test --locked` 中我实测该测试
   `... ok`，另直接运行**已构建**测试二进制（sha `8fa10311…`，mtime 00:02:16，未重建）：
   单测 1 passed/4 filtered、全量 5 passed/0 failed/0 ignored。
6. **门禁**：我在主树重跑 fmt/clippy/clippy-cuda exit 0、unit **987/0/373**（117 binaries）、
   all-features **1036/0/431**（117 binaries）、NVRTC 冒烟指纹
   `254485 / 00810ddb… / 3883740 / entries 6`、`runtime::collision_gpu`（含 ignored）**33 passed**；
   `normify gate/check` exit 0（见 `.normify/runs/2026-10-26-mesh-multicontact-m2-independent-check/`）。

## 1. 复核对象

| 项 | 值 |
| --- | --- |
| M2 主体提交 | `16214a22`（32 文件：device 6 + runtime 5 + tests 3 + 证据 15 + runs/任务） |
| 测试入口提交 | `85cce21b`（`src/runtime/collision_gpu.rs` 只 +10 行：doc 注释 + 3 行函数） |
| 复核时 HEAD | `1fb2a695`；`16214a22..HEAD` 对 src/tests/fixtures/Cargo.* 零 diff |
| 复核环境 | Windows / `rustc 1.99.0` / 未开 GPU 窗口、未建 CUDA context |

## 2. 移植审计（逐条对 pinned 源码）

设备实现 `src/physics/collision/cuda/multicontact_mesh.cu`（新增 191 行）逐函数核对，详见
`qa-port-audit.json`。要点：

| pinned | 设备 | 我核的点 |
| --- | --- | --- |
| `_intersect1` L1591-1602 | `g5_mc_intersect1` | 双层 `start..start+len` 循环、命中写 `res[count]`、`count==2` 提前返回 2、返回 count |
| `_intersect2` L1605-1616 | `g5_mc_intersect2` | 参数顺序 `(a1[], a2[], start2, len1, len2)` 与 caller 传 `n` 一致 |
| `_mesh_normals` L1620-1686 | `g5_mc_mesh_normals` | dim3 三段 `polymapadr[vertadr+v]`、两次交集、`faceset[0]` 唯一面、`mat @ polynormal[polyadr+idx]`；dim2 逐面；dim1 逐 vertex 相邻 polygon；`return 0` 分支一致 |
| `_mesh_edge_normals` L1690-1734 | `g5_mc_mesh_edge_normals` | dim2 单边 `normalize(v2-v1)`；dim1 `polyvertadr/polyvertnum[polyadr+idx]`、找 `v1i`、前驱顶点 `k=(j==0?nvert-1:j-1)`、`mat@vert[vertadr+polyvert[adr+k]]+pos`、`normalize(end-v1)` |
| `_mesh_face` L1926-1948 | `g5_mc_mesh_face` | `polyvertadr[polyadr+idx]`、`nvert-1..0` **逆序**、`mat@vert[vertadr+polyvert[adr+i]]+pos`、返回 nvert |
| 数据视图 L2245-2262 | `g5_mc_mesh_view_of` + `build_geom` | 9 指针 + `vert`；`vertadr`/`mesh_polyadr` 由 `dataid` 解析；单 mesh 组合选 geom1 优先（与 pinned `if geom1==MESH … elif geom2==MESH` 一致，含 mesh-mesh 时也取 geom1 的边界，见 O4） |
| 分派 L2278-2310 / L2326-2379 / L2393-2436 | `multicontact.cu` | 三个分支位置、参数顺序与 pinned 一一对应 |

`mat @ v` 约定：`common.cu::mul_mat3_vec(m,v) = (m0x+m1y+m2z, m3x+m4y+m5z, m6x+m7y+m8z)`（行主序），
与 MuJoCo xmat 行主序一致，也是既有 box/cylinder 路径使用的同一 helper（G5 位级回归已验证该约定）。
`v3_normalize` = `a/|a|`（零向量返回 0）与 `wp.normalize` 同形。

**容量 clamp（新增，不影响已覆盖 case）**：degree>10 / polygon 顶点>16 时助手返回 0 →
`multicontact.cu` 在 clip 前 `if (nface1==0 || nface2==0) return out;` 回退单接触；pinned 是动态上界
（`max(m.npolygonmax,16)`、`nmeshdegmax≥3`）。本片 cube fixture（degree=3、polygon=4）不触发，
证据 §5.1 已声明为未覆盖边界；我核对该声明与代码一致。

## 3. 既有路径不回退（代码级）

- `multicontact_geom.cu`、`product.cu`、`hfield.cu`、`geom.cu` 的改动**全部是新增行**：
  `git diff ac01d86d 16214a22 -- <file>` 中非注释删除行 = 0（我逐文件过滤过）。
- `multicontact.cu` 唯一非注释删除行 = 旧的 `if (geomtype1 == G5_MESH || geomtype2 == G5_MESH) {
  return out; }` 里的 `return out;`（mesh 分支重写）；box/cylinder 的 normals / edge-normals /
  face 分派在 diff 中只作为未改动上下文出现。
- `nface==0` 防御：`g5_mc_box_face` 恒返回 4、`g5_mc_cylinder_face` 恒返回 16、`g5_mc_set_edge`
  返回 2，`nface` 在恢复前为 `_feature_dim` 的 1..3 → box/cylinder 组合永不触发该早退。
- sidecar 缺失（`mesh_poly_args==nullptr` 或 `dataid<0` 或 `mesh_polyadr<0`）：9 指针清空 →
  mesh 参与时直接回退单接触，与 M0a 的 null sidecar 语义一致（`product.cu::build_geom` +
  `g5_mc_mesh_view_of.has_poly`）。

## 4. 窗口证据口径（attempt4）

- 措辞核对：README/`window-attempts-summary.txt`/receipt 统一写
  「attempt4 = 数值全绿 + guard 因 free floor 环境边界 abort，经 Chief 裁定接受为有效轮」，
  **不写 window-pass 式结论**（我全目录扫过，0 命中，见 qa-window-attempts.json）。
- 6 条补偿事实我逐条对原始文件核过：① 开窗基线 free **11200 ≥ 10240**（attempt4-baseline.txt）；
  ② post `query-compute-apps` **31 条、0 条本任务/test exe**、residual 0、process_exited=true；
  ③ delta（guard 基线口径）**1499 MiB**；④ util_max **48%**、util_streak **0**；⑤ 数值全 bit-equal/
  判据内（§5）；⑥ 8/8 exit 0（attempt4-facts.txt）。
- guard 只触发一条规则：`guard_stop reason=agg_free_below_floor free_min=9696 floor=10240`
  （attempt4/guard.log）；`foreign_gpu_context` / `agg_increment_over_limit` / `agg_sustained_util`
  均未出现，delta_peak 1499 < limit 3072。
- 我按 `target/m2/attempt4/samples.csv`（658 样本）独立重算：used_peak **6366**、free_min **9696**、
  util_max **48**、util≥80 连续 streak **0**、首个/末个时间戳 00:10:14.124 / 00:13:16.147
  —— 与 guard 汇总一致（`logs/qa-window-recompute.log`）。
- attempt1：8 个 `.err.log` 均为 `Compile("NVRTC库缺失（nvrtc 不可加载）")` 加载期 panic，无 device
  工作；attempt2/3：guard.log 分别 `free_min=8587` / `9824` 触发 free floor，8 个日志均
  `test result: ok`（README 已如实注明 runner 退出码采集有缺陷，attempt4 起改为直接调用）。

## 5. 数值证据（device 数字只来自窗口日志，未重跑）

我解析了 attempt4 的 8 个日志（`logs/` 有副本），与参考/判据交叉核对：

- `mesh-box` / `mesh-cylinder`：`nacon=4 ref_count=4`、`bit_equal=4/4`、`max_dist/pos/frame_ulp=0`、
  `contact.geom=[1,0]`、`failures=0`。我独立核了 M1 参考：sha `c83baffe…`（文件实测一致）、
  `count=4`、`geom=[[1,0]]×4`、`worldid=0`、`geomcollisionid=0..3`、`two_captures_bitwise_identical=true`、
  **无 `efc_address` 字段**（与比较器「设备全 -1」不变量口径一致）。
- G5 9 个设备 case：multicontact（cylinder-box 314/314、box-box-margin 391/392 max_pos_ulp=1、
  mesh-mesh 80/80）、launcher（hfield-sphere 314/314、sdf-box 3122/3122、ellipsoid-box 80/80）、
  primitive（sphere-box-shallow/-deep/capsule-box 各 80/80）—— `failures=0` 全命中日志。
  `mesh-mesh` 的内核是 **`kind=Contact`（单接触）**，不是 ContactMulti（见 O3）。
- B1 6 / B2 4 / B3 6 分支 + 4 swap / primitive 3 / launcher 3：日志 `failures=0`（B3 的 4 条
  swap 行也在日志中）。
- `hfield-capsule`：**未覆盖**（窗口 9 个设备 case 不含它）；但证据里「仅 CPU
  `g10_p5_ccd_overflow` 口径」的说法不成立（见 O1）。
- stage2 9 synth（非 G5 bins）：18 dump 完成，窗口日志每行 `两次一致=true`；窗外 CPU 比较按
  「记录 C 只记录」口径，`stage2-cpu-compare.log` 有 7 个 case/kernel 组合的 FAIL 行（6 个 case），
  属跨 recipe 已知差、非 M2 回归——但「逐位一致」措辞不精确（见 O2）。
- UNC-6 深穿透：mesh 两 case 全字段 ULP=0 → 未触发（仅记为未覆盖类未被命中）。

## 6. 输入与参考（我独立复算的部分）

- **M0b 32 bins 是真实 pinned mjModel 数据**：我用 pinned mujoco 3.15 重新 dump 两 fixture 的
  9 个 mesh-poly 数组 + `mesh_vert/mesh_vertadr/mesh_vertnum/mesh_graph/mesh_graphadr`，
  与 M0b bins **逐值相等（15 组 × 2 case，0 mismatch）**；`geom_type=[7,6]`（mesh-box）/
  `[7,5]`（mesh-cylinder）、`geom_dataid=[0,-1]`（`logs/qa-bins-vs-model.log`、
  `scripts/verify_m0b_bins.py`）。
- **判据零改动**：M2 提交只新增 `tests/support/mesh_multicontact_compare_support.rs`，
  既有 support（`primitive_compare*`、`multicontact_compare_support`、`launcher_compare_support`、
  `stage2_equivalence_compare`）零 diff；`FOUR_ULP_AT_ONE=4*2^-23`、dist 4 ULP@量级
  （ref=0→4 ULP@1.0）、pos 4·ULP(scale)、frame 4 ULP@1.0、零容差字段集
  （friction/solref/solreffriction/solimp/includemargin/adhesion/worldid/type/dim/geom/
  geomcollisionid）、overflow 全 0、efc 全 -1 与既有片同口径，未见放宽。

## 7. 冻结与范围

- 8 个二进制：sha256/字节数与 `attempt4-binary-shas.txt`、README §7、receipt 三处一致
  （`qa-frozen-binaries.json`）；mtime 00:01:20–00:02:41 < 窗口开始 00:10:14；窗口日志 0 条
  `Compiling`/`rustc`。
- `DEVICE_SOURCES` 27→28：`multicontact_mesh.cu` 位于 `multicontact_geom.cu` 之后、
  `multicontact_clip.cu` 之前；`multicontact_tests.rs` 同步断言 28 段 + 段位置 + 三个 mesh 符号；
  `primitive_tests.rs` 同步 28；入口数仍 6（`product.cu` `extern "C"` 计数 6）。
- `src/physics/mod.rs` 零 diff（device source 用 `source.rs` 本地 `include_str!`，保住 T4 身份）。
- 派生口径：open 门 `free≥10240` 不变；M3 起 guard floor=`min(10240, baseline_free−3072)`，
  M2 数据不追溯——与证据一致（口径说明，不影响本片数据）。

## 8. 常驻真实校验（Chief 追加点，逐条）

1. **`85cce21b` 定性**：diff 只新增 `#[doc(hidden)] pub fn validate_collision_inputs`（3 行代码
   + doc 注释），函数体仅 `validate_inputs(inputs)`；`CollisionGpuProgram::run` 在 L129 仍**直接**
   调用 `abi::validate_inputs`，没有走新函数、没有新 env/feature 分支、没有改变分配/发射顺序 →
   仅测试入口、行为零变更。（`pub` 会让它成为 doc(hidden) 公共 API 面，见 O7。）
2. **常驻测试真的在跑**：
   - 源码：`tests/g10_p5_mesh_poly_case_assembly.rs:390` `#[test]`，**无 `#[ignore]`**；
     `load_case(m0b_cases_root().join(case))` 读真实 32 bins → `mesh_poly_present()` 断言 9 项 present
     → `validate_collision_inputs(&inputs.inputs())` == `Ok(())`；随后遍历 `g5_cases_root()` 下
     **全部 10 个**旧 case 目录，断言空 sidecar + `Ok(())`。
   - 默认套件：我重跑 `cargo test --locked`，输出中
     `test real_cpu_validation_accepts_m0b_and_old_cases ... ok`（unit **987/0/373**）。
   - 直接跑已构建二进制（未重建）：`g10_p5_mesh_poly_case_assembly-242b14a010d48be2.exe`
     （sha `8fa10311…`，mtime 00:02:16）→ 单测 `1 passed; 4 filtered out`；全量
     `5 passed; 0 failed; 0 ignored`（与 README §6 一致）。
   - 不是镜像测试：它调用的是 `run()` 同一个 `abi::validate_inputs`（非重写规则副本）。

## 9. 门禁数字（我在主树复跑）

| 项 | 命令 | 我的结果 |
| --- | --- | --- |
| fmt | `cargo fmt --check` | exit 0 |
| clippy | `cargo clippy --all-targets --locked -- -D warnings` | exit 0 |
| clippy-cuda | `cargo clippy --all-targets --locked --features cuda -- -D warnings` | exit 0 |
| unit | `cargo test --locked` | **987 passed / 0 failed / 373 ignored**（117 binaries，exit 0） |
| all-features | `cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features`（msvc + `MJWARP_MUJOCO_ROOT`） | **1036 / 0 / 431**（117 binaries，exit 0） |
| NVRTC 冒烟 | `cargo test --locked --features cuda-probe --lib runtime::collision_gpu::nvrtc_smoke -- --include-ignored` | `254485 / 00810ddb… / 3883740 / entries 6`，exit 0 |
| collision_gpu 子集 | `cargo test --locked --features cuda-probe --lib runtime::collision_gpu -- --include-ignored` | **33 passed / 0 failed**，exit 0 |

与交付声明（987/0/373、1036/0/431、33 passed、冒烟指纹）逐项一致；`normify gate/check` exit 0
（run 目录记录在 `.normify/runs/2026-10-26-mesh-multicontact-m2-independent-check/`）。

## 10. 观察（按级别，均不阻断）

- **O1 一般**：`hfield-capsule` 的覆盖补偿表述不成立。README §7.2/§7.3/§11 与 receipt 写
  「仅 CPU `g10_p5_ccd_overflow` 口径」，但 `tests/support/ccd_overflow_support.rs` 的
  `FORCED_HORIZON_CASES` 只有 `cylinder-box / box-box-margin / mesh-mesh`，**不含 hfield-capsule**；
  实际覆盖 = 无接触回归，仅有旧 G5 bin 装配/输入校验（`old_g5_cases_keep_23_bin_compat`、
  常驻真实校验里的空 sidecar 加载）+ §7.3 的两条补偿（launcher hfield-sphere 位级回归、ABI 504
  hfield 偏移断言）。结论「未覆盖」本身是诚实的，但第三项补偿表述需更正，避免高估覆盖。
- **O2 一般**：stage2 的「每 case 两次运行逐位一致（18 dump）」措辞不精确。`deterministic()` 是
  **行序归一化后**比较；按原始 JSON 逐位比，`cylinder-box.Contact` 与 `ellipsoid-box.Contact`
  两个 dump 的 `run1 != run2`（`stage2-cpu-compare.log` 自身有「两次运行（run1/run2）不逐位一致」
  行）。建议写成「行序归一化后一致、2 个 Contact dump 行序不同」，或直接引用 stage2 的记录 C。
- **O3 建议**：`delta` 数字有两套基线：guard.log 的 `baseline_used=4867`→delta **1499**；开窗 3 点
  基线 `4862`→delta **1504**（M2 commit message 用的 1504）。README §8.4 给了 4867 的公式，
  但 §8.1 表格只写 1499、commit 写 1504，建议统一标注「相对哪条基线」，避免读者以为矛盾。
- **O4 建议**：`mesh-mesh` 在窗口日志是 `kind=Contact`（单接触），README §8.2 把它列在
  「G5 multicontact(3)」组里；建议注明 kernel=Contact、mesh-mesh 多接触属 M3。
- **O5 建议**：guard 的 `own_patterns=mjwarp_rs-*` 是通配（guard.log 自身提示
  `prefer explicit -OwnExeNames`），且 post `query-compute-apps` 有 4 条
  `Insufficient Permissions` 无法排除身份；本窗口串行执行、8/8 exit 0、residual 0，
  「无本任务进程」结论不受影响，但证据里宜写明通配副作用（会把其它任务的 test exe 也当 own）。
- **O6 建议**：receipt 的 `files[].change` 对 `geom.cu` 写「d_support_point restored」，
  实际 diff 只有新增 9 指针/`dataid`/`mesh_polyadr`/`g5_mesh_clear_poly`/`g5_mesh_vertex`，
  没有「恢复」动作；建议改成实际改动描述。
- **O7 建议**：`validate_collision_inputs` 是 `pub`（doc(hidden)），技术上成为公共 API 面；
  另外 M2 让 `polyvert/polymap` 的**元素值域**第一次成为 device 可达索引（M0a 校验仍只到长度/区间/
  求和，O6 边界沿用）。当前唯一生产者是 M0b（pinned 模型数据、我用 mjModel 逐值核过），风险低；
  建议 M3 补值域校验或在 M3 证据里显式声明「sidecar 只接受同源 mjModel 数组」。
- **O8 建议**：证据 15 个文件里 11 个是 UTF-8 **带 BOM**（8 个 attempt 日志 + stage2 两个日志 +
  binary-shas）；内容无损、行数全部 ≤500，但 Arch 要求「UTF-8 无 BOM」，建议统一转换。
- （另：双 mesh 组合在 `g5_multicontact` 中会按 pinned `if geom1==MESH … elif geom2==MESH` 语义
  复用 geom1 的 polygon 数组视图（上游同样如此）；M2 已声明 mesh-mesh 延后 M3，记为边界，
  不作为观察项重复计数。）

## 11. 边界 / 未覆盖

- 我**没有重跑 device**（未开窗、未建 CUDA context）：mesh 两 case 的 `bit_equal=4/4`、G5 9 个
  case 的逐项数全部来自窗口日志与冻结二进制，我核的是「日志与参考/判据/冻结实体自洽」，
  不是独立重算。判据实现本身我逐行核过（§6）。
- 未覆盖：一般 mesh（degree>10 / polygon>16）→ 单接触回退；`hfield-capsule` 无接触回归；
  mesh-mesh 多接触属 M3；UNC-6 类深穿透未被命中但仍记未覆盖。
- 「Chief 裁定 (b)」与「Arch 会签/补签 plane 64 位」在仓内是**叙述性证据**（无签名产物）；
  我核的是冻结清单自洽 + 8/8 实体 sha 与当前文件一致 + 该轮未跑（attempt1 无 device 工作）。
- 不宣称 mesh 已支持、不宣称跨环境/全类型。

## 12. 复现

```text
# 输入真实性与窗口聚合（CPU）
target/mjwarp-parallel/venv/Scripts/python.exe target/qa-m2/verify_m0b_bins.py
target/mjwarp-parallel/venv/Scripts/python.exe target/qa-m2/recompute_window.py

# 常驻真实校验（已构建二进制，不重建）
target/debug/deps/g10_p5_mesh_poly_case_assembly-242b14a010d48be2.exe --nocapture

# 门禁
cargo fmt --check; cargo clippy --all-targets --locked -- -D warnings
cargo clippy --all-targets --locked --features cuda -- -D warnings
cargo test --locked
```
