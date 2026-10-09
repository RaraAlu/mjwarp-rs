#![cfg(target_os = "linux")]

use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Evidence(PathBuf);

impl Evidence {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "mjwarp-linux-runner-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        fs::write(path.join("checks.json"), "[]\n").unwrap();
        Self(path)
    }
}

impl Drop for Evidence {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn check(output: &str, exit: &str, expected: &str, minimum: &str) -> bool {
    let evidence = Evidence::new();
    let result = Command::new("bash")
        .arg("-c")
        .arg(
            r#"set -euo pipefail
source "$COMMON"
directory="$EVIDENCE"
emit() { printf '%s\n' "$OUTPUT"; return "$CODE"; }
run_checked sample "$EXPECTED" "$MINIMUM" emit
"#,
        )
        .env(
            "COMMON",
            format!(
                "{}/scripts/linux-probe-common.sh",
                env!("CARGO_MANIFEST_DIR")
            ),
        )
        .env("EVIDENCE", &evidence.0)
        .env("OUTPUT", output)
        .env("CODE", exit)
        .env("EXPECTED", expected)
        .env("MINIMUM", minimum)
        .output()
        .unwrap();
    assert!(evidence.0.join("sample.log").is_file());
    let checks = fs::read_to_string(evidence.0.join("checks.json")).unwrap();
    assert!(checks.contains("sample"));
    result.status.success()
}

#[test]
fn accepts_exact_counts_and_non_test_commands() {
    assert!(check(
        "test result: ok. 7 passed; 0 failed; 0 ignored;",
        "0",
        "7",
        "7"
    ));
    assert!(check("format check", "0", "-1", "0"));
    assert!(check(
        "test result: ok. 2 passed; 0 failed; 0 ignored;\ntest result: ok. 5 passed; 0 failed; 0 ignored;",
        "0",
        "7",
        "7"
    ));
}

#[test]
fn rejects_zero_missing_failed_and_incomplete_test_evidence() {
    for output in [
        "test result: ok. 0 passed; 0 failed; 7 ignored;",
        "no summary",
        "test result: ok. 7 passed; 1 failed; 0 ignored;",
        "test result: ok. 6 passed; 0 failed; 0 ignored;",
        "test result: ok. 7 passed; 0 failed; 1 ignored;",
    ] {
        assert!(!check(output, "0", "7", "7"));
    }
    assert!(!check(
        "test result: ok. 7 passed; 0 failed; 0 ignored;",
        "1",
        "7",
        "7"
    ));
}

#[test]
fn rejects_empty_invalid_and_partly_invalid_fixture_manifests() {
    let evidence = Evidence::new();
    let fixtures = evidence.0.join("fixtures");
    fs::create_dir(&fixtures).unwrap();
    fs::write(fixtures.join("empty.bin"), []).unwrap();
    let valid = r#"{"path":"empty.bin","sha256":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"}"#;
    for (manifest, expected) in [
        (None, false),
        (Some(format!(r#"{{"files":[{valid}]}}"#)), true),
        (Some(r#"{"files":[]}"#.into()), false),
        (
            Some(format!(r#"{{"files":[{valid},{{"path":"bad"}}]}}"#)),
            false,
        ),
        (Some("invalid json".into()), false),
        (
            Some(r#"{"files":[{"path":"missing.bin","sha256":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"}]}"#.into()),
            false,
        ),
    ] {
        if let Some(manifest) = manifest {
            fs::write(fixtures.join("manifest.json"), manifest).unwrap();
        }
        let result = Command::new("bash")
            .arg("-c")
            .arg("set -euo pipefail; source \"$COMMON\"; root=\"$ROOT\"; verify_fixtures")
            .env(
                "COMMON",
                format!(
                    "{}/scripts/linux-probe-common.sh",
                    env!("CARGO_MANIFEST_DIR")
                ),
            )
            .env("ROOT", &evidence.0)
            .output()
            .unwrap();
        assert_eq!(result.status.success(), expected);
    }
}

#[test]
fn com_velocity_gate_fixes_counts_and_keeps_full_g02_unverified() {
    let gate = include_str!("../scripts/test-linux-com-velocity.sh");
    assert!(gate.contains("run_checked gpu 6 6"));
    assert!(gate.contains("run_checked guards 1 1"));
    assert!(gate.contains("g02Verified:false,t4Verified:false"));
    assert!(gate.contains("--require-t4"));
    assert!(gate.contains("error(\"invalid fixture manifest\")"));
    let g01 = include_str!("../scripts/test-linux-g01.sh");
    assert!(g01.contains("expected=168"));
    assert!(g01.contains("expected=176"));
}
