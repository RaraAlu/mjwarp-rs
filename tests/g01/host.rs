//! G01 参考夹具哈希与覆盖。

use serde_json::Value;

use crate::support::fixture;

#[test]
fn reference_hashes_and_coverage() {
    use sha2::{Digest, Sha256};
    let manifest: Value =
        serde_json::from_str(include_str!("../../fixtures/g01/manifest.json")).unwrap();
    for f in manifest["files"].as_array().unwrap() {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/g01")
            .join(f["path"].as_str().unwrap());
        assert_eq!(
            format!("{:x}", Sha256::digest(std::fs::read(p).unwrap())),
            f["sha256"].as_str().unwrap()
        );
    }
    let v = fixture();
    assert_eq!(v["cases"].as_array().unwrap().len(), 16);
    assert_eq!(
        v["model"]["jnt_type"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_i64().unwrap())
            .collect::<std::collections::BTreeSet<_>>(),
        std::collections::BTreeSet::from([0, 1, 2, 3])
    );
    assert!(
        v["model"]["flex_interp"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x == -1)
    );
}
