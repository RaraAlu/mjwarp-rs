# T4源码身份重钉（G03矩阵链对齐后）

任务：`2026-10-22-t4-source-hash-repin-9`（前身 -6→-8 因范围不足依次放弃，本任务一次登记全）。
依赖：`2026-10-22-g03-matrix-chain-alignment`（提交 `d5b02d5`）。
起因：`d5b02d5` 改动 `src/physics/mod.rs`、`mass_matrix.rs`、`mass_solve/cuda.rs` 后，T4固定源码身份（PHYSICS锚点组合、CUDA模板摘要、precision夹具、gravity内联contract、rigid重采集清单、L1摘要表、frame393诊断轨迹）全部过期。

## 结论

全量非ignored测试恢复：`cargo +1.99.0-x86_64-pc-windows-msvc test --locked --all-features --no-fail-fast` = **709 passed / 0 failed / 346 ignored，78/78目标ok**。
全部只改哈希数值、期望字面量与诚实的谱系逆变换；断言、容差、映射条目语义、W-D5排除清单未改。

## 重钉清单

| 对象 | 重钉内容 |
| --- | --- |
| `src/physics/mod.rs` 组合摘要 | `ecc55abc…`→`9d10f056…`（PHYSICS锚点组合） |
| `fixtures/precision/t4-current-kernels.json` | 10旋转变体 source+PTX+receipt；rigid_mass_matrix/rigid_mass_solve 两个保留变体 source+PTX+receipt；product_sources mod.rs 同步 |
| `docs/evidence/t4-g03-retest/rigid-rotate-recapture/previous-kernels.json` | 10旋转变体 source=digest(before_rotate(当前源码))；两保留变体行与current一致 |
| 同上 `source-identity.json` | rigid_mass_matrix/rigid_mass_solve 两行 source+PTX 同步 |
| `docs/development-evidence/toolchain-evidence-revalidation/gpu-candidate-digests.txt` | rigid_kinematics / rigid_mass_matrix / rigid_mass_solve 三行 source+PTX（NVRTC 12.8实编译：4b5e948b… / 53c2b6dc… / 9ca25bc7…） |
| `docs/evidence/t4-g03-retest/{camlight,tendon-wake}-internal/map.json` | mod.rs组合摘要 |
| `fixtures/precision/g01-{field,rigid,state-packing,topology-frames}.json` | mod.rs组合摘要；g01-rigid 两处映射锚点 `matrix`→`matrix_natural` |
| `fixtures/gravity-upstream-abi/nonzero-product-chain-package.json` | physics_core=`9d10f056…`；整包SHA256=`39426708…`（2223B不变） |
| `tests/t4_gravity_nonzero_product_chain.rs` | `PACKAGE_SHA256` 与内联 physics_core 字面量 |
| `src/physics/resident/rigid_kinematics_tests.rs` | PLAIN/MOCAP/G01 三个冻结PTX常量：`4b5e948b…`/`0bbc1f9a…`/`442bcf33…`（NVRTC实编译，先用旧模板复算命中历史值） |
| `fixtures/precision/t4-kernels.json` | **未改**：保持历史捕获 `cd781388` 与历史 .cu/receipt 自洽 |
| `tests/support/t4_rigid_internal/{matrix_lineage.rs,matrix-repaired.txt,matrix-historical.txt}` | 新增矩阵修复逆变换：把当前KINEMATICS中G03 matrix片段整体还原为历史写法，使 `preserves_zero_quaternion…` 的 `pre_zero` 诚实重建回历史捕获 `cd781388`（不改BASE夹具、不伪造捕获） |
| `fixtures/mjwarp-geom-tendon/trace393.json` + `tests/frame393_trace.rs` | 诊断轨迹按对齐后产品重钉：`inputs[15]` 由 `3195361247` 改为 `3195361249`（2 ULP），同步 inputs_sha256 与整文件 sha；该项为诊断探针（scope 标注 no tolerance acceptance），无容差准入 |

## 失败目标转绿

- lib 重采集2项：PRE_ROTATE 10行source + current 12行source/ptx/receipt。
- launcher_cache L1：三行 source+PTX。
- t4_camlight_precision / t4_tendon_wake_precision：map mod.rs组合摘要。
- t4_gravity_nonzero_product_chain contract：包 physics_core、整包SHA256、测试常量与内联字面量。
- t4_input_precision 5项：4个g01夹具mod.rs摘要、g01-rigid映射锚点、t4_rigid_internal.rs OUTPUTS、pre_rotation摘要`714839bf…`、新增matrix逆变换使pre_zero回到`cd781388`。
- frame393诊断轨迹。

## 复验

| 编号 | 内容 | 结果 |
| --- | --- | --- |
| run-01 | 全量非ignored `--all-features --no-fail-fast` | 709 passed / 0 failed / 346 ignored，78目标ok（`target/host-repin9-final.out`） |
| run-02 | `t4_input_precision` | 36 passed / 0 failed / 12 ignored |
| run-03 | `launcher_cache`、`t4_camlight_precision`、`t4_gravity_nonzero_product_chain`、`t4_tendon_wake_precision` | 全绿 |
| run-04 | lib recapture 两项 | 全绿 |
| run-05 | `frame393_trace --ignored` | 1 passed / 0 failed |

## all-probes环境依赖枚举（非源码漂移，另立接线任务）

all-probes等价命令枚举25项失败，逐条分类：源码漂移0项，全部为env/外部依赖类——
`MJWARP_FROZEN_UPSTREAM_ROOT` 8项、报告/证据目录env 6项、native测试DLL/环境10项、RNE数值探针1项（`2026-10-22-g02-rne-acc-native-tolerance-debt` 跟踪）。
G01 debug在all-probes前各阶段已全绿：fmt 0警告、native 27、host-msvc 709、g01 7、native-g01 2、native-only-g01 1、resident 102。

## 未验收边界

- all-probes env接线完成前，不宣称G01双模式全绿。
- Linux、release、性能与完整上游等价未验收。
- 未改断言/容差/排除清单；诊断类失败归因外部产物/环境。
