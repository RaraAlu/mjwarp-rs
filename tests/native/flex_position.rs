//! G01原生柔体位置转换验收。
use super::{EMPTY, dll, load, mock};
use mjwarp_rs::{diagnostics::NativeProbeError, model::FlexPositionFields};
pub(super) const FLEX: &[u8] = include_bytes!("../../fixtures/flex-position/flex-tree.mjb");

#[test]
#[ignore = "需要可信MuJoCo DLL"]
fn captures_flex_fields_after_releasing_native_model_and_accepts_empty_flexes() {
    let p = load(&dll(), FLEX).unwrap();
    let s = p.flex_position_snapshot().unwrap();
    drop(p);
    assert_eq!((s.nflex, s.nflexnode, s.nflexvert), (4, 20, 80));
    assert_eq!((s.info.nq, s.info.nv, s.info.nbody), (66, 65, 25));
    assert_eq!(s.flex_interp, [0, 0, 1, 1]);
    assert_eq!(s.flex_centered, [0, 1, 0, 1]);
    assert_eq!(s.flex_nodeadr, [0, 0, 0, 12]);
    assert_eq!(s.flex_nodenum, [0, 0, 12, 8]);
    assert_eq!(s.flex_vertnum, [5, 3, 45, 27]);
    assert_eq!(s.flex_node.len(), 60);
    assert_eq!(s.flex_vert.len(), 240);
    assert_eq!(s.flex_vert0.len(), 240);
    s.into_fields().unwrap();
    let p = load(&dll(), EMPTY).unwrap();
    let s = p.flex_position_snapshot().unwrap();
    drop(p);
    assert_eq!((s.nflex, s.nflexnode, s.nflexvert), (0, 0, 0));
    assert_eq!(s.into_fields().unwrap(), FlexPositionFields::default());
    let p = load(
        &dll(),
        include_bytes!("../../fixtures/flex-stretch/stretch.mjb"),
    )
    .unwrap();
    let s = p.flex_position_snapshot().unwrap();
    drop(p);
    assert_eq!(s.nflexnode, 8);
    assert!(s.flex_interp.contains(&0));
    s.into_fields().unwrap();
}

#[test]
#[ignore = "需要原生测试DLL与独立日志"]
fn rejects_missing_flex_source_and_invalid_counts_without_losing_owner() {
    use mjwarp_rs::diagnostics::InputError;
    let path = std::path::PathBuf::from(std::env::var_os("MJWARP_NATIVE_LIFETIME_LOG").unwrap());
    for name in [
        "missing-flex-pointer",
        "invalid-flex-counts",
        "oversized-flex-counts",
    ] {
        std::fs::write(&path, []).unwrap();
        let p = load(&mock(name), EMPTY).unwrap();
        let error = p.flex_position_snapshot().unwrap_err();
        let expected = match name {
            "missing-flex-pointer" => NativeProbeError::Native {
                stage: "copy_flex_position",
                code: 8,
            },
            "invalid-flex-counts" => NativeProbeError::Native {
                stage: "flex_position_info",
                code: 8,
            },
            _ => NativeProbeError::Input(InputError::Overflow { field: "nflexnode" }),
        };
        assert_eq!(error, expected);
        assert!(std::fs::read(&path).unwrap().is_empty());
        assert_eq!(p.snapshot().unwrap().body_mass, [0.0]);
        drop(p);
        assert_eq!(std::fs::read(&path).unwrap(), b"DU");
    }
    let p = load(&mock("direct-flex"), EMPTY).unwrap();
    let s = p.flex_position_snapshot().unwrap();
    drop(p);
    assert_eq!((s.nflex, s.nflexnode, s.nflexvert), (1, 0, 1));
    assert!(s.flex_node.is_empty());
    assert!(s.flex_nodebodyid.is_empty());
    let f = s.into_fields().unwrap();
    assert_eq!(f.flex_interp, [0]);
    assert_eq!(f.flex_vert, [0.0; 3]);
}
