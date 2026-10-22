# 开发总览：仓库内 Normify 数据适配器

任务：`2026-10-22-dev-overview-repo-adapter`
结论：repo 模式可用，22 项自动测试通过，53643 看板已切到仓库实时数据；dsh 模式一条命令可切回。

## 背景

原看板经外部 `dsh-normify` 引擎读 `~/.codex/normify` 旧快照（10-19，300 任务），看不到仓库内进度。
仓库 `.normify` 与 dsh 引擎格式不兼容：模块文档用 `+++` TOML frontmatter，变更文件含 `checks`、`closed_at` 与 `backlog` 状态。
因此新增自带读取器，直接读仓库数据，不改 `.normify` 数据，也不走引擎。

## 实现

| 文件 | 改动 |
| --- | --- |
| `scripts/development-overview/repo-reader.mjs` | 新增：读 `.normify/config.toml` 与 `.normify/changes/*.json`，输出与 `api/schedule` 相同契约 |
| `scripts/development-overview/server.mjs` | 增加 `--source repo`（默认）与 `--source dsh`；dsh 仍用 `--normify-server`，保留 `--normify-root` 语义 |
| `scripts/development-overview/test.mjs` | 新增 8 项测试：readiness、字段映射、缺依赖、冲突分波、空目录、坏文件、配置、回环服务 |

`RepoReader.call('normify_change_list', …)` 与 `NormifyReader` 同调用面，因此 server 路由与校验逻辑未分叉。

契约字段：`schema_version`、`ok`、`tasks`、`ready`、`running`、`blocked`、`unscheduled`、`completed`、`waves`、`conflicts`、`errors`、`warnings`、`mermaid`。
每个任务：`id`、`title`、`status`、`phase`、`owner`、`depends_on`、`modules`、`files`、`resources`、`priority`、`readiness`、`blockers`。

## 字段映射

| 仓库字段 | 排程字段 |
| --- | --- |
| `title` | `title` |
| `status` | `status` |
| `schedule.phase` | `phase` |
| `schedule.owner` | `owner` |
| `schedule.depends_on` | `depends_on` |
| `schedule.files` | `files` |
| `schedule.resources` | `resources` |
| `schedule.priority`（缺省 50） | `priority` |
| `modules.modify` | `modules` |

`checks`、`closed_at`、`intent`、`revision` 等仓库字段不进入任务输出。
坏文件只记 `schedule/change-unreadable` 警告并跳过；空目录直接报错，不返回空排程冒充正常。

## readiness 规则

- `verified` / `abandoned` / `in_progress` 原样。
- `backlog`（待办）记为 `unscheduled`，不计入 `ready` / `blocked`。
- `proposed` 且前置全部为 `verified`（或前置任务不存在）→ `ready`。
- 其余 `proposed` → `blocked`，`blockers` 写 `dependency:<id>` 或 `conflict:<id>`。
- 候选任务与运行任务争用写入范围或独占资源时也记为阻塞，与 dsh 契约一致。
- 前置任务不存在时按已满足处理，并记 `schedule/dependency-missing` 警告。

## 已知差异与限制

- 冲突只按 `schedule.files` 与模块名判定，未展开模块源码路径。
  仓库 `normify status` 当前为 88 冲突 / 1 就绪 / 14 阻塞；本看板为 79 冲突 / 2 就绪 / 13 阻塞。
  差异来自 `2026-10-18-linux-toolchain-validation`：它声明 `mjwarp` 模块，仓库按模块源码展开后与运行任务冲突，本看板按声明判定为 `ready`。
- 属只读工具：不写任务文件，不派发任务，不承诺并行安全。
- dsh 模式仍读旧快照（300 任务），只作回退，不修其数据。

## 验证

环境：Windows，Node v24.14.1，仓库 `D:\Rust\mjwarp-rs`。

```text
node scripts/development-overview/test.mjs
# 22 pass / 0 fail（原有 15 项 + 新增 7 项）
```

真实仓库数据：

```text
任务 404；verified 252；in_progress 2（本任务与 g01-all-probes）；ready 2；blocked 13；unscheduled 124；
2026-10-22-* 共 36 项；2026-10-18-layers-m3 为 verified；errors 0；warnings 0。
```

HTTP 实测（repo 模式）：

```text
GET /api/schedule → 404 任务，头部提交 76db5fedb5a9
GET /api/task?id=2026-10-18-layers-m3 → 4 条验收；回执状态「已读取；未复验」
```

dsh 模式回归（临时端口）：

```text
node scripts/development-overview/server.mjs --source dsh --normify-server C:/Users/zhang/Documents/Work/Personal/dsh-normify/src/stdio.mjs
GET /api/schedule → 300 任务，errors 0；原行为未变。
```

## 看板

当前地址（base path 每次重启随机）：

```text
http://127.0.0.1:53643/7862fefe6ea668df4700362cd5e984c30ae2c4695169fb7e/
```

repo 模式启动：

```text
node scripts/development-overview/server.mjs --source repo --port 53643
```

一条命令切回 dsh 模式：

```text
node scripts/development-overview/server.mjs --source dsh --normify-server C:/Users/zhang/Documents/Work/Personal/dsh-normify/src/stdio.mjs --port 53643
```

## 门禁与缺口

登记检查：`gate fmt`、`gate unit`、`unchanged`（src/tests/fixtures/native/include/build.rs/Cargo.*/rust-toolchain）、回执 `json`、`forbid_claim`。
门禁结果见 `.normify/runs/2026-10-22-dev-overview-repo-adapter/`。

未做浏览器点击回归：本机无浏览器自动化工具，仅验证 HTTP 契约与首页资源；界面交互仍需人工点开一次确认。
未运行产品物理、GPU 与 Linux 验收；本次只改只读 Node 看板，产品精度接口不在范围。
