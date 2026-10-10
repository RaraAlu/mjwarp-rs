# Linux常驻空间速度子集

日期：2026-10-09。
范围：G02首批刚体速度子集。
完整G02仍待实现。

2026-10-10补充：
速度链新增私有f64工作区。
它复用既有模型与状态上传。
公开输入与结果仍采用f32。
下文保留首批历史证据。
最新复验见[RNE报告](linux-resident-rne-bias.md)。

## 实现与接口

GPU计算cvel与cdof_dot。
六维顺序先角向再线向。
公开输入与结果采用f32。
位置与质心沿用严格常驻底座。
计划只上传一次模型。
计划只编译一次内核。
状态更新不回读宿主中间量。
显式回读检查前置结果与守卫。
默认qpos采用既有qpos0。
默认qvel全零。
默认结果保持未就绪。

```text
ComVelocityPlan::new -> create_data
write_qpos / write_world_qpos
write_qvel / write_world_qvel
ComVelocityPlan::update -> ComVelocityData::readback
ComVelocitySnapshot::world -> cvel / cdof_dot
```

入口需要cuda-probe与NVRTC。
默认构建不需要GPU工具链。
产品链没有CPU回退。
产品测试不启动Python。
本批没有新增第三方依赖。

## 冻结语义与边界

内核复用冻结com_vel顺序。
自由关节先累计平移速度。
三个旋转导数使用同一前置速度。
球关节也先计算三个导数。
静态体继承父体速度。
世界体速度保持零。
模型与状态检查沿用严格子集。
计划拒绝同尺寸跨模型数据。
输入错误保留已有速度结果。
合法写入废弃整组速度结果。
设备更新失败不发布结果。
显式回读拒绝越界与非有限量。
快照独占宿主结果。
状态与缓冲保留设备会话。

本子集不实现RNE偏置力。
它不实现子树速度汇总。
它不处理完整休眠或柔体动力学。
它不提供逐世界模型参数。
它不提供完整fwd_velocity。
它不关闭U058或完整G02门禁。
Windows与T4仍缺本批验收。
本批未注入真实设备失联。

算法见[冻结速度源码](https://github.com/google-deepmind/mujoco_warp/blob/71da24d956378a87a703b6e1442b13aec0c4ac29/mujoco_warp/_src/smooth.py#L2363-L2457)。
许可见[第三方说明](../THIRD_PARTY_NOTICES.md)。
参考见[样本说明](../fixtures/com-velocity/README.md)。

## 验收入口

脚本拒绝零用例与参考漂移。
脚本生成逐项日志与JSON回执。
回执保留源码摘要与脏状态。
回执明确标记完整G02未验收。

```bash
bash scripts/test-linux-com-velocity.sh
bash scripts/test-linux-com-velocity.sh --release
```

T4需要真实T4设备零号。
RTX 5080不能替代T4证据。

```bash
bash scripts/test-linux-com-velocity.sh --require-t4
```

G01完整回归新增七项速度检查。
纯探针回归由161增至168。
全feature回归由169增至176。
既有G01物理逻辑保持不变。
历史报告继续保留原计数。

## 验收记录

本次通过Linux双模式门禁。
GPU采用NVIDIA RTX 5080。
驱动采用595.99.02。
NVRTC采用12.8.93。
Rust采用1.96.1。
每模式通过七项新GPU检查。
参考校验通过一项。
CUDA与默认宿主各通过两项。
格式、Clippy与rustdoc均通过。
Clippy与rustdoc拒绝警告。
Git差异检查也通过。

| 样本 | 状态数 | 比较标量 | 最大绝对误差 |
| --- | ---: | ---: | ---: |
| 五组原生参考 | 40 | 2976 | `4.124216822276594e-6` |
| 跨块世界批量 | 513 | 52326 | `1.7145784019234611e-6` |

双模式得到上述同一误差。
本批没有放宽数值容限。

```text
2e-5 + 2e-5*abs(reference)
```
边界测试覆盖输入与模型身份。
它们也覆盖守卫与非有限结果。
零自由度测试覆盖257个世界。

G01双模式完整回归也通过。
每模式通过七项G01检查。
每模式通过98项常驻检查。
全探针每模式通过176项。
全feature宿主每模式通过267项。
默认宿主每模式通过207项。
G01各比较2084组完整状态。
每模式比较7704548个标量。
G01最大绝对误差保持如下值：

```text
1.3855896387748867e-6
```

实际执行以下完整回归：

```bash
bash scripts/test-linux-g01.sh --all-features --full-regression
bash scripts/test-linux-g01.sh --all-features --full-regression --release
```

四份本次回执如下。
回执记录提交前脏状态。
当时HEAD仍指向以下提交：

```text
05ac2c467b04ca943729c0e1f1b83a46f3cc890e
```

| 门禁 | 回执路径 | 回执SHA-256 |
| --- | --- | --- |
| 速度debug | `target/linux-com-velocity-JMRUrdeh/report.json` | `e141e6ee5ac20e75e36533117cba78acf749210d3289141fc7ffd56b59f7b20a` |
| 速度release | `target/linux-com-velocity-2q8lQZBq/report.json` | `b520649f8526b5944d9c0be32c83f02a96504d814682153de1d3bb4ec29b4ea6` |
| G01 debug | `target/linux-g01-Z2kTMr95/report.json` | `a1ede9032bb211cf66f3fe3d4a1fb4e2da0866d191160713349385eb93f37baf` |
| G01 release | `target/linux-g01-ec90k0P2/report.json` | `67dd985c371ee4730d3190e3dbe8002f45d425e3aa335f1f8f23c3d6601fbefc` |

两份速度回执冻结同一源码。
它们也冻结同一参考清单。

| 对象 | SHA-256 |
| --- | --- |
| `src/physics/com_velocity.rs` | `96be588f3bafa736b9231ff8e327230618a5539843a8a278b9347d8838d3bc38` |
| `fixtures/com-velocity/manifest.json` | `82f73f1c5c7ce5490c01cc54f6db8d1e62663ae684261ec9b9b3461192b2d0e1` |

T4负向门禁按预期拒绝本机。
该检查返回退出码1。
它不构成T4正向验收。
Windows与T4仍缺速度复验。
完整G02与RNE仍待实现。
