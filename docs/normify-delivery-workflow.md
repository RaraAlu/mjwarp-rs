# 排程拆分与交付同步

日期：2026-10-16。
本次只修复治理流程。
产品物理代码保持不变。

## 迁移结果

旧G01任务混入多类功能。
长备注掩盖了独立交付。
本次保留全部历史内容。
系统禁止运行态回退。
因此终止旧执行容器。
终止不表示通过验收。
新总门禁承接全部旧条款。
G01前置改指新总门禁。
其余正式前置全部保留。
P10保持原候选态。

历史节点共有21项。
其中RNE节点只确认取证。
它仍保留1042块超差。
其他节点也只确认所列子集。
失败与未测项保持原记录。
三项既有决议不重复建项。
剩余工作另建12项。
原始基线为提交`8a5bac0`。
当前捕获未提交项不计完成。

详见[机器账本](development-evidence/schedule-reconciliation.json)。
原状见[迁移快照](development-evidence/schedule-before-2026-10-16.json)。

## 后续顺序

1. 核对旋转变体与提交。
2. 补齐全路径精度类型。
3. 补齐当前内核证据。
4. 完成双模式数值复验。
5. 实测公平性能与显存。
6. 最后核验完整T4门槛。

前两项可先做独立核查。
共享文件仍要求串行写入。
GPU任务仍采用独占资源。
系统波次不承诺完工日期。

功能开发另走独立支线。
G02先处理RNE剩余超差。
G03补完整分支与精度。
U038先补共享与同代状态。
随后补独立参考及归约。
三维状态也有独立任务。
G26另补完整能量链。
各正式阶段保留原前置。

## 强制交付流程

编码前建立独立交付节点。
节点登记实际文件与资源。
先满足前置，再启动节点。
总门禁不能代替交付节点。
历史核验节点不能接新代码。

交付分为三个状态。
三个状态不得互相替代。

| 状态 | 仓库记录 | 完成条件 |
| --- | --- | --- |
| 实现完成 | `milestones.implementation` | 本次范围已落地 |
| 验证通过 | `milestones.validation` | 范围验证已通过 |
| 交付记录完成 | Normify关闭记录 | 完整SHA已回写 |

提交前不知道自身SHA。
因此回执先写`pending_commit`。
提交后再完成交付记录。

先运行任务预检：

```powershell
pwsh -NoProfile -File scripts/check-normify-delivery.ps1 -Mode Preflight -TaskId <task-id>
```

预检核对任务仍在进行。
预检核对验收条件存在。
预检核对文件范围存在。
预检核对回执路径绑定。
预检核对前置均已完成。
预检核对上次交付收尾。

提交前新增JSON交付回执。
目录采用以下固定位置：

```text
docs/development-evidence/changes/<task-id>.json
```

回执使用`schema: 1`。
`task_id`引用运行中的节点。
`kind`采用`delivery`或`governance`。
`milestones`登记三项状态。
实现采用`complete`。
治理可用`not_applicable`。
验证必须采用`passed`。
交付先用`pending_commit`。
`files`逐项登记路径与暂存对象。
删除项的`blob`写为`deleted`。
其他`blob`采用Git对象标识。
回执自身不参与递归摘要。
`validation`登记命令和结果。
每项也登记真实证据文件。
`remaining`列出未验收边界。
任务备注登记如下绑定：

```text
delivery-receipt: docs/development-evidence/changes/<task-id>.json
```

先暂存目标文件，再取对象。
可用`git rev-parse :<path>`。
最后暂存回执并运行检查。
钩子只读取暂存内容。
它拒绝漏项及陈旧对象。
它拒绝缺任务或缺验证。
它也检查实际任务前置。
它不验证数值结果的真伪。
审查者仍须核对原始证据。

```powershell
pwsh -NoProfile -File scripts/check-normify-delivery.ps1 -Mode Staged
```

提交后先核验提交对象：

```powershell
pwsh -NoProfile -File scripts/check-normify-delivery.ps1 -Mode PostCommit -TaskId <task-id>
```

此时任务必须仍可写。
核验读取真实HEAD。
核验联合全部回执范围。
核验拒绝重复文件归属。
核验重查提交内证据。
删除项必须真的不存在。
核验输出完整SHA。

随后更新任务备注。
备注登记完整SHA。
任务仍保持运行状态。
再核验登记结果：

```powershell
pwsh -NoProfile -File scripts/check-normify-delivery.ps1 -Mode Recorded -TaskId <task-id>
```

登记核验通过后关闭任务。
关闭后执行最终核验：

```powershell
pwsh -NoProfile -File scripts/check-normify-delivery.ps1 -Mode Final -TaskId <task-id>
```

最终核验要求任务已关闭。
最终核验要求完整SHA。
最终核验要求修订一致。
下次提交前须完成上次收尾。
钩子拒绝悬挂的上次交付。
检查失败时不得跳过钩子。
不得用改名回避验收。
不得使用`--no-verify`。
关闭失败时必须停止。

已关闭任务不可补写。
正式接口会返回`change/closed`。
此时不得直接修改原数据。
应新建关联纠正任务。
纠正记录保留原任务ID。
纠正记录保留完整SHA。
纠正记录保留真实错误。
纠正记录保留绕过事实。
纠正任务也走正常钩子。

## 安装与限制

本仓库安装以下配置：

```powershell
git config --local core.hooksPath .githooks
git config --local normify.directory C:/Users/zhang/.codex/normify/normify-mjwarp-rs
```

其他机器应设置真实目录。
工具要求PowerShell 7。
缺服务目录时检查失败。
钩子不修改Normify数据。
所有排程写入仍走MCP工具。
钩子不自动关闭验收任务。
新克隆须安装上述配置。
人工仍可绕过本地钩子。
因此本机制不是绝对保证。
交付时必须报告回执与状态。

历史迁移可另行核验：

```powershell
pwsh -NoProfile -File scripts/check-normify-delivery.ps1 -Mode Audit
pwsh -NoProfile -File scripts/test-normify-delivery.ps1
```

历史核验只检查冻结迁移。
它不运行任何物理测试。
后续任务推进会改变状态。
届时应保留本轮核验报告。
