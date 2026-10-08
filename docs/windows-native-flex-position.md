# Windows原生柔体位置转换

日期：2026-10-08。
范围：G01原生位置字段增量。
完整G01仍保留计划态。

## 冻结范围纠正

冻结上游拒绝二次插值。
拒绝条件为插值绝对值等于二。
正负二阶均不属支持集。
内核分支不代表合法模型范围。
G01不以实现二次插值为前提。
本轮保持直接与线性支持。
线性壳体继续沿用既有语义。
矩阵计算与乘法推进G22。
G01只负责废弃矩阵缓存。

依据：[冻结转换检查](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L363-L366)。
阶段见[运动学契约](physics-stage-contracts.md)。

## 本轮接口

| 接口 | 作用 |
| --- | --- |
| `NativeModelProbe::flex_position_snapshot()` | 复制十二项原生位置字段 |
| `NativeFlexPositionSnapshot` | 独占计数、索引、标志与f64位置 |
| `NativeFlexPositionSnapshot::into_fields()` | 严格转换并校验位置字段 |
| `NativeFlexPositionSnapshot::into_model(tendons)` | 组合调用方准备的肌腱输入 |

源代码位于`src/io/flex.rs`。
桥接位于`native/model_probe.cpp`。
ABI位于`include/mjwarp_native_probe.h`。
既有核心DTO仍占96字节。
新增计数DTO独占32字节。
复制目标DTO独占128字节。
C++与Rust分别断言布局。
快照不借用模型或DLL。
调用方可先释放原生所有者。
GPU计划仍保留自己的会话。

## 字段与转换

| 字段 | 原生快照 | 转换结果 |
| --- | --- | --- |
| `flex_interp` | `nflex`项i32 | 保留直接与线性模式 |
| `flex_cellnum` | `3*nflex`项i32 | 检查正格数与网格容量 |
| `flex_nodeadr`、`flex_nodenum` | 各`nflex`项i32 | 检查连续节点区间 |
| `flex_vertadr`、`flex_vertnum` | 各`nflex`项i32 | 检查连续顶点区间 |
| `flex_centered` | `nflex`项u8 | 仅允许零或一，再转bool |
| `flex_nodebodyid` | `nflexnode`项i32 | 检查身体引用 |
| `flex_vertbodyid` | `nflexvert`项i32 | 检查直接与插值引用 |
| `flex_node` | `3*nflexnode`项f64 | 严格转换至f32 |
| `flex_vert`、`flex_vert0` | 各`3*nflexvert`项f64 | 严格转换至f32 |

桥接先核对全部计数。
桥接先核对全部所需指针。
任一检查失败都不复制字段。
零计数字段允许空指针。
Rust先核对GPU展开容量。
检查通过后才分配暂存。
Rust再核对十二项数组长度。
转换拒绝非有限值与溢出。
转换保留有限负零。
居中检查也读取原生精度。
f64下溢不能隐藏非零偏移。
模型检查沿用既有位置校验。

组合入口核对nbody、nq与nv。
这些维度不证明模型同源。
调用方须保证身体编号一致。
本轮不冻结完整模型身份ABI。

## 样本与产品边界

样本复用[柔体位置参考](../fixtures/flex-position/README.md)。
样本包含四项柔体。
样本包含20节点与80顶点。
样本包含十二组固定状态。
测试读取可信静态MJB。
产品测试不重新编译XML。
产品测试不生成物理参考。
产品测试不启动Python。
原生脚本核对三个样本清单。
原生桥接只复制模型字段。
GPU运行全部位置计算。
其余模型字段仍用静态DTO。
本轮不宣称完整put_model。

测试共用既有位置DTO辅助。
原生测试单独核对所有权。
FFI测试独立声明目标布局。
它逐项破坏十二个目标指针。
它核对计数、schema与空目标。
失败时所有目标保留哨兵值。
模拟DLL覆盖晚期缺失源。
模拟DLL也覆盖坏计数与超限。
释放日志核对先删模型后卸载。
空柔体与零节点路径单独覆盖。

内核验收独占测试目标。
目标名为`native_flex_position`。
该目标需要NVRTC与驱动。
原生受限PATH验收不运行它。
原生脚本仍核对准确测试数。
本轮不削减既有原生验收。

## 验证记录

| 环境 | 记录 |
| --- | --- |
| 系统 | Windows x86_64，10.0.26200.0 |
| GPU | NVIDIA RTX 4070 Ti SUPER |
| 驱动 | 596.36 |
| Rust | 1.99.0，GNU与MSVC |
| CUDA | 项目私有12.8.1工具链 |
| MuJoCo | 可信3.12.0候选工具包 |

本轮不新增第三方依赖。
DLL哈希沿用冻结清单。

| 实际验证 | 结果 |
| --- | --- |
| `cargo fmt --check` | 通过 |
| `cargo test --locked` | GNU通过188项宿主与12项文档测试 |
| MSVC `cargo test --locked --all-features` | 调试与发布各通过244项宿主及12项文档测试 |
| MSVC `cargo test --locked --all-features -- --ignored --test-threads=1 --nocapture` | 调试与发布各通过159项探针 |
| `scripts/test-windows-native.ps1 -AllFeatures -RestrictedRuntime` | 调试模式通过25项，受限重跑也通过25项 |
| 上述原生脚本追加`-Release` | 发布模式通过25项，受限重跑也通过25项 |
| `scripts/test-windows-native.ps1` | 仅原生feature通过20项 |
| `scripts/test-windows-resident-kinematics.ps1 -AllFeatures` | 调试模式通过98项 |
| 上述常驻脚本追加`-Release` | 发布模式通过98项 |
| GNU默认与cuda feature Clippy | 两路均零警告 |
| MSVC全feature调试与发布Clippy | 两路均零警告 |
| MSVC全feature Rustdoc | `RUSTDOCFLAGS=-D warnings`通过 |
| `git diff --check` | 通过 |

全量探针包含25项原生测试。
它也包含134项GPU测试。
本轮新增五项宿主单元测试。
本轮新增三项原生验收测试。
本轮新增一项GPU验收测试。
全量探针不代表完整G01验收。

Clippy命令沿用以下参数。

```text
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --all-targets --features cuda-probe -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --all-features -- -D warnings
cargo +stable-x86_64-pc-windows-msvc clippy --locked --all-targets --all-features --release -- -D warnings
```

GPU对比采用1、2、5、513世界。
每组执行四轮不同状态。
每种构建验证2084组状态。
每种构建比较625200项标量。
两种构建取得相同最大误差。
最大绝对误差为3.8035251e-7。
容限仍为`2e-5+2e-5*abs(ref)`。
本轮没有放宽数值容限。
GPU测试也核对四项失效标志。
GPU测试先释放原生模型。
GPU测试随后释放宿主会话。
计划继续保留所需设备资源。

初跑发现样本计数误判。
另一份样本实际含八个节点。
测试现核对真实节点数量。
受限环境初跑缺少NVRTC。
内核测试现使用独立目标。
原生受限验收继续覆盖旧用例。

验证日志集中保存在：

```text
target/native-flex-verification/results.json
target/native-flex-verification/all-probes-debug.log
target/native-flex-verification/all-probes-release.log
target/native-flex-verification/host-msvc-debug-final.log
```

原生受限证据分别保存在：

```text
target/native-probe/306bf2dc00c544d99672e37ccd586e7a/report.json
target/native-probe/0e47d8636a5542249ec8c7721eb838d4/report.json
target/native-probe/cb612281bb534b1db6a4839c0a307623/report.json
```

常驻证据分别保存在：

```text
target/resident-kinematics/75c81e4934284924a8717ddb9b80f9d7/report.json
target/resident-kinematics/6c157219d1fd494fa9a2aaf7cfa3d7ac/report.json
```

## 文档与架构复核

范围检查读取17份源码。
它核对12份相关文档。
它核对158条本地链接。
它核对八项冻结文件哈希。
范围检查确认零错误。
人工复核字段与阶段边界。
本轮纠正二次插值支持范围。
本轮区分G01与G22推进。

外部审计扫描183份源码。
它扫描63份文档。
工具报告九项路径提示。
提示涉及上游路径与脚本变量。
本地PTX与配置库均存在。
这些提示不构成本轮文档错误。
报告保存在范围检查目录。

架构叶子记录原生位置转换。
它保持完整产品模块计划态。
既有柔体叶子保留粒度提示。
该叶子目前拥有两份源码。
本轮不为消除提示拆分它。

## 剩余范围

边、面与拉伸字段转换仍待补齐。
附着、相机与肌腱转换仍待补齐。
完整Data导入仍待实现。
等价运动学入口仍待收口。
全阶段组合仍需独立证据。
Linux真实GPU验收仍待执行。
本轮不关闭G01、G22或G25。
