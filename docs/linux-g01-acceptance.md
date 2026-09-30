# Linux G01全阶段验收

状态：Linux双模式验收通过。
矩阵：G01。
本轮GPU采用RTX 5080。
本轮没有T4环境。

## 验收边界

本轮补齐Linux原生准入。
本轮复验完整G01调度。
本轮复用冻结模型与参考。
本轮不修改物理内核。
本轮不放宽数值容限。
本轮不重新生成参考。
产品测试不启动Python。
产品测试不调用CPU物理。
原生桥接只读取已编译MJB。
LRsLab继续负责模型编译。
完整引擎与发布门禁仍待完成。
Windows记录保留历史证据。
本轮不重新执行Windows。

既有范围见[Windows报告](windows-g01-acceptance.md)。
冻结模型见[参考说明](../fixtures/g01/README.md)。

```text
upstream: google-deepmind/mujoco_warp
revision: 71da24d956378a87a703b6e1442b13aec0c4ac29
source baseline: 12cea1e834f315be9d730e00281ec985d1ce9bf3
native port: 562fa37
Linux gates: 039445a
```

## Linux原生准入

可选feature支持两种工具链。
Windows采用x64 MSVC。
Linux采用x86_64 GNU。
默认构建仍不编译C++桥接。
Linux使用dlopen与dlsym。
桥接仅加载绝对可信路径。
桥接使用局部符号作用域。
Linux路径保留原生字节。
Windows继续使用UTF-16。
入口拒绝相对路径与NUL。
桥接先删除模型再卸载库。
快照继续独占复制字段。
G01快照继续覆盖78项字段。
生产Model与Data契约仍待实现。

环境变量沿用历史名称。
Linux的DLL变量指向so文件。

```text
MJWARP_MUJOCO_ROOT=target/toolchains/mujoco-3.12.0/linux-package
MJWARP_MUJOCO_DLL=<absolute root>/lib/libmujoco.so.3.12.0
CUDA_PATH=target/toolchains/cuda-12.8.1
```

本轮没有新增Cargo依赖。
MuJoCo保持3.12.0候选版本。
它采用Apache-2.0许可。
cc保持1.6.0与原有许可。
CUDA组件采用官方许可。
CCCL另保留开源通知。
下载脚本保留原包许可文件。
本轮不提交原生库与工具链。

| 组件 | 版本 |
| --- | --- |
| MuJoCo | 3.12.0 |
| NVRTC | 12.8.93 |
| CUDA Runtime | 12.8.90 |
| CCCL | 12.8.90 |
| NVCC组件 | 12.8.93 |
| LLVM包 | 23.1.0 |

原生摘要见[运行包清单](../fixtures/native-probe/linux-runtime.json)。
许可依据见[第三方说明](../THIRD_PARTY_NOTICES.md)。
CUDA依据见[官方清单](https://developer.download.nvidia.com/compute/cuda/redist/redistrib_12.8.1.json)。

## 复验命令

在仓库根目录执行命令。
主机需要NVIDIA驱动。
主机需要Rust与g++。
脚本需要Bash与jq。
下载需要curl与tar。
脚本只在target保存工具链。
脚本只修改自身进程环境。

```bash
bash scripts/prepare-linux-native.sh
bash scripts/prepare-linux-cuda.sh
bash scripts/test-linux-g01.sh --all-features --full-regression
bash scripts/test-linux-g01.sh --all-features --full-regression --release
```

脚本逐项核对冻结参考摘要。
脚本拒绝损坏或缺项清单。
脚本拒绝零用例物理验收。
指定门禁拒绝遗漏用例。
完整回归覆盖169项显式探针。
非全feature回归覆盖161项。
脚本生成checks与report文件。
报告记录HEAD与脏状态。
报告记录工具链与原生摘要。
target目录不进入Git。

T4复验需要真实T4设备。
当前脚本只选择设备零。
T4环境可执行以下命令：

```bash
bash scripts/test-linux-g01.sh --all-features --full-regression --require-t4
bash scripts/test-linux-g01.sh --all-features --full-regression --release --require-t4
```

RTX 5080不能替代T4证据。
本轮验证了设备拒绝路径。
本机T4命令返回退出码1。
报告不能据此标记T4通过。

## 双模式结果

主机采用以下配置：

| 项目 | 实测配置 |
| --- | --- |
| 系统 | Ubuntu 24.04.4 LTS |
| 内核 | 7.0.0-38-generic |
| 架构 | x86_64 GNU |
| GPU零号 | NVIDIA GeForce RTX 5080 |
| 驱动 | 595.99.02 |
| 显存 | 16303 MiB |
| 计算能力 | 12.0 |
| Rust | 1.96.1 |
| C++ | g++ 13.3.0 |

两个模式均通过完整门禁。

| 门禁 | debug | release |
| --- | ---: | ---: |
| 原生桥接 | 27 | 27 |
| 受限原生运行 | 27 | 27 |
| 全feature宿主测试 | 263 | 263 |
| G01门禁 | 7 | 7 |
| 原生G01组合 | 2 | 2 |
| 纯原生G01转换 | 1 | 1 |
| 纯原生桥接 | 22 | 22 |
| 常驻运动学回归 | 98 | 98 |
| 完整显式探针回归 | 169 | 169 |
| 默认宿主测试 | 204 | 204 |

两个模式覆盖以下检查：
fmt、四组clippy与rustdoc。
clippy与rustdoc均拒绝警告。
Rust新增三个脚本边界测试。
Linux另增原生路径字节测试。
计数差异不表示物理范围扩张。
受限进程清除动态库搜索变量。
它只保留系统命令路径。
它不代表清洁机器部署。

完整链检查2084组世界状态。
完整链比较7704548个浮点量。
低层入口另比较50512个量。
以上计数均按每模式记录。
所有参考继续保留原有容限。

```text
abs(actual - expected) <= 2e-5 + 2e-5 * abs(expected)
debug max_abs_error: 1.3855896387748867e-6
debug raw_max_abs_error: 4.977585623677783e-7
release max_abs_error: 1.3855896387748867e-6
release raw_max_abs_error: 4.977585623677783e-7
debug: target/linux-g01-L0HL7W12/report.json
debug report sha256: 09e6ff3e255ca6b876f859b8be9a5e8080fa2d53bb0699eca2d8e3962956a9f5
release: target/linux-g01-jhGwVtao/report.json
release report sha256: 77553a91f65fc12c9cfe0fd530a9912c43ea870d064469028fb0544a53ff44ca
fixture manifest sha256: 7ead0bc9dc49c77c644be3de47862c68728404904628208f42b6f9b5b240acf4
```

报告记录提交039445a。
报告如实标记工作区脏状态。
用户改动与交接包保持原样。
测试期间只新增说明文档。
报告时间采用主机UTC。
本轮不修改历史报告时间。

## Normify同步

用户提供了原始架构导出。
本轮先验证交接包摘要。
本轮恢复原树到默认目录。
本轮保留原有模块UID。
本轮不另建替代结构树。
本轮保留计划态产品契约。
本轮不关闭历史开放变更。
早期debug先于结构预检。
本轮补齐真实接口调用。
本轮不追认早期预检。

```text
project: mjwarp-rs
directory: /home/zkbot/.codex/normify/normify-mjwarp-rs
change: 2026-10-09-g01-linux-acceptance
```

实际流程包含以下接口：

```text
normify_brief -> normify_check -> normify_change_open
normify_module_get -> normify_module_patch
normify_sync -> normify_module_refresh -> normify_validate
normify_change_close -> normify_validate -> normify_build -> normify_render
```

开发期校验返回零错误。
它提示两项结构警告。
柔体叶子保留既有粗粒度。
本轮变更并存历史开放变更。
收尾结果另见变更记录。
本轮不修订粗粒度模块边界。
交接包继续保留原始字节。

```text
change record: /home/zkbot/.codex/normify/normify-mjwarp-rs/changes/2026-10-09-g01-linux-acceptance.json
receipt: /home/zkbot/.codex/normify/normify-mjwarp-rs/receipt.json
viewer: /home/zkbot/.codex/normify/normify-mjwarp-rs/normify.html
```

## 未完成项

T4完整G01复验仍待环境。
清洁机器部署仍待执行。
性能验收与渲染仍待完成。
完整G02与G03仍待实现。
完整G06、G22与G25仍待实现。
本轮通过不代表完整引擎完成。
