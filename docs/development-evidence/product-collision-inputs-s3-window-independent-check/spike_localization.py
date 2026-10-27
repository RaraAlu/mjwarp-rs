"""QA evidence: spike localization vs ENTITY_DONE/log mtimes, and freeze provenance cross-check."""
import json
from datetime import datetime, timedelta, timezone
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
ATT = ROOT / "target/s3-product-wiring/attempt1"
FREEZE = ROOT / "target/s3-product-wiring-freeze"
EV = ROOT / "docs/development-evidence/product-collision-inputs-wiring"
OUT = ROOT / "target/qa-s3win/spike-localization.txt"
OUT2 = ROOT / "target/qa-s3win/freeze-provenance.txt"
TZ = timezone(timedelta(hours=8))


def ts(text: str) -> datetime:
    main, _, off = text.partition("+")
    head, _, frac = main.partition(".")
    return datetime.fromisoformat(head + "." + (frac + "000000")[:6] + "+" + off)


def fmt(value: datetime) -> str:
    return value.strftime("%Y-%m-%dT%H:%M:%S.%f")[:-3]


def mtime(path: Path) -> datetime:
    return datetime.fromtimestamp(path.stat().st_mtime, tz=TZ)


def read_manifest(path: Path) -> dict:
    rows = {}
    for line in path.read_text(encoding="utf-8-sig").splitlines():
        if line.startswith("#") or "|" not in line:
            continue
        parts = [p.strip() for p in line.split("|")]
        rows[parts[0]] = parts[3]
    return rows


def main():
    receipt = json.loads((EV / "window-receipt.json").read_text(encoding="utf-8"))
    spike = receipt["sampler"]["observation_OW1"]
    spike_t = ts(spike["timestamp"])
    window_start = ts(receipt["window_facts"]["window_start"])
    window_end = ts(receipt["window_facts"]["window_end"])
    logs = {path.name: mtime(path) for path in sorted(ATT.glob("g10*.log"))}
    logs["stage2_equivalence_window.log"] = mtime(ATT / "stage2_equivalence_window.log")
    stage2_start_by_mtime = logs["stage2_equivalence_window.log"] - timedelta(seconds=21.47)
    stage2_start_by_end = window_end - timedelta(seconds=21.5)
    launcher_mtime = logs["g10_p5_nvrtc_launcher.log"]

    lines = [
        "# Spike localization, independent recomputation (attempt1)",
        "",
        f"spike: seq={spike['seq']} timestamp={spike['timestamp']} used={spike['used_mib']} MiB",
        "",
        "# runner writes each entity log AFTER the process finishes",
        "#   (run-validation-attempt1.ps1 line 253 Write-Utf8, then line 256 ENTITY_DONE),",
        "#   so a log mtime is >= the entity exit time.",
        f"launcher log mtime            = {fmt(launcher_mtime)}",
        f"prev entity (m3) log mtime    = {fmt(logs['g10_p5_mesh_multicontact_m3_cuda.log'])}",
        f"spike timestamp               = {fmt(spike_t)}",
        f"spike - launcher log mtime    = {(spike_t - launcher_mtime).total_seconds():+.3f} s",
        "",
        "stage2 start bounds:",
        f"  from stage2 log mtime - 21.47 s wall = {fmt(stage2_start_by_mtime)}",
        f"  from window_end - 21.5 s wall        = {fmt(stage2_start_by_end)}",
        f"spike - earliest stage2 start bound    = {(spike_t - stage2_start_by_mtime).total_seconds():+.3f} s",
        "",
        "=> the spike is within about 0.1 s of the launcher exit/log-write boundary and at least",
        "   ~0.9 s BEFORE the stage2 start. The record's 'launcher exit + about 0.7 s' is not",
        "   supported by the artifacts: the launcher log was written at 17:03:36.808,",
        "   86 ms after the spike sample.",
        "",
        f"window start/end: {receipt['window_facts']['window_start']} .. {receipt['window_facts']['window_end']}",
        f"body wall from start/end: {(window_end - window_start).total_seconds():.3f} s (receipt 238.5)",
        "",
        "entity log mtimes (write-after-exit):",
    ]
    for name, value in logs.items():
        lines.append(f"  {name} {fmt(value)}")
    OUT.write_bytes(("\n".join(lines) + "\n").encode("utf-8"))

    m3_prev = read_manifest(ROOT / "target/m3-freeze/frozen-shas.txt")
    s4_prev = read_manifest(ROOT / "target/s4-freeze/frozen-shas.txt")
    s3_rows = {}
    for line in (FREEZE / "frozen-shas.txt").read_text(encoding="utf-8").splitlines():
        if line.startswith("#") or "|" not in line:
            continue
        parts = [p.strip() for p in line.split("|")]
        s3_rows[parts[0]] = {"sha256": parts[3],
                             "source": next(p.split("=", 1)[1] for p in parts if p.startswith("source="))}
    lines2 = ["# Freeze provenance cross-check (S3 freeze rows vs the signed M3/S4 freeze manifests)", "",
              f"S3 freeze rows: {len(s3_rows)}", ""]
    matched = 0
    for file, row in s3_rows.items():
        if row["source"].startswith("s3-build"):
            lines2.append(f"{file} source=s3-build -> new build from commit 0c9497e4 (no prior signed copy expected)")
            continue
        prior = (s4_prev if row["source"].startswith("s4-build") else m3_prev).get(file)
        ok = prior == row["sha256"]
        matched += 1 if ok else 0
        lines2.append(f"{file} source={row['source']} sha256={row['sha256']} prior_signed={prior} match={ok}")
    lines2 += ["", f"non-s3-build rows matching their prior signed manifest: {matched}/9",
               "note: the launcher row is the S4 rebuild (hfield-capsule) and matches s4-freeze, not the older m3-freeze copy."]
    OUT2.write_bytes(("\n".join(lines2) + "\n").encode("utf-8"))
    print("wrote", OUT, OUT2)
    print("provenance matched:", matched, "of 9")
    print("spike-launcher:", f"{(spike_t - launcher_mtime).total_seconds():+.3f} s")


if __name__ == "__main__":
    main()
