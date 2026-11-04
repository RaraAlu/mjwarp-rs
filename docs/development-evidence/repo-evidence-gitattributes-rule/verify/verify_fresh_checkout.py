"""Fresh-checkout verification for the docs/development-evidence/** -text -diff rule.

- clone-before: plain fresh clone of HEAD (rule absent) with core.autocrlf=true -> text evidence
  files are EOL-converted on checkout (sha256 != worktree).
- clone-after: same clone, then apply the new .gitattributes inside the clone only, make a
  verify-only commit, force a full re-materialisation (`git rm -r --cached .` + `git reset --hard`)
  -> checkout honours -text, bytes identical to blob == main worktree.
Compares >=10 docs/development-evidence/**/manifest.txt sha256 (clone vs main worktree).
Writes target/gitattributes-verify/clone-compare.json and prints a compact report.
"""
import json
import shutil
import subprocess
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
OUT = ROOT / "target/gitattributes-verify"


def run(args, cwd, check=True):
    result = subprocess.run(args, cwd=str(cwd), capture_output=True)
    return result.returncode, result.stdout.decode("utf-8", "replace"), result.stderr.decode("utf-8", "replace")


def sha256(path):
    import hashlib

    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def manifests():
    code, out, err = run(["git", "ls-files", "docs/development-evidence/**/manifest.txt"], ROOT)
    assert code == 0, err
    files = sorted(line.strip() for line in out.splitlines() if line.strip())
    assert len(files) >= 18, len(files)
    step = max(1, len(files) // 18)
    picked = [files[i] for i in range(0, len(files), step)][:18]
    return picked


def compare(clone, rels):
    rows = []
    equal = 0
    for rel in rels:
        wt = sha256(ROOT / rel)
        co = sha256(clone / rel) if (clone / rel).is_file() else None
        rows.append({"path": rel, "worktree_sha256": wt, "clone_sha256": co, "equal": wt == co})
        equal += int(wt == co)
    return rows, equal


def clone(dirname):
    dest = OUT / dirname
    if dest.exists():
        shutil.rmtree(dest)
    code, out, err = run(
        [
            "git",
            "clone",
            "--no-hardlinks",
            "--quiet",
            "--config",
            "core.autocrlf=true",
            "--config",
            "core.longpaths=true",
            str(ROOT),
            str(dest),
        ],
        ROOT,
    )
    assert code == 0, err
    return dest


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    rels = manifests()
    result = {
        "task": "2026-11-05-repo-evidence-gitattributes-rule",
        "head_before": run(["git", "rev-parse", "HEAD"], ROOT)[1].strip(),
        "repo": str(ROOT),
        "git_version": run(["git", "--version"], ROOT)[1].strip(),
        "core_autocrlf": run(["git", "config", "--get", "core.autocrlf"], ROOT)[1].strip(),
        "rule": "docs/development-evidence/** -text -diff",
        "manifest_sample_count": len(rels),
        "manifest_samples": rels,
        "steps": [],
    }

    # --- clone-before: fresh clone of HEAD (rule absent) ---
    before = clone("clone-before")
    cmd = "git clone --no-hardlinks --config core.autocrlf=true --config core.longpaths=true <repo> target/gitattributes-verify/clone-before"
    rows, equal = compare(before, rels)
    result["clone_before"] = {
        "clone_dir": "target/gitattributes-verify/clone-before",
        "command": cmd,
        "manifests_equal": equal,
        "manifests_total": len(rels),
        "mismatched": [row["path"] for row in rows if not row["equal"]],
        "note": "rule absent -> text evidence files are EOL-converted by core.autocrlf=true on checkout",
    }
    result["steps"].append(cmd)

    # --- clone-after: apply new rule inside the clone only, then force full re-checkout ---
    after = clone("clone-after")
    shutil.copyfile(ROOT / ".gitattributes", after / ".gitattributes")
    cmds = [
        "cp <main-worktree>/.gitattributes target/gitattributes-verify/clone-after/.gitattributes",
        "git add .gitattributes && git -c user.name=verify -c user.email=verify@local commit -m 'verify-only: docs/development-evidence/** -text -diff'",
        "git rm -r --cached .",
        "git reset --hard HEAD",
    ]
    for argv in (
        ["git", "add", ".gitattributes"],
        ["git", "-c", "user.name=verify", "-c", "user.email=verify@local", "commit", "-m", "verify-only: docs/development-evidence/** -text -diff"],
        ["git", "rm", "-r", "--cached", "."],
        ["git", "reset", "--hard", "HEAD"],
    ):
        code, out, err = run(argv, after)
        assert code == 0, f"{argv}: {err}"
    status = run(["git", "status", "--porcelain"], after)[1].strip()
    attrs = run(
        ["git", "check-attr", "text", "diff", "--", "docs/development-evidence/g10-p5-summary/receipt.json", "docs/development-evidence/np01-e-criterion-a2/manifest.txt", "docs/development-evidence/write-lock/manifest.txt"],
        after,
    )[1].strip()
    rows, equal = compare(after, rels)
    result["clone_after"] = {
        "clone_dir": "target/gitattributes-verify/clone-after",
        "commands": cmds,
        "git_status_porcelain": status,
        "check_attr_samples": attrs.splitlines(),
        "manifests_equal": equal,
        "manifests_total": len(rels),
        "mismatched": [row["path"] for row in rows if not row["equal"]],
        "rows": rows,
        "note": "rule applied -> checkout keeps blob bytes (-text), sha256 == main worktree",
    }
    result["steps"].extend(cmds)

    (OUT / "clone-compare.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({
        "head_before": result["head_before"],
        "clone_before_equal": f"{result['clone_before']['manifests_equal']}/{result['clone_before']['manifests_total']}",
        "clone_after_equal": f"{result['clone_after']['manifests_equal']}/{result['clone_after']['manifests_total']}",
        "clone_after_git_status": repr(status),
        "clone_after_check_attr": attrs.splitlines()[:4],
        "clone_before_mismatched": result["clone_before"]["mismatched"][:6],
    }, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
