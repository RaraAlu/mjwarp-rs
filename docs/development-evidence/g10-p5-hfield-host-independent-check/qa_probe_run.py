"""在临时 `git worktree`（G4 提交 1f882361 副本）里编译 QA 探针，导出产品 hfield 输出。

产品 `physics::collision` 是私有模块，crate 外无法调用；因此把产品代码的**只读副本**
放到 `target/qa-g4/worktree`（git 忽略、提交后即删），在其 `hfield` 模块内加一个
`#[cfg(test)] mod qa_probe;`（仅该副本），调用产品 `hfield_pair_contacts` 与
`hfield_filter` 导出行为。**产品工作树、产品文件、Cargo.toml/lock 均不变。**

用法：python qa_probe_run.py
"""

import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

EVIDENCE = Path(__file__).resolve().parent
ROOT = EVIDENCE.parents[2]
WORK = ROOT / "target" / "qa-g4"
WORKTREE = WORK / "worktree"
PROBE_TARGET = WORK / "probe-target"
COMMIT = "1f882361"
CASES = EVIDENCE / "qa-upstream-cases.json"
OUT = EVIDENCE / "qa-product-output.json"


def run(command, cwd, env=None, check=True):
    print("+", " ".join(str(part) for part in command))
    result = subprocess.run(command, cwd=str(cwd), env=env, text=True,
                            capture_output=True, check=False)
    if result.stdout:
        print(result.stdout[-4000:])
    if result.stderr:
        print(result.stderr[-4000:])
    if check and result.returncode != 0:
        raise SystemExit(f"命令失败（{result.returncode}）：{command}")
    return result


def main():
    WORK.mkdir(parents=True, exist_ok=True)
    if WORKTREE.exists():
        run(["git", "worktree", "remove", "--force", str(WORKTREE)], ROOT, check=False)
        if WORKTREE.exists():
            shutil.rmtree(WORKTREE, ignore_errors=True)
    run(["git", "worktree", "prune"], ROOT, check=False)
    run(["git", "worktree", "add", "--detach", str(WORKTREE), COMMIT], ROOT)

    probe_source = EVIDENCE / "probe" / "qa_probe.rs"
    target_probe = WORKTREE / "src" / "physics" / "collision" / "hfield" / "qa_probe.rs"
    shutil.copyfile(probe_source, target_probe)
    mod_path = WORKTREE / "src" / "physics" / "collision" / "hfield" / "mod.rs"
    text = mod_path.read_text(encoding="utf-8")
    anchor = "#[cfg(test)]\nmod tests;"
    assert anchor in text, "hfield/mod.rs 结构不符合预期"
    mod_path.write_text(text.replace(anchor, anchor + "\n#[cfg(test)]\nmod qa_probe;", 1),
                        encoding="utf-8")

    env = dict(os.environ)
    env["QA_CASES"] = str(CASES)
    env["QA_OUT"] = str(OUT)
    env["CARGO_TARGET_DIR"] = str(PROBE_TARGET)
    result = run(
        ["cargo", "test", "--locked", "--lib", "physics::collision::hfield::qa_probe", "--",
         "--nocapture"],
        WORKTREE, env=env,
    )
    ok = OUT.exists()
    print("probe output exists:", ok)
    if ok:
        value = json.loads(OUT.read_text(encoding="utf-8"))
        print("probe scenes:", len(value["scenes"]))
    # 清理：移除临时 worktree（不动产品树）
    run(["git", "worktree", "remove", "--force", str(WORKTREE)], ROOT, check=False)
    run(["git", "worktree", "prune"], ROOT, check=False)
    if not ok:
        sys.exit(1)
    return result


if __name__ == "__main__":
    main()
