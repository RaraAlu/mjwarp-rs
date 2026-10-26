# -*- coding: utf-8 -*-
"""Re-run all QA test commands with byte-exact stdout capture (UTF-8, no PowerShell recoding).

Builds are already cached in the target dirs; this only re-executes the tests and rewrites
the evidence logs. Writes logs/exits.txt and the default-gate summary extract.
"""
import json
import os
import subprocess
from pathlib import Path

REPO = Path(r"D:\Rust\mjwarp-rs")
WORK = REPO / "target/qa-a-check"
LOGS = REPO / "docs/development-evidence/g10-p5-faithful-fma-fix-independent-check/logs"
LOGS.mkdir(parents=True, exist_ok=True)


def base_env():
    env = {key: value for key, value in os.environ.items()
           if not key.startswith(("CARGO", "RUST"))}
    env["CARGO_TERM_COLOR"] = "never"
    return env


def run(label: str, cwd: Path, target_dir: Path, args: list, log_name: str,
        rustflags: str = "") -> int:
    env = base_env()
    env["CARGO_TARGET_DIR"] = str(target_dir)
    if rustflags:
        env["RUSTFLAGS"] = rustflags
    proc = subprocess.run(["cargo", *args], cwd=str(cwd), env=env,
                          stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    (LOGS / log_name).write_bytes(proc.stdout)
    print(f"{label}: exit={proc.returncode} -> {log_name}")
    return proc.returncode


exits = []
MAIN = REPO
PRE = WORK / "pre-tree"
DUMP = WORK / "dump-tree"
CASE = ["--lib", "physics::collision::gjk::case_evidence", "--", "--nocapture"]
ASSERT = ["--lib", "physics::collision::gjk::case_evidence::upstream_f32_within_4ulp", "--", "--exact"]

# three builds, case_evidence
for mode, toolchain, rustflags in (
        ("gnu-default", [], ""),
        ("gnu-fma", [], "-C target-feature=+fma"),
        ("msvc", ["+1.99.0-x86_64-pc-windows-msvc"], "")):
    td = WORK / f"target-{mode}"
    exits.append((f"{mode}-case-evidence", run(mode, MAIN, td, [*toolchain, "test", "--locked", *CASE],
                                               f"{mode}-case-evidence.log", rustflags)))
    exits.append((f"{mode}-case-evidence-assert", run(mode, MAIN, td,
                                                      [*toolchain, "test", "--locked", *ASSERT],
                                                      f"{mode}-case-evidence-assert.log", rustflags)))

# G5 post (r2 main tree)
td = WORK / "target-gnu-default"
for name, args in (
        ("relmat", ["test", "--locked", "--lib", "physics::collision::relmat"]),
        ("sdf-relmat", ["test", "--locked", "--test", "g10_p5_sdf_relmat", "--", "--nocapture"]),
        ("sdf-host", ["test", "--locked", "--test", "g10_p5_sdf_host"]),
        ("gjk-epa", ["test", "--locked", "--test", "gjk_epa", "--", "--nocapture"])):
    exits.append((f"g5-post-{name}", run("g5-post-" + name, MAIN, td, args, f"g5-post-{name}.log")))

# default full gate (raw -> target; summary extract -> evidence)
raw_log = WORK / "default-full-test.log"
proc = subprocess.run(["cargo", "test", "--locked"], cwd=str(MAIN), env={**base_env(),
                      "CARGO_TARGET_DIR": str(td)}, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
raw_log.write_bytes(proc.stdout)
print("default-full-test exit=", proc.returncode)
lines = proc.stdout.decode("utf-8", errors="replace").splitlines()
summary = [line for line in lines if line.startswith("test result:")]
summary.append(f"test_binaries_with_running_lines={sum(1 for line in lines if line.startswith('running '))}")
(LOGS / "default-gate-summary.txt").write_text("\n".join(summary) + "\n", encoding="utf-8")
(LOGS / "default-gate-case-evidence-lines.txt").write_text(
    "\n".join(line for line in lines if "case_evidence" in line) + "\n", encoding="utf-8")
exits.append(("default-full-test", proc.returncode))

# G5 pre (parent commit copy)
td = WORK / "target-pre"
for name, args in (
        ("relmat", ["test", "--locked", "--lib", "physics::collision::relmat"]),
        ("sdf-relmat", ["test", "--locked", "--test", "g10_p5_sdf_relmat", "--", "--nocapture"]),
        ("sdf-host", ["test", "--locked", "--test", "g10_p5_sdf_host"]),
        ("gjk-epa", ["test", "--locked", "--test", "gjk_epa", "--", "--nocapture"])):
    exits.append((f"g5-pre-{name}", run("g5-pre-" + name, PRE, td, args, f"g5-pre-{name}.log")))

# pre-fix case_evidence fact check (GNU default + MSVC) on parent tree
for mode, toolchain, td_name in (
        ("gnu-default", [], "target-pre-fact-gnu"),
        ("msvc", ["+1.99.0-x86_64-pc-windows-msvc"], "target-pre-fact-msvc")):
    td = WORK / td_name
    exits.append((f"pre-{mode}-case-evidence",
                  run(f"pre-{mode}", PRE, td, [*toolchain, "test", "--locked", *CASE],
                      f"pre-fix-case-evidence-{mode}-qa.log")))
    exits.append((f"pre-{mode}-case-evidence-assert",
                  run(f"pre-{mode}-assert", PRE, td, [*toolchain, "test", "--locked", *ASSERT],
                      f"pre-fix-case-evidence-{mode}-assert-qa.log")))

# 5-case dump (r2 + QA test-only dump)
env = {**base_env(), "CARGO_TARGET_DIR": str(WORK / "target-dump"),
       "QA_A_DUMP": str(WORK / "qa-contacts-dump.json")}
proc = subprocess.run(["cargo", "test", "--locked", "--lib",
                       "physics::collision::gjk::case_evidence::qa_dump_upstream_contacts",
                       "--", "--exact", "--nocapture"],
                      cwd=str(DUMP), env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
(LOGS / "dump-test.log").write_bytes(proc.stdout)
print("dump-test exit=", proc.returncode)
exits.append(("dump-test", proc.returncode))
(LOGS / "dump-test-summary.txt").write_text(
    f"dump_exists={(WORK / 'qa-contacts-dump.json').is_file()} "
    f"dump_bytes={(WORK / 'qa-contacts-dump.json').stat().st_size if (WORK / 'qa-contacts-dump.json').is_file() else 0}\n",
    encoding="utf-8")

(LOGS / "exits.txt").write_text("\n".join(f"{label} exit={code}" for label, code in exits) + "\n",
                                encoding="utf-8")
print("QA_A_PY_RUNNER_DONE")
