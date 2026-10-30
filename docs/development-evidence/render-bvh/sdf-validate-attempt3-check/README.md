# U4 attempt3（INVALID_ROUND）+ 球面 fix 回滚 — 独立复核

任务：`2026-10-31-render-sdf-validate-attempt3-check`（QA；只读）
被核：U4 窗口 attempt3（`2026-10-31-render-sdf-validate`）、球面 fix 回滚（`2026-10-31-render-sdf-sphere-dot-a-order-revert`，`ee1bea57`）、
fix 片修正/单数据集 addendum（`c7172e37` / `7f9da6b2`）、attempt 证据 4 处修正 + attempt3 入库（`1ccb3446`）
pin：`1ccb3446`；start 时 HEAD（`revision.before`）：`694bb5aa33319980459eedb25cecba026f914219`
复核时间：2026-10-31 03:3x–04:0x（+08:00），空闲段；**零 GPU、未重跑窗口、未重冻结、未编译**

## 1. 结论

**pass / 阻断 0 / 需修 5 / 观察 7**

六个复核点全部成立：
1. **两个结论拆分正确**：①流程无效——attempt3 在 U1b 仍回滚态时开跑，`render_sdf_reference` 的 5 个 GPU 用例全部 panic
   `Unsupported("S7b3 不含 SDF 求交")`（`sdf_scene.rs:267`），记录明确「本 attempt 的 SDF 侧不作为任何判据依据」；
   ②真实参考冲突——另 6 个 scene-a 回归红在 scene-a cam1 球面轮廓（首差 28969/(169,120)，`max_ulp=73`>4，fix 前位级 0）。
   我按 committed 日志逐条核过；**未把成因写成定论**（成因指向 target 只读 RCA）。
2. **attempt3 冻结身份**：`8b34570e…`/33,651 B/aggregate `013babf4…`（我用 Windows PowerShell 5.1 从 committed blobs 独立重算，全等）；
   `9 exe + 83 inputs = 92`（10/10/47/16）；与 attempt2 比 **9/9 exe 全变、83/83 inputs 不变**；stash 9/9；`generated_at 03:31:17 < window_start 03:31:40`；
   **作废、不复用** ✓。
3. **之前 4 处证据修正**：`attempt1 util 3–76` ✓、`attempt1 例外 69 样本` ✓、§3b 五要素明文块 ✓、attempt2 `failed_tests` 走
   **errata 不改字节**（attempt1/2 归档 blob 与 d52dc774 逐字节相同）✓。
4. **fix 片 addendum**（单数据集范围）已入 fix 任务 addenda；3 处文档修正（6³=216×2=432、gate 快照来源措辞、2 个并列 winner）已落地；
   回滚关系在三处留痕（fix 卡 addendum 引用 revert、revert 证据引用 fix、窗口 README §3c）。
5. **回滚正确**：`render_math.cu` blob `7a43e575…` → **`33efcb27… == gen1**；我用 `MEGAKERNEL_SOURCE` 6 段 `include_str!` 在 LF 工作树**独立重算** source = `778216b2…`/88,865 B（= gen1）；
   只改该 `.cu` + 新证据 + 卡片/模块记录；unit 1111/0/397、`--no-run` EXIT=0、validate 0 err。
6. **判据/参考零改动、U4 仍未通过**：`MAX_DEPTH_ULP=4` 未动；`2162044b..694bb5aa` 的 `tests/ fixtures/ scripts/ Cargo*` 零改动（src 只有回滚那一处）；窗口卡仍 `in_progress`，无任何「通过」声明。

需修 5 条（全部文档/证据级，不重跑、不改判据）：
- **P1** 窗口 README 仍按「两次 attempt」写：标题/第 3–4 行、§2 表（无 attempt3 行）、"两次 approach"、文末"后续 → U4 attempt=3（已发生并失败）"、§4 guards-only 无 attempt3。
- **P2** attempts-evidence `receipt.json` 仍是 attempt1/2 口径：`total_artifacts=10`、`target_copies {attempt1,attempt2}`、`window_state "attempt1/attempt2 …"`；实际目录已 3 attempts / 35 条 manifest / 36 文件。
- **P3** fix 与 revert 两个证据目录的 `manifest.txt` 各 4 条与 committed blob 不符（CRLF 工作树 vs LF blob）：fix `kernel-identity.txt` 1152/`303cc894…` vs 1141/`10d7fadd…`、`no-run.txt` 382/`4eb631af…` vs 376/`24bd8851…`、`src-diff.patch` 3908/`fe850614…` vs 3857/`9411be0d…`、`validate.txt` 520/`2768a2a4…` vs 513/`61ee58e4…`；revert 同名文件 4 条同样问题。修法：manifest 从 committed blob 重算 + 加 `.gitattributes`（`docs/development-evidence/render-bvh/** -text -diff`，窗口目录已有此规则故其 manifest 干净）。
- **P4** 内核 `.cu` 的 EOL 未钉：`git check-attr` 对 `src/render/kernel/cuda/render_math.cu`、`src/render/bvh/cuda/scene.cu` 返回 `text/eol unspecified`，
   而 `MEGAKERNEL_SOURCE` 是 `include_str!` ⇒ **source 身份随 checkout EOL 变**（revert 日志自述 CRLF checkout 得 `86a55721…`；我把 6 文件全转 CRLF 模拟得 90,908 B / `e5719c81…`，方向一致）。
   建议 `.gitattributes` 加 `src/render/kernel/cuda/*.cu text eol=lf`、`src/render/bvh/cuda/*.cu text eol=lf`（或 repo-local `core.autocrlf=false`），否则 fresh clone 复现不出 gen1/gen2 身份。
- **P5** attempt3 记录/README §3c 的「6 实体首差 28969、max_ulp=73」应为：共享球面轮廓集（e2e/rgb/depth_seg/skybox/r10 + flags 的 baseline/culling 臂）首差 28969/73；
  但 `render_kernel_flags` 的 **aa2 臂**首差是 **29448/(168,122)、max_ulp=91**（仍 ≫4，结论不变）。建议补限定语。

观察（记录级）：O1 fix/revert 的 `no-run.txt` 是截断转写（非原文）；O2 validate 快照时点差异（现跑的多出 `mjwarp.physics.collision` fingerprint drift，
源于复核时工作树里**与本片无关的**`src/physics/collision/**` 在途改动）；O3 RCA 只在 target 未入库、committed 文档正确地标「成因待 RCA」，
我抽核了其机器证据（两份 megakernel PTX 身份与各自 capture cache 全等；`_ray_quad_0` 生成源码 sha 相同；recipe 环境相同；逐像素表与日志一致），
建议用户拍板后把 RCA 摘要+决策一起入库；O4 attempt3 冻结完整性（stash 9/9、front-freeze 时间线）成立；
O5 errata 方案有效（归档不改字节、更正另置）；O6 窗口 manifest 35 条/37 行/36 文件全对；O7 attempt3 例外 monitor 真 load-bearing（ge40_clean=6、ge60=0）、
常规失败件保留，body util_max 28、零编译、floor 未破——两个 INVALID_ROUND 结论不是环境假象。

## 2. 边界 / 不宣称

只读：不跑 GPU、不开窗、不重跑、不重冻、不编译；不改任何被核文件。
不宣称渲染数值验收通过 / 上游等价 / 精度达标；**U4 仍未通过**；不宣称参考冲突成因已定论（只记录 RCA 已定论部分与分级）。

## 3. 复现命令（只读）

```
# attempt3 冻结身份（committed blob）
git cat-file blob HEAD:docs/development-evidence/render-bvh/sdf-validate-window/attempts/attempt3/guards-sdf-validate.json > t.json   # 33,651 B
# aggregate：Windows PowerShell 5.1 按 path 排序拼接 92 条 sha 后 sha256 => 013babf4..
# 回滚身份
git ls-tree 1ccb3446 -- src/render/kernel/cuda/render_math.cu        # 33efcb27..（= gen1）
# 内核 source 身份（LF 字节拼接 6 个 include_str! 文件）=> 778216b2../88,865 B
git check-attr text eol -- src/render/kernel/cuda/render_math.cu     # unspecified/unspecified（P4）
# manifest 对 committed blob 重算：fix/revert 目录各 4 条不符（P3）
```

`materials/selfcheck-*.txt` 是本次重算的原始输出（attempt3 / manifests / eol / rca / corrections）。
