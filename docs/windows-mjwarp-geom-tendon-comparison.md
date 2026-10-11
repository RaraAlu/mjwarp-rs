# Windows肌腱上游并列诊断

日期：2026-10-11。
状态：局部实测；完整T4仍开放。

本报告保留首次并列诊断。
后续已修复路径与结构漂移。
最新状态见[T4补测](windows-t4-recheck.md)。

T4表示Windows G01验收。
它不表示显卡型号。
本报告只覆盖几何肌腱子集。
它不关闭完整G01验收。

## 参考顺序更新

2026-10-11按用户要求调整。
第一参考标准为原生MuJoCo。
原生通过即通过该项数值门禁。
仅原生超差后启用MJWarp。
第二参考通过也通过数值门禁。
原生通过无需上游再次通过。
规则详见[精度契约](numerical-precision-contract.md)。

本报告保留原始测量结果。
现有并列测试仍只提供诊断。
两级判定尚未接入测试。
第310帧可用于第二级复核。
本次不宣称已完成该项复核。
第393帧仍须按两路容差判定。
本次不修改样本或容差。

## 冻结条件

| 项目 | 实际条件 |
| --- | --- |
| MJWarp提交 | `71da24d956378a87a703b6e1442b13aec0c4ac29` |
| 上游包 | MJWarp 3.15.0；Warp 1.15.0 |
| 离线模型编译器 | MuJoCo 3.15.0 |
| 作者环境 | Python 3.12.13；NumPy 2.5.4 |
| GPU | RTX 4070 Ti SUPER |
| 驱动 | 596.36 |
| 上游编译器 | CUDA 12.9.86；sm_89 |
| 本项目编译器 | NVRTC 12.8；compute_70 |
| 物理精度 | f32 |
| 输入批量 | 3与513世界 |
| 输出采样 | 37组世界状态 |
| 浮点字段 | 16项 |
| 整数字段 | 3项 |

作者隔离工具环境。
产品构建不依赖此环境。
产品测试读取静态参考。
产品测试不启动Python。
脚本不修改上游源码。
脚本核对安装源码哈希。

MuJoCo 3.12轮子缺少枚举。
冻结上游引用该枚举。
作者改用3.15编译源XML。
随后覆盖全部既有浮点输入。
作者逐项核对原有整数拓扑。
Rust再核对设备输入哈希。
双方采用相同f32量化。
本次不修改产品原生ABI。

新资料位于[参考目录](../fixtures/mjwarp-geom-tendon/)。
清单冻结输入与输出哈希。
来源记录保存上游源码哈希。
它也保存生成内核哈希。
本地缓存保留CUDA与PTX。
清单记录离线依赖许可。
本次不新增Cargo依赖。
作者固定LF换行。
Git检出不得改变参考哈希。

## 诊断与修复

上游默认启用FMA。
本项目此前关闭FMA。
同步内核现已跟随上游。
双方仍关闭fast-math。
回执记录实际FMA设置。
GPU回归验证融合运算。

精度门禁识别库级角度约简。
它仅新增等价指令拼写。
它继续拒绝物理f64运算。
它继续拒绝f16等低精度。
PTX默认采用最近偶数舍入。
依据：[NVIDIA乘法规则](https://docs.nvidia.com/cuda/parallel-thread-execution/#floating-point-instructions-mul)。

下表比较同一稀疏力臂项。
该项为`ten_J[3]`。
世界2与512给出相同结果。

| 源帧 | 本项目修复后 | 冻结MJWarp | 原生MuJoCo参考 |
| --- | --- | --- | --- |
| 310 | -0.675108314 | -0.675108314 | -0.674780926 |
| 344 | -0.194405019 | -0.194405019 | -0.194396268 |
| 393 | -0.168292493 | -0.168039322 | -0.168325342 |

第310帧仍超原生旧容差。
但它已逐位匹配MJWarp。
所以旧超限不能单独判错。
不能为此恢复内部f64。

第393帧仍存在上游差异。
力臂最大绝对差为`2.532e-4`。
FMA修复未关闭此项。
下一步核查绕行计算顺序。
还需核查编译工具链差异。

## 判断边界

测试报告绝对差、相对差与ULP。
相对差采用对称尺度。
公式为`abs(a-b)/max(abs(a),abs(b))`。
双方为零时相对差为零。
ULP统计折叠正负零。
位差统计仍区分正负零。
整数比较要求完全一致。
测试拒绝非有限值。

本次不设置浮点容差门槛。
报告标记`parity=not_evaluated`。
诊断测试通过不等于数值验收。
原生参考仍保留原有容差。
本次不改写失败参考。
本次未测稳态性能与显存。
本次未执行Linux复验。
完整NP01至NP04仍有缺口。

## 实际验证

debug与release结果相同。
每种模式比较27121个浮点量。
每种模式比较3478个整数量。
整数失配为零。

| 检查 | 实际结果 |
| --- | --- |
| 上游并列诊断 | 双模式各4过0败 |
| 内核专项 | debug 7过0败 |
| 完整G01原生测试 | 双模式各7过0败 |
| G01全链比较 | 2084状态；7704548标量 |
| G01低层比较 | 50512标量 |
| `cuda-probe`全套 | 双模式各385过5败 |
| 默认宿主测试 | 205过0败 |
| 格式与CUDA Clippy | 通过 |

全套CUDA回归仍有5项失败。
质量求解占2项原生超限。
运动肌腱占1项原生超限。
G02参考哈希占2项失败。
后两项来自Windows换行。
按LF读取后匹配原清单。
本次未改动这些G02资料。

专项G01另启用原生feature。
只启用CUDA不会运行该目标。
上述7项来自实际原生测试。
零用例不计入验收。
本次未运行全feature脚本。
原生通过仍不关闭上游差异。

本机Normify已登记本次进展。
已审查并刷新两个相关叶子。
仍有8处其他源码漂移。
完整结构校验尚未归零。
文档工具扫描224个源码文件。
它扫描73个文档文件。
它仍报告12处既有路径提示。
本次修正一处T4现状描述。
新增报告未触发工具提示。

## 复现入口

Rust入口：
[并列诊断](../tests/mjwarp_geom_tendon.rs)。
两类测试共用[输入构造](../tests/support/geom_tendon_input.rs)。

```powershell
$env:CUDA_PATH = Join-Path $PWD 'target/toolchains/cuda-12.8.1'
$env:PATH = "$env:CUDA_PATH/bin;$env:PATH"
$env:MJWARP_PARALLEL_REPORT = Join-Path $PWD 'target/mjwarp-parallel/report.json'
$env:MJWARP_PHYSICS_PTX_DIR = Join-Path $PWD 'target/mjwarp-parallel/ptx'
New-Item -ItemType Directory -Force target/mjwarp-parallel | Out-Null
cargo +stable-x86_64-pc-windows-msvc test --locked --features cuda-probe --test mjwarp_geom_tendon -- --include-ignored --test-threads=1
cargo +stable-x86_64-pc-windows-msvc test --release --locked --features cuda-probe --test mjwarp_geom_tendon -- --include-ignored --test-threads=1
```

作者脚本仅供离线更新参考。
脚本要求独立安装冻结工具。
先在`target/mjwarp-parallel/upstream`准备冻结提交。
再用独立环境执行作者脚本。
脚本只向`target/mjwarp-parallel`写出结果。
审查者核验后再冻结文件。
此步骤不进入Cargo或验收脚本。

```powershell
target/mjwarp-parallel/venv/Scripts/python.exe fixtures/mjwarp-geom-tendon/reference.py
```

本地原始证据位于：

```text
target/mjwarp-parallel/debug-report.json
target/mjwarp-parallel/frozen-debug-report.json
target/mjwarp-parallel/frozen-release-report.json
target/mjwarp-parallel/verified-debug.log
target/mjwarp-parallel/verified-release.log
target/mjwarp-parallel/g01-debug.log
target/mjwarp-parallel/g01-release.log
target/mjwarp-parallel/warp-cache/
```
