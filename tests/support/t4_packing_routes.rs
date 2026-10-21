//! 追踪其余打包器的字段路由。
//! 私有表示不充当完整上游ABI。

#[path = "t4_packing_routes/flex_internal.rs"]
mod flex_internal;
#[path = "t4_packing_routes/routes.rs"]
mod routes;

const RESIDENT: &str = include!("../../docs/development-evidence/t4-resident-anchors/resident.rs");
