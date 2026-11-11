# QA 复核：z2vec 片 1（`19a68b49`）

- 任务：`2026-11-11-render-flex-z2vec-pin-check`（owner QA）。复核对象：提交 **`19a68b49`**（19 files；HEAD 当时、树 clean）+ 卡 `2026-11-11-render-flex-z2vec-pin`（verified）。零 GPU、只读复核；结论均为自算/自测。
- 参考：Arch 对片 1 的独立复核 PASS（0 阻断，1 观察：镜像 `warp_length3` 用 Rust 左结合求和 vs `.cu` 显式 `fmaf` 链，非位级一致、1 ulp 级、容差 1e-6 内，建议片 2 注释注明「数值对应、非位级 oracle」）。本卡采纳记录该观察，不阻断。

## 结论

**复核通过（0 阻断，1 低 + 1 观察）。**

## 一、改动面（✅）

- `src/render/kernel/cuda/render_flex.cu`：**+6/−1**，只动 `flex_quat_z2vec` 的近平行分支——`quat[0]=1.0f` 后补 `quat[1]=0.0f; quat[2]=0.0f; quat[3]=0.0f;`（四位齐），注释补上游 `math.py::quat_z2vec` 整体替换语义与旧版 `(1,0,0,1)` 缺陷。同一分支的 `if (unit.z < 0.0f)` 判定、`return`、初始化 `quat=[0,0,0,1]`（:21-25）、一般分支 `axis normalize/atan2f/half/sinf/cosf`、`flex_quat_to_mat` 均未动。
- `tests/render_flex_z2vec.rs`：**新增 215 行**（CPU-only 宿主镜像，测试专用）。
- 其余：证据目录 11 文件 + `.normify` 卡/门禁；`fixtures/`、`scripts/` **0 改动**（`git show --name-only` 仅 `flex.cu` + 测试 + 证据，探测 `.cu` 在证据目录内）。guards JSON `94478b3d…`/43,729 B 未变（与 attempt4 冻结件同值）。
- 编码：`render_flex.cu` 7,426 B、UTF-8 无 BOM、CR=0、NUL=0。

## 二、测试（✅ 我独立跑）

- `cargo test --locked --test render_flex_z2vec` ⇒ **9 passed / 0 failed / 0 ignored**（`probe_1…`, `probe_2…`, `probe_3_y_axis`, `probe_4_x_axis`, `probe_5_zero_vector…`, `probe_6_and_7_diagonal_planar_vectors`, `threshold_and_near_parallel_branches`, `negative_zero_vector`, `cu_source_anchors_match_mirror_contract`）。
- 口径核对：`#1`（`(0,0,-1)`）用 **`assert_eq!` 精确单位阵**；`#2–#7` 容差 `1e-6`（文件头注释明示 #3/#4/#6/#7 有 1 ulp CPU 三角函数库差）；阈值两侧 `1e-16`/`1e-20`、近平行两支 `(1e-30,0,∓1)`、`-0.0` 均在用例内。
- 文本锚点自检 + 不静默回退声明在**文件头**（`//!` 硬约束：镜像不进产品路径、不得运行时回退、`.cu` 与镜像不一致必须测试失败、不允许 skip/fallback）与测试 `cu_source_anchors_match_mirror_contract`（读取真实 `.cu`、锚定 `warp_length3`/`atan2f(a, unit.z)`/`quat[0..3]`）双处落地 ✅。
- **观察（采纳 Arch）**：镜像 `warp_length3` 为 Rust 左结合求和，`.cu` 为显式 `fmaf` 链 ⇒ 非位级 oracle（1 ulp 级，容差内）；建议片 2 在镜像函数注释注明「数值对应、非位级 oracle」。

## 三、内核身份（✅ 我独立复算，NVRTC 纯编译，零 GPU）

按 `src/render/kernel/mod.rs` 的 `concat!`/`include_str!` 顺序拼接源、仓库 pinned `cuda-12.8.1` NVRTC + product 选项（`--fmad=true --std=c++17 --gpu-architecture=compute_70 --device-as-default-execution-space`）编译；PTX 长度按去尾部 NUL 口径（`nvrtcGetPTXSize` 含终止 NUL，去掉后与声明一致）：

| artifact | 我的复算 | 卡声明 | 结论 |
| --- | --- | --- | --- |
| megakernel source（新） | `cf356b728c563f7da89e33e1d06b0765028e4861bee990c4a1db54843395ab48` / 110,879 B | 同 | ✅ |
| megakernel source（旧，`19a68b49^` 的 `render_flex.cu`） | `6b1ade37958facfd48f485a7433f93bcf7256a6b08a167c1ece078e0744de278` / 110,509 B | 同 | ✅ |
| megakernel PTX（新） | `b63a51ddcc41a5ec7520fdc38a5234a2d44a1874bc4b38b298f4338bfb11df14` / 348,459 B | 同 | ✅ |
| megakernel PTX（旧） | `48f8e7f820f7a0a931a82e07efbebef07b831b3895bb7c6423dcd4975c31fa02` / 348,399 B | 同 | ✅ |
| depth_seg source/PTX | `1e3f273c…` / `31912377…` / 138,857 B | 未变 | ✅ 未变 |
| aa_resolve source/PTX | `0bf8af5a68f9cae50d0cb9cc4da2a01da2607066cf1fe3072c8345e6e104bc76` / `b28e155c…` / 27,536 B | 未变 | ✅ 未变 |

- source +5 行文本 ⇒ +370 B（110,879−110,509）；PTX +60 B（348,459−348,399）——量级与「只在分支块多两组 mov/寄存器赋值」一致（Arch 的 +1 系按其它抽取口径，见其观察，不影响身份值）。
- 身份值与我此前 P10 基线（`6b1ade37…`/`48f8e7f8…`）无缝衔接：本次只是 `render_flex.cu` 变化引起 megakernel 新基线，depth_seg/aa_resolve 逐字未变。

## 四、证据与边界（✅）

- 证据 manifest **11/11** 独立重算（bad=0/missing=0/extra=0、不自列）；README `e4cb5098…`/5,256、receipt `50388381…`/2,798、manifest `bf9eac4e…`/1,251（我另算出，非采信）。
- 离线分支证明（`logs/offline-probe-branch.txt` + 探针 `.cu`/`.ptx`）：结论为 OLD 只置 x（w 保持入口 1.0）⇒ `(1,0,0,1)` ⇒ `diag(0,0,2)`（max|diff|=2.0）；NEW 同时置 x=1、w=0 ⇒ 精确单位阵——与我复算的 PTX 字节数变化方向一致；探针源与 PTX 均已归档（非仅文字）。
- 边界：`fixtures/`、`scripts/` 零改动；guards/冻结件未动（`94478b3d…`/43,729 B）；不改判据/参考/容差/相机/recipe；无 GPU 会话、未重启窗口。

## 五、记录面低项 + 观察

- **低 L1（措辞）**：卡/提交/README 写「**不**改 `(0,1,0,0)`」，但 `render_flex.cu` 与全 `src/` 中**不存在**该四元组字面（`grep` 仅命中 `src/physics/**` 无关元组）。实际未改动的是**同向路径保存的初始单位四元数 `(0,0,0,1)`**（:21-25 与 `return` 早退），建议把措辞改成「不改同向路径的 `(0,0,0,1)`」以免后续读者去源码里找不存在的锚点。
- **观察 O1（Arch 同项）**：镜像 `warp_length3` 求和次序与 `.cu` 非位级一致（1 ulp 级），建议片 2 注释明示「数值对应、非位级 oracle」。
- **观察 O2（口径）**：PTX 长度存在「含/不含终止 NUL」两种口径（差 1 B，内容同）；本复核统一按去 NUL 计，与卡/README 一致。

## 六、口径（非声明）

不宣称设备端渲染变化/修复生效；CPU 镜像断言不等价于设备端验收；设备端行为只由内核身份（source/PTX 前后基线）+ 离线 PTX 证明承载；不改判据/参考/容差；零 GPU。**后续任何渲染 GPU 窗口都需重新冻结**（megakernel 身份已由 `6b1ade37`/`48f8e7f8` 变为 `cf356b72`/`b63a51dd`）——建议记入 backlog。
