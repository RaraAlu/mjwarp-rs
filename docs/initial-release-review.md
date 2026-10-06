# mjwarp-rs 初代完整对标复审

复审日期：2026-10-06。
状态：设计修订，尚未验收。

## 结论

首版必须完整对齐GPU引擎。
现有功能族覆盖主要方向。
现有契约仍存在语义偏差。
本轮记录9项复审发现。
本轮修正7项文档偏差。
2项准入问题仍待关闭。
本轮不证明引擎功能等价。

初次复审只看到问候语。
后续代码新增P1驱动探针。
探针不关闭本报告准入项。
当前证据见[探针报告](gpu-probe.md)。
仓库没有GPU引擎实现。
仓库没有物理测试用例。
因此当前不能发布对标首版。

## 冻结证据

```text
repository: google-deepmind/mujoco_warp
revision: 71da24d956378a87a703b6e1442b13aec0c4ac29
public_exports: 99
export_source_modules: 21
upstream_test_modules: 30
local_engine_implementation: none
```

本轮通过原生Git核对提交。
本轮只读取上游源码。
本轮不启动Python解释器。
行号采用冻结Git对象的行号。
网页提取行号不替代源码行号。

依据：[冻结入口](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/__init__.py)、[当前入口](../src/main.rs)。

## 按风险排序的发现

“已修正文档”只关闭设计偏差。
实现与GPU测试仍须独立验收。
下表不把文档修改计为功能完成。

| ID | 等级 | 原契约或缺口 | 本轮处理 | 状态 |
| --- | --- | --- | --- | --- |
| RV01 | P1 | 功能分母只含功能族与符号；字段、参数、组合及证据未齐 | 保留全量展开门槛；禁止据此声明完整 | 准入未关闭 |
| RV02 | P1 | step2强制分步票据；无法导入已准备的中间量 | 等价step2不收票据；严格门面另设接口 | 已修正文档 |
| RV03 | P1 | 插件排除过宽；契约遗漏geom用户SDF扩展 | 仅排除对应插件类别；补入距离、梯度与属性契约 | 已修正文档 |
| RV04 | P1 | 强制派生版本、BVH及回调限制缩小调用范围 | 保留资源安全；将物理新鲜度检查设为可选 | 已修正文档 |
| RV09 | P1 | “完整发行包”与既有GPU产品边界不同 | 列出CPU调试及工具差异；等待范围确认 | 边界待确认 |
| RV05 | P2 | 渲染默认关闭快速数学；默认容量仍待定 | 恢复上游渲染默认值；要求容量算法对齐 | 已修正文档 |
| RV06 | P2 | 深度缩放强制有限正数；上游没有此限制 | 等价入口保留浮点公式；严格入口另设检查 | 已修正文档 |
| RV07 | P2 | 溢出后只允许诊断输出；上游仍可导出截断数据 | 保留已有字段导出；同时报告计数与截断 | 已修正文档 |
| RV08 | P2 | 常量刷新默认保护全部状态；改变Data副作用 | 保留调用方Data与restore语义；独立工作区仅供门面 | 已修正文档 |

### RV01：完整功能分母

99个符号只是公开入口。
61个功能族只是粗粒度清单。
30个测试模块不是用例总数。
这些数量不能证明完整等价。

后续细化已经补入声明盘点。
盘点覆盖895项直接字段声明。
它保留31类枚举及73项签名。
阶段契约补入回调和注入路径。
组合清单核对43组入口分支。
字段语义、内部组合仍不完整。
原生取值与数值证据仍待冻结。
因此RV01继续保持未关闭。

细节见[声明盘点](upstream-contract-inventory.md)、
[阶段契约](physics-stage-contracts.md)、
[组合清单](supported-combinations.md)。

正式验收还需登记：

- 全部字段、单位与布局。
- 全部参数、默认值与覆盖规则。
- 全部公开及内部枚举值。
- 几何对、资产与能力开关。
- 回调、SDF及设备视图扩展。
- 拒绝条件、警告与降级行为。
- 阶段输入、输出与副作用。
- 全部测试意图与独立证据。

依据：[类型定义](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/types.py)、[上传条件](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L288-L431)。

### RV02：独立分阶段调用

上游step2不要求step1票据。
上游测试先准备原生中间量。
它随后上传Data并调用step2。
强制票据会拒绝这条路径。
Rust须提供等价阶段入口。
RK4分步仍采用Euler积分。

依据：[step2源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward.py#L2138-L2156)、[导入中间量测试](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/forward_test.py#L715-L740)。

### RV03：插件与SDF边界

上游拒绝body插件。
它也拒绝执行器和传感器插件。
它仍提取geom插件属性。
SDF内核仍调用用户距离与梯度。
用户代码必须提供对应实现。
这些钩子不保证任意插件可用。

Rust须提供等价GPU扩展机制。
Rust不必复制Python补丁语法。
首版还须保留体积SDF路径。
插件状态与geom配置不能混同。

依据：[插件拒绝条件](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L354-L361)、[geom属性上传](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L902-L929)、[用户钩子](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/collision_sdf.py#L298-L315)、[体积与用户分支](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/collision_sdf.py#L511-L566)。

### RV04：资源安全与物理新鲜度

上游render读取现有位姿。
它不自动调用forward或refit。
上游set_state只写签名字段。
它不自动重建派生量。
Rust不能用新鲜度检查缩减入口。

资源寿命与越界检查始终强制。
物理新鲜度检查可以选用。
严格门面必须使用独立调用形状。
诊断不能隐式重置非有限状态。
迭代上限不能默认改成异常。

局部重置还涉及共享计数。
上游仅在选中世界0时清零nacon。
等价入口不能擅自重建接触池。
严格门面可以提示派生量陈旧。

依据：[渲染提交](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/render.py#L1491-L1604)、[状态写入](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/support.py#L829-L948)、[共享计数重置](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L2749-L2757)。

### RV05至RV08：默认与副作用

渲染默认开启快速数学。
数值对比应匹配双方配置。
关闭快速数学属于独立测试配置。
它不能替代默认配置验收。

深度提取采用除法与clamp。
等价入口不额外要求正缩放值。
零值及非有限行为仍需实测。
文档不预设这些GPU数值结果。

上游导出限制接触与约束数量。
数量溢出后仍可导出已有字段。
Rust须报告截断，避免误判完整。

restore默认采用true。
false仍恢复qpos本身。
true还刷新当前姿态派生量。
该选项不保证完整Data回滚。

依据：[渲染默认值](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/render_util.py#L312-L342)、[深度公式](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/render_util.py#L192-L243)、[截断导出](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/io.py#L2410-L2419)、[常量恢复](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/set_const.py#L804-L849)。

### RV09：引擎对标与发行包差异

现有项目边界只对标GPU引擎。
“完全对标”不能掩盖交付差异。
本轮沿用既有边界开展复审。
本轮不增加CPU后端或窗口工具。
范围确认仍需用户决议。

| 层次 | 上游能力 | 当前项目决议 |
| --- | --- | --- |
| GPU引擎 | 物理、数据、查询与批量渲染 | 全量等价；不能缩减 |
| CPU调试 | README明确提供开发调试路径 | 既有范围不交付；等待确认 |
| 发行工具 | testspeed、viewer、record入口 | 保留无头测速及导出；前端归LRsLab |
| Python及生态桥 | Python接口及JAX等路径 | 无Python产品；提供通用Rust数据接口 |
| 原生模型 | 接收已编译MjModel | 保留等价转换；模型编译归LRsLab |
| 发布系统 | 使用上游支持及依赖配置 | 首版另要求Windows原生与Linux验收 |

CPU调试不等于CPU play。
不能用play排除解释全部CPU差异。
Rust接口适配不改变GPU功能分母。
产品交付边界需明确声明差异。
完整发行包对标需要重新决议。

依据：[CPU调试声明](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/README.md#getting-started)、[工具入口](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/pyproject.toml#L64-L75)、[既有项目边界](project-positioning.md)。

## 首版准入补充

每项功能都需完成三层验收。

1. 等价入口覆盖上游调用范围。
2. 数值与阶段副作用符合证据。
3. 双平台资源与部署测试通过。

每条验收还需记录：

| 维度 | 必须记录 |
| --- | --- |
| 默认 | 全部默认值、None及推导规则 |
| 输入 | 类型、形状、批量与有效组合 |
| 输出 | 字段、单位、布局与阶段对应 |
| 副作用 | 状态、派生量、计数与休眠变化 |
| 边界 | 错误、警告、截断与迭代上限 |
| 扩展 | 回调、SDF、外部缓冲与图捕获 |
| 差异 | 语言适配、交付差异与可选门面 |
| 证据 | 固定提交、测试、样本及双平台报告 |

严格门面不能抵扣等价入口缺口。
额外安全检查不能缩减GPU支持。
上游警告路径不能全部改成拒绝。
上游限制不能扩大为整族排除。
上游未支持项不进入完整分母。
功能缺口仍须阻断正式发布。

补充任务采用MC25至MC30。
任务细节见[模块契约](module-contracts.md)。
完整分母继续由P0-03展开。
正式验收继续遵守P11门槛。

## 初次复审验证

本轮只改文档。
本轮检查1个本地Rust文件。
本轮检查9份设计文档。

| 检查 | 实际结果 |
| --- | --- |
| 冻结提交与公开映射 | 提交一致；99个符号与21个来源模块一致 |
| 上游测试入口 | 30个模块与映射一致；未执行这些测试 |
| 契约任务与功能族 | MC01至MC30连续；矩阵保留61个功能族 |
| 本地文档链接 | 38处检查通过 |
| 固定源码链接 | 76处文件及适用行号检查通过 |
| 文档句长 | 正文检查通过；表格与代码单独处理 |
| 七模块依赖 | 无环；模块数量未变 |
| 源码与配置 | 5个受保护文件的SHA256保持不变 |
| 文档差异空白检查 | 无错误 |
| `cargo fmt --check` | 通过 |
| `cargo test` | 通过；0个测试用例 |
| `cargo clippy --all-targets -- -D warnings` | 通过 |
| Python文档审计脚本 | 未运行；本轮采用原生检查 |
| Normify漂移工具 | 未运行；仓库没有现有结构 |

原生检查采用临时脚本。
脚本不进入产品构建与测试链。
本轮未生成或改写架构树。

实际静态检查命令：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File C:\Users\zhang\AppData\Local\Temp\mjwarp-parity-review-check.ps1
```

本轮不运行上游Python测试。
本轮不执行GPU物理或渲染。
Linux与GPU验收仍未执行。
零个Rust用例不代表物理验收。

## 契约细化验证

本轮继续展开P0-03。
本轮只修改设计文档。
源码仍保留问候语入口。
本轮不关闭RV01与RV09。

本轮采用独立的静态检查。
检查直接读取冻结Git对象。
检查不复用声明提取脚本。
字段检查排除文档与方法体。
签名检查逐段比较源文。
表格和代码不参与正文句长统计。
静态一致不等于算法正确。

| 检查 | 实际结果 |
| --- | --- |
| 声明盘点 | 12类记录、895项直接字段声明一致 |
| 枚举盘点 | 31类枚举、218项成员表达式一致；未求值原生枚举 |
| 功能签名 | 73项签名与冻结源码逐段一致；公开映射仍为99项 |
| 边界与任务编号 | SC01至SC43连续；MC01至MC42连续；任务尚未实现 |
| 功能分母 | 矩阵保留61个功能族；上游保留30个测试模块 |
| 文档链接 | 12份文档；70处本地链接、244处冻结源码链接检查通过 |
| 文档句长与空白 | 正文句长检查通过；文档差异空白检查通过 |
| 模块与文件保护 | 七模块依赖无环；5个受保护文件SHA256不变 |
| `cargo fmt --check` | 通过 |
| `cargo test` | 通过；0个测试用例；不构成物理验收 |
| `cargo clippy --all-targets -- -D warnings` | 通过 |
| Python文档审计工具 | 未运行；项目排除Python测试链，本轮采用原生检查 |
| Normify同步与收尾工具 | 未运行；本轮未找到现有架构树，不自动创建 |
| GPU与Linux验收 | 未执行；本轮没有引擎实现与GPU测试 |

实际静态检查命令：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File C:\Users\zhang\AppData\Local\Temp\mjwarp-contract-detail-check.ps1
```

临时检查脚本不进入产品链。
本轮未冻结原生ABI与依赖版本。
字段单位与实际布局仍需核对。
内部组合与逐用例证据仍需展开。
所有GPU功能继续标记未实现。
