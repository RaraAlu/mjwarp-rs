# 渲染/BVH S2a 独立复核（主机侧 primitive AABB、场景 BVH 构建/重拟合、分组与拒绝面）

- 任务：`2026-10-24-render-bvh-s2a-independent-check`（owner QA）
- 日期：2026-10-24
- 复核对象：`f9adf14`（`功能(渲染): 落地S2a主机侧场景BVH与primitive AABB位等`；任务 `2026-10-24-render-bvh-s2a`，注册 `2a62e2b`）
- 复核 HEAD：`0b63829`（本轮期间 S2b 的实现仍以未提交 WIP 存在，仅触碰 S2b 自有路径；`git diff f9adf14..HEAD -- src tests fixtures scripts` 为空）
- 口径：快验独立复核，**CPU/离线、无 GPU**，未跑全量 G01；只对已覆盖的主机语义下结论。
- 结论：**5/5 项成立**，未发现阻断问题；口径建议见 §7。

## 1. AABB 位等（成立）

**独立重算**（不读交付 aabb-parity 数字）：QA 用 Python 按上游
`mujoco_warp/_src/bvh.py` 的 `_compute_{box,sphere,capsule,plane,ellipsoid,cylinder}_bounds`
公式重新实现（7 类 leaf），显式 f32 逐步运算并复刻文档化的 NVRTC 收缩顺序
（3 项和 `fma(c2,b2,fma(c0,b0,round(c1*b1)))`、2 项和 `fma(c0,b0,round(c1*b1))`、
`pos ± z*half` 的 `fma(±z,half,pos)`），输入取 fixture 记录的 f32 位姿位型，输出与
`bounds.json` 的 lower/upper **逐位比较**：

| 项 | 结果 |
| --- | --- |
| 位姿数 × leaf 数 | 8 × 7 = **56** |
| 比较的 f32 分量 | 336（56×3×2） |
| 位差 | **0** |
| max ULP | **0** |
| 模型侧输入 | `geom_size` 由 model-312 f64→f32 **0 差**；mesh bounds 由 `mesh_vert` 重算 **0 差** |

实现细节：宿主 fma 口径（`(a as f64 * b as f64 + c as f64) as f32`）与交付 README 一致；
QA 复核 `src/render/bvh/aabb.rs` 确认未调用 `f32::mul_add`（仅注释/测试名出现），
与「debug 回退不保证单次舍入」的说明相符。证据 `aabb-recompute.json`。

## 2. 构建 / refit 语义不变量（成立）

**独立 Rust harness**（不导入交付测试）：用 QA 自己的 `qa-s2a-harness`，把该仓按 `f9adf14` 的
干净 worktree 作为路径依赖，从 fixture AABB 构建场景 BVH，独立断言：

- 每个 leaf 的区间等于 fixture；group root 包围盒 = 组内 leaf 并集；
- 每个内部节点包围盒 = 两个子节点包围盒的并集；从 group root 遍历每个 leaf **恰好可达一次**；
- 依次以 8 个位姿 refit（初始 + moved-base + 6 个校准），每次 refit 后上述不变量全部成立；
- 初始 vs moved 的 leaf 变化集合 = **`[1,2,3,4,5,6]`**（floor 静态）。

结果：`poses=8 errors=0 changed_leaves=[1,2,3,4,5,6]`（`bvh-semantics.json`、`bvh-harness-result.txt`）。

## 3. 双 world 分组与 32×2 查询（成立）

同一 harness 构造两个 world 的复制场景（14 leaf、groups=0×7/1×7），用与交付相同的 32 条射线
逐 world 查询（语义：候选 leaf 的 group 必须等于查询 world，两 world 的候选 geom 集合一致）：

| 项 | 结果 |
| --- | --- |
| 32 射线 × 2 world | **64/64 命中**（每次查询候选非空） |
| 跨 world 候选 / 集合不一致 | **0** |
| QA 追加的 24 条自有射线 × 2 world | **48/48 命中**，跨 world 0 |
| 8 条向上（远离场景）射线 × 2 world | 候选 **0** |

证据同 `bvh-semantics.json`。

## 4. 拒绝面（成立）

独立 harness 对 14 个具体拒绝调用逐一断言必须 `Err` 且全部成立：
空场景、bounds/groups 长度不符、groups/geom_ids 长度不符、world 数 0、group 越界、空 world 分组、
构建期非法 AABB（lower>upper）、refit 长度不符、refit 非法 AABB、hfield 等不支持 geom 类型、
非有限位姿、空 mesh bounds、`Aabb::new` 非有限、查询 world 越界（`WorldOutOfRange`）。
计数：**14/14 rejected**。

口径说明：任务验收写「13 项拒绝面」，交付 README 归纳为 10 类、交付 host 测试断言 11 个具体调用；
本次独立枚举了 14 个具体调用（覆盖全部 13 项要求行为并多出「groups/geom_ids 长度不符」与
「Aabb::new 非有限」两个等价入口），全部被拒绝。建议后续把「13」对应的具体清单固定下来（§7）。

## 5. 证据、测试与哈希（成立）

- 交付回执 12 个 artifact：**12/12 sha256、12/12 字节、12/12 行数与 `f9adf14` 的 blob 一致**
  （`artifact-hashes.json`）。
- 交付测试（在 `f9adf14` 干净 worktree 重跑）：定向 `tests/render_scene_bvh_host` **5 passed / 0 failed**；
  模块 `--lib render::bvh` **16 passed / 0 failed**；全量 `cargo test --locked` 合计
  **506 passed / 0 failed / 0 ignored**（与交付 gate 记录一致，`delivered-tests.json`）。
- 交叉校验：`s2/bvh_initial.npz`、`s2/bvh_moved.npz` 的 lower/upper 位型与 S2a fixture
  `initial`/`moved-base` **4/4 位等**（`s2-cross-check.json`）。

## 6. 边界与并发（成立）

- **无 GPU**：全部复核为 CPU（Python 重算 + 纯 Rust harness + 原生测试）；未启用 cuda-probe，
  未运行 GPU 命令（复核期间 `nvidia-smi` 为 3,424 MiB 桌面基线）。
- **产品测试不启动 Python**：S2a 的源码/测试是纯 Rust，`python`/`GPU` 只出现在注释与 authoring
  脚本；QA harness 也不调 Python。
- **行数**：`src/render/bvh/aabb.rs` 400、`builder.rs` 440、`refit.rs` 123、`query.rs` 111、
  `mod.rs` 184、`src/render/mod.rs` 242、`tests/render_scene_bvh_host.rs` 472
  ——新建代码文件均 ≤500，blob 行数与交付记录一致（`boundary-and-concurrency.json`）。
- **模块源码登记**：`mjwarp.render` 模块列 `src/render/mod.rs` 与 `bvh/{mod,aabb,builder,refit,query}.rs`。
- **并发**：复核期间 render/BVH S2b 实现落库（`c7b5d7f`）。`git diff f9adf14..HEAD -- src tests fixtures scripts`
  的已提交变化共 7 个路径，**全部属 S2b 自有范围**：`src/render/bvh/cuda/{mod.rs,scene.cu}`、
  `src/render/bvh/mod.rs`（cuda 挂载）、`tests/render_scene_bvh_gpu.rs`、`tests/support/render_bvh/mod.rs`、
  `fixtures/render-bvh/scene-a/s2/reference_hits.json`、`capture_s2b_query.py`；
  **S2a 的 unchanged 清单文件（aabb/builder/refit/query.rs、host 测试、s2a fixture、capture/verify 脚本、
  s2a 交付证据）均未被触碰**。本轮 receipt 记录该并发状态。

## 7. 观察项 / 建议项（不影响结论）

1. 「13 项拒绝面」的计数口径不统一（任务 13、README 10 类、测试 11 个断言、本复核 14 个具体调用）；
   建议在 S2 收口时固定成一份带名字的拒绝清单。
2. `src/render/bvh/mod.rs` 在本轮期间被 S2b WIP 修改（加 `pub mod cuda`）；这不在 S2a 复核的
   unchanged 清单内，但后续 S2a/S2b 交叉阶段要确保 mod.rs 的挂载语义有对应验证。
3. 交付未覆盖「旋转 plane」的 GPU 位型参考（scene-a floor 静态），已在交付 README §9 登记。

## 8. 未满足项（沿用并如实）

- 无 GPU 遍历/命中：不宣称 S2 六条全部通过（①③ 的射线层属 S2b）。
- mesh 三角形 BVH 与 mesh 命中语义属 S3；S2a 只到包围盒。
- 旋转 plane AABB 无 GPU 位型参考；host fma 为 fixture 验证的 f64 复刻，任意输入可能与 CUDA
  `fma.rn` 差 ≤1 ULP；AABB 参考输入是设备侧 f32 位姿。
- 未做 fuzz/任意 MJB 性质测试；未覆盖 hfield 完整语义。
- 本轮无全量 G01、无产品渲染验收、不宣称 S2b 已实现。

## 复现命令

```powershell
$py = 'target\mjwarp-parallel\venv\Scripts\python.exe'
# 独立 AABB 重算
& $py -B target\qa-render-bvh-s2a\aabb_recompute.py
# 独立 Rust harness（干净 worktree 作为依赖）
cd target\qa-s2a-harness
$env:CARGO_TARGET_DIR = (Join-Path (Get-Location).Path 'target')
cargo +1.99.0-x86_64-pc-windows-msvc run --quiet
# 交付测试（干净 worktree）
cd ..\qa-s2a-head
$env:CARGO_TARGET_DIR = (Join-Path (Get-Location).Path '..\qa-s2a-head-target')
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --test render_scene_bvh_host
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --lib render::bvh
cargo +1.99.0-x86_64-pc-windows-msvc test --locked
# 证据汇总
cd ..\..
& $py -B target\qa-render-bvh-s2a\evidence_gen.py
```

## 证据索引

- `aabb-recompute.json`：56/56 位等、模型输入位等
- `bvh-semantics.json` / `bvh-harness-result.txt`：构建/refit 不变量、双 world 查询、14 项拒绝
- `delivered-tests.json`：5 / 16 / 506 测试
- `s2-cross-check.json`：与 S2 npz 的 4/4 位等
- `artifact-hashes.json`：12/12 哈希、字节、行数
- `boundary-and-concurrency.json`：行数、无 GPU/Python、模块登记、并发 diff
- `claim-mapping.json`：交付声明 ↔ 独立证据映射
- `tools/`：QA 独立脚本与 harness 源码
