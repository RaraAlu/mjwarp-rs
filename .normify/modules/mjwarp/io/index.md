+++
uid = "47e35774"
id = "mjwarp.io"
state = "planned"
tags = ["planned", "engine", "io"]
fingerprint = "9ca7e34cf063ff3b729ee0afc21b9e9569bfac75d1592deb2b6f51c115813148"

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
