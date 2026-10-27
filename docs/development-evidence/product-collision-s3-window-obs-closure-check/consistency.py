"""Line counts, closure-receipt line-ref block check and delivery-record consistency (v3, post-fix)."""
import json
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
EV = ROOT / "docs/development-evidence/product-collision-inputs-wiring"
DEL = ROOT / "docs/development-evidence/changes"
SELF = ROOT / "docs/development-evidence/product-collision-s3-window-obs-closure-check"
OUT = SELF / "logs/line-counts-and-refs.txt"

FILES = {
    "stdout copy": EV / "logs/attempt1-runner-stdout.log",
    "window-receipt": EV / "window-receipt.json",
    "receipt": EV / "receipt.json",
    "README": EV / "README.md",
    "closure receipt": EV / "s3-window-observation-closure-receipt.json",
    "window delivery": DEL / "2026-10-27-product-collision-inputs-s3-window.json",
    "closure delivery": DEL / "2026-10-27-product-collision-inputs-s3-window-observation-closure.json",
}

EXPECTED = {
    "stdout copy": 183,
    "window-receipt": 620,
    "receipt": 750,
    "README": 475,
    "closure receipt": 238,
    "window delivery": 325,
    "closure delivery": 78,
}


def text(path: Path) -> str:
    raw = path.read_bytes()
    return raw.decode("utf-16") if raw.startswith(b"\xff\xfe") else raw.decode("utf-8")


def rows_of(path: Path) -> list[str]:
    return text(path).splitlines()


def main():
    out = ["# line counts / encodings (v3, after evidence-only fixes 71e78aae / ee5d0ca4)", "#"]
    stats = {}
    count_bad = []
    for name, path in FILES.items():
        raw = path.read_bytes()
        value = text(path)
        stats[name] = {"bytes": len(raw), "lines": len(value.splitlines()),
                       "bom": raw.startswith(b"\xef\xbb\xbf"), "crlf": b"\r\n" in raw,
                       "utf16": raw.startswith(b"\xff\xfe")}
        flag = "" if stats[name]["lines"] == EXPECTED[name] else " MISMATCH"
        if flag:
            count_bad.append(name)
        out.append(f"{name}: bytes={len(raw)} lines={len(value.splitlines())} "
                   f"bom={stats[name]['bom']} utf16={stats[name]['utf16']} crlf={stats[name]['crlf']}{flag}")

    closure = json.loads((EV / "s3-window-observation-closure-receipt.json").read_text(encoding="utf-8"))
    targets = {
        "window_receipt": EV / "window-receipt.json",
        "receipt": EV / "receipt.json",
        "window_delivery": DEL / "2026-10-27-product-collision-inputs-s3-window.json",
        "readme": EV / "README.md",
    }
    markers = {
        "O1": ["stdout_log_encoding", "UTF-16", "40380", "183"],
        "O2": ["17:03:36.808", "86 ms", "0.1", "17:03:37.658", "38.443", "0.9"],
        "O3": ["neighbors_used_mib", "3092", "4142"],
        "O4": ["sample_interval", "0.276", "0.2721", "0.2901"],
        "O5": ["evidence_format", "receipt.json", "750"],
        "O6": ["runner_outer_wall_s", "derived", "252.1"],
        "O7": ["statistics_basis", "统计口径", "7"],
    }
    problems = []
    out += ["", "# closure receipt line_refs: cited line plus the following block must contain the markers", "#"]
    for obs, refs in closure["observations"].items():
        for key, line_no in refs.get("line_refs", {}).items():
            if key not in targets:
                continue
            rows = rows_of(targets[key])
            if not 1 <= line_no <= len(rows):
                problems.append(f"{obs}.{key} out of range {line_no}/{len(rows)}")
                continue
            window = "\n".join(rows[max(0, line_no - 1):line_no + 12])
            wanted = [marker for marker in markers.get(obs, []) if marker in window]
            if not wanted:
                problems.append(f"{obs}.{key} line {line_no} block lacks {markers.get(obs)}")
            out.append(f"{obs}.{key} line {line_no}: markers={wanted}")

    delivery = json.loads((DEL / "2026-10-27-product-collision-inputs-s3-window.json").read_text(encoding="utf-8"))
    wd_text = json.dumps(delivery["evidence_format"], ensure_ascii=False)
    wr_text = text(EV / "window-receipt.json")
    readme_text = text(EV / "README.md")
    actual = stats["receipt"]["lines"]
    findings = []
    out += ["", f"# receipt.json actual lines = {actual}; window-receipt says 750: {'750' in wr_text}; "
                f"README says 750: {'750' in readme_text}; window delivery says 745: {'745' in wd_text}, "
                f"750: {'750' in wd_text}"]
    out.append("window delivery machine_readable_exceptions receipt.json entry: "
               + next((row.strip() for row in wd_text.splitlines() if "receipt.json (" in row), "(not found)"))
    if "745" in wd_text and "750" not in wd_text:
        findings.append("window delivery record lists receipt.json as 745 lines while actual/window-receipt/README say 750")
    if actual != 750 or "750" not in wd_text:
        findings.append("receipt.json line count / delivery declaration mismatch")
    readme_bytes_rows = [row for row in readme_text.splitlines() if "7453" in row]
    out.append("# README 7453 occurrences (e2e target bytes; not a line-count exception): "
               + json.dumps(readme_bytes_rows, ensure_ascii=False))
    closure_delivery = json.loads((DEL / "2026-10-27-product-collision-inputs-s3-window-observation-closure.json").read_text(encoding="utf-8"))
    out.append("closure delivery milestones O4: " + json.dumps(closure_delivery["milestones"].get("O4"), ensure_ascii=False))
    out.append("closure receipt O4 detail range: " + json.dumps(closure["observations"]["O4"]["detail"], ensure_ascii=False))
    out += ["", f"# problems: {len(problems)}"] + problems
    out += [f"# findings: {len(findings)}"] + findings
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_bytes(("\n".join(out) + "\n").encode("utf-8"))
    print("\n".join(out[-8:]))
    print("problems:", len(problems), "findings:", len(findings))


if __name__ == "__main__":
    main()
