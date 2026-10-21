//! 谱系清单校验。

use super::recipe::{retained_recipe, rotate_recipe};
use super::*;

/// 保留十份刚体更新；另允许一份面算子修复。
/// 其他字段、参考、变体、排除项和未验收边界全部继承。
pub(super) fn lineage(value: &Inventory) -> Result<(), String> {
    validate_inventory(value)?;
    let mut expected: Inventory = serde_json::from_str(PRE_ROTATE).unwrap();
    expected.source_revision = REVISION.into();
    expected
        .product_sources
        .iter_mut()
        .find(|stamp| stamp.path == "src/physics/mod.rs")
        .unwrap()
        .sha256 = digest(normalized(PHYSICS).as_bytes());
    let mut replacements = 0;
    let mut retained = 0;
    for (old, new) in expected.artifacts.iter_mut().zip(&value.artifacts) {
        if let Some((entry, source)) = rotate_recipe(old) {
            if new.entry != entry || digest(source.as_bytes()) != new.source_sha256 {
                return Err("rigid template coverage drift".into());
            }
            *old = new.clone();
            replacements += 1;
        } else {
            let source = retained_recipe(old).ok_or("unmapped retained variant")?;
            if digest(source.as_bytes()) != old.source_sha256
                || old.entry != new.entry
                || old.source_sha256 != new.source_sha256
                || old.ptx_sha256 != new.ptx_sha256
                || old.receipt_sha256 != new.receipt_sha256
            {
                return Err("retained template coverage drift".into());
            }
            retained += 1;
        }
    }
    if replacements != 10
        || retained != 20
        || serde_json::to_value(expected).unwrap() != serde_json::to_value(value).unwrap()
    {
        return Err("unexpected lineage change".into());
    }
    Ok(())
}
