"""QA 判据自检（反向判别力）：不跑 GPU，用上游参考合成 dump，再逐项做受控变异，
确认 `qa_primitive_check.py` 会在实现/数据错误时报错、在容差内不误报。

用途：覆盖验收项「新增测试是否真能区分正确/错误实现（反向判别）」的**判据侧**证据；
产品侧由窗口内真实 dump 的位级比较承担。

用法：python qa_primitive_selfcheck.py --json target/qa-primitive/selfcheck.json
"""

import argparse
import json
import pathlib
import shutil
import subprocess
import sys

import numpy as np

REPO = pathlib.Path(r"D:\Rust\mjwarp-rs")
EVIDENCE = REPO / "docs/development-evidence/g10-p5-primitive-narrowphase-cuda-independent-check"
REFERENCE = REPO / "docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-upstream-gpu.json"
QA_CASES = REPO / "docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-gpu/cases"
CHECK = EVIDENCE / "qa_primitive_check.py"
PYTHON = REPO / "target/mjwarp-parallel/venv/Scripts/python.exe"
SYNTH = REPO / "target/qa-primitive/synth"
CASE = "sphere-box-shallow"
F32_FIELDS = [("dist", 1), ("pos", 3), ("frame", 9), ("includemargin", 1), ("friction", 5),
              ("solref", 2), ("solreffriction", 2), ("solimp", 5), ("adhesion", 1)]
I32_FIELDS = [("dim", 1), ("geom", 2), ("worldid", 1), ("geomcollisionid", 1), ("type", 1)]


def meta_of(case):
    meta = {}
    for line in (QA_CASES / case / "meta.txt").read_text(encoding="utf-8").splitlines():
        if " " in line:
            key, value = line.split(" ", 1)
            meta[key] = value
    return meta


def reference_arrays(case):
    ref = json.loads(REFERENCE.read_text(encoding="utf-8"))["cases"][case]
    nacon = int(ref["count"])
    efc_width = int(meta_of(case)["efc_width"])
    arrays = {}
    for name, width in F32_FIELDS:
        values = np.asarray([np.float32(v) for v in ref["fields"][name]], dtype="<f4")
        assert len(values) == nacon * width, (name, len(values), nacon * width)
        arrays[name] = values
    for name, width in I32_FIELDS:
        values = np.asarray([int(v) for v in ref["fields"][name]], dtype="<i4")
        assert len(values) == nacon * width, (name, len(values), nacon * width)
        arrays[name] = values
    arrays["nacon"] = np.asarray([nacon], dtype="<i4")
    arrays["nccd"] = np.asarray([0], dtype="<i4")
    arrays["overflow"] = np.asarray([int(v) for v in ref["overflow"]], dtype="<i4")
    arrays["efc_address"] = np.full(nacon * efc_width, -1, dtype="<i4")
    return arrays, ref


def write_case(dest, arrays):
    case_dir = dest / CASE
    if case_dir.exists():
        shutil.rmtree(case_dir)
    case_dir.mkdir(parents=True)
    for name, values in arrays.items():
        values.astype(values.dtype).tofile(case_dir / f"{name}.bin")


def run_check(dumps):
    proc = subprocess.run(
        [str(PYTHON), str(CHECK), "--numeric-only", "--case", CASE, "--dumps", str(dumps)],
        capture_output=True, text=True, encoding="utf-8", errors="replace")
    lines = (proc.stdout or "").splitlines() + (proc.stderr or "").splitlines()
    summary = {}
    for line in lines:
        if line.startswith(f"case={CASE}"):
            for token in line.split():
                if "=" in token:
                    key, value = token.split("=", 1)
                    summary[key] = value
    return proc.returncode, summary, lines


def mutate(base, name, fn):
    arrays = {key: (value.copy() if hasattr(value, "copy") else value) for key, value in base.items()}
    fn(arrays)
    return arrays


def plus_ulp(values, index, ulps=1):
    values[index] = np.frombuffer(
        (np.uint32(values.view("<u4")[index]) + np.uint32(ulps)).tobytes(), dtype="<f4")[0]


def main():
    try:
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass
    parser = argparse.ArgumentParser()
    parser.add_argument("--json", default=None)
    args = parser.parse_args()

    arrays, ref = reference_arrays(CASE)
    SYNTH.mkdir(parents=True, exist_ok=True)
    cases = {}

    def case_entry(label, expectation):
        return {"label": label, "expectation": expectation}

    scenarios = []
    scenarios.append(("baseline_reference_dump", None))
    scenarios.append(("dist_plus_1ulp", lambda a: plus_ulp(a["dist"], 0, 1)))
    scenarios.append(("dist_plus_1e-4", lambda a: a["dist"].__setitem__(0, a["dist"][0] + np.float32(1e-4))))
    scenarios.append(("pos_plus_1e-3", lambda a: a["pos"].__setitem__(0, a["pos"][0] + np.float32(1e-3))))
    scenarios.append(("frame_plus_1e-6", lambda a: a["frame"].__setitem__(0, a["frame"][0] + np.float32(1e-6))))
    scenarios.append(("frame_plus_2e-7", lambda a: a["frame"].__setitem__(0, a["frame"][0] + np.float32(2e-7))))
    scenarios.append(("nacon_minus_one", lambda a: a["nacon"].__setitem__(0, int(a["nacon"][0]) - 1)))
    scenarios.append(("nccd_one", lambda a: a["nccd"].__setitem__(0, 1)))
    scenarios.append(("overflow_bit_set", lambda a: a["overflow"].__setitem__(0, int(a["overflow"][0]) | 1)))
    scenarios.append(("efc_address_zero", lambda a: a["efc_address"].__setitem__(0, 0)))
    def swap_rows(a):
        a["dist"][[0, 1]] = a["dist"][[1, 0]]
    scenarios.append(("swap_dist_rows", swap_rows))

    expectations = {
        "baseline_reference_dump": "pass",
        "dist_plus_1ulp": "pass_with_1_ulp",
        "dist_plus_1e-4": "fail",
        "pos_plus_1e-3": "fail",
        "frame_plus_1e-6": "fail",
        "frame_plus_2e-7": "pass_with_1_ulp",
        "nacon_minus_one": "fail",
        "nccd_one": "fail",
        "overflow_bit_set": "fail",
        "efc_address_zero": "fail",
        "swap_dist_rows": "pass",
    }

    results = []
    all_ok = True
    for name, fn in scenarios:
        mutated = arrays if fn is None else mutate(arrays, name, fn)
        dest = SYNTH / name
        write_case(dest, mutated)
        code, summary, lines = run_check(dest)
        expected = expectations[name]
        passed = code == 0
        if expected == "pass":
            ok = passed
        elif expected == "fail":
            ok = not passed
        elif expected == "pass_with_1_ulp":
            parts = summary.get("bit_equal", "0/0").split("/")
            ok = passed and len(parts) == 2 and int(parts[0]) == int(parts[1]) - 1
            if name == "dist_plus_1ulp":
                ok = ok and summary.get("max_dist_ulp") == "1"
        else:
            ok = False
        all_ok = all_ok and ok
        results.append({
            "scenario": name,
            "expectation": expected,
            "check_exit": code,
            "summary": summary,
            "fail_lines": [line for line in lines if line.startswith("FAIL")][:3],
            "as_expected": ok,
        })

    data = {
        "case": CASE,
        "reference": str(REFERENCE.relative_to(REPO)),
        "synth_dumps": str(SYNTH.relative_to(REPO)),
        "scenarios": results,
        "all_as_expected": all_ok,
        "note": "基线 dump 由上游参考合成（不跑 GPU），用于证明判据/比较器有反向判别力；产品侧证据由窗口真实 dump 承担",
    }
    text = json.dumps(data, indent=2, ensure_ascii=False)
    print(text)
    if args.json:
        pathlib.Path(args.json).write_text(text + "\n", encoding="utf-8")
    return 0 if all_ok else 1


if __name__ == "__main__":
    sys.exit(main())
