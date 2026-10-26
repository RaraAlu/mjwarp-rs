# -*- coding: utf-8 -*-
"""Prepare two read-only source copies for the option-A QA check.

  target/qa-a-check/dump-tree  = r2 commit + an appended QA dump test (case_evidence.rs)
  target/qa-a-check/pre-tree   = r2 parent commit (for G5 pre/post comparison)

The main tree is never modified; both trees come from `git archive <rev>`.
Writes qa-copies-info.json.
"""
import hashlib
import json
import shutil
import subprocess
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
WORK = REPO / "target/qa-a-check"
TASK = REPO / "docs/development-evidence/g10-p5-faithful-fma-fix-independent-check"
R2 = "50a0acd1"
PARENT = "025d47fc"

DUMP_TEST = r'''
// ===== QA independent check: dump expected/actual contact rows (test-only, copy tree) =====
fn qa_dump_rows(rows: &[RefContact]) -> String {
    let mut out = String::from("[");
    for (index, row) in rows.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(&format!("[{},{}", row.worldid, row.geomcollisionid));
        out.push_str(&format!(",{}", row.dist.to_bits()));
        for value in row.pos {
            out.push_str(&format!(",{}", value.to_bits()));
        }
        for value in row.frame {
            out.push_str(&format!(",{}", value.to_bits()));
        }
        out.push(']');
    }
    out.push(']');
    out
}

#[test]
fn qa_dump_upstream_contacts() {
    let path = std::env::var("QA_A_DUMP").expect("QA_A_DUMP");
    let mut out = String::from("{\"schema\":1,\"cases\":{");
    let all = cases();
    for (case_index, case) in all.iter().enumerate() {
        if case_index > 0 {
            out.push(',');
        }
        out.push('"');
        out.push_str(&case.name);
        out.push_str("\":[");
        for world in 0..case.worlds.len() {
            let mut expected: Vec<RefContact> = super::case_compare::upstream_mjwarp_contacts(case)
                .iter()
                .filter(|contact| contact.worldid == world as i32)
                .copied()
                .collect();
            let mut actual = super::case_compare::product_contacts_upstream(case, world);
            super::case_compare::sort_contacts(&mut expected);
            super::case_compare::sort_contacts(&mut actual);
            if world > 0 {
                out.push(',');
            }
            out.push_str(&format!(
                "{{\"world\":{},\"expected\":{},\"actual\":{}}}",
                world,
                qa_dump_rows(&expected),
                qa_dump_rows(&actual)
            ));
        }
        out.push(']');
    }
    out.push_str("}}");
    std::fs::write(&path, out).expect("write QA dump");
    println!("wrote QA dump to {path}");
}
'''


def archive(rev: str, dest: Path) -> None:
    if dest.exists():
        shutil.rmtree(dest)
    dest.parent.mkdir(parents=True, exist_ok=True)
    archive_file = WORK / f"{dest.name}.tar"
    subprocess.run(["git", "-C", str(REPO), "archive", "--format=tar", rev,
                    "-o", str(archive_file)], check=True)
    dest.mkdir(parents=True, exist_ok=True)
    subprocess.run(["tar", "-xf", str(archive_file), "-C", str(dest)], check=True)
    archive_file.unlink()


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


WORK.mkdir(parents=True, exist_ok=True)
archive(R2, WORK / "dump-tree")
archive(PARENT, WORK / "pre-tree")

dump_file = WORK / "dump-tree/src/physics/collision/gjk/case_evidence.rs"
original = dump_file.read_text(encoding="utf-8")
dump_file.write_text(original + DUMP_TEST, encoding="utf-8")

info = {
    "r2": R2,
    "parent": PARENT,
    "dump_tree": str(WORK / "dump-tree"),
    "pre_tree": str(WORK / "pre-tree"),
    "dump_test_appended_to": "src/physics/collision/gjk/case_evidence.rs",
    "dump_test_source": DUMP_TEST.strip(),
    "dump_tree_case_evidence_sha256_after_patch": sha256(dump_file),
    "pre_tree_case_evidence_sha256": sha256(WORK / "pre-tree/src/physics/collision/gjk/case_evidence.rs"),
    "polytope_dump_tree_sha256": sha256(WORK / "dump-tree/src/physics/collision/gjk/polytope.rs"),
    "polytope_pre_tree_sha256": sha256(WORK / "pre-tree/src/physics/collision/gjk/polytope.rs"),
}
(TASK / "qa-copies-info.json").write_text(
    json.dumps(info, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({k: v for k, v in info.items() if k != "dump_test_source"}, ensure_ascii=False, indent=2))
