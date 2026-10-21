+++
uid = "ccf4d023"
id = "mjwarp.runtime.cache"
fingerprint = "7144e8ce66e3ab79327400cfb041d11d1748ee3d408c6150b0667bb4eab0f9f3"

[name]
zh = "可信产物缓存"
en = "Trusted artifact cache"

[description]
zh = """
检查可信PTX缓存与配方。 产品目标接口仍属计划。
"""
en = """
Own explicit trusted PTX recipes, schema validation, integrity and compiler provenance checks. Production target APIs remain planned.
"""

[[source]]
path = "src/runtime/cache.rs"

[[source]]
path = "src/runtime/cache/tests.rs"

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::build_probe_artifact"

[apis.description]
zh = """
现有局部探针入口。 不提供完整引擎API。 rpc仅作契约标记。
"""
en = """
Existing working prototype entry point, not a complete engine API. RPC is an abstract contract notation, not a network service.
"""

[[apis]]
protocol = "rpc"
path = "mjwarp_rs::runtime::run_cached_probe"

[apis.description]
zh = """
现有局部探针入口。 不提供完整引擎API。 rpc仅作契约标记。
"""
en = """
Existing working prototype entry point, not a complete engine API. RPC is an abstract contract notation, not a network service.
"""
+++
