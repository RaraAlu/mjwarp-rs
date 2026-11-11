# A2-3 验证窗计划（草稿；不排窗、不开窗）

> 状态：**计划稿**，写入 target 工作材料（不入仓）。窗口由 Chief 按四前提开（fresh freeze / monitor 60/60 / 三点基线 / floor 取严），
> 开窗前在「开发团队」群留痕。本文件不等于窗口执行记录，不含任何通过声明。

## 0. 依据与边界

- 用户 2026-11-12 已签：U7/U8 提升为 pass/fail；A2 容量 32/32。签名级。
- 判据：`docs/development-evidence/plan/u7-u8-promotion/u7-u8-criteria.json`（pass_rule：nacon 零容差、结构键配对、
  位级优先 / dist 4 ULP@自身量级、pos 4·ULP(参考三轴最大绝对值)、frame 4 ULP@1.0、11 个零容差字段、
  overflow==0、efc_address==-1、contact.geom 序）；U7 额外强制 dist/pos。
- 范围：`a2-slices.json` A2-3 replay_matrix；**不新增判据/不放宽容差/不改参考**。
- 非声明：不宣称 G10@P5 整族通过、不宣称 U8「已验收」以外的任何族级结论、不宣称与上游一致（over-32）、
  不宣称 mesh 容量缺口已清。
- 前置：A2-1 verified（`2ee0540d`）；A2-2 身份重采 + 全量重冻完成；A2-3 窗为**全新 suite + 全新冻结**，
  旧 guards/旧 exe 一律不复用。

## 1. 窗口内容（1 个 GPU 窗；单窗互斥）

### 1.1 目标 exe（10 个，S4 冻结集同名，新冻）

| # | exe target | 覆盖 |
| --- | --- | --- |
| 1 | `g10_p5_branch_closure_cuda` | S4 branch 9 case + limits U7/U8（改判 pass/fail）+ A2 边界 record-only |
| 2 | `g10_p5_nvrtc_launcher` | launcher 4 case（hfield-sphere/sdf-box/ellipsoid-box/hfield-capsule） |
| 3 | `g10_p5_multicontact_cuda` | multicontact 3 case（cylinder-box/box-box-margin/mesh-mesh） |
| 4 | `g10_p5_primitive_cuda` | primitive 3 case（sphere-box-shallow/deep/capsule-box） |
| 5 | `g10_p5_primitive_plane_cuda` | B1 plane 6 case |
| 6 | `g10_p5_primitive_sphere_capsule_cuda` | B2 4 case |
| 7 | `g10_p5_primitive_branches_cuda` | B3 6 分支 + 4 swap |
| 8 | `g10_p5_mesh_multicontact_m2_cuda` | M2 2 case（mesh-box/mesh-cylinder） |
| 9 | `g10_p5_mesh_multicontact_m3_cuda` | M3 1 case（mesh-mesh） |
| 10 | `stage2_equivalence_window` | stage2 9 synth（record-C，只记录） |

### 1.2 判据矩阵（逐项）

| 项 | case | 判据 | 期望/记录口径 |
| --- | --- | --- | --- |
| M2 | mesh-box、mesh-cylinder（2） | mesh 比较器（nacon 零容差/结构键排序/位级优先或 4·ULP@scale；`contact.geom=[1,0]`） | 保持 M2 已验收：4/4 位级、ULP 0；A2 不得引入回退 |
| M3 | mesh-mesh（1，count=4） | 同上（参考 `96e7895d…`） | bit_equal 4/4、ULP 0；CPU 18 条自检 PASS |
| S4 branch | 9 case（U1/U3×5/U4/U5/U6） | 原 S4 判据：nacon==ref count、failures=0、位级或 ≤4 ULP | 9/9 failures=0 保持 |
| S4 U7 | `mesh-limits-cone-box`（fan-cone-24，24/24） | **pass/fail**：pass_rule 全条件；count=1 且 dist/pos 必须在容差内（防单接触偶合） | 上游 count=1、dist `-1.0000001639127731e-2`；设备需同时满足 dist/pos |
| S4 U8 | `mesh-limits-prism-box`（prism-20，20/3） | **pass/fail**：pass_rule 全条件 | 设备 count **1→4**（提升前 1）、dist `-9.999997913837433e-3`、pos 四条 `(±0.05, ±0.05, 0.045)` |
| A1 回归 | 20-vertex clip（CPU） | 原 A1 断言 | 保持 PASS（同窗 CPU 复跑日志） |
| A2 边界 | A2-1 的 33/32 边界 CPU 测试 | 原 A2-1 断言 | 全 PASS；33 不 panic、host 容量错误可判 |
| over-32 | host 容量门（CPU） | 确定性 `MeshMulticontactCapacity` 错误 | CPU 断言；**设备直调 over-32 行为 record-only、不进分母、不写等价**（是否加设备 over-32 case 待定，见 §5） |
| NP02 | 6 条 collision 回执（CPU-only） | NVRTC 12.8.93、6 入口、file/PTX hash | A2 新 PTX sha `78da9dea…` 入 NP02（回执已产，索引/清单更新见 A2-2） |
| stage2 | 9 synth | record-C | 只记录，不判 fail |

### 1.3 窗口输出（证据）

- U7/U8（及可选 over-32）设备 32 数组（count/dist/pos/frame/geom…）与逐项比对结果；
- `contact.geom` 顺序断言（box 6 < mesh 7 → [1,0]）；
- 不变量：`overflow == 0`、`efc_address == -1`（设备不变量）、guard 无 abort；
- 每实体独立 exit code + stdout/stderr 日志 + 两跑一致性（同窗内 key 输出 sha）。

## 2. runner 纪律（沿用 S4 窗口）

1. **fresh freeze**：全部 10 exe 用 A2 后源码、固定 toolchain（`+1.99.0-x86_64-pc-windows-msvc`）、
   `MJWARP_MUJOCO_ROOT=target/toolchains/mujoco-3.12.0/package`、`--all-features --no-run` 构建；
   复制到 `target/a2-freeze/` 后重哈希，记录 sha256/bytes/mtime；源码 mtime 必须早于构建时间。
2. **零 Rust 编译**：窗内只跑冻结 exe；runner 开跑前检查 `cargo/rustc/cl/nvcc` 不存在（存在即 abort exit 9）。
3. **单窗互斥**：全队同一时间只一个 GPU 满载任务；开窗前群里报窗口。
4. **monitor 60/60**：`nvidia-smi` 100 ms 采样；连续 60 个干净样本；compute-apps 过滤后为 0；
   util/free 阈值取严；任一污染 → abort（不重试、不放宽）。
5. **三点基线**：开窗前 3 点（util/used/free/temp/driver）记录并贴群；开窗后 pre/post 基线复核；
   窗内 0.25 s 采样；结束后无残留进程。
6. **floor 取严**：`free ≥ 10240 MiB` 硬门；`floor = min(10240, baseline_free − 3072)`；取两者更严者。
7. **fail-stop**：任一实体失败/崩溃/超时 → 立即停；不重试、不放宽；保留失败轮证据并报 Chief。
8. **sha 守卫**：10 exe + 15 fixture（A2-2 后 13→15）+ 2 参考；`SHA_OK …` 行写 `sha-guards.txt` 且进 `facts.txt`；
   不一致 → abort。
9. **参考绑定**：`upstream-mesh-limits-gpu.json`（`b6abb8fa…`/4,581 B）与 `upstream-branch-closure-gpu.json`
   pinned raw 未变，只做 sha 绑定；**不重采参考**。
10. **attempt 上限**：≤2 有效轮；abort/污染轮留档不拼。

## 3. 开窗前检查清单（Chief 四前提）

- [ ] A2-1 verified + QA 复核 PASS；
- [ ] A2-2 verified：身份重采（source `fb03c5b1…`/254,434 B、PTX `78da9dea…`/3,883,643 B/6 入口）、
      10 exe 新冻 + sha 表、fixture 13→15、render guards 不受影响声明；
- [ ] 10 exe + 15 fixture + 2 参考 sha 守卫表冻结（新 `guards-a2.json`，不复用旧 guards）；
- [ ] 环境 HEAD 记录 + `src/` 与冻结源码零 diff（`git diff` 为空）；
- [ ] monitor 60/60 PASS、三点基线记录、floor 取严值；
- [ ] 群里留痕（窗口计划 + 基线 + floor）；
- [ ] 失败预案（fail-stop、留档、不重试）。

## 4. 时间/资源预算（基于 S4 实测）

- S4 10 实体墙上 ≈220–300 s；A2 新增 U7/U8 pass 判定逻辑与（可选）over-32 case，预计 ≤360 s。
- 显存：单进程峰值 ~1.5 GiB；基线 free ≈12 GiB → free_min 预计 ≥11 GiB > floor（开窗时按实测重算 floor）。
- 零 Rust 编译；NVRTC 在产品 launcher 内编译是既有路径（与 S4 相同），不是窗口外编译。

## 5. 待 Chief/Arch 裁决的点

1. **over-32 设备 case 是否入窗**：若入窗需 A2-2/A2-3 准备 `fan-cone-33`/`prism-33` 的 XML 模型 +
   MeshData 常量 + LIMIT 记录 case（目前 A2-2 只落 2 个 OBJ fixture 13→15）；否则 over-32 只由 host 容量门 CPU 覆盖。
2. U7 若设备返回非单接触或 dist/pos 超容差 → 按 pass/fail 判红并 fail-stop（不放宽、不归因偶合）。
3. NP02 回执索引/清单更新是否在 A2-2 落仓（现计划只产新回执 + 索引追加；inventory 重算归 NP5-03 面）。
