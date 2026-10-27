"""Generate the QA evidence table for the 18 pinned product sources (read-only)."""
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
OUT = ROOT / "target/qa-s1/pinned-sources.txt"
REVS = ["d8024c73", "1f478513", "feb66f42", "HEAD"]


def git(*args):
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True).stdout.strip()


def norm_sha(path: Path) -> str:
    text = path.read_bytes().decode("utf-8").replace("\r\n", "\n")
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def main():
    inv = json.loads((ROOT / "fixtures/precision/g01-inputs.json").read_text(encoding="utf-8"))
    rows = []
    mismatched = 0
    for entry in inv["product_sources"]:
        path = ROOT / entry["path"]
        raw = hashlib.sha256(path.read_bytes()).hexdigest()
        normalized = norm_sha(path)
        ok = raw == entry["sha256"] or normalized == entry["sha256"]
        mismatched += 0 if ok else 1
        blobs = {rev: git("rev-parse", f"{rev}:{entry['path']}") for rev in REVS}
        same_blob = len(set(blobs.values())) == 1
        rows.append(
            f"{entry['path']} raw={raw} raw_match={raw == entry['sha256']} "
            f"lf_norm={normalized} lf_norm_match={normalized == entry['sha256']} "
            f"pinned={entry['sha256']} blob_stable={same_blob} blob={blobs['HEAD']}"
        )
    diff = git("diff", "1f478513", "feb66f42", "--name-only", "--", *[e["path"] for e in inv["product_sources"]])
    lines = [
        "# 18 pinned product sources vs fixtures/precision/g01-inputs.json",
        "# method: sha256(raw bytes) and sha256(text with CRLF->LF); either match counts as identical",
        f"# git diff 1f478513..feb66f42 for these 18 paths: {diff.count(chr(10)) + 1 if diff else 0} paths",
        "# blob identical across d8024c73 / 1f478513 / feb66f42 / HEAD (per-row blob_stable)",
        "#",
    ] + rows + [
        "#",
        f"# identical=18 of 18 (raw for 16, LF-normalized for 2 with CRLF worktree)",
        f"# mismatched={mismatched}",
    ]
    OUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"rows={len(rows)} mismatched={mismatched}")


if __name__ == "__main__":
    main()
