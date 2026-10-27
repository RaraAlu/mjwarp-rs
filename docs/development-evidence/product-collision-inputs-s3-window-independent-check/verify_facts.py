"""QA independent verification of facts.txt, guard caveats, entity exits, budget wording and stage2 dumps."""
import json
import re
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
ATT = ROOT / "target/s3-product-wiring/attempt1"
EV = ROOT / "docs/development-evidence/product-collision-inputs-wiring"
OUT = ROOT / "target/qa-s3win/verify_facts.json"
problems = []
checks = 0


def check(cond, message):
    global checks
    checks += 1
    if not cond:
        problems.append(message)


def parse_pairs(text: str) -> dict:
    pairs = {}
    for token in re.finditer(r"([A-Za-z_][\w.]*)=([^\s]+)", text):
        pairs.setdefault(token.group(1), token.group(2))
    return pairs


def main():
    facts = {}
    exit_codes = {}
    entity_walls = {}
    for line in (ATT / "facts.txt").read_text(encoding="utf-8").splitlines():
        if line.startswith("exit_codes:"):
            for item in line.split(":", 1)[1].split():
                name, _, value = item.partition("=")
                exit_codes[name] = int(value)
            continue
        if line.startswith("entity_wall_s:"):
            for item in line.split(":", 1)[1].split():
                name, _, value = item.partition("=")
                entity_walls[name] = float(value)
            continue
        if line.startswith(("SHA_OK ", "#")):
            continue
        for key, value in parse_pairs(line).items():
            facts.setdefault(key, value)

    raw_facts = (ATT / "facts.txt").read_text(encoding="utf-8")
    sha_ok = sum(1 for line in raw_facts.splitlines() if line.startswith("SHA_OK "))
    receipt = json.loads((EV / "window-receipt.json").read_text(encoding="utf-8"))
    wf = receipt["window_facts"]

    check(len(exit_codes) == 10, f"exit codes {len(exit_codes)}")
    check(all(value == 0 for value in exit_codes.values()), f"non-zero exit: {exit_codes}")
    check(set(exit_codes) == {e["entity"] for e in receipt["entities"]}, "entity set matches receipt")
    check(sha_ok == 133, f"SHA_OK lines {sha_ok}")
    check(wf["attempt"] == 1 and wf["valid_attempts"] == 1 and wf["aborted_or_corrupted_attempts"] == 0,
          "attempt counts in receipt")
    check(not (ROOT / "target/s3-product-wiring/attempt2").exists(), "no attempt2 directory")
    check(wf["feature_profile"] == "all-features" and facts.get("feature_profile", "").startswith("all-features"),
          "feature profile")
    check("mujoco.dll" in facts.get("env_mujoco_dll", ""), "facts env_mujoco_dll")
    check("cuda-12.8.1" in facts.get("env_cuda_bin", ""), "facts env_cuda_bin")
    check(facts.get("baseline_free") == "12976" and facts.get("floor_mib") == "9904", "floor facts")
    check("min(10240" in facts.get("floor_formula", ""), "floor formula")
    check(int(facts.get("baseline_free", 0)) >= 10240, "open free >= 10240")
    check(receipt["window_facts"]["open_free_mib"] == 12976 and receipt["window_facts"]["open_free_ge_10240"] is True,
          "receipt open gate")
    check(receipt["window_facts"]["floor_mib"] == 9904, "receipt floor")

    deps_ok = facts.get("deps_exe_before") == facts.get("deps_exe_after") == "1812"
    check(deps_ok, f"deps exe count {facts.get('deps_exe_before')}/{facts.get('deps_exe_after')}")
    check(facts.get("deps_max_write_before") == facts.get("deps_max_write_after"), "deps max write unchanged")
    check(facts.get("zero_rust_compile", "").startswith("True"), "zero_rust_compile True")
    check("link absent" in raw_facts, "machine load precheck no compilers")
    check(facts.get("cpu_load_samples") == "6,5,6", "cpu load samples")
    check(facts.get("compute_apps_raw_count") == "29" and facts.get("compute_apps_filtered_count") == "0",
          "pre compute apps filtered 0")
    check(facts.get("post_compute_apps_raw_count") == "29" and facts.get("post_compute_apps_filtered_count") == "0",
          "post compute apps filtered 0")

    for key, value in [("guard_aggregate.samples", "867"), ("guard_aggregate.baseline_used_mib", "3091"),
                       ("guard_aggregate.used_peak_mib", "4142"), ("guard_aggregate.delta_peak_mib", "1051"),
                       ("guard_aggregate.increment_limit_mib", "3072"), ("guard_aggregate.free_min_mib", "11920"),
                       ("guard_aggregate.util_max_percent", "31"), ("guard_aggregate.util_streak_max", "0")]:
        check(facts.get(key) == value, f"facts {key}={facts.get(key)}")
    check(facts.get("guard_complete.samples") == "1" and facts.get("guard_complete.peak_used_mib") == "3091",
          "guard_complete caveat facts")
    caveats = receipt["sampler"]["guard_complete_caveats"]
    check("3091" in caveats["peak"] and "4142" in caveats["peak"] and "NOT the window peak" in caveats["peak"],
          "guard_complete peak caveat")
    check("867" in caveats["samples"] and "samples=1" in caveats["samples"], "guard_complete samples caveat")
    check(receipt["sampler"]["used_peak_mib"] == 4142 and receipt["sampler"]["delta_vs_guard_baseline_mib"] == 1051,
          "sampler peak quoted from aggregate")

    stage2 = next(e for e in receipt["entities"] if e["entity"] == "stage2_equivalence_window")
    check(stage2["result"]["product_dumps"] == 17 and stage2["result"]["two_captures_identical"] == 17,
          "stage2 dumps in receipt")
    check(receipt["stage2_record_only"]["product_dumps"] == 17, "stage2 record-only block")

    text_files = {
        "window-receipt.json": EV / "window-receipt.json",
        "receipt.json": EV / "receipt.json",
        "delivery record": ROOT / "docs/development-evidence/changes/2026-10-27-product-collision-inputs-s3-window.json",
        "README.md": EV / "README.md",
    }
    texts = {name: path.read_text(encoding="utf-8") for name, path in text_files.items()}
    counts = {}
    for name, text in texts.items():
        counts[name] = {"budget_deviation_zh": text.count("预算偏差"),
                        "within_budget_zh": text.count("在预算内"),
                        "attribution_inference": text.count("归因推断")}
        check(counts[name]["within_budget_zh"] == 0, f"{name} still contains within-budget wording")
    total_dev = sum(item["budget_deviation_zh"] for item in counts.values())
    check(total_dev >= 3, f"budget deviation total {total_dev}")
    check("实测 1051 MiB 单样本瞬态" in texts["window-receipt.json"], "window receipt budget sentence")
    check(counts["window-receipt.json"]["attribution_inference"] >= 1, "attribution inference wording")
    check("未证实到具体分配点" in texts["window-receipt.json"], "not proven to a specific allocation point")

    result = {
        "exit_codes": exit_codes,
        "entity_walls": entity_walls,
        "sha_ok_lines": sha_ok,
        "facts": {key: facts.get(key) for key in [
            "feature_profile", "env_mujoco_dll", "env_cuda_bin", "floor_mib", "baseline_free",
            "deps_exe_before", "deps_exe_after", "deps_max_write_before", "deps_max_write_after",
            "zero_rust_compile", "compute_apps_raw_count", "compute_apps_filtered_count",
            "guard_complete.samples", "guard_complete.peak_used_mib"]},
        "budget_wording_counts": counts,
        "budget_deviation_total": total_dev,
        "checks": checks,
        "problems": problems,
        "result": "PASS" if not problems else "FAIL",
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_bytes((json.dumps(result, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))
    print(f"checks={checks} problems={len(problems)} result={result['result']}")
    for problem in problems:
        print("PROBLEM:", problem)
    print("budget wording:", json.dumps(counts, ensure_ascii=True), "total", total_dev)


if __name__ == "__main__":
    main()
