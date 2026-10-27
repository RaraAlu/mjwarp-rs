"""QA verification of the S3 MJB provenance: fixtures, XML, references, qpos, 96 arrays."""
import hashlib
import json
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
PROV = ROOT / "docs/development-evidence/product-collision-inputs-wiring/fixtures/mjb-312/provenance.json"
OUT = ROOT / "target/qa-s3/verify_provenance.json"
problems = []
checks = 0


def check(cond, message):
    global checks
    checks += 1
    if not cond:
        problems.append(message)


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    prov = json.loads(PROV.read_text(encoding="utf-8"))
    report = {
        "mujoco": prov.get("mujoco"),
        "python": prov.get("python"),
        "numpy": prov.get("numpy"),
        "result": prov.get("result"),
        "array_checks": prov.get("array_checks"),
        "array_mismatches": prov.get("array_mismatches"),
        "authoring_only": prov.get("authoring_only"),
        "cases": {},
    }
    check(prov.get("mujoco") == "3.12.0", f"provenance mujoco {prov.get('mujoco')}")
    check(prov.get("result") == "PASS", "provenance result")
    check(prov.get("array_checks") == 96, "provenance array_checks")
    check(prov.get("array_mismatches") == 0, "provenance array_mismatches")
    check(prov.get("failures") == [], "provenance failures")
    check(prov.get("authoring_only") is True, "provenance authoring_only")

    for name, case in prov["cases"].items():
        base = ROOT / "docs/development-evidence/product-collision-inputs-wiring/fixtures/mjb-312"
        mjb = base / Path(case["mjb"]).name
        xml = ROOT / case["xml"]
        reference = ROOT / case["reference"]
        check(mjb.stat().st_size == case["mjb_bytes"], f"{name} mjb bytes")
        check(sha(mjb) == case["mjb_sha256"], f"{name} mjb sha")
        check(xml.stat().st_size == case["xml_bytes"], f"{name} xml bytes")
        check(sha(xml) == case["xml_sha256"], f"{name} xml sha")
        check(sha(reference) == case["reference_sha256"], f"{name} reference sha")
        # qpos vs frozen reference qpos (f64 exact)
        ref = json.loads(reference.read_text(encoding="utf-8"))
        ref_qpos = ref["cases"][name]["qpos"]
        check([float(v) for v in case["qpos"]] == [float(v) for v in ref_qpos],
              f"{name} qpos vs reference")
        import struct
        # provenance qpos_bits are f32 bit patterns (the frozen reference qpos values are
        # f32-representable f64 literals); verify both the f32 bits and the f64 equality above.
        bits = [struct.unpack("<I", struct.pack("<f", float(v)))[0] for v in case["qpos"]]
        check(bits == case["qpos_bits"], f"{name} qpos bits (f32)")
        # 32 arrays vs the frozen bins
        bins_dir = ROOT / case["bins"]
        arrays = case["arrays"]
        check(len(arrays) == 32, f"{name} arrays {len(arrays)}")
        bad = []
        for field, entry in arrays.items():
            bin_path = bins_dir / f"{field}.bin"
            raw = bin_path.read_bytes()
            count = len(raw) // 4
            if (sha(bin_path) != entry.get("bin_sha256") or
                    sha(bin_path) != entry.get("sha256") or
                    count != entry.get("count") or
                    entry.get("bin_match") is not True or
                    entry.get("round_trip_match") is not True):
                bad.append(field)
        check(not bad, f"{name} array mismatches: {bad}")
        check(case.get("field_mismatches") == [], f"{name} field_mismatches")
        report["cases"][name] = {"mjb_sha256": case["mjb_sha256"], "xml_sha256": case["xml_sha256"],
                                 "reference_sha256": case["reference_sha256"], "arrays": len(arrays)}
    report["checks"] = checks
    report["problems"] = problems
    report["result"] = "PASS" if not problems else "FAIL"
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_bytes((json.dumps(report, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))
    print(f"checks={checks} problems={len(problems)} result={report['result']}")
    for problem in problems:
        print("PROBLEM:", problem)
    print("cases:", {k: v["arrays"] for k, v in report["cases"].items()})


if __name__ == "__main__":
    main()
