"""在临时 `git worktree`（产品 HEAD 只读副本）里编译 SDF 相对旋转探针并导出结果。

产品 `physics::collision` 私有，crate 外无法调用；因此把产品代码只读副本放
`target/qa-g4/sdf-worktree`，在其 `sdf` 模块内加 `#[cfg(test)] mod qa_probe_sdf;`
（仅副本），调产品 `sdf_pair_contacts` 与「只换 rel_mat 的上游语义变体」，跑完删除 worktree。
**产品工作树、产品文件、Cargo.toml/lock 均不变。**

用法：python qa_probe_sdf_run.py
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
WORKTREE = WORK / "sdf-worktree"
PROBE_TARGET = WORK / "sdf-probe-target"
PRODUCT_COMMIT = "0eeb3766"  # G4 复核落盘后的 HEAD（产品 SDF host 与 1f882361 相同）
CASES = EVIDENCE / "qa-sdf-rotation-cases.json"
OCT = ROOT / "fixtures" / "g10-p5" / "sdf-f32"
OUT = EVIDENCE / "qa-sdf-rotation-product.json"


def run(command, cwd, env=None, check=True):
    print("+", " ".join(str(part) for part in command))
    result = subprocess.run([str(part) for part in command], cwd=str(cwd), env=env,
                            text=True, capture_output=True, check=False)
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
    run(["git", "worktree", "add", "--detach", str(WORKTREE), PRODUCT_COMMIT], ROOT)

    shutil.copyfile(EVIDENCE / "probe" / "qa_probe_sdf.rs",
                    WORKTREE / "src" / "physics" / "collision" / "sdf" / "qa_probe_sdf.rs")
    mod_path = WORKTREE / "src" / "physics" / "collision" / "sdf" / "mod.rs"
    text = mod_path.read_text(encoding="utf-8")
    anchor = "#[cfg(test)]\nmod tests;"
    assert anchor in text, "sdf/mod.rs 结构不符合预期"
    mod_path.write_text(text.replace(anchor, anchor + "\n#[cfg(test)]\nmod qa_probe_sdf;", 1),
                        encoding="utf-8")

    env = dict(os.environ)
    env["QA_SDF_CASES"] = str(CASES)
    env["QA_SDF_OCT"] = str(OCT)
    env["QA_SDF_OUT"] = str(OUT)
    env["CARGO_TARGET_DIR"] = str(PROBE_TARGET)
    run(["cargo", "test", "--locked", "--lib", "physics::collision::sdf::qa_probe_sdf", "--",
         "--nocapture"], WORKTREE, env=env)
    ok = OUT.exists()
    print("probe output exists:", ok)
    if ok:
        print("probe scenes:", len(json.loads(OUT.read_text(encoding="utf-8"))["scenes"]))
    run(["git", "worktree", "remove", "--force", str(WORKTREE)], ROOT, check=False)
    run(["git", "worktree", "prune"], ROOT, check=False)
    if not ok:
        sys.exit(1)


if __name__ == "__main__":
    main()
