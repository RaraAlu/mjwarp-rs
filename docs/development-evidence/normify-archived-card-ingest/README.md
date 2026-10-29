# normify 历史卡归档入库（方案 A 执行记录）

任务：`2026-10-29-normify-archived-card-ingest`（Arch 2026-10-29 登记；Chief 2026-10-29 22:3x 裁决：批准方案 A 今晚执行，方案 B 只登记 backlog 卡）。
性质：只动 `.normify/changes/`（规划提交 + 本任务提交）与 `.normify/runs/<abandoned-id>/`；不改产品代码、不改 hook、不改 normify 工具。

## 1. 机制事实（读码结论，`D:\Rust\normify-rs@4bc7fba`）

- 规划类提交免任务：新增/修改 `proposed`/`backlog` 卡的迁移分类为 `Move::Planning`，被计入 `planning_paths`，不受 `E commit/no-task` 与 `E commit/scope` 约束（`src/hooks/mod.rs:178-235`）。
- 新卡不能直接为 `abandoned`（`src/hooks/transition.rs:72-80`）；`abandoned → abandoned` 落入兜底报错（`transition.rs:109-111`），因此 **abandoned 卡无法原样新增、也无法事后修订**，只能用「先 proposed 再 abandoned」的两步迁移重建。
- `.normify/runs/<id>/` 只在任务自己的 `allowed_paths` 内（`hooks/mod.rs:37-38`）；abandoned 任务无提交途径，故用本任务 `schedule.files` 携带该目录（`hooks/mod.rs:31-36` + `gates::scope_paths`）。
- 先例：`9499e1f3` 为「单文件、无 trailer、新增 proposed 卡」的规划提交，证明方案 A 的规划提交路径成立。

## 2. 执行记录（提交）

| 步骤 | 提交 | 内容 |
| --- | --- | --- |
| 规划落地 | `8732a60b` | host-quat 审计卡（P3 结论）入库（backlog）；`archive` 子命令提案卡入库（backlog）；remaining 卡与 4 张 P4 缺口卡 note 追加排期/守则；本卡转 proposed 并补 checks/schedule.files |
| 重建步骤 1 | `8253b2eb` | 4 张 abandoned 卡以 `proposed` 快照新增入库（仅 `status`/`closed_at` 两个字段与原始不同，其余字段原样保留） |
| 重建步骤 2 | `93c89e24` | 4 张卡恢复为原始 `abandoned` 字节（`git hash-object` 与原始文件、HEAD blob 三者逐字节相同） |
| 本任务 | 见 receipt `commits` | 本卡 in_progress（含证据）→ verified（门禁 + 收口） |

说明：abandoned 卡的「原始字节」在步骤 1 前备份于 `target/normify-p2-originals/`（临时目录，不入库），步骤 2 用备份恢复；因此 **committed blob == 原始字节**，历史状态只是被重建了两个快照（proposed → abandoned），原始字段（title/intent/acceptance/schedule/note/closed_at/revision/checks 等）一个未丢。

## 3. 4 张 abandoned 卡原始身份（全量记录）

| id | bytes | sha256 | closed_at | revision.before | 原 note 保留 |
| --- | --- | --- | --- | --- | --- |
| `2026-10-28-render-bvh-r05-s1-hfield-host` | 5016 | `e9e6eaef2c911277412d73b0936ef21ab3bee4d0514152ea1fe0e2bab3d03fa8` | `2026-10-28T13:22:04.877Z` | null | 是（含「放弃：…」全文） |
| `2026-10-28-render-bvh-r05-s2-hfield-intersect` | 6222 | `fed40ee9ba610dbfe1374c72450d1c081125750ee2c624fc7d13638c7d3c772a` | `2026-10-28T15:00:04.138Z` | `7ff6abd5dc710cf705d54143792c2983ec5f9275` | 是（含「放弃：…」全文） |
| `2026-10-28-render-bvh-r05-s3b-hfield-empty-mesh-fix-check` | 5643 | `73ff94cf2b5b3018dff448d112df0b18ba83b14a082ec5bc31a64df520647613` | `2026-10-28T21:00:50.713Z` | null | 是（含「放弃：…」全文） |
| `2026-10-28-render-bvh-r09-s2b-e2e-negative-control` | 3413 | `f2658a72ce5f71a0ac59959bf6e904dedb554b3fc15196411d4175f8bc1f330d` | `2026-10-28T12:21:56.523Z` | `cbee11271a1c78595f2c828c3e1abe541a627b2e` | 是（含「放弃：…」全文） |

逐字节一致性（git 对象 id；`orig`=备份、`HEAD`=步骤 2 提交后 blob、`disk`=现盘）：

| id | orig == HEAD | disk == HEAD |
| --- | --- | --- |
| r05-s1-hfield-host | 是（`e18f9747…`） | 是 |
| r05-s2-hfield-intersect | 是（`060f0ea1…`） | 是 |
| r05-s3b-hfield-empty-mesh-fix-check | 是（`994d7e1c…`） | 是 |
| r09-s2b-e2e-negative-control | 是（`92e834c1…`） | 是 |

## 4. abandoned 任务的 runs 目录

`2026-10-28-render-bvh-r05-s2-hfield-intersect` 的 4 份门禁结果随本任务提交入库（`schedule.files` 携带）：

| 文件 | bytes | sha256 |
| --- | --- | --- |
| `clippy-cuda.json` | 460 | `f3a0d3069177fde450439481243b78e52ad18fa60d2db5e54e9897a39619e77a` |
| `clippy.json` | 455 | `dc9a0e48ec9b8911c39f3138486c4718e3f66a95a76a61ab9cd40011f7321881` |
| `fmt.json` | 453 | `bf6908c02af5aac525e01b488b76dea103d7bd8ccf7e1ed57a3fabd51f68296c` |
| `unit.json` | 529 | `02a4ac9ad1c1f4cd61131d83d9563868eea896a604e1c20ad6a9f64aa5f7cd6f` |

## 5. backlog 卡入库（直接规划提交）

- `2026-10-29-render-bvh-host-quat-derivation-audit`（P3 审计结论：无 fix 卡 + host quat→matrix 守则）。
- `2026-10-29-normify-archive-subcommand`（方案 B：`normify archive` 子命令提案，backlog，不实施、动共享工具须用户批准）。
- `2026-10-29-arch-unregistered-sources-remaining` 与 4 张 P4 缺口卡的 note 追加（规划提交 `8732a60b`）。

## 6. 验证

- `normify validate --fingerprints`：错误 0、警告 5（与归档前一致，见 receipt）。
- `normify status`：错误 0、警告 0。
- 归档后工作树不再残留这些历史卡/runs 的未跟踪项；剩余 `git status` 说明见最终报告。
- 未使用 `--no-verify`，未手改 hook，未改 `.normify` 之外状态语义。

## 7. 非声明

- 本卡只做「历史记录入库途径」的方案 A 执行；不宣称 hook 语义变化，也不把两步重建描述为「从未放弃过的卡」——证据如实记录状态快照经两步重建。
- 方案 B 未实施；归档命令、archive 目录布局与 hook 豁免语义均在 backlog 卡中作为提案，动共享工具须用户批准。

## 8. 证据文件

| 文件 | 内容 |
| --- | --- |
| `receipt.json` | 机器可读记录（原卡身份、提交、对象 id、runs 目录、验证） |
| `logs/originals.txt` | 4 张原卡的字段摘要（note/closed_at/revision 全量） |
| `logs/object-ids.txt` | 逐卡 git 对象 id 一致性 |
| `logs/runs-sha.txt` | runs 目录文件 sha |
| `logs/validate.txt` | 归档后 validate 输出 |
