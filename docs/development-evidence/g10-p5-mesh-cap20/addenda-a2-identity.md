# A1 身份 supersession（append-only 追加；A1 正文与历史字节不改）

- 任务：`2026-11-12-w2-a2-2-identity-refreeze`（A2-2，0 窗、CPU-only）；本文件只做 A1 身份表的 supersession 声明。
- 被说明文件：`docs/development-evidence/g10-p5-mesh-cap20/README.md` §3「内核身份」与 `logs/nvrtc-identity.txt`（A1，`2026-10-31-g10-p5-mesh-cap20`，提交 `307219fe`）。

## 1. 身份链

| 状态 | collision 组装源 sha256 | source bytes | PTX bytes | 入口 | 说明 |
| --- | --- | --- | --- | --- | --- |
| A1 前 | `9b7435ab…` | 254,005 | 3,883,740 | 6 | A1 测量，已被 A1 取代 |
| **A1** | `12d50d68…` | 254,263 | 3,883,769 | 6 | A1 证据（本目录 README §3 / `logs/nvrtc-identity.txt`）；**A2 后作废** |
| **A2（现行）** | `fb03c5b1…`（全值见 A2-2 证据 `logs/nvrtc-identity.txt`） | **254,434** | **3,883,643** | 6 | A2 容量 32/32 后实测（NVRTC 12.8.93、LF worktree、CPU-only）；PTX sha `78da9dea…` |

## 2. 效力

- 自 A2 验证申请起，**A1 的 254,263/`12d50d68` 与 3,883,769 不再作为现行身份**；任何新的 GPU 窗口必须使用 A2 身份的新冻结，
  旧 guards/旧 exe 一律不复用（A1 README §3/§5 的「后续窗口须新冻结」条款继续有效）。
- 本文件只做 supersession 声明：**不改写** A1 README 正文、receipt、logs 与既有 manifest 条目（manifest 追加本文件一条，历史条目字节不变）。
- 不宣称数值/行为变化：A2 容量只影响 >20/10 的 mesh 拓扑路径；已验收的单帧/族级结论仍绑定各自窗口内核身份，不自动继承到新内核。

## 3. 边界

- 本文件不是验收依据；不宣称 U7/U8 通过、mesh 容量缺口已清、与上游等价（over-32）。
