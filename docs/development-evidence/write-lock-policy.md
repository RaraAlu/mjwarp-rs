# 写锁口径（跨会话单写者）

Status: adopted 2026-11-03（夜间两次双写者险情后）。
Applies to: 所有写 mjwarp-rs 工作区的会话——主会话、群会话、各 Bot 会话；窗口执行者、材料作者、证据作者都在内。
实施：`scripts/validate/repo-write-lock.ps1`；机制与演练见 [`write-lock/README.md`](write-lock/README.md)。

## 1. 规则

1. **写仓前先持锁**：任何写动作（改代码、写证据、改窗口材料、`git add` / `git commit`、`normify start/gate/done`）
   之前先 `acquire`，拿不到（exit 17 `REPO_WRITE_LOCK_HELD`）就停手，把持有者与年龄报给 Chief。
2. **一个会话同一时刻只持一把锁**，owner 用会话名（如 `arch-chat`、`arch-group`、`backend`、`qa`），不混用。
3. **群会话在写窗期间只读**：群里的留痕贴是记录，不是执行信号；要写先按第 1 条取锁。
4. **不抢锁、不 `break`**：存活锁只能由持有者 `release`；确认持有会话已停写后，才允许
   `-Action break -Reason "..."` 清除，并把原因写进当天记录。
5. **写窗结束就释放**：`release` 后 `status` 必须为 `REPO_WRITE_LOCK_FREE`；异常退出也要补释放。
6. **提交用显式路径**：持锁会话的提交一律 `git commit -- <路径>`（或先 `git add` 只加自己的路径），
   禁止裸 `git add -A` / 裸 `git commit` 把别人的在途改动带走。
7. **写锁先于窗口锁**：窗口执行者先持写锁，再进 `<out-dir>/.window-lock` 流程；两层都过。

## 2. 与窗口锁的关系

| 锁 | 粒度 | 保护 |
| --- | --- | --- |
| `.normify/locks/repo-write.lock` | 整仓 / 会话 | 任意仓写（代码、证据、材料、提交） |
| `<out-dir>/.window-lock` | 单窗口目录 / 进程 | 同目录不被两个执行者同时跑 |

窗口期间的第二执行者、群会话误起跑，都被第二层挡住；普通写窗冲突由第一层挡住。

## 3. 故障处置

- **锁残留**：持有者崩溃或忘记释放 → 等 TTL（默认 90 分钟）自然过期后 `acquire` 会打印
  `REPO_WRITE_LOCK_STALE_CLEARED` 并接管；急用则由 Chief 或持有者 `break` 并留痕。
- **锁内容损坏**：`expires` 解析失败按「已过期」处理，`acquire` 会清陈旧锁接管。
- **两个会话同时 `acquire`**：先写成的持有，另一个 exit 17 并停手；这是预期行为，不是错误。
- **锁文件进版本库**：`.normify/locks/` 是运行时状态，不提交；本克隆已在 `.git/info/exclude` 加入 `/.normify/locks/`（当前卡 scope 不含根 `.gitignore`，仓库级规则待后续治理卡）；
  若发现它出现在 `git status`，先补忽略规则再继续，不要提交它。

## 4. 与「干净树」的关系

窗口前置要求 worktree clean。写锁文件若未被忽略会污染 `git status`，因此忽略规则是机制的一部分；
`release` 之后 `git status` 必须干净（除本卡正在写的路径）。
