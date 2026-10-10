# Linux常驻RNE偏置力子集

日期：2026-10-10。
时间采用北京时间。
范围：G02第二批刚体子集。
完整G02仍待实现。

## 实现与接口

GPU计算广义偏置力。
公开结果仅含qfrc_bias。
本子集固定采用flg_acc=false。
计划显式接收共享重力。
零重力关闭重力贡献。
本入口不转换完整原生Option。
默认qpos采用既有qpos0。
默认qvel全零。
默认结果保持未就绪。

```text
RneBiasPlan::new(session, model, gravity) -> create_data
write_qpos / write_world_qpos
write_qvel / write_world_qvel
RneBiasPlan::update -> RneBiasData::readback
RneBiasSnapshot::world -> qfrc_bias
```

它复用常驻模型与状态上传。
模型与重力只共享一份。
私有GPU工作区采用f64。
工作区覆盖刚体、质心与速度。
它也覆盖加速度与内力。
公开输入与结果仍采用f32。
模型上传与编译只发生一次。
状态更新不回读宿主中间量。
G01继续沿用原有f32路径。
产品链没有CPU回退。
产品测试不启动Python。
本批没有新增第三方依赖。

## 阶段与数值边界

完整子集update刷新前置字段。
前置入口只准备字段。
入口名为update_velocity。
它同时废弃已有偏置力。
update_bias读取新鲜前置字段。
它不隐式重算位置或速度。
合法状态写入废弃派生结果。
输入错误保留已有结果。
计划拒绝同尺寸跨模型数据。
更新失败不发布结果。
显式回读检查全部工作区。
它也检查首尾守卫与f32溢出。
快照独占宿主结果。
状态与缓冲保留设备会话。

首轮原生对比发现精度损失。
仅提升RNE累加仍未达容限。
只提升速度累加也未达容限。
本批改用内部f64前置链。
本批没有替换冻结参考。
本批没有放宽固定容限。

```text
2e-5 + 2e-5*abs(reference)
```

本子集不处理qacc贡献。
它不处理外力与后约束阶段。
它不提供完整休眠或被动力。
它不关闭完整U063或G02。
Windows与T4仍缺本批验收。
本批未注入真实设备失联。
严格新鲜度仅约束本子集。
未来等价入口另行验收。

算法见[冻结RNE源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L1243-L1394)。
代数见[冻结空间代数](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/math.py#L111-L147)。
许可见[第三方说明](../THIRD_PARTY_NOTICES.md)。
参考见[样本说明](../fixtures/rne-bias/README.md)。
时序见[阶段契约](physics-stage-contracts.md)。

## 验收入口

脚本拒绝零用例与参考漂移。
脚本生成逐项日志与JSON回执。
回执保留源码摘要与脏状态。
回执明确标记完整G02未验收。

```bash
bash scripts/test-linux-rne-bias.sh
bash scripts/test-linux-rne-bias.sh --release
bash scripts/test-linux-g01.sh --all-features --full-regression
bash scripts/test-linux-g01.sh --all-features --full-regression --release
```

T4需要真实T4设备零号。
RTX 5080不能替代T4证据。

```bash
bash scripts/test-linux-rne-bias.sh --require-t4
```

G01完整回归新增八项检查。
纯探针回归由168增至176。
全feature回归由176增至184。
既有G01物理逻辑保持不变。
历史报告继续保留原计数。

## 本次验收记录

Linux双模式门禁均通过。
GPU采用NVIDIA RTX 5080。
驱动采用595.99.02。
NVRTC采用12.8.93。
Rust采用1.96.1。
每模式通过八项新GPU检查。
每模式通过七项速度回归。
参考校验通过一项。
CUDA与默认宿主各通过两项。
门禁契约另通过五项。
格式、Clippy与rustdoc均通过。
Clippy与rustdoc拒绝警告。
Git差异检查也通过。

| 样本 | 状态数 | 比较标量 | 最大绝对误差 |
| --- | ---: | ---: | ---: |
| 五组原生参考及三种重力 | 120 | 864 | `1.4651743981630716e-5` |
| 跨块世界批量 | 513 | 5643 | `7.139005901990458e-6` |

双模式得到上述同一误差。
边界断言另行执行。
零自由度另测257个世界。
边界测试覆盖身份与状态写入。
它们也覆盖守卫与非有限量。
阶段测试覆盖分步与完整更新。
快照测试覆盖独占与计划析构。
溢出测试也覆盖恢复能力。

G01双模式完整回归也通过。
每模式通过七项G01检查。
每模式通过98项常驻检查。
全探针每模式通过184项。
全feature宿主各通过271项。
默认宿主每模式通过210项。
每模式比较2084组完整状态。
每模式比较7704548个标量。
低层另比较50512个标量。
G01最大绝对误差保持不变：

```text
1.3855896387748867e-6
```

四份本次回执如下。
回执记录提交前脏状态。
当时HEAD仍指向以下提交：

```text
95a3bd403f210e2dfdd5dc8568fba53237ba920e
```

| 门禁 | 回执路径 | 回执SHA-256 |
| --- | --- | --- |
| RNE debug | `target/linux-rne-bias-hcaveV7M/report.json` | `0fe3b53ea324ee70c7abaefeb781a84e4c5c0c65428766be0ad2a58e3385e689` |
| RNE release | `target/linux-rne-bias-YrDrXcIb/report.json` | `0d4fc964f042b47196c9a229311c3fe5d1923c7496761030d7d9f96164a02f81` |
| G01 debug | `target/linux-g01-nwUdBK3u/report.json` | `750948695331bff63b46c948cafe7adc0bb155c8440f1acee4c1c223a471a740` |
| G01 release | `target/linux-g01-yKcLnMqm/report.json` | `441f98d7e67f87597e741249b95f733e521ea625f484f709cbe37c44b23c93e9` |

两份RNE回执冻结同一源码。
它们也冻结同一参考清单。

| 对象 | SHA-256 |
| --- | --- |
| `src/physics/rne_bias.rs` | `6e10d62346217a80b6fcede61421f75378723a08abd26ebc3599c7a8533daa3a` |
| `src/physics/com_velocity.rs` | `1f2f028b6de65bedf0b141a9df95b881ff679f65f03b2a7f8741cb6289ebea3c` |
| `src/physics/resident.rs` | `d04a2fbc36584c9ba3a5fdcb1cea35ffc5133c1b73a83f17e472c3ffbdab5fea` |
| `src/runtime/transfer/kernel.rs` | `53c550f6162ff3e6fe453704470f25252dd203d48bacb0a72748409fcc3c3b7f` |
| `fixtures/rne-bias/manifest.json` | `6bda0de463c284b1cc6811015d9f18c4b199cd08c2191aae92a837e70634b562` |

T4设备门禁正确拒绝本机。
该命令退出码为1。
它没有执行T4产品验收。
Windows本批验收仍待环境。
完整G02与U063继续计划。
本批不关闭无关历史变更。
