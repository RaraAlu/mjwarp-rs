+++
uid = "26d0e749"
id = "mjwarp.runtime.tooling"
fingerprint = "5fb770492ea9a7d15f629012fd6efed0e634dc2cb88fb9ce5ef54ee4d222dfdc"

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

[[source]]
path = "src/runtime/tooling/mod.rs"

[[source]]
path = "src/runtime/tooling/tests.rs"

[[source]]
path = "src/runtime/tooling/perf.rs"

[[source]]
path = "src/runtime/tooling/gpu.rs"

[[source]]
path = "tests/a10_p11_gpu_perf_probe.rs"

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
