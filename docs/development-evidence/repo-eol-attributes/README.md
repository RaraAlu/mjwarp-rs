# 仓库卫生：补齐 G10@P5 夹具的 eol=lf 属性（fresh checkout 可复现）

- 任务：`2026-10-28-repo-fixture-eol-attributes`（p40，仓库卫生，owner Backend）
- 背景：QA 在 R08-S1 复核中记录的 **O8** —— 本机 `core.autocrlf=true` 且 `.gitattributes` 只覆盖部分
  fixture 目录；`fixtures/g10-p5-primitive-branches/*` 与 `fixtures/g10-p5-mesh-multicontact/*` 没有规则，
  fresh checkout 时会被检成 CRLF，而既有 S3 签名清单按**字节 sha256**复核 → 干净 checkout 下
  `g10_p5_branch_closure_cuda::fixtures_match_s3_signature` 失败（**非 S1 引入**，属既有仓库配置缺口）。

## 1. 改动（只 `.gitattributes`）

只补两行窄规则（连同说明共 +8 行）：

```text
fixtures/g10-p5-primitive-branches/* text eol=lf
fixtures/g10-p5-mesh-multicontact/* text eol=lf
```

- 选择依据：`tests/support/branch_closure_case.rs` 的 `FIXTURE_SHA256` 共 **13** 项，恰好全部落在上述
  两个目录（8 个 `primitive-branches` + 5 个 `mesh-multicontact`）；改动前 `git check-attr text eol`
  对两个目录均为 `unspecified`。
- **未改任何 fixture blob**、未做全仓 renormalize、未动其他已有规则目录
  （`fixtures/precision`、`fixtures/render-bvh`、`fixtures/native-probe` 等）与 `target/`、`.normify/`。
- 加规则后 `git status --porcelain` **仅 `.gitattributes`**（fixture 零 modified，见
  `logs/main-status.txt`）；`git diff --stat` = **1 file changed, +8**。

## 2. 主工作区验证

- `git check-attr text eol -- <13 个签名文件>`：**13/13 `text: set` + `eol: lf`**
  （改动前 13/13 为 `unspecified`；对照 `logs/before-attr-main.txt` / `logs/after-attr-main.txt`）。
- `git ls-files --eol <两个目录>`：**24/24** 为 `attr/text eol=lf` 且 `w/lf`
  （对照 `logs/before-eol-main.txt` / `logs/after-eol-main.txt`）。
- fixture 内容零改动：`git status --porcelain -- fixtures/` 为空、`git diff --stat -- fixtures/` 为空。

## 3. fresh clone 验证（`core.autocrlf=true`）

- 命令：`git -c core.longpaths=true clone --no-hardlinks . target/eol-verify/clone`
  ——`core.longpaths` 只用于 checkout 时 `docs/evidence/**/warp-cache` 的超长文件名，**与 EOL 判定无关**；
  clone 内 `core.autocrlf=true`（日志 `logs/clone-status.txt`）。
- clone HEAD = `d1626e51`（含本规则），`git status` **0 行**（干净）。
- `git ls-files --eol <两个目录>`：**24/24 `w/lf`、24/24 `attr/text eol=lf`**、`w/crlf` = **0**
  （`logs/clone-eol.txt`）；13 个签名文件字节级 **CR=0**。
- 既有 S3 签名测试（CPU 用例，只需 `cuda-probe` 特性编译，不发射内核）：
  `cargo test --locked --features cuda-probe --test g10_p5_branch_closure_cuda fixtures_match_s3_signature -- --nocapture`
  → **1 passed / 0 failed**（`logs/clone-s3-signature-test.log`）。
- **父提交对照（negative control）**：在无规则的 `ae90eb55` 上强制按 `core.autocrlf=true` 重检两个目录，
  得到 **24/24 `w/crlf`**；逐文件 sha256 与 S3 签名清单 **13/13 不一致**；同一批文件在加规则后的
  fresh clone 中 **13/13 一致**（`logs/clone-before-eol.txt`、`logs/sha-compare.txt`）。
  该对照解释 O8 的失败路径，也证明本规则正是缺口所在。

## 4. gate / 检查

- 本任务卡**未登记 gate**（只有 `json` + `forbid_claim` 两项检查）→ **无门禁属预期**；本任务不跑
  fmt/clippy/unit 门禁（只改 `.gitattributes` 与证据）。
- 证据自检：`receipt.json` 有效且 `/task_id` 正确、UTF-8 无 BOM / LF/CR=0、禁词 0。

## 5. 边界与不做

- 不改任何 fixture/code blob、不做全仓 renormalize、不扩大到其他 attr 空缺目录
  （本轮只修 S3 签名清单覆盖的两个目录）。
- 未动 `fixtures/precision`、`fixtures/render-bvh`、`fixtures/native-probe` 等已有规则目录。
- 不声明仓库级 EOL 全量合规；仅完成 QA O8 指出的签名夹具缺口。
