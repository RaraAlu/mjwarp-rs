# 跨会话统一写锁（repo write lock）

Status: adopted 2026-11-03（夜间两次双写者险情后落地）。
Task: `2026-11-03-repo-write-lock-and-window-policy`。
Scope: 所有会写 mjwarp-rs 工作区的会话（主会话、群会话、各 Bot 会话），窗口执行者与材料作者都在内。

## 1. 为什么需要它

2026-11-03 夜间出现两次「双写者」险情：两个会话（同一 Bot 的两个对话）同时写同一棵工作树，
其中一个在另一个改到一半时执行了 `normify done` 并把在途改动一并 `git add -A` 提交。两次都靠事后
逐条核对才确认没有把别人的文件带进提交，但风险是真的：可能把半成品提交、把别人的卡状态改掉、
或让窗口跑在未提交的测试支撑改动上（perworld S1 的 `SHA_DRIFT` 作废轮就是这么来的）。

已有的 `<out-dir>/.window-lock` 只解决「同一窗口目录被两个执行者同时跑」，管不到普通的仓写。
本卡的写锁补上另一半：**写仓前先拿锁，拿不到就停手**。

## 2. 机制

脚本：`scripts/validate/repo-write-lock.ps1`（Windows PowerShell 5.1 兼容，纯 ASCII）。
锁文件：`<repo>/.normify/locks/repo-write.lock`，键值行 `pid/owner/host/started/expires/reason`。

| 动作 | 行为 | 输出 / 退出码 |
| --- | --- | --- |
| `-Action acquire` | 锁空闲或已过期 → 写入锁；同一 owner → 续期；别的 owner 持有 → 拒绝，且**不改锁文件** | `REPO_WRITE_LOCK_ACQUIRED` / `REPO_WRITE_LOCK_RENEWED` / `REPO_WRITE_LOCK_STALE_CLEARED`，拒绝时 `REPO_WRITE_LOCK_HELD`，exit 17 |
| `-Action release` | 只有 owner 相同才删 | `REPO_WRITE_LOCK_RELEASED`，非属主 `REPO_WRITE_LOCK_NOT_OWNER`，exit 18 |
| `-Action status` | 只读，打印持有者与年龄，或 `REPO_WRITE_LOCK_FREE` | exit 0 |
| `-Action break` | 强制清除（必须给 `-Reason`） | `REPO_WRITE_LOCK_BROKEN`，无原因 `REPO_WRITE_LOCK_BREAK_NEEDS_REASON`，exit 19 |

### 2.1 为什么用 TTL 而不是 pid 判活

`normify`/shell 的每个命令都是短进程，`acquire` 的进程在返回时立刻退出。若按 pid 判活，
正在写作的会话会被误判成「持有者已死、锁已释放」，机制直接失效。所以判活以 `expires` 为准
（默认 90 分钟，`-TtlMinutes` 可调），pid 只作为排障信息记录；持有期间每次写命令前重新
`acquire` 即续期。

这条与卡片 acceptance 原文（「持有者 pid 存活时第二次 acquire 拒绝、pid 已死则清除」）有意偏离：
等价目标（同一时刻只允许一个写者、存活锁不许被抢）由「未过期即拒绝 + 过期才清」实现，且比 pid 判活更保守
（死 pid 不能证明会话已停止写入）。**该偏离记录在此，不改验收口径**。

### 2.2 与窗口锁的分工

| 锁 | 粒度 | 保护什么 |
| --- | --- | --- |
| `repo-write.lock` | 整个仓库 / 会话 | 任意仓写（代码、证据、材料、提交） |
| `<out-dir>/.window-lock` | 单个窗口输出目录 / 进程 | 同一窗口目录不被两个执行者同时跑（窗口步骤、采样、冻结） |

窗口执行者必须先持有写锁，再进入窗口流程；两层都要过。

## 3. 使用（会话开头 / 结尾）

```powershell
powershell -NoProfile -File scripts/validate/repo-write-lock.ps1 -Action acquire -Owner arch-chat -Reason "night CPU slice"
# ... 写代码 / 证据 / 材料 / 提交 ...
powershell -NoProfile -File scripts/validate/repo-write-lock.ps1 -Action release -Owner arch-chat
```

- 拿不到锁（exit 17）：立即停手，把 `REPO_WRITE_LOCK_HELD` 的 owner/age 报给 Chief，不抢、不 `break`。
- 锁被误留（持有会话已确认停写）：由 Chief 或持有者用 `-Action break -Reason "..."` 清除并留痕。
- 群会话在写窗期间只读；要写先 `acquire`，失败就不写。

## 4. 演练（`drill.ps1`）

`drill.ps1` 在本机 Windows PowerShell 5.1 下跑 11 项断言，全部通过（`DRILL_OK`），结果见
[`drill.log`](drill.log)：

1. PS 5.1 解析 0 错误；2. 初始 `FREE`；3. A 取锁成功；4. B 取锁被拒 exit 17 且锁文件字节不变；
5. A 续期；6. B 释放被拒 exit 18 且锁仍在；7. A 释放；8. 释放后 `FREE`；
9. `TtlMinutes=0` 后 B 看到陈旧锁并取得；10. `break` 无原因 exit 19、有原因成功；11. 收尾 `FREE`。

演练只碰 `.normify/locks/repo-write.lock`，不碰 git index，不跑 cargo、不占 GPU。

## 5. 边界

- 本卡只加 harness 脚本与口径文档：`src/`、`tests/`、`*.cu`、判据/容差/夹具零改动。
- 写锁是协作约定 + 机器拒绝，不是内核级互斥：任何会话都可以绕过它直接写，但绕过即违反口径，
  并且 `window-execution-policy.md` 把它列为窗口前置。
- 锁文件是运行时状态，不应进版本库。本卡 scope 不含根 `.gitignore`，因此本克隆用 `.git/info/exclude` 加了 `/.normify/locks/`（本地忽略，所有会话共享同一克隆即生效）；把该规则落到仓库 `.gitignore` 留作后续治理项（需要一张 scope 含 `.gitignore` 的卡）。
