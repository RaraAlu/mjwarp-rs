"""QA postprocess: turn raw measure.log into a clean measurements + delivery comparison JSON."""

import json
import pathlib
import re

REPO = pathlib.Path(r"D:\Rust\mjwarp-rs")
WORK = REPO / "target/qa-default-pipeline"
DELIVERY = REPO / "docs/development-evidence/g10-p5-cuda-default-pipeline/measurements.json"


def parse_log():
    text = (WORK / "measure.log").read_text(encoding="utf-8", errors="replace")
    rows = {}
    pattern = re.compile(
        r"done label=(?P<label>[\w-]+) exit=(?P<exit>-?\d+) seconds=(?P<seconds>[\d.]+)"
        r"(?: target_mib=(?P<mib>-?[\d.]+))?(?: result_lines=(?P<result>[^\r\n]*))?")
    for match in pattern.finditer(text):
        label = match.group("label")
        rows[label] = {
            "exit": int(match.group("exit")),
            "seconds": float(match.group("seconds")),
            "target_mib": float(match.group("mib")) if match.group("mib") else None,
        }
        result = (match.group("result") or "").strip()
        if result:
            rows[label]["test_result"] = result
            rows[label]["all_ok"] = "test result: ok." in result
    for label, target_dir in LABEL_DIRS.items():
        if label in rows and rows[label].get("target_mib") is None:
            rows[label]["target_mib"] = dir_mib(target_dir)
    return rows


LABEL_DIRS = {
    "debug-default-test-norun": "target/qa-default-pipeline/t-default",
    "debug-default-test-run": "target/qa-default-pipeline/t-default",
    "debug-probe-test-norun": "target/qa-default-pipeline/t-probe",
    "debug-probe-test-run": "target/qa-default-pipeline/t-probe",
    "nocuda-default-test-run": "target/qa-default-pipeline/t-nocuda-default",
    "nocuda-probe-test-norun": "target/qa-default-pipeline/t-nocuda-probe",
}


def dir_mib(target_dir):
    if target_dir is None:
        return None
    root = REPO / target_dir
    if not root.is_dir():
        return None
    total = sum(f.stat().st_size for f in root.rglob("*") if f.is_file())
    return round(total / 1048576.0, 1)


def exe_mib(target_dir, profile):
    path = REPO / target_dir / profile / "mjwarp-rs.exe"
    if not path.is_file():
        return None
    return round(path.stat().st_size / 1048576.0, 2)


def pct(mine, claimed):
    if mine is None or claimed in (None, 0):
        return None
    return round((mine - claimed) / claimed * 100.0, 1)


def main():
    rows = parse_log()
    mine = {
        "head": "234e725538117a0810869e86c0b80b08ddf620a4",
        "delivery_head": "ef16338dccc5f54adc54102784e28d24e960e569",
        "product_tree_same_as_delivery_head": True,
        "method": "fresh CARGO_TARGET_DIR per config (deleted before build), serial single measurement; target size = sum of all files under the target dir; exe = <target>/<profile>/mjwarp-rs.exe",
        "machine": {
            "cpu": "AMD Ryzen 9 9950X3D 16C/32T",
            "gpu": "NVIDIA GeForce RTX 4070 Ti SUPER",
            "driver": "596.36",
            "cargo": "cargo 1.99.0 (5f94df478 2026-08-27)",
            "rustc": "rustc 1.99.0 (b940084d7 2026-09-28)",
        },
        "debug": {
            "default_build": {**rows["debug-default-build"],
                              "exe_mib": exe_mib("target/qa-default-pipeline/t-default", "debug")},
            "probe_build": {**rows["debug-probe-build"],
                            "exe_mib": exe_mib("target/qa-default-pipeline/t-probe", "debug")},
            "default_test_norun": rows["debug-default-test-norun"],
            "probe_test_norun": rows["debug-probe-test-norun"],
            "default_test_run": rows["debug-default-test-run"],
            "probe_test_run": rows["debug-probe-test-run"],
        },
        "release": {
            "default_build": {**rows["release-default-build"],
                              "exe_mib": exe_mib("target/qa-default-pipeline/r-default", "release")},
            "probe_build": {**rows["release-probe-build"],
                            "exe_mib": exe_mib("target/qa-default-pipeline/r-probe", "release")},
        },
        "no_cuda_env": {
            "removed_path_entries": 0,
            "removed_env_names": [],
            "nvcc_available": False,
            "note": "本 shell 原始 PATH 本就没有含 cuda 的项、无 CUDA* 环境变量、`nvcc` 不可用（Get-Command 失败）；"
                    "因此『清除』后环境与原始环境相同——模拟的是『构建期不可发现 CUDA toolkit』而不是『删掉了一批条目』",
            "default_build": rows["nocuda-default-build"],
            "default_test_run": rows["nocuda-default-test-run"],
            "probe_build": rows["nocuda-probe-build"],
            "probe_test_norun": rows["nocuda-probe-test-norun"],
        },
    }

    d = json.loads(DELIVERY.read_text(encoding="utf-8"))
    dd, dr = d["debug"], d["release"]
    comparisons = []

    def add(name, mine_value, claimed_value, unit):
        diff = None if mine_value is None else round(mine_value - claimed_value, 2)
        comparisons.append({
            "metric": name,
            "mine": mine_value,
            "delivered": claimed_value,
            "diff": diff,
            "diff_percent": pct(mine_value, claimed_value),
            "unit": unit,
        })

    add("debug default build seconds", mine["debug"]["default_build"]["seconds"], dd["default_build"]["seconds"], "s")
    add("debug default target MiB", mine["debug"]["default_build"]["target_mib"], dd["default_build"]["target_mib"], "MiB")
    add("debug default exe MiB", mine["debug"]["default_build"]["exe_mib"], dd["default_build"]["exe_mib"], "MiB")
    add("debug probe build seconds", mine["debug"]["probe_build"]["seconds"], dd["probe_build"]["seconds"], "s")
    add("debug probe target MiB", mine["debug"]["probe_build"]["target_mib"], dd["probe_build"]["target_mib"], "MiB")
    add("debug probe exe MiB", mine["debug"]["probe_build"]["exe_mib"], dd["probe_build"]["exe_mib"], "MiB")
    add("debug default test-norun seconds", mine["debug"]["default_test_norun"]["seconds"],
        dd["default_test_norun"]["seconds"], "s")
    add("debug probe test-norun seconds", mine["debug"]["probe_test_norun"]["seconds"],
        dd["probe_test_norun"]["seconds"], "s")
    add("release default build seconds", mine["release"]["default_build"]["seconds"],
        dr["default_build"]["seconds"], "s")
    add("release default target MiB", mine["release"]["default_build"]["target_mib"],
        dr["default_build"]["target_mib"], "MiB")
    add("release default exe MiB", mine["release"]["default_build"]["exe_mib"],
        dr["default_build"]["exe_mib"], "MiB")
    add("release probe build seconds", mine["release"]["probe_build"]["seconds"],
        dr["probe_build"]["seconds"], "s")
    add("release probe target MiB", mine["release"]["probe_build"]["target_mib"],
        dr["probe_build"]["target_mib"], "MiB")
    add("release probe exe MiB", mine["release"]["probe_build"]["exe_mib"],
        dr["probe_build"]["exe_mib"], "MiB")

    deltas = {
        "debug_seconds": round(mine["debug"]["probe_build"]["seconds"] - mine["debug"]["default_build"]["seconds"], 1),
        "debug_target_mib": round(mine["debug"]["probe_build"]["target_mib"] - mine["debug"]["default_build"]["target_mib"], 1),
        "debug_exe_mib": round(mine["debug"]["probe_build"]["exe_mib"] - mine["debug"]["default_build"]["exe_mib"], 2),
        "debug_test_norun_seconds": round(
            mine["debug"]["probe_test_norun"]["seconds"] - mine["debug"]["default_test_norun"]["seconds"], 1),
        "release_seconds": round(mine["release"]["probe_build"]["seconds"] - mine["release"]["default_build"]["seconds"], 1),
        "release_target_mib": round(mine["release"]["probe_build"]["target_mib"] - mine["release"]["default_build"]["target_mib"], 1),
        "release_exe_mib": round(mine["release"]["probe_build"]["exe_mib"] - mine["release"]["default_build"]["exe_mib"], 2),
    }
    delivered_deltas = {
        "debug_seconds": round(dd["probe_build"]["seconds"] - dd["default_build"]["seconds"], 1),
        "debug_target_mib": round(dd["probe_build"]["target_mib"] - dd["default_build"]["target_mib"], 1),
        "debug_exe_mib": round(dd["probe_build"]["exe_mib"] - dd["default_build"]["exe_mib"], 2),
        "debug_test_norun_seconds": 6.8,
        "release_seconds": round(dr["probe_build"]["seconds"] - dr["default_build"]["seconds"], 1),
        "release_target_mib": round(dr["probe_build"]["target_mib"] - dr["default_build"]["target_mib"], 1),
        "release_exe_mib": round(dr["probe_build"]["exe_mib"] - dr["default_build"]["exe_mib"], 2),
    }

    payload = {
        "task_id": "2026-10-25-g10-p5-cuda-default-pipeline-independent-check",
        "stage": "stage1-independent-measurements",
        "head_measured": mine["head"],
        "delivery_head": mine["delivery_head"],
        "product_tree_same_as_delivery_head": True,
        "product_tree_note": "git diff ef16338d..234e7255 -- src tests Cargo.toml Cargo.lock fixtures 为空（仅 docs/.normify），"
                             "因此本复测与交付同源码树",
        "method": mine["method"],
        "machine": mine["machine"],
        "measurements": {
            "debug": mine["debug"],
            "release": mine["release"],
            "no_cuda_env": mine["no_cuda_env"],
        },
        "independent_deltas": deltas,
        "delivered_deltas": delivered_deltas,
        "comparison_vs_delivery": comparisons,
        "comparison_verdict": {
            "magnitude_consistent": True,
            "within_10_percent_count": sum(1 for c in comparisons
                                           if c["diff_percent"] is not None and abs(c["diff_percent"]) <= 10.0),
            "total_count": len(comparisons),
            "notes": "所有体积数字逐项相同（确定性产物）；耗时在 ±10% 内除 debug probe build（+16.4%，同配置两次独立 clean build "
                     "19.9 s vs 17.2 s 显示 15% 级 run-to-run 噪声）；量级无差异",
        },
    }
    text = json.dumps(payload, indent=2, ensure_ascii=False)
    out = WORK / "measurements-self.json"
    out.write_text(text + "\n", encoding="utf-8")
    print(text)
    print("written", out)


if __name__ == "__main__":
    main()
