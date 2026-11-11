# z2vec 片 1：flex 近平行分支显式四位四元数 + 宿主镜像 CPU 单测（零 GPU）

- 任务：`2026-11-11-render-flex-z2vec-pin`（owner Backend；p97；P8-渲染与BVH）。依据 Arch 规格
  `target/render-bvh/gl2-ptx-diff/z2vec-probe-expectations.md`（`b815d932…`/8,200 B）与 `r06-s3b-flex-z2vec/` 证据。
- 结论口径：修复 `render_flex.cu::flex_quat_z2vec` 近平行反向分支只写 `quat[0]` 的缺陷（上游 `math.py::quat_z2vec`
  是**整体替换** `(1,0,0,0)`）；宿主镜像 9 tests 全绿；megakernel 换新内核身份（`render_flex.cu` 只在
  `MEGAKERNEL_SOURCE`）；depth_seg/aa_resolve 不变。**不宣称设备端验收**（见 §6 强制标注）。

## 1. 改动（仅 1 处 `.cu` 分支 + 1 个新测试文件）

| 文件 | 改动 |
| --- | --- |
| `src/render/kernel/cuda/render_flex.cu` | 近平行反向分支显式写全四位 `quat[0]=1.0f; quat[1]=0.0f; quat[2]=0.0f; quat[3]=0.0f;`（**不**改 `(0,1,0,0)`）；注释补上游整体替换语义与旧版缺陷 |
| `tests/render_flex_z2vec.rs`（新增） | 宿主逐式镜像 CPU 单测：7 探针 + 阈值两侧 + 近平行两支 + `-0.0` + 文本锚点自检（9 tests） |

```text
 render_flex.cu  | 8 ++++++--
 tests/render_flex_z2vec.rs | NEW  (9 tests)
```

## 2. 镜像 ↔ `.cu` 逐行对照（Arch 防漂移要求）

| 镜像函数（`tests/render_flex_z2vec.rs`） | 产品 `.cu`（行号为当前 worktree） |
| --- | --- |
| `warp_length3` | `render_flex.cu:30-32`（`sqrtf(fmaf(z,z,fmaf(y,y,x*x)))`） |
| `flex_quat_z2vec` 初始化 `[0,0,0,1]` | `render_flex.cu:36-39` |
| `norm < FLEX_MINVAL` 早退 | `render_flex.cu:40-43`（`FLEX_MINVAL=1.0e-15`，`:27`） |
| `axis = (-unit.y, unit.x, 0)` / `a = length3(axis)` | `render_flex.cu:45-46` |
| 近平行分支（反向 `(1,0,0,0)`；同向保持初值） | `render_flex.cu:47-58`（修复点） |
| 一般分支 `axis normalize` / `atan2f(a, unit.z)` / `half` / `sinf/cosf` | `render_flex.cu:59-68` |
| `flex_quat_to_mat`（行主序 3x3） | `render_flex.cu:66-91` |

文本锚点自检（`cu_source_anchors_match_mirror_contract`）：读取真实 `.cu`，断言四位赋值齐备、旧注释/旧形态不存在、
分支内恰 4 个 `quat[` 赋值，并锚定 `warp_length3`/`atan2f(a, unit.z)`/`quat[0..3]` 行——源改而镜像不跟改时测试必红。

## 3. 测试结果（`logs/host-mirror-tests.log`）

- **9 tests passed / 0 failed**（CPU-only）：#1 `(0,0,-1)` **精确单位阵**（`assert_eq!`）；#2–#7 容差 1e-6
  （#2 `(0,0,1)`→`diag(-1,-1,1)`；#5 `(0,0,0)` 按上游 quat_to_mat 口径 `diag(-1,-1,1)`）；
  阈值两侧 `1e-16`/`1e-20`；近平行两支 `(1e-30,0,-1)`→单位阵（修复点）/`(1e-30,0,1)`→`diag(-1,-1,1)`；`-0.0`。
- 期望值来源：Arch 规格 §2（上游真值 `probe_quat.log`；#3/#4/#6/#7 有 1 ulp CPU 三角函数库差，故用容差）。

## 4. 内核身份（megakernel 新基线；depth_seg/aa_resolve 不变）

| artifact | old（改动前） | new（改动后） |
| --- | --- | --- |
| megakernel source | `6b1ade37958facfd48f485a7433f93bcf7256a6b08a167c1ece078e0744de278` | **`cf356b728c563f7da89e33e1d06b0765028e4861bee990c4a1db54843395ab48`** |
| megakernel PTX | `48f8e7f820f7a0a931a82e07efbebef07b831b3895bb7c6423dcd4975c31fa02` / 348,399 B | **`b63a51ddcc41a5ec7520fdc38a5234a2d44a1874bc4b38b298f4338bfb11df14` / 348,459 B** |
| aa_resolve | `0bf8af5a…` / `b28e155c…` / 27,536 B | 未变 |
| depth_seg | `1e3f273c…` / `31912377…` / 138,857 B | 未变（`render_flex.cu` 不在 DEPTH_SEG_SOURCE） |

采集：identity 用例（NVRTC 纯编译，无 GPU 会话），`logs/old-identity-flags.log`、`logs/new-identity-flags.log`、
`logs/old-new-depth-seg-identity.log`。

## 5. 离线 NVRTC 分支证明（`logs/offline-probe-branch.txt`）

- 探针源从 HEAD/worktree 的 `render_flex.cu` **逐字节抽取** `warp_length3`+`flex_quat_z2vec`，以常量
  `vec=(0,0,-1)` 调入 kernel 并落盘 `quat[4]`；product 选项 NVRTC 编译后读 PTX。
- **OLD**：近平行分支只 `mov.f32 %f137, 0f3F800000`（x=+1），w 寄存器 `%f134` 保持入口初始化的
  `0f3F800000`（w=+1）⇒ 四元数 `(1,0,0,1)` ⇒ `quat_to_mat = diag(0,0,2)`（与单位阵 max|diff| = **2.0**，即红例）。
- **NEW**：分支同时 `mov.f32 %f137, 0f3F800000`（x=+1）与 `mov.f32 %f134, 0f00000000`（w=0）
  ⇒ 四元数 `(1,0,0,0)` ⇒ **精确单位阵**（#1 转绿）。

## 6. 边界与非声明（含强制标注）

- 边界：只改 1 个 `.cu` 分支（+注释）与新增 1 个测试文件；`fixtures/`、`scripts/` 零改动；不改判据/参考/容差/相机/recipe；零 GPU。
- **强制标注（逐字）**：「设备端行为仅由内核身份（megakernel source/PTX 前后基线）+ 离线 PTX 证明；CPU 镜像断言不等价于设备端验收。」
- **不静默回退**：宿主镜像仅 host 断言，产品路径始终使用 `render_flex.cu` 设备侧实现；镜像不进产品路径、
  不得作为运行时回退；`.cu` 与镜像不一致时测试必须失败（不允许 skip/fallback）。
- 非声明：不宣称设备端渲染结果变化/修复生效（需后续诊断轮或 GPU 验证）；历史证据与提交说明不改写。

## 7. 记录面 errata（2026-11-11 卡 A；append-only，原文字节不改）

本段只更正记录面表述，不改变片 1 的任何结论（9 tests 全绿、内核身份、离线 PTX 分支证明与边界不变）。
完整留证见 `perworld-l-v2-flex-z2vec-record-errata/README.md`。

- **L1 措辞更正（QA 复核 `ebc8e97e`）**：§1 表与提交说明里的「**不**改 `(0,1,0,0)`」措辞不对——
  `(0,1,0,0)` 四元组**字面**在产品 `render_flex.cu` 与全 `src/` 中都不存在（QA grep 仅命中
  `src/physics/**` 的无关元组）。实际未改动的是**同向路径保存的初始单位四元数 `(0,0,0,1)`**
  （`render_flex.cu:36-39` 入口初始化，`unit.z >= 0` 的近平行同向分支原样返回）。正确表述应为
  「不改同向路径的 `(0,0,0,1)`」；`tests/render_flex_z2vec.rs` 文件头与函数注释已同步更正。
- **O1 镜像 `warp_length3` 非位级 oracle（Arch/QA 采纳）**：镜像用 Rust 左结合求和，`.cu` 用显式
  `fmaf(z,z,fmaf(y,y,x*x))` 收缩链，1 ulp 级差异；测试文件已注明「数值对应、非位级 oracle」，
  该链相关断言只做 1e-6 容差（#1 的 `assert_eq!` 针对常量四元数，不经过该链）。
- **O2 PTX 长度双口径**：§4 表内 348,459 B 为**去尾部 NUL** 口径（产品/测试路径
  `artifact.ptx().len()`）；`nvrtcGetPTXSize` 原始值**含终止 NUL = 348,460 B**（旧 PTX
  348,399 → 348,400 同理）。sha256 按**去 NUL** 字节计：`src/runtime/launcher/artifact/nvrtc.rs:104-105`
  的 `compile_ptx_with_opts(...).map(|ptx| ptx.to_src())` 经 cudarc 0.18.2 `Ptx::to_src()`
  （`src/nvrtc/safe.rs:41-46`，`CStr::from_ptr` 在 NUL 处截止）截断，缓冲区长度来自
  `src/nvrtc/result.rs:114-122` 的 `nvrtcGetPTXSize`（含 NUL）。两口径差 1 B、内容同；身份值以 §4 为准。
- **Backlog（不阻断）**：megakernel 身份已由 `6b1ade37…`/`48f8e7f8…`（348,399 B）变为
  **`cf356b72…`/`b63a51dd…`**（348,459 B）⇒ **后续任何渲染 GPU 窗口都必须重新冻结** guards/制品；
  本片不重冻、不占窗，冻结动作留给下一个渲染窗口执行。
