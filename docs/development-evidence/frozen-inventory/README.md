# 精度冻结清单盘点（NP02 证据基线）

任务：`2026-10-23-frozen-inventory`。对象：`docs/numerical-precision-contract.md` 的 NP02 要求——「记录冻结提交与具体符号，也记录生成内核与编译选项」。
本目录只做**只读盘点与证据登记**：不改产品代码、测试、断言、容差与 fixtures，不跑 GPU。

## 结论

清单已建立，可核对部分已逐项核对；**NP02 未通过**：上游生成内核与编译器库覆盖仍不完整、历史捕获实体不全在仓、产品源身份相对冻结捕获已漂移。

- 上游冻结提交 `71da24d956378a87a703b6e1442b13aec0c4ac29`（Warp 1.15.0）：`fixtures/precision/t4-kernels.json` 记录的 7 个上游源文件与 pinned checkout（`target/mjwarp-parallel/upstream`）**7/7 哈希一致**。
- T4 冻结清单 24 条目 / 30 变体：22 个变体可按 `ptx_sha256` 在仓内定位到内核回执，19 个回执文件哈希与冻结记录的 `receipt_sha256` **逐字节一致**；13 个变体在中基线 `8a5bac00` 有重采集哈希。
- 仓内（`docs/` 与 `fixtures/`）共 688 份内核回执、79 个 entry、98 个不同 PTX：**编译选项全部一致**（见下），无选项变体。
- 产品源 20 个：当前 HEAD 有 **10 个**与冻结捕获身份（`4dc88934`）不一致；9 个在中基线 `8a5bac00` 之后仍有变化。旧 30 份集合不能直接证明当前源码。
- NP01 的 30 条产品路径：16 条有 T4 冻结条目、24 条能在仓内定位内核回执、3 条有 fixtures 冻结证据链；**3 条无任何内核证据**（`rne_acc`、`tendon_wrap`、`field_batch`）。

## 编译选项（实测，非估计）

仓内 688 份回执逐份解析，字段完全一致：

```json
{
  "entry": "<内核符号>",
  "fmad": true,
  "useFastMath": false,
  "nvrtcVersion": [12, 8],
  "options": ["--std=c++17", "--gpu-architecture=compute_70"],
  "precisionGate": "f32-physics",
  "ptxSha256": "<生成 PTX 哈希>",
  "sourceSha256": "<捕获时的 CUDA 源哈希>"
}
```

与产品配方一致：`src/runtime/launcher/recipe.rs` 的 `CompileRecipe::WARP_1_15_PHYSICS`（`fmad=true`、`fast_math=false`、`--std=c++17`、`--gpu-architecture=compute_70`、`defines=[]`、`PrecisionGate::F32Physics`），捕获路径为 `src/runtime/transfer/kernel.rs` 的 `check_physics_ptx`。

fixtures 冻结证据链是**离线 authoring 产物**，编译身份与产品 nvrtc 不同，已在 `inventory.json` 的 `compileOptions.frozenChainAuthoring` 单列：`target_arch=89`（RTX 4070 Ti SUPER）、`mode=release`、`fast_math=false`、`fuse_fp=true`、CUDA toolkit 12.9、PTX 编译器 12.9.86；产品直接加载其 PTX（前向兼容），不重编译。

## 冻结证据链（`fixtures/gravity-upstream-abi/`）

| 链 | 生成内核条目 | PTX 实体 | 校验 |
| --- | --- | --- | --- |
| `independent-kinematics`（smooth 模块） | 7 个 `*_cuda_kernel_forward`（kinematics branch、body matrices、inertial frames、subtree_com init/acc/div、cdof） | `smooth.ptx` | 实体 SHA = receipt `ptx_sha256`，一致 |
| `generated`（passive 模块） | `_gravity_force_7c63ac28_cuda_kernel_forward` | `module.ptx` | 一致 |
| `three-d-kinematics` | 复用 independent-kinematics 的 7 条目 | 引用 `../independent-kinematics/smooth.ptx` | 引用 SHA 一致 |

## 缺口（按 NP02 分类，逐条记录在 `inventory.json` 的 `gaps`）

1. **上游生成内核覆盖（待补）**：24 条目只覆盖 `smooth`/`passive`/`sleep`/`math`/`block_cholesky`/`types`/`util_misc` 的显式符号；上游 Warp 生成内核（codegen/builtin 头文件级）无逐符号清单。W15 头文件已落地（`b1de2db`，`include/warp-headers/`），可作为上游源码路径对照，但本任务未做逐头文件清单（不扩大范围）。
2. **编译器库覆盖（待补）**：回执只证明 nvrtc 12.8 + 统一选项 + f32 PTX 门禁；Warp 编译器库内部精度边界无逐项证据。
3. **历史捕获实体（部分在仓）**：22/30 变体可按 PTX 哈希定位回执，19/30 回执逐字节一致；其余依赖历史 `target/t4-g03-retest` 产物。对应 6 项 audit 已按用户裁决从 all-probes 显式排除（`docs/three-layer-acceptance.md` §5.2）。
4. **产品源身份漂移（已量化）**：HEAD 漂移 10/20，列出：`src/physics/attached.rs`、`com_velocity.rs`、`flex_face.rs`、`mass_matrix.rs`、`mass_solve.rs`、`mod.rs`、`resident.rs`、`rne_bias.rs`、`src/runtime/transfer.rs`、`src/runtime/transfer/kernel.rs`。重采集/重基线需另行裁决登记，本任务不改。
5. **NP01 无内核证据路径**：`rne_acc`、`tendon_wrap`、`field_batch`；另有 10 条 NP01「待补」路径虽有回执/条目，仍缺逐字段类型表（见 NP01 清单）。
6. **排除条目**：`adapter_probe`、`fma_probe`、`hessian_probe` 不在 T4 分母；排除理由沿用 `fixtures/precision/t4-kernels.json` 的 `remaining` 与 `fixtures/precision/audit.json`。

## 复现

脚本按自身位置定位仓库根，可从仓库根目录运行（只读）：

```powershell
node docs/development-evidence/frozen-inventory/collect.mjs
```

脚本读取：`fixtures/precision/t4-kernels.json`、`fixtures/precision/t4-current-kernels.json`、`docs/development-evidence/np01-type-mapping/inventory.json`、`fixtures/gravity-upstream-abi/*/receipt.json`、`src/runtime/launcher/recipe.rs`、`src/runtime/transfer/kernel.rs`、pinned 上游 checkout，以及 `docs/`、`fixtures/` 下全部 `ptxSha256` 内核回执；输出 `inventory.json`（主清单）与 `receipts.json`（回执目录）。

## 边界

- 本清单**不构成 NP02 通过**：覆盖率不是 100%，缺口仍存在；也不代表 NP01/NP03/NP04 或完整 T4 验收。
- 所有数字均来自脚本实测/已提交回执；未验证项明确标为待补/外部条件，不写成通过。
- 未改 `fixtures/`、产品源码、测试与容差；未跑 GPU；未引用 Python/venv 产物。
