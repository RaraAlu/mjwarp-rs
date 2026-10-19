# 接入Rust版normify

任务：`2026-10-19-normify-rust-cutover`（旧版流程最后一次交付）。
基线：`3379e108225741a03ba33c657dcaa09955c1aae2`。
主机：Rara（Windows）。

## 工具

- normify-rs提交：`f358414d7dc32d1d64d6179f4bacd07adec1c8de`。
- 二进制：`C:\Users\zhang\.cargo\bin\normify.exe`，由`cargo install --path . --offline --locked`安装。
- normify-rs测试（f358414）：单元23项、端到端7项、旧版对照5项全部通过。

## 导入

命令：

```text
normify import --from C:\Users\zhang\.codex\normify\normify-mjwarp-rs --name mjwarp-rs
```

输出：

```text
导入完成：模块60 任务300（逐字节一致241） 布局6 规则有
指纹：换算59 漂移0
未导入：development-order.md，development-order.mmd，development-overview.mmd，initialization-report.md，reports，schedule-20261012-021248.html，schedule-integrity.json，schedule-registration-report.json，schedule-registration.json，schedule.mmd
```

- 59个模块的旧指纹与当前源码一致，已换算为基于Git blob的新指纹。
- 58个任务文件只有键顺序不同；值经对照测试确认一致。
- 生成产物与旧报告未导入；旧结构目录保持不动。

## 本次调整

- `config.toml`：提交类型与禁止的署名尾部字段。
- `gates.toml`：fmt、clippy、unit三项门禁。
- 124项10-15前登记且未就绪的候选任务转为待办。
- 新增切换任务`2026-10-19-normify-rust-switch`，由下一个提交按新流程完成。
- AGENTS.md：改写Normify伴随开发与进展汇报两节，结构表新增`.normify/`。

## 校验与状态

```text
normify 0.7.0
校验：错误0 警告2
  W structure/leaf-too-coarse 18 个叶子可能过大（多文件或超220行），建议拆分
  W evidence/fingerprint-drift 1 个模块指纹已过期，如 mjwarp
normify mjwarp-rs · 60 模块 · 301 任务
任务：已验收140 运行0 就绪15 阻塞17 待办124 放弃5 未排0
冲突226 · 错误0 · 警告0
运行：-
就绪：2026-10-19-normify-rust-switch 切换到Rust伴随流程(p0)；2026-10-18-layers-m3 M3 安全包装同步迁移(p1)；2026-10-18-layers-m7-headers M7 内核来源与头文件评审(p1)；2026-10-18-layers-m7-link M7 动态链接评审(p1)；2026-10-18-layers-m7-tile M7 共享内存发射评审(p1) +10
下一波：4 项，共 15 波
最近验收：2026-10-19-t4-resident-source-drift；2026-10-19-t4-input-precision-source-drift；2026-10-19-t4-gravity-chain-live-rerun
```

根模块漂移来自本次AGENTS.md改动；切换任务关闭时刷新。

## 未覆盖

- Linux主机未构建或运行normify。
- 架构图渲染尚未实现。
- 本次未运行mjwarp产品测试；产品代码未改动。
