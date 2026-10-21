# W15 内核来源与头文件评审

任务：`2026-10-18-layers-m7-headers`。
日期：2026-10-21。
依据：`docs/architecture.md`、`docs/unified-launcher.md`、`docs/module-contracts.md`。
本轮只评审，不实现。
本评审不修改产品内核与浮点顺序。
获准路线须另立交付任务。

## 结论

产品内核来源采用**上游源码与头文件原样复制**路线。
手写移植与冻结PTX保留为既有来源，不删除、不改写。
W15由“待决”转为“需要”，但本轮不落地。
原样复制只指上游文件字节，不改写、不转译。
复制来源固定Warp `v1.15.0`，提交`ffa038b1adf3927a8e893d4536f1a7562f17d749`。
许可证为Apache-2.0，与仓内`LICENSES/Apache-2.0.txt`同源。
复制路线仍受配方、摘要、f32门禁与许可约束。
评审通过不等于产品精度验收。

## 现状两条来源

| 来源 | 位置 | 证据 | 现状 |
| --- | --- | --- | --- |
| 手写移植 | `src/physics/`内嵌CUDA源，31个`*_CUDA`常量 | NVRTC现场编译；源码摘要与PTX摘要分别记录 | 已用于产品路径 |
| 冻结PTX | `src/runtime/warp_*.rs`按`PTX_SHA256`加载 | 加载前核对摘要，例如运动学`4179acd0…` | 已用于产品路径 |

两条来源都绑定`warp-1.15-physics`配方。
配方取值为`fmad=true`、`fast-math=false`、`--std=c++17`、`compute_70`。
物理产物必过f32检查。
手写移植的源码与冻结PTX的摘要不能互相替代。
原样复制是第三条来源，不取代前两条的既有证据。

## 上游盘点

克隆位置：`target/research/warp`，不进仓库。
标签`v1.15.0`，提交如上。
许可：`LICENSE.md`为Apache-2.0全文。
`warp/native/`共84个文件，含C++宿主与CUDA内核两种用途。
内核编译需要同一棵自包含头树。
`builtin.h`依次引入`crt.h`、`cuda_fp16.h`、`vec.h`、`mat.h`、`quat.h`、`spatial.h`、`array.h`、`tile*.h`等。
`aot.h`引入`<cuda.h>`与`<cuda_runtime.h>`。
`warp.h`是宿主导出接口，编译内核时不使用。
本轮核对的关键摘要如下：

| 文件 | 字节 | SHA-256 |
| --- | ---: | --- |
| `warp/native/builtin.h` | 94374 | `0f05d5fb339e5c91effd6c53a03bbcafa60f5b2928e96009aa5af24174b82468` |
| `warp/native/mat.h` | 160812 | `27510f294838de1f33e0c54dc1bb531b373109008f36b80e7960e8b5a26f2c03` |
| `warp/native/vec.h` | 62325 | `8a3dbd39c6cea762447699a98d4115b0183077bdcb5f2d3a676bec25ffd3d2fa` |
| `warp/native/quat.h` | 54924 | `d3e7f43f71dad326919ced3ad5d7f9afcc8124d8e8f957a2ab816b4288a4e905` |
| `warp/native/spatial.h` | 37337 | `56af9577ceb18cc81758c161feea20add31988be50c59dc8aaa45c2021cc8305` |
| `warp/native/warp.h` | 32905 | `53303073981c678c81d9e6d07b32afe90f329a7a8c36e99692f36879e451f161` |

## 原样复制路线

上游文件以字节复制进入仓内固定目录。
复制时记录上游提交、路径与摘要。
不得就地修改头文件；需要改动时改在包装层。
编译器通过`--include-path`指向复制后的目录。
产物键继续覆盖源码摘要、头文件逐项摘要、配方、架构与NVRTC版本。
头文件摘要进入产物键已有实现基础，见`src/runtime/launcher/artifact.rs`。
当前实现对非空头文件明确拒绝，因为W15未启用。
原样复制落地时须放开该拒绝，并补对应拒绝面测试。

复制集合暂定三级：

| 级别 | 内容 | 用途 |
| --- | --- | --- |
| 必需 | `builtin.h`、`crt.h`、`vec.h`、`mat.h`、`quat.h`、`spatial.h`、`array.h`、`tuple.h`、`range.h`、`intersect.h`、`svd.h`、`mesh.h`、`bvh.h` | GENERIC与数学内核 |
| 条件 | `tile*.h`、`hashgrid.h`、`volume.h`、`texture.h`、`noise.h`、`rand.h`、`matnn.h` | 仅对应上游内核需要时复制 |
| 排除 | `warp.h`、`aot.h`、宿主`*.cpp` | 宿主API与导出，不进内核编译 |

`cuda_fp16.h`来自CUDA SDK，不在Warp头树内。
因此原样复制还须固定一份CUDA SDK头来源。
固定方式与现有`target/toolchains/cuda-12.8.1`一致。
构建不得依赖Python。

## 部署

原样复制是编译期依赖，不进入运行期。
运行期仍只依赖NVIDIA驱动与既有加载路径。
头文件随二进制或构建缓存分发，须登记许可与通知。
上游许可为Apache-2.0，复制须保留许可文本与来源声明。
本轮不复制，不产生分发义务。

## 精度

直接编译上游源码会按上游类型展开。
`builtin.h`有69行`double`，`mat.h`有13行，`quat.h`、`spatial.h`、`vec.h`各有少量。
产品路径必须保持f32，不得升精度或降精度。
原样复制因此需要逐字段精度映射。
映射未完成前，上游源码产物不得进入物理配方。
手写移植仍服务现有产品路径。
四类已知恢复f32的路径不受本轮影响。

## 风险与缓解

| 风险 | 缓解 |
| --- | --- |
| 头树带入未使用依赖 | 按级复制，逐文件登记摘要 |
| `cuda_runtime.h`来源漂移 | 固定CUDA SDK头与版本，写入产物键 |
| 上游源码含f64路径 | 逐字段映射后再接物理配方 |
| 许可义务遗漏 | 保留上游许可文本与来源清单 |
| 复制后误改头文件 | 复制目录只读，改动走包装层 |

## 后续任务

本轮只批准路线。
落地须另立交付任务，建议名`2026-10-21-w15-verbatim-headers`。
该任务至少覆盖：复制目录与固定提交、头文件摘要清单、`--include-path`接线、产物键扩展、非空头文件拒绝面改写、CUDA SDK头固定、许可与通知、上游源码编译复现、逐字段精度映射记录。
该任务不默认修改产品内核。
产品内核切换另立任务。

## 本轮验证

| 命令 | 结果 |
| --- | --- |
| `cargo fmt --check` | 通过 |
| `cargo test --locked` | 通过；退出码0 |
| 上游克隆与摘要核对 | 通过；`v1.15.0`，`ffa038b1` |
| `git status` | 仅`.claude/`未跟踪 |

本轮未编译任何上游源码。
本轮未运行GPU测试。
本轮未做许可法律审阅。
本轮未改产品浮点类型、配方与容差。
