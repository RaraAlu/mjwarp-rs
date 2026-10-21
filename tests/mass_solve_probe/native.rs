use super::compare::compare;
use super::*;

#[test]
#[ignore = "requires trusted Windows MuJoCo DLL, NVIDIA driver and NVRTC"]
fn native_snapshots_feed_gpu_solves_without_native_physics_at_runtime() {
    let mut audit = Audit::new("G03-native-snapshots");
    use mjwarp_rs::io::NativeModelProbe;
    let dll = std::path::PathBuf::from(
        std::env::var_os("MJWARP_MUJOCO_DLL").expect("provide trusted DLL"),
    );
    let session = TransferSession::new(0).unwrap();
    let mjbs: [&[u8]; 6] = [
        include_bytes!("../../fixtures/native-probe/mixed-joints.mjb"),
        include_bytes!("../../fixtures/native-probe/inertial-tree.mjb"),
        include_bytes!("../../fixtures/kinematics/rotated-tree.mjb"),
        include_bytes!("../../fixtures/native-probe/zero-dof.mjb"),
        include_bytes!("../../fixtures/com-position/massless-tree.mjb"),
        include_bytes!("../../fixtures/mass-matrix/armature-chain.mjb"),
    ];
    for ((m, source), mjb) in MODELS.into_iter().zip(SOLVES).zip(mjbs) {
        // SAFETY: The caller supplies the hash-verified trusted candidate DLL.
        // These static compiled models use that exact known ABI.
        let native = unsafe { NativeModelProbe::load_trusted(&dll, mjb) }.unwrap();
        let m_native = native.inertial_snapshot().unwrap().into_model().unwrap();
        assert_eq!(m_native, model(&serde_json::from_str(m).unwrap()));
        drop(native);
        let s: Value = serde_json::from_str(source).unwrap();
        let cases = s["cases"].as_array().unwrap();
        let q: Vec<_> = cases.iter().flat_map(|c| floats(c, "qpos")).collect();
        let b: Vec<_> = cases.iter().flat_map(|c| floats(c, "rhs")).collect();
        let out = probe_mass_solve(&session, &m_native, 8, &q, 3, &b).unwrap();
        for (w, c) in cases.iter().enumerate() {
            compare(&out, w, c, source, &mut audit);
        }
    }
    audit.finish();
}
