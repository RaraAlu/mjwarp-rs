# S1 QA 观察 O1–O11 收口（证据勘误）

任务：6-10-27-unc3-branch-closure-s1-observation-closure（owner Arch）。触发：S1 独立复核 aec54a1（pass / 0 阻断 / 观察 11）。

逐条处置见 S1 README §7 表与 eceipt.json.observation_closure。要点：O1 U3 pose 5/8→8/8（含 Rx(180)）；O2 现有 capsule case 已是退化帧基，改补横躺 case；O3 U5 真增量需深穿透 ~0.17；O4 hfield-capsule 参考 9712 B + 位级优先/4ULP 兜底；O5 pinned 容量出处改 collision_convex.py L1262-1289 + 	ypes.py L1661-1662；O6 S3 显存按 M3 捕获窗实测 delta 238 MiB；O7/O8 U2 补 f32 溢出注记、证据=静态 diff（加测试须标镜像）；O9 行号精修；O10 S3/S4 files/实体口径补齐；O11 docs-only QA 的 gate 口径写入排程清单 §7.3。

未改动：代码/判据/容差/ABI/fixtures；未跑 GPU/未捕获；S1 结论方向（U1/U3/U4/U5/U6/U9 可验证、U2 不可达防御、U7/U8 仅限制存在）不变，只勘误 pose/构造/数字/出处/行号。
