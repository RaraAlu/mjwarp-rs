# U038-3D 已知债务登记（用户裁决 B）

- 日期：2026-10-23。
- 任务：`2026-10-23-u038-3d-known-deviation`。
- 裁决：用户 2026-10-23 裁决 **B**——U038-3D 三维链的全链 `qfrc_gravcomp` **8/77 超差**
  登记为**已知债务**，**不宣称通过**（不宣称三维链验收、不宣称精度达标、不宣称完整验收）。
- 债务跟踪任务：`2026-10-23-u038-3d-gravity-propagation-debt-2`（2026-10-24 启动；已完成有界
  判定：**不改容差/断言/fixtures 语义且最小改产品源码时不可达 1477 项 0 超差**，
  复跑 122 位差 / 8 超差 / max 4.29e-6 不变，见 `debt-2/README.md`）。
  验收条件不变（1477 项 0 超差、2e-6 容差/断言不变、覆盖 free+ball+hinge+slide 混合关节）。
  旧任务 `2026-10-23-u038-3d-gravity-propagation-debt` 在有界修复尝试后已 **abandon**；
  引擎不允许对 abandoned 任务追加修正记录（`normify fix` 拒绝，直接改 note 被迁移校验拒绝），
  故 B 裁决、PTX 证据路径与后继任务指引改由本文件与后继 proposed 任务承接。

## 偏差数据（run5 `run.json` 为准）

| 项目 | 数值 |
| --- | --- |
| 比较项总数 | 1477 |
| `qfrc_gravcomp` 超差 | **8/77** |
| 最大绝对偏差 | 4.291534423828125e-6 |
| 冻结容差 | 2e-6（未放宽） |
| 总位差 | 122 |
| 运动学八字段 | 1400 项 0 超差（max 1.79e-7） |
| `gravity_operator`（同输入算子） | 77/77 0 超差（位差 0~1） |

8 项超差的 world/dof/产品值/上游值/绝对差逐项清单见 `closure.md`「超差清单」。
历史口径说明：早期 70/77、max 80.93、161.87、277 位差等数字是参考矩阵修复前
（提交 `89e9910` 之前）的状态；当前以 `run5/run.json` 为准（8/77、max 4.29e-6、位差 122）。

## 根因（U038 PTX 逐表达式对照结论）

证据：`docs/development-evidence/u038-3d/ptx-diff/`（工具提交 `71d9d42`、修正 `722e979`）。

1. **自然校验**：本机 NVRTC 12.8.93 按 Warp release 选项重编译 pinned 生成 CUDA，
   与冻结 12.9.86 PTX **48/48 个入口指令流完全一致**（含 lineinfo 变体）→ 对照可本机复现。
2. **源形态无差异**：同编译器、同选项、同形状包装下，`rotate`/`multiply`/`normalize`
   三表达式的产品源码与上游生成源码指令计数、fma 融合数完全一致（`rotate` 仅一条非融合
   `add` 的调度位置不同）→ 不存在"表达式写错或少融合"。
3. **收缩由内联上下文决定**：完整内核内探针命中 `rotate` 产品 3/5、上游 q-first 0/5、
   v-first 3/5，`multiply` 1/3 vs 2/3，`normalize` 3/3；同一上游 `rotate` 源码只换包装
   加载顺序即 13 fma ↔ 10 fma，而产品源码与"照上游语句重写"的候选转录恒为 13 fma（不敏感），
   候选模式在冻结上游内核 0/5 命中 → **源级转录无法复现上游内核收缩**。
   这解释此前两轮有界失败：第 1 轮改源形态无变化；第 2 轮强制 0 fma 与上游混合收缩
   （`_kinematics_branch` 99 fma / 284 条 f32 算术）方向相反，位差变差。
4. **com/cdof 结构性差异**：上游拆 4 个独立内核（init / acc / div / cdof），`subtree_com`
   累加使用 `atom.global.add.f32`（6 条）按树层进行；产品为单内核顺序反向累加（0 原子），
   属内核分解/浮点加法顺序差异，无法用 FMA 转录消除。

## 复现入口

PTX 逐表达式对照（只读、只编译不跑内核）：

```powershell
pwsh -NoProfile -File scripts/u038-ptx-diff.ps1
```

前置：NVRTC 12.8 工具链（`target/toolchains/cuda-12.8.1/bin`）、pinned Warp 缓存、
Warp wheel native 头文件（详 `ptx-diff/README.md`）。完整数据在
`docs/development-evidence/u038-3d/ptx-diff/`（`ptx-diff.json`、产品 PTX 摘录、探针源码）。

数值验收复跑入口（容差/断言不变，定向 GPU 测试）：

```powershell
$env:PATH = "target\toolchains\cuda-12.8.1\bin;target\toolchains\mujoco-3.12.0\package\bin;$env:PATH"
$env:MJWARP_MUJOCO_ROOT = "target\toolchains\mujoco-3.12.0\package"
$env:MJWARP_GRAVITY_3D_EVIDENCE_DIR = "<新证据目录>"
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --test t4_gravity_3d_product_chain -- --ignored --test-threads=1
```

## 不宣称与边界

- 未宣称 U038-3D 通过；8/77 超差如实保留为已知债务。
- 未改产品源码、测试、容差、断言、fixtures、排除清单或 ABI；未做 T4 repin。
- 未验收边界：完整三维状态空间、NP03/NP04、release/Linux、性能、满额回归、完整上游等价。
- W13（nvJitLink）、Linux、`g03-parity` 保持不动。
