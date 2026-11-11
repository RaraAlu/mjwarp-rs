# W1 p20 清理项：关闭 `2026-11-05-render-r06-flex-multiframe-recipe-params-backlog`（治理片证据）

- 治理卡：`2026-11-11-w1-r06-recipe-params-card-closure`（owner Backend）；被关闭卡：`2026-11-05-render-r06-flex-multiframe-recipe-params-backlog`（原 status=backlog / checks=[]）。
- 批准：Chief 2026-11-11 批准「由治理片按既有实现关闭」。
- 关闭依据：既有测试侧实现 + 本片逐条 acceptance 核对 + CPU 测试复跑；**不改代码/判据/参考/容差/夹具**。

## 1. 关闭依据（实现提交与证据）

| 提交 | 内容 |
| --- | --- |
| `20015625` | 28 项口径 `assert_params_match_capture`（26 recorded + nworld；cam_res 名义项）+ `RECIPE_PARAM_KEYS` + 防漏守卫 + 4 个 CPU 用例（`tests/support/render_bvh/multiframe_recipe.rs`） |
| `8618e447` | 新增键负例 `use_textures`（证明 21 项新增断言非空转） |
| `a20efe2a` | 卡 L 注释措辞修正（cam_res 口径；注释 only） |

- 范围核查（`w1-r06-recipe-params-resolution`）：实现提交只动 `.normify/`、`docs/development-evidence/`、`tests/`；
  `src/`/`fixtures/`/`scripts/` 零改动；未触碰产品 recipe 常量/判据/容差；未涉及 eol/.gitattributes。

## 2. acceptance 逐条核对

| # | acceptance（原文） | 核对 | 满足 |
| --- | --- | --- | --- |
| 1 | 不排期：本卡只作为已知项记录，不启动、不产生验收结论；触发条件=验证窗或后续审计暴露 recipe 参数漂移风险，由 Chief/用户决定是否补齐。 | 实现已补齐，本卡不再作为「已知项待决」存在；无排期需求 | ✅ |
| 2 | 若补齐：把实体 A 的 recipe 比对补齐到与单帧验收同口径的 28 项（差异逐项列出），零 GPU 的 CPU 自证即可覆盖；不得改写已签 spec/判据/参考，不得改变 G-2 夹具。 | 28 项口径已在 `multiframe_recipe.rs` 落地（26 recorded + nworld + cam_res 名义；实际 27 项断言 + 防漏守卫）；CPU 自证 4 用例通过；diff 面仅 .normify/docs/tests | ✅ |

- 原卡 `checks=[]`（无机器检查项）；本片为该卡补 3 条机器检查（json 指向本证据 receipt / acceptance / 上一证据片），
  满足后执行 `normify done` 关闭。

## 3. 测试复跑（CPU-only，零 GPU）

```
cargo test --locked --features cuda-probe --test render_flex_multiframe recipe -- --test-threads=1
```

完整输出见 `logs/test-output.txt`；摘要：`test result: ok.`（5 passed / 0 failed / 37 filtered；含 4 个 multiframe_recipe CPU 用例 + 1 个 filter 命中的 s7b1 用例）。

## 4. 非声明与边界

- 不改代码/判据/参考/容差/夹具；不宣称渲染数值/等价/发布结论；不宣称任何门禁通过。
- 零 GPU、零窗口；本片只做治理关闭与证据登记。
