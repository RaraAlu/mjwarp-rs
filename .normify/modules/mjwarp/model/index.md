+++
uid = "b1afcc5a"
id = "mjwarp.model"
state = "planned"
tags = ["planned", "engine", "model"]
fingerprint = "f50ba14e28fa9fbc2af1bf0c1ef79f2d3f62ef2872f78263dd8ffb3920e4f75e"

[name]
zh = "模型与状态布局"
en = "Model and Data layouts"

[description]
zh = """
规划完整模型与状态。柔体边输入已有子集。完整模型仍待实现。
"""
en = """
Planned full model with checked flex positions and edges.
"""

[[source]]
path = "src/model/mod.rs"
+++

## 职责容器

本模块保留原有顶层职责。
API显式下沉到职责叶子。
原API路径与模块方向保持不变。
本层不复制叶子API。
各叶子按真实文件记录证据。
完整产品模块继续保持计划态。

证据：C:/Rust/mjwarp-rs/docs/windows-kinematic-fields.md。

## 惯性辅助叶子

本层新增惯性职责叶子。
辅助子集单独记录局部验收。
产品模块继续保持计划态。

证据：C:/Rust/mjwarp-rs/docs/windows-inertial-fields.md。
