+++
uid = "47e35774"
id = "mjwarp.io"
tags = ["planned", "engine", "io"]
fingerprint = "d4ed42d1ed3f59b97f467edb44fd9cce47289991aa80012afdf76e240a3bfe86"

[name]
zh = "模型与状态IO"
en = "Model and state IO"

[description]
zh = """
转换已编译模型。交换状态与组织重置。不编译MJCF。
"""
en = """
Convert compiled model inputs, exchange state and coordinate resets. Do not compile MJCF.
"""

[[source]]
path = "src/io/mod.rs"
+++

## 本轮源码复审

IO容器继续导出既有路径。
G01快照另按feature导出。
原生适配支持MSVC与Linux GNU。
Windows采用UTF-16路径。
Linux采用原始字节路径。
职责叶子继续独占API声明。
本层不复制叶子接口。
完整状态交换仍保持计划态。
