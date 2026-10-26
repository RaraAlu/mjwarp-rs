# G10@P5 M3（mesh-mesh multicontact）QA 独立复核

- 任务：`2026-10-26-mesh-multicontact-m3-independent-check`（owner QA；纯 CPU 复核，未开窗、未建 CUDA context）。
- 被复核提交：`f1e5cda1`（父 `11a13f15`）。复核期间 HEAD 前进到 `1f08d3b2`（治理/工具提交），
  `git diff f1e5cda1..1f08d3b2 -- src tests fixtures Cargo.toml Cargo.lock` **为空** → 代码树仍等于 M3 提交。
- 结论：**pass（阻断 0 / 观察 7：3 一般 + 4 建议）**；未 commit、未 done。
- device 数字未由我重跑（无窗口）：我核的是「日志/参考/冻结实体/原始 samples 自洽 + 代码/分派/校验的独立审读」，
  并独立复算了 bins/参考/capture 窗口/validation 窗口/拆分/门禁。

## 0. 结论

1. **「0 行设备改动」成立**：`git diff 11a13f15..f1e5cda1 -- src/physics/collision/cuda/` **0 行**；
   `src/physics/mod.rs` 零 diff；`DEVICE_SOURCES` **28→28**（source.rs 不在改动内）；入口 **6→6**；
   M3 的 src 改动只有 `abi.rs`（O7 值域校验）+ 两个测试模块文件。我另独立重跑 NVRTC 冒烟：
   **`254485 / 00810ddb… / 3883740 / entries 6`**（与 M2 完全相同）→ 设备源文本/PTX 口径未变。
2. **O7 值域校验正确且只有 CPU 测试入口**：`validate_mesh_poly` 末尾新增 4 组规则（逐 mesh
   `vertadr/vertnum` 长度/非负/区间、`polyvert` 元素 ∈ [0, vertnum[owner])、`polymap` 元素 ∈
   [0, polynum[owner])、sidecar 存在时 mesh geom 的 `dataid ∈ [0,nmesh)`）；我逐条对 pinned 设备语义
   核过（`polyvert` 是逐 mesh 局部索引、设备加 `vertadr` 后用；`polymap` 是逐 mesh 局部 polygon 索引、
   设备加 `polyadr` 后用；`build_geom` 用 `mesh_polyadr[dataid]`）→ 规则与设备消费方式一致。
   `run()` 仍直接调 `abi::validate_inputs`（collision_gpu.rs 不在改动内，wrapper 未变）。
   实测 `cargo test --lib runtime::collision_gpu::validate` → **8 passed / 0 failed**
   （父模块 4 + 子模块 4，测试名不变；新增 1 条值域负例含 6 类负例 + 真实 cube 正例）。
3. **bins/同源正确**：`cases/mesh-mesh/` 32 bin + meta；32/32 数组 sha/count/dtype 与
   `mesh-case-manifest.json` 一致；9 个 polygon slice 实测 **`[2,2,36,12,12,48,16,16,48]`**；
   manifest `result=PASS`、`failures=[]`、**18 条自检**（12 M0a + 4 adr==cumsum + 2 值域）全 PASS、
   `cross_check` 带 actual/reference 且全 true；9 数组数值与我从 pinned mjModel 现场 dump 的
   **逐值相等**；sum/区间/cumsum/值域/`dataid∈[0,nmesh)` 全部成立；pinned CPU 后端复算
   collision count **4/4**。同源逐位（我现场重算）：`xml_sha256`、`qpos_bits`、`geom_xpos`、
   `geom_xmat` 与 M3 参考**逐位一致**；meta 的 friction/solref/solreffriction/solimp/adhesion
   与参考首行**逐位一致**。
4. **两份参考**：M3 参考实测 `96e7895d…`（3676 B；count=4、`geom=[[0,1]]×4`、
   `geomcollisionid=[0,1,2,3]`、dist 全 −0.009999997913837433、`two_captures=true`、
   `captured_at=00:48:22.577`= 捕获窗开始、flags nativeccd off/multiccd on、`geom_dataid=[0,1]`）；
   旧 M1 参考实测仍 `c83baffe…`（5703 B），且 M3 提交对 M1 参考/G5 bins/旧 fixture **零 diff**。
   捕获窗原始 A/B/提交参考三者**逐字节相同**（同一 sha），RUN A/B 独立进程 + 独立 warp cache
   （日志可见两个 cache 目录），RUN A/B exit 0、compare exit 0。
5. **验证窗口径**：9 实体 exit **9/9 = 0**、failed_binaries=0（facts 与 9 个日志 `test result: ok` 对上）；
   `facts.txt` 的 `guard_aggregate=System.Collections.Specialized.OrderedDictionary` 确为**无效行**
   （我核过原文，且 `guard.log`+`samples.csv` 才是权威来源）。我按 `validation-attempt1-samples.csv`
   独立重算：**732 样本 / used_peak 2505 / free_min 13557 / util_max 60 / util≥80 streak 0 / residual 0**，
   guard baseline used **2227** → delta 278；第一基线点 2222 / 采样最小值 2222 → 283/283；
   floor=`min(10240, 13840−3072)` = **10240**（open 门 free 13840 ≥ 10240）。
   零 Rust 编译（日志 0 条 `Compiling|rustc`）；post compute-apps 按 `python|g10_p5|cargo|mjwarp`
   过滤 **0**；9 个冻结二进制 sha 与 receipt/`frozen-binaries.txt`/`validation-attempt1-binary-shas`
   **逐项一致**（`target/m3-freeze/` 副本我重新哈希 9/9 一致）。
6. **拆分**：测试名/数量不变（validate 8 = 4+4；assembly 7 = 5+2），3 条搬移测试函数体**逐字不差**
   （whitespace-normalised 比对）；拆分后门禁 unit **990/0/374**、fmt/clippy/clippy-cuda exit 0；
   `scope-selfcheck` 我用同一匹配规则对提交改动集复现 = **75 路径 / 越界 0**（与声明一致）；
   M3 任务 JSON 有 addenda 记录（commit hook 500 行拒 → 拆分后重跑门禁）。
   注意：`CubeMeshPoly` 数据块**不是**逐字节搬移（见 O1），测试函数体与 helper 是。
7. **门禁**：M3 提交时 runs unit **990/0/374**（我读 runs 核对）；我在当前树跑 `normify gate` 也是
   **990/0/374**（fmt/clippy/clippy-cuda/unit exit 0），`normify check` exit 0。

## 1. 复核对象

| 项 | 值 |
| --- | --- |
| M3 提交 | `f1e5cda18bb4d0841b465e6e7a290d2060f56339`（80 文件：证据 60+ / src 3 / tests 5 / fixture 1 / .normify 5） |
| 父提交 | `11a13f154776b8e7a3d28b60e9db5f9e950787e4` |
| 复核时 HEAD | `f1e5cda1`（无后续代码提交） |
| 复核环境 | Windows / rustc 1.99.0 / pinned venv python 3.12.13 + mujoco 3.15.0 + warp 1.15.0 + mujoco_warp 3.15.0（CPU device）；未开 GPU 窗口 |

## 2. 「0 行设备改动」（独立核）

- `git diff 11a13f15..f1e5cda1 -- src/physics/collision/cuda/` = **0 行**；`src/physics/mod.rs` = 0 行。
- `DEVICE_SOURCES` 声明 `[(&str,&str); 28]`；`source.rs` 不在 M3 改动文件里 → 28→28。
- `product.cu` 不在改动文件里，`extern "C" __global__ void` 计数 = **6** → 入口 6→6。
- M3 的 src 改动文件只有：`abi.rs`（+80 行 O7）、`validate_mesh_poly_tests.rs`（新）、`validate_tests.rs`（拆出）。
- **冒烟指纹独立重跑**：`254485 / 00810ddb5355564eed422e9d992a6843baf1f5e3541714230463df45a16ee253 /
  3883740 / entries 6`，与 M2 证据值完全相同 → 设备源内容未变（文本级）。
- M3 的「device bit_equal 4/4」是运行期证据（窗口日志，见 §5/§8），我只能核日志与冻结实体自洽，
  不能替代设备重跑。

## 3. O7 值域校验（逐条审计）

`abi.rs::validate_mesh_poly` 末尾新增（在 M0a 长度/区间/sum 规则之后）：

| # | 规则 | 我对设备语义的核 | 判定 |
| --- | --- | --- | --- |
| 1 | `mesh_vertadr.len()==mesh_vertnum.len()==nmesh`；逐 mesh 非负且 `vertadr+vertnum ≤ nmeshvert` | `nmeshvert = polymapadr.len()`；设备用 `vertadr` 定位 vertex 区间 | 正确 |
| 2 | 逐 polygon（owner mesh = `mesh_polyadr[m]..+mesh_polynum[m]`）的 `polyvert` 元素 ∈ [0, `mesh_vertnum[m]`) | `polyvert` 是逐 mesh 局部索引，设备 `_mesh_face/_mesh_normals` 加 `vertadr` 用 | 正确 |
| 3 | 逐 vertex（owner mesh = `mesh_vertadr[m]..+mesh_vertnum[m]`）的 `polymap` 元素 ∈ [0, `mesh_polynum[m]`) | `polymap` 是逐 mesh 局部 polygon 索引，设备加 `polyadr` 用 | 正确 |
| 4 | sidecar 存在时，所有 type==7 geom 的 `geom_dataid ∈ [0,nmesh)` | 设备 `build_geom` 用 `mesh_polyadr[dataid]`；`geom_type/geom_dataid` 长度已由既有校验锁 ngeom | 正确 |

- 索引安全前提：规则 2/3 的区间循环在规则 1 与 M0a 区间规则之后，`polyadr+polynum ≤ nmeshpoly`、
  `vertadr+vertnum ≤ nmeshvert`、逐 polygon/vertex 的 `adr+num ≤ len(polyvert/polymap)` 都已先行保证，
  故新增循环的下标访问在已校验范围内。
- 旧 23 项 case（9 数组全空）在函数开头的 null-sidecar 分支直接 `Ok`，不受新规则影响（我跑
  `real_cpu_validation_accepts_m0b_and_old_cases` 通过）。
- `run()` 路径：`collision_gpu.rs` 未改；`run()` 仍直接调 `abi::validate_inputs`；测试入口
  `validate_collision_inputs` 未变 → **无新开关、不是间接假验证**。
- 单测：`validate_mesh_poly_tests.rs` 新增 `rejects_mesh_poly_value_domain_violations`，含 6 条负例
  （vertadr 长度/区间/负值、polyvert 越界、polymap 越界、dataid 越界）+ 1 条真实 cube 正例；
  连同搬移的 3 条 = 4 条子模块测试；父模块 4 条 → **8 passed**（我实跑）。

## 4. bins / 同源（独立复算）

- 32/32 数组 sha/count/dtype/size 与 manifest 一致；`arrays_order` 32 项；case 目录 32 bin + meta.txt。
- 9 个 polygon slice：**`[2,2,36,12,12,48,16,16,48]`**；`nmesh=2 / nmeshpoly=12 / nmeshvert=16 /
  nmeshpolyvert=48 / nmeshpolymap=48`；`mesh_polyadr=[0,6]`、`mesh_polynum=[6,6]`、
  `mesh_vertadr=[0,8]`、`mesh_vertnum=[8,8]`；9 数组与 pinned mjModel dump **逐值相等**。
- 规则复算：sum(polynum)=12、sum(polyvertnum)=48=len(polyvert)、sum(polymapnum)=48=len(polymap)；
  vertadr/polyadr/polyvertadr/polymapadr 均 == cumsum(num)；逐 mesh/polygon/vertex 区间在界；
  `polyvert∈[0,vertnum[owner])`、`polymap∈[0,polynum[owner])`；mesh geom dataid ∈ [0,2)。
- manifest：sha 实测 `abc44369c219b2284ff176f8c22d7ed82fbecfda503366e67256ee321a6f66fc` = receipt；
  `result=PASS`、`failures=[]`、`cpu_selfcheck.rules=18` 全 PASS、`cross_check` 各项带 actual/reference 且 true。
- 同一来源（我现场重算，不读交付日志）：`xml_sha256`（fixture 实测 = manifest = 参考）、
  `qpos_bits`（capture keyframe f32 = 参考）、`geom_xpos`/`geom_xmat` 位型（pinned `mj_forward` 复算
  = bins = 参考）全逐位一致；meta 的 friction/solref/solreffriction/solimp/adhesion 位型 = 参考首行；
  pinned CPU 后端 collision count **4/4**（仅 fixture 有效性交叉证据）。

## 5. 两份参考 + 捕获窗

- M3 参考：实测 sha `96e7895d13c3ba23475a013fcaa586485e93f95cced5351e2874973917e149a1`、3676 B；
  count=4、`geom=[[0,1]]×4`、`geomcollisionid=[0,1,2,3]`、dist 全 `-0.009999997913837433`、
  `two_captures_bitwise_identical=true`、`captured_at=2026-10-27T00:48:22.577+08:00`（= 捕获窗 start）、
  flags `nativeccd_disabled=true / multiccd_enabled=true`、`geom_dataid=[0,1]`。
- 旧 M1 参考：实测仍 `c83baffe…`、5703 B；M3 提交对 M1 参考/G5 bins/旧 fixture 零 diff。
- 捕获窗原始文件（`target/m3-mesh-mesh/capture-attempt1/`）：RUN A json、RUN B json、提交参考
  **三者逐字节相同**（同一 sha `96e7895d…`）；RUN A/B 日志显示独立 warp cache 目录、exit 0；
  compare 日志 `RAW BYTE-IDENTICAL: PASS`。
- 捕获窗采样（samples.csv 87 样本）我重算：used_peak **2278**、free_min **13784**、util_max **80**、
  util≥80 **streak 2**、residual 0、baseline used 2040 → delta 238。

## 6. 验证窗（attempt1）

- 9 实体 exit **9/9 = 0**、failed_binaries=0；9 个日志均 `test result: ok`；mesh-mesh 结果
  `nacon=4 ref_count=4 bit_equal=4/4 max ULP=0/0/0 contact.geom=[0,1] failures=0`；M2 两 case 4/4 `[1,0]`；
  G5/launcher/primitive/B1/B2/B3 回归日志 failures=0（我逐日志读过）。
- **我在 RAW samples 上独立重算**（权威来源 `validation-attempt1-guard.log` + `samples.csv`）：
  732 样本、used_peak **2505**、free_min **13557**、util_max **60**、util≥80 streak **0**、residual 0；
  guard baseline used **2227** → delta **278**；第一基线点 used 2222 / 采样最小 used 2222 → **283/283**；
  floor = `min(10240, 13840−3072)` = **10240**；open 门 free 13840 ≥ 10240。
- `facts.txt` 的 `guard_aggregate=System.Collections.Specialized.OrderedDictionary` 一行确为无效行
  （runner 字典对象字符串化；原文保留不改写），证据已明确「权威数字取 guard.log + samples.csv」。
- 零 Rust 编译：`target/m3-mesh-mesh/attempt1/*.log` 中 `Compiling|rustc` **0 命中**；只跑 9 个冻结 exe
  + nvidia-smi + guard。
- 冻结：9 个二进制 sha/size 与 receipt、`logs/frozen-binaries.txt`、
  `logs/validation-attempt1-binary-shas.txt` 三处一致；`target/m3-freeze/` 副本我重新哈希 **9/9 一致**。
- post compute-apps 28 条，按 `python|g10_p5|cargo|mjwarp` 过滤 **0**（其中 5 条 `[Insufficient Permissions]`
  身份不可见，baseline 同口径也有 → 观察 O5）；机器负载 precheck 记录 `cargo/rustc/cl/nvcc` 无、CPU 负载 1/3/3。

## 7. 拆分（500 行棘轮）

- 测试名/数量不变：validate **8 = 4 父 + 4 子**（`validate_mesh_poly_tests`），assembly **7 = 5 + 2**
  （`g10_p5_mesh_poly_case_assembly` + `g10_p5_mesh_m3_case_assembly`）；实跑 8 passed、5 passed、2 passed。
- 搬移一致性：我按 whitespace-normalised 逐文本比对——3 条搬移测试函数体（accepts_real_cube…
  / rejects_inconsistent_sidecar / rejects_inconsistent_sums）在子文件**逐字不差**；assembly 的
  `sha256_hex`/`f32_bits` helper 体逐字不差，`json_f32_bits` 体逐字一致但签名类型路径
  `serde_json::Value`→`Value`（import 变化，行为同）。
- **但 `CubeMeshPoly` 数据块不是逐字节搬移**：子文件里该 struct/impl 新增了 `vertadr/vertnum` 两个字段
  并改 `over()` 填入（O7 规则 1 的前提）。交付 README §13「CubeMeshPoly 数据块原样搬 / verbatim True」
  不成立（见 O1）；测试函数体与 helper 的 verbatim 成立。
- 门禁：拆后 runs unit **990/0/374**、fmt/clippy/clippy-cuda exit 0；我在当前树 gate 复跑同样
  **990/0/374**。拆前数字（990/0/374）仅 addenda 记录（`commit hook 拒：两测试文件超 500 行，拆分后重跑门禁`），
  拆前 WIP 未入 git，故「拆前=拆后」我只能核「addenda 记载 + 拆后重跑一致」（见 O6）。
- `scope-selfcheck`：我用与 `scripts/scope-selfcheck.mjs` 相同匹配规则对 `11a13f15..f1e5cda1` 的
  75 个非 .normify/target 路径复现 = **75 / 范围内 75 / 越界 0**（与声明一致）。

## 8. 观察（按级别，均不阻断）

- **O1 一般**：拆分「纯搬移」表述不准确。`CubeMeshPoly` struct/impl 在搬移时新增 `vertadr/vertnum`
  并扩展 `over()`（O7 需要），不是逐字节搬移；3 条测试函数体与 `sha256_hex`/`f32_bits` helper 体
  确实逐字不变（`json_f32_bits` 仅类型路径随 import 变化）。对判据/产品代码/窗口数据/测试计数无影响，
  但 README §13「数据块原样搬 / verbatim True」应改为「测试函数体 verbatim；CubeMeshPoly 因 O7 增加
  vertadr/vertnum 字段」。
- **O2 一般**：stage2 record-C 数字表述错位。README §9.1/receipt 写「17 条 `failures=` 行」；实际
  **17 = dump 行数**（`两次一致=true` 17 行），CPU 比较的 **case/kernel 汇总行是 16 条**
  （8 对 Contact/ContactMulti + sdf-box），其中 failures>0 的 **11 条**（sdf-box failures=112 条明细）。
  语义（只记录、含已知 recipe 分岔、非 M3 回归）成立，数字应改为「17 dump / 16 比较行 / 11 条有失败」。
- **O3 一般**：capture 窗 delta 第三口径错。证据写「vs 采样最小 used=2032 → 246」；2032 是**首样本**
  （samples.csv first=2032），实际采样最小 used=**2012**（seq 81）→ delta **266**。前两口径
  （guard baseline 2040→238、第一基线点 2044→234）正确。三者都 ≪3072，不影响 guard 结论。
- **O4 建议**：交付证据 17 个日志 UTF-8 带 BOM；`validation-attempt1-samples.csv` 734 行 >500 行
  常规（原始窗口数据，建议注明「数据文件例外」或压缩/引用 target 原始件）。
- **O5 建议**：validation 窗 post compute-apps 有 5 条 `[Insufficient Permissions]`（baseline 同口径也有，
  属环境噪声；按关键词过滤 =0），建议照 M2 观察继续注明「身份不可见的条目不影响结论」。
- **O6 建议**：拆分前门禁数字（990/0/374）只有 addenda 文字记录，拆前 WIP 未入 git / 未留独立日志；
  「拆前=拆后」无法从仓内独立复算，建议以后把拆前/拆后的命令与数字留成证据日志。
- **O7 建议**：capture 窗 util 80% 连续 2 采样的「warp JIT 编译瞬态」归因是合理推断（RUN A 日志可见
  ccd kernel 编译 10.35 s、RUN B 0.63 s），但 samples 本身不能证明归因；建议注明「推断」。

（另有两点如实记录、不作扣分项：① M3 fixture 的 `mesh-mesh.xml` 未显式写 multiccd flag，
默认启用已由参考 flags/预检覆盖；② 采样最后时间戳比 window_end 晚约 1.8 s，属 sampler 停止抖动。）

## 9. 边界

- 未重跑 device（未开窗、未建 CUDA context）：mesh-mesh 的 `bit_equal 4/4`、9 实体 exit 0、窗口采样等
  均来自交付日志/原始 samples，我核的是日志/参考/冻结实体/规则自洽，不是独立设备执行。
- 一般 mesh 未覆盖（nmesh>1 通用、degree>10、polygon>16）；hfield-capsule 设备回归未覆盖；`efc_address`
  缺口；UNC-6 未被本 fixture 触发；`contact.geom` 顺序为调用方职责；stage2 9 synth 只按记录 C。
  不宣称 mesh 全类型/全分支、不宣称 G10@P5 整体通过或跨平台。
- 「Chief 双签/Arch 实测签名集合」为叙述性证据（仓内无签名产物）；我核的是 9 个冻结实体 sha 与
  runner 记录自洽。

## 10. 复现

```text
target/mjwarp-parallel/venv/Scripts/python.exe target/qa-m3/verify_m3.py
target/mjwarp-parallel/venv/Scripts/python.exe target/qa-m3/split_audit.py
cargo test --locked --lib runtime::collision_gpu::validate
cargo test --locked --test g10_p5_mesh_poly_case_assembly --test g10_p5_mesh_m3_case_assembly
cargo test --locked --features cuda-probe --lib runtime::collision_gpu::nvrtc_smoke -- --include-ignored --nocapture
```
