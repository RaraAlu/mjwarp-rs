# G10@P5 primitive narrowphase 设备版 · QA 独立复核

- 任务：`2026-10-25-g10-p5-primitive-narrowphase-cuda-independent-check`（p54，QA）。
- 被复核片：`2026-10-25-g10-p5-primitive-narrowphase-cuda`（Backend），产品提交 `9d72159c`。
- 复核 HEAD：**`6e2bd529`**（= `9d72159c` + 覆盖口径文案修正）。窗口日志有硬守卫：
  `git diff 9d72159c..HEAD` 只含 7 个 docs/evidence + `.normify` 文件、`src/tests/Cargo/fixtures`
  零改动、产品工作树干净（`product_diff_empty=true`、`worktree_product_changes_count=0`）。
- 上游：pinned `71da24d956378a87a703b6e1442b13aec0c4ac29`；冻结源
  `docs/evidence/d04-complex-physics/authoring-runs/run-5/frozen-source/_src/`
  （`collision_primitive_core.py`、`collision_primitive.py`、`collision_driver.py`、`collision_core.py`）。
- 参考：QA 已提交的上游 f32 GPU 捕获
  `docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-upstream-gpu.json`
  （device `cuda`；3 个 primitive case 均 `two_captures_raw_bitwise_identical=true`）。

## 1. 结论

**pass（阻断 0 / 发现 1（已由 `6e2bd529` 修正并复核）/ 观察 6）**，`normify gate` + `normify check`
全绿，可串行提交。

- **3/3 case 逐字节一致**（QA 自建 dumper 落盘 + 自写比较器对上游 f32 GPU 参考，未用交付
  `bit_equal` 汇总）：`sphere-box-shallow` **80/80**、`sphere-box-deep` **80/80**、
  `capsule-box` **80/80**，`max_dist/pos/frame_ulp = 0`、failures 0；分母
  `nworld + nacon*(35+efc_width)` 独立复算 = 80，与交付分母一致。
- **同窗口 launcher 回归（16 段拼装无回归）**：`hfield-sphere` **314/314**、`sdf-box`
  **3122/3122**、`ellipsoid-box` **80/80**，逐字节对 G5 `out1`，分母 314/3122/80 独立复算一致。
- **入口/拼装/ABI**：入口真实走 NVRTC 运行期编译 + 真实 launch/回读（`cuda-probe` 门控，非测试专用）；
  拼装独立复现 **len=138520 / sha256=`a45169bc…f8bd38`**；PTX 独立编译 **1,983,583 B / 4 入口**；
  `abi.rs` 未改；`product.cu` 纯追加 22 行入口壳、无数值内核重定义。
- **覆盖口径**：发现 1（BOX-BOX 在 `nativeccd=disable` 下属 PRIMITIVE → 13 组合/未实现 11）已由
  Backend 修正 §2/表格/receipt/commit message，本复核逐字核对与 frozen 源码一致。
- 边界：仅 3 case × 2 world × 1 接触/world；CAPSULE-BOX 第二接触机制 0 覆盖（交付已声明）；
  未接入默认产品管线；未作精度达标/等价/放行类结论。

## 2. 复核方法（不复用交付比较器/汇总）

1. **自己的 dumper**：`dumper/`（独立 crate，只依赖 `mjwarp-rs` 公开 API + `cuda-probe`）用产品入口
   `CollisionGpuProgram::new(0)` + `run(CollisionKernel::Primitive, ..)` 跑 3 个冻结 case，落盘 18 个
   原始输出数组 + `array-lengths.txt`；**每 case 一次独立进程**，任一失败立即停。
2. **自己的比较器**：`qa_primitive_check.py`（不 import 交付 `tests/support/*`）：结构/计数/参数零容差、
   位级优先、非位级按 G5 口径算 ULP；行序按 `(worldid, dist, pos, frame)` 位级全序排序后按多重集比较；
   分母独立复算；窗口内现场重算 23 项输入 bin 的 sha256/count 与参考 `input_bins` 比对。
3. **拼装/PTX 独立复算**：`qa_assemble_check.py` 自己解析 `source.rs` 的 `DEVICE_SOURCES` 顺序、
   读 16 个 `.cu`、删本地 `#include "*.cu"`、追加 `product.cu` 后算 len/sha256；再用 ctypes 直调
   `nvrtc64_120_0.dll`（选项同产品：`--fmad=true --std=c++17 --gpu-architecture=sm_89`）得到 PTX
   长度与入口名，不经过产品 `assemble_device_source()`/`compile_program()`。
4. **判据反向判别力自检**：`qa_primitive_selfcheck.py` 用上游参考**合成 dump**（不跑 GPU）做受控
   变异，证明判据会失败/不误报；产品侧结论由窗口真实 dump 承担。
5. **分支归因**：`qa_branch_analysis.py` 用 numpy.float32 逐式转写上游 `sphere_box`/`capsule_box`
   （影子实现，不参与判据），从 QA 自己的冻结输入 bin 复算走了哪个分支。
6. **launcher 回归**：`qa_launcher_regression_check.py` 用同一 dumper 落盘，与 G5 窗口 `out1`
   逐字节比较（零容差）。

## 3. 语义逐行对照（设备实现 vs pinned 上游）

对照：`src/physics/collision/cuda/primitive.cu`（371 行）、`primitive_wrapper.cu`（37 行）、
`product.cu::mjwarp_collision_primitive`（22 行）vs 冻结 `collision_primitive_core.py::sphere_box`
（L1049-1102）/`capsule_box`（L1105-1460）、`collision_primitive.py::sphere_box_wrapper`
（L1087-1157）/`capsule_box_wrapper`（L1160-1246）、`collision_core.py::write_contact`（L214-294）。

| # | 上游语义 | 设备实现 | 结论 |
| --- | --- | --- | --- |
| 1 | `center = transpose(box_rot) @ (sphere_pos - box_pos)` | `transpose_mul_mat3_vec(box_rot, 差)` | 一致（转置方向一致） |
| 2 | `clamped = wp.max(-size, wp.min(size, center))` | 分量 `fmaxf(-size, fminf(size, center))` | 一致 |
| 3 | `normalize_with_norm`：`n=length(x)`；`n==0` 返回 `(x,0.0)` 否则 `(x/n,n)` | `sqrtf(dot)`；`length==0` 返回原向量+0 | 一致（不除零） |
| 4 | `safe_div(x,y) = x / where(y!=0, y, MJ_MINVAL)` | `x / (y!=0 ? y : 1e-15f)` | 一致 |
| 5 | 盒内分支（`dist<=MINVAL`）：6 面枚举 `closest > face_dist` 严格更小、同距取首个；`nearest[k//2]=k%2?-1:1`；`pos=center+nearest*(r-closest)/2`；`dist=-closest-r`；`normal=box_rot@nearest` | 同（`k` 与 `i%2` 同向） | 一致 |
| 6 | 盒外分支：`deepest=center+clamped_dir*r`；`pos=0.5*(clamped+deepest)`；`dist=dist-r`；`normal=box_rot@clamped_dir` | 同 | 一致 |
| 7 | `contact_position = box_pos + box_rot @ pos` | 同 | 一致 |
| 8 | capsule：`pos/axis = boxmatT @ (…差值/axis)`；`halfaxis=axis*half_length`；`axisdir=Σ2^j*(halfaxis[j]>0)` | 同 | 一致 |
| 9 | 初值 `bestdist=1e32`/`bestsegmentpos=-12`/`cltype=-4`/`clface=-12`；`best=0.0` 后 `best=-1.0`（死赋值） | 初值同，未写死赋值 | 一致（无行为差异） |
| 10 | 第一段 loop `i∈{-1,1}`：端点逐轴 clamp、`n_out>1` continue、`length_sq<bestdist` 更新、`cltype=-2+i`、`clface=ax_out` | 同（`-1..1 step 2`） | 一致 |
| 11 | 第二段 loop `i<8×j<3`（跳 `i&(1<<j)`）：`box_pt` 角点把 j 轴置 0；`u/v/ma/mb/mc/det`、`abs(det)<MINVAL` continue；`x1/x2` 两轮 clamp + `safe_div`（`s1/s2`）；`dif -= halfaxis*x2; dif[j] += size[j]*x1`；`ct=s1*3+s2`；`dif_sq<bestdist-MINVAL` 更新 `c2=ct//6`、`clcorner`、`cledge` | 同 | 一致 |
| 12 | `c1` 编码：`p/dd/s` 取 x,y；`w_neg`；`ee1/ee2` 与 `best=-1` 顺序比较 | 同 | 一致 |
| 13 | `cltype==-4` → `vec2(MJ_MAXVAL)` + 空 `mat23f()` | 两 `dist=1e10`、`pos/normal=0` | 一致 |
| 14 | 角点分支（`cltype>=0 and cltype//3!=1`）：`c1=axisdir^clcorner`；`{0,7}` 无第二接触；`mul`+`c1=7-c1`；`ax/ax1/ax2`；`axis[ax]^2>0.5` 走盒边 `m=2*safe_div(...)` 否则盒面 `m=2*min(...)`；`secondpos*=mul` | 同 | 一致 |
| 15 | 盒边分支（`cltype//3==1`）：`c1 &= 7-(1<<cledge)`；仅 `{1,2,4}` 产生第二接触；`ax1/ax2`、两次 `min`、`secondpos*=mul` | 同 | 一致 |
| 16 | 端点-盒面分支（`cltype<0`）：`clface!=-1` 时 `mul`（`cltype==-3`→1）、`secondpos=2`、另两轴 `ha_r` 两个候选取小、`secondpos*=mul` | 同 | 一致 |
| 17 | 第一接触 `sphere_box(pos+halfaxis*bestsegmentpos→世界系)`；`secondpos>-3` 才生成第二接触，否则 `dist=MJ_MAXVAL`、`pos/normal=0` | 同 | 一致 |
| 18 | `sphere_box_wrapper`：`write_contact(id_=0,..)` 单接触 | 同 | 一致 |
| 19 | `capsule_box_wrapper`：`axis=vec3(cap.rot[0,2],cap.rot[1,2],cap.rot[2,2])`；循环 2 次 `write_contact(id_=i,..)` | `rot[2],[5],[8]`（行优先第三列）+ id 0/1 | 一致 |
| 20 | `write_contact`：`active`/`detected` 门、先 `atomicAdd(nacon)` 再判容量、`contact_type` 位、`geomcollisionid=id_`、`efc_address=-1` | `contact.cu::g5_write_contact` 同（G5 已 verified；本片关键点：`detected` 门用于跳过 `MJ_MAXVAL` 第二接触） | 一致 |
| 21 | dispatch：上游按 `(type1,type2)` 精确匹配 `_PRIMITIVE_COLLISIONS`；pair 顺序由 `_add_geom_pair` 按 **type 升序**构造（`collision_driver.py` L373-376） | `product.cu` 按 `a.type1/a.type2` 分派，**额外**支持 `(BOX,SPHERE)`/`(BOX,CAPSULE)` 交换 | 升序路径一致；交换路径上游不可达（观察 3） |

**结论**：逐式 f32 转写，无语义偏差；未重定义任何数值内核。

## 4. 入口真实性、拼装与 ABI（CPU 独立复算）

- **入口真实**：`runtime::collision_gpu` 整模块在 `#[cfg(feature = "cuda-probe")]` 下（不在 default
  features）；`new` 走 `cudarc::nvrtc::compile_ptx_with_opts`（`fmad=true`、`use_fast_math=false`、
  `--std=c++17`、`--gpu-architecture=sm_89`）→ `load_module` + 4 个 `load_function`；`run(Primitive)`
  真实 launch/回读；不引用探针、不是 `#[ignore]` 测试专用路径。
- **拼装**：独立复现 **16 段 / len=138520 / sha256=`a45169bc…f8bd38`**，与交付一致；顺序
  `common→…→hfield→contact→primitive→primitive_wrapper→product.cu`。
- **PTX**：ctypes 直调 NVRTC（同选项）→ **1,983,583 B / 4 入口**（contact/hfield/sdf/primitive）。
- **ABI**：`abi.rs` 在 `7e5f4f43..9d72159c` 未改动。
- **胶水边界**：`product.cu` 只加唯一入口壳（1 处定义）；`primitive.cu`/`primitive_wrapper.cu`
  不重定义 `make_frame`/`g5_write_contact`（拼装源统计各 1 处定义）。

## 5. 分支归因（设备到底跑了哪些分支）

自写 numpy.f32 上游转写从 QA 冻结输入 bin 复算（`qa_branch_analysis.py` 输出，见 receipt
`branch_analysis`）：

| case | world | 执行分支 | 证据 |
| --- | --- | --- | --- |
| `sphere-box-shallow` | 0/1 | `sphere_box` **盒外** | 球心局部 z=0.100 > size.z=0.06；dist=-0.06 |
| `sphere-box-deep` | 0 | `sphere_box` **盒内（深穿透）** | 中心局部 (0.02,0,0.02) 全在 size (0.10,0.08,0.06) 内；closest=0.04；dist=-0.14 |
| `sphere-box-deep` | 1 | `sphere_box` **盒内** | 中心局部 z=-0.04；closest=0.02；dist=-0.12 |
| `capsule-box` | 0 | `g5_capsule_box` **`cltype<0` 端点-盒面分支** + `sphere_box` 盒内 | `cltype=-3`、`clface=-1`（端点 z=0.02 在盒内）、`secondpos=-4`（无第二接触）；dist=-0.09 |
| `capsule-box` | 1 | 同上 | `cltype=-3`、`clface=-1`、`secondpos=-4`；dist=-0.10 |

**未执行**：`sphere_box` 盒内/盒外两分支均被覆盖；但 `g5_capsule_box` 的**角点分支、盒边分支
与全部第二接触机制**（`secondpos` 三处赋值路径 + 第二 `write_contact`）本片 **0 覆盖**——交付已
在 §6 声明，本复核独立确认并量化（对应 `primitive.cu` L254-353 约 100/371 行）。

## 6. 覆盖缺口核对

- **发现 1（已由 `6e2bd529` 修正）**：交付原写「上游 primitive dispatch 共 12 组合、未实现 10」并
  断言 `(BOX, BOX)` 不属于 primitive dispatch。实际本项目**全部 fixture 为 `nativeccd="disable"`**，
  frozen `collision_driver.py` L909-912 会把 `(BOX, BOX)` 覆盖为 `CollisionType.PRIMITIVE`
  （走 `box_box_wrapper`）→ 本项目配置下 **13 组合 / 未实现 11**；且 `box-box-margin` 的上游
  primitive 参考已存在于 `qa-upstream-gpu.json`（抓取脚本 `qa_prepare_capture.py::capture_contacts`
  调 `mjw.collision`，provenance sha256 `5cf2f746…`）。Backend 已按此修正 §2/表格/receipt/commit
  message，本复核逐字核对与源码一致，无残留错误表述。
- 「无既有 host 参考」成立：G5 host 交付 9 case 不含 SPHERE-BOX/CAPSULE-BOX，本片未伪造 host 参考。
- 「CAPSULE-BOX 第二接触分支未被样本触发」成立（§5），未写成通过。
- 「产品入口仍经 `cuda-probe`、未接入默认产品管线」成立。
- 修正后未发现把「未覆盖」写成「通过」或省略未覆盖。

## 7. 判据独立复算（窗口结果）

窗口 HEAD `6e2bd529`，只跑**窗口外预构建**的 dumper；每 case 独立进程 + 跑完立刻比较，任一失败
即停（本次 6/6 全过，未触发早停）。

| case | kernel | nacon | nccd | 逐字节 | 独立分母 | 交付分母 | max_dist/pos/frame ULP | failures |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `sphere-box-shallow` | Primitive | 2 | 0 | **80/80** | 80 | 80 | 0 / 0 / 0 | 0 |
| `sphere-box-deep` | Primitive | 2 | 0 | **80/80** | 80 | 80 | 0 / 0 / 0 | 0 |
| `capsule-box` | Primitive | 2 | 0 | **80/80** | 80 | 80 | 0 / 0 / 0 | 0 |
| `hfield-sphere`（回归） | Hfield | 8 | 0 | **314/314** | 314 | 314 | 0 / 0 / 0 | 0 |
| `sdf-box`（回归） | Sdf | 80 | 0 | **3122/3122** | 3122 | 3122 | 0 / 0 / 0 | 0 |
| `ellipsoid-box`（回归） | Contact | 2 | 2 | **80/80** | 80 | 80 | 0 / 0 / 0 | 0 |

- 18 个原始数组逐 4 字节比较；结构/计数/参数/整数零容差；`overflow` 全 0；`efc_address` 逐项 -1。
- **输入守卫**：3 case × 23 项 bin 的 sha256 与 count 全部等于参考 `input_bins`，且与交付 tracked
  `gpu-window/cases` 逐项相同（`qa-primitive-check.json` 的 `inputs` 字段）。
- 行序：设备 `nacon` 行序跨 world 由 `atomicAdd` 竞态决定（观察 1）；按判据排序后多重集全等。

## 8. GPU 窗口事实

- 窗口：2026-10-25 23:21:25.450 → 23:22:23.413（**elapsed 57.2 s**），0.25 s 采样 **201 点**。
- 起点 HEAD **`6e2bd5296091e6803e1c24d931bd216b6c4917ed`**；`product_head=9d72159c`；
  `git diff 9d72159c..HEAD` = 7 文件（primitive 片 README/receipt + 5 个 `.normify` 记录），
  **`src/tests/Cargo.toml/Cargo.lock/fixtures` diff 为空**（`product_diff_empty=true`）、产品工作树
  改动 0（`worktree_product_changes_count=0`）。
- 基线：used **3737 MiB** / 16376（相对 3693 漂移 44，<200），32°C，34% util（桌面瞬时），driver
  596.36；`query-compute-apps` **34 条全桌面图形上下文**，本任务相关 **0**。
- 峰值：采样峰值 **3882 MiB = +145 MiB**（序列 min 3663、max 3882；峰值 1 点 3882、1 点 3850、
  其余 ≤3737）；软阈值 +1536 未超、硬守卫 3072 MiB/50%（8188）未触发；结束 used **3668**；
  跑后 `query-compute-apps` 34 条（全桌面）、本任务相关 **0**、`qa-primitive-dumper` 残留 **0**。
- 工具链：`PATH` 前置 `target/toolchains/cuda-12.8.1/bin`；nvcc `Build cuda_12.8.r12.8/compiler.35583870_0`
  （release 12.8, V12.8.93）；NVRTC DLL sha256 `nvrtc64_120_0.dll=02d2d7ef…346e93`、
  `nvrtc64_120_0.alt.dll=5464ea01…cf2ae`、`nvrtc-builtins64_128.dll=a7afa2a4…668f5`。
- dumper（窗口外预构建，窗口内未编译）：`target\qa-primitive\dumper-target\debug\qa-primitive-dumper.exe`，
  sha256 `f3a3fbf953beb5489f62eb29575f56c5a591cc39aa3b8a65b3a07aae2648f104`，构建时间 23:10:55。
- 逐 case wall：9.44 s / 9.37 s / 9.45 s（primitive）、9.57 s / 9.64 s / 9.67 s（回归）；
  NVRTC 编译在 CPU 侧，GPU 满载极短。
- 桌面 top5（工作集）：vmware-vmx 17714 MiB、DeepSeek Harness 1701 MiB、Code 1049 MiB、
  MsMpEng 760 MiB、explorer 668 MiB。

## 9. 反向判别力与错误路径

- **判据侧**（`qa_primitive_selfcheck.py`，**合成 dump 自检**，不跑 GPU、不是真实设备数据）：
  11 场景全部符合预期——基线 80/80；`dist+1 ULP` → 79/80、max ULP=1（判据内，位级计数敏感）；
  `dist+1e-4`（26844 ULP）、`pos+1e-3`、`frame+1e-6`、`nacon−1`、`nccd=1`、`overflow` 置位、
  `efc_address=0` → 全部失败；`frame+2e-7`（≤4·ULP@1.0）→ 79/80 但判据内（不误报）；交换两行 →
  排序恢复 80/80。
- **launcher 比较器**：用 G5 `out1` 合成 dump 得 314/3122/80；把 `hfield-sphere dist[0]` 翻 1 bit →
  313/314、failures=1 正确报错。
- **产品侧**：本片新增 CPU 单测 4 条（`primitive_tests.rs`）：4 种顺序接受 / 10 种非法组合拒绝、
  `DEVICE_SOURCES` 片段数与顺序、primitive 入口唯一性 + `make_frame` 单定义；均为可失败断言。
  错误路径 12 条 CPU 单测为真覆盖（编译 Err、编译期 panic 映射不 panic、无设备、编译错误优先、
  非法 meta/slice、ABI 偏移、源拼装确定性等），不是只断言 `Ok`。
- **缺口（沿用 NVRTC 片观察）**：`CollisionGpuError::Load`/`Launch` 分支无 CPU 可注入 seam，仅
  Display 覆盖；未实现组合在设备入口内是「不写接触」的空分支，但产品 `run` 在 launch 前
  `validate_primitive_types` 直接返回 `InvalidArgument`（有 10 组合拒绝单测），不会静默通过。

## 10. 观察项（均非阻断）

1. **行序跨 world 由 `atomicAdd` 竞态决定**：本次 `sphere-box-shallow/deep` 设备 raw 行序为
   world1→world0、参考为 world0→world1（`capsule-box` 同序），排序后 80/80 全等。与上游同为
   `atomic_add` 分配行号的机制，非本片缺陷；但**不能用 raw 行序作判据**，逐行 diff 也不可靠
   （交付与 QA 均用排序口径，正确）。
2. **峰值采样口径**：本次 201 点 0.25 s 采样峰值仅 **+145 MiB**，未复现 NVRTC 片 run1 的
   **+1325 MiB** context 创建瞬时（同一配方、同一机器）。说明该瞬时是否被采到取决于采样时机
   （Windows/WDDM 显存记账 + 进程寿命），**交付 +197 与本次 +145 都不能证明「无瞬时」**；因此
   窗口峰值数字只作资源观测，不用作设备占用结论；功能/数值结论不受影响。
3. **反向实参顺序分支无参考**：`product.cu` 支持 `(BOX,SPHERE)`/`(BOX,CAPSULE)` 交换，但上游 pair
   由 `_add_geom_pair` 按 type 升序构造（`collision_driver.py` L373-376），交换顺序在上游流程中
   不可达、也无 case/参考。不影响本片数值结论；建议交付 README §2 该行标注「单方向覆盖」
   （现为 `(SPHERE, BOX) / (BOX, SPHERE)` 合并成一行给 80/80），下片再定是否补 host 交叉或删分支。
4. **dispatch 依据是 meta.type1/type2 而非 `inputs.geom_type`**：上游按 `geom_type[geoms[0/1]]`
   分派，设备按元参数分派，未与 `geom_type/g1/g2` 交叉校验。QA harness 两者一致；作为产品 API
   健壮性可加一致性校验（建议级）。
5. **CAPSULE-BOX 第二接触机制 0 覆盖**（角点/盒边/第二 `write_contact`）：交付如实声明，未写成
   通过；下片如需覆盖要造 corner/edge 姿态 case（上游参考需重新 authoring）。
6. **`nvrtc_smoke` 只作诊断**：`ptx_bytes=1983583`/`source_sha256` 已由 QA 独立 NVRTC 编译与
   独立拼装复算，不依赖该冒烟输出。

## 11. 边界与未覆盖

- 仅 3 个冻结 case × 2 world × 1 接触/world = 6 个接触点；只覆盖 primitive dispatch 的 2/13 组合。
- 未覆盖：其余 11 组合（含 BOX-BOX，其参考已存在）；CAPSULE-BOX 第二接触机制；无 host 交叉参考；
  产品入口未接入默认产品管线；只有 Windows/RTX 4070 Ti SUPER 16 GiB/NVRTC 12.8 环境。
- 未做性能测试；未改判据/已 verified 数字/产品代码/`Cargo.*`/fixtures。
- 不宣称精度达标、上游完全等价、原生放行或双平台通过。

## 12. 可复现路径与命令

- dumper：`dumper/`（`cargo build --manifest-path dumper/Cargo.toml --offline`，独立
  `CARGO_TARGET_DIR=target/qa-primitive/dumper-target`；窗口内不编译）。
- 比较器：`qa_primitive_check.py`（primitive vs 上游参考）、`qa_launcher_regression_check.py`
  （launcher 回归 vs G5 `out1`）、`qa_assemble_check.py`（拼装/PTX）、`qa_branch_analysis.py`
  （分支归因）、`qa_primitive_selfcheck.py`（判据自检）、`qa_artifacts_sha256.py`（清单）。
- 窗口：`run_window.ps1`（HEAD/diff 守卫 + 0.25 s 采样 + 守卫 + per-case 早停）。
- 原始输出：`target/qa-primitive/prod-out/<case>/`（不入库，可由 dumper 重跑重建）；
  诊断中间物 `target/qa-primitive/{assemble-check.json,selfcheck.json,case-branch-analysis.json}`（不入库）。
