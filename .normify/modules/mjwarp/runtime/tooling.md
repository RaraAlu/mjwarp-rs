+++
uid = "26d0e749"
id = "mjwarp.runtime.tooling"
state = "planned"
fingerprint = "27c0c52ec120ce18b5cdb6ae193b8fe91d7df22642f0d3d3581e667c3fe6be6c"

[name]
zh = "工具与命令入口"
en = "Tooling and commands"

[description]
zh = """
解析探针命令与显式配置。 产品目标接口仍属计划。
"""
en = """
Own probe CLI parsing and explicit route/configuration selection. Script integrations remain recorded in the runtime container. Production target APIs remain planned.
"""

[[source]]
path = "src/main.rs"

[[apis]]
protocol = "file"
path = "scripts/prepare-cuda-probe.ps1"

[apis.description]
zh = """
现有PowerShell执行入口。 只验证Windows局部能力。
"""
en = """
Existing PowerShell command entry for bounded Windows tooling and probe verification.
"""

[[apis]]
protocol = "file"
path = "scripts/prepare-gnu-probe.ps1"

[apis.description]
zh = """
现有PowerShell执行入口。 只验证Windows局部能力。
"""
en = """
Existing PowerShell command entry for bounded Windows tooling and probe verification.
"""

[[apis]]
protocol = "file"
path = "scripts/run-cubecl-probe.ps1"

[apis.description]
zh = """
现有PowerShell执行入口。 只验证Windows局部能力。
"""
en = """
Existing PowerShell command entry for bounded Windows tooling and probe verification.
"""

[[apis]]
protocol = "file"
path = "scripts/test-windows-deployment.ps1"

[apis.description]
zh = """
现有PowerShell执行入口。 只验证Windows局部能力。
"""
en = """
Existing PowerShell command entry for bounded Windows tooling and probe verification.
"""

[[apis]]
protocol = "file"
path = "scripts/test-windows-external.ps1"

[apis.description]
zh = """
现有PowerShell执行入口。 只验证Windows局部能力。
"""
en = """
Existing PowerShell command entry for bounded Windows tooling and probe verification.
"""

[[apis]]
protocol = "file"
path = "scripts/test-windows-transfers.ps1"

[apis.description]
zh = """
在受限子进程验证同步传输。 不代表独立清洁机器验收。
"""
en = """
Run the CUDA-only transfer tests in a restricted-PATH process with NVRTC invisibility assertions. Not clean-machine deployment acceptance.
"""
+++
