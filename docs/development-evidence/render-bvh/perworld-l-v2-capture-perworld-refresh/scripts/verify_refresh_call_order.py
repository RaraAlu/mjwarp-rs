#!/usr/bin/env python
"""Static call-order assertion for the P0-1 per-world scene refresh fix (CPU only).

Checks (on the real repository files):
1. `scripts/capture_render_perworld.py`:
   - exactly one `mjw.render` call, inside `render_images`;
   - the statement immediately before it is `refit_scene_before_render(...)`
     (no conditional wrapping: both are direct children of the function body);
   - `capture_once` calls `render_images` exactly once, and `main` calls
     `capture_once` exactly twice (first pass + `--repeat 2` second pass) after
     `create_render_context`, so every render is preceded by a refit;
   - `refit_scene_before_render` is imported from `perworld_capture_meta`.
2. `scripts/perworld_capture_meta.py`: helper body has no control flow and
   exactly one `mjw.refit_bvh(m, d, rc)` call with parameter order (m, d, rc).
3. pinned upstream: `mujoco_warp/__init__.py` exports `refit_bvh`;
   `_src/bvh.py` defines `refit_bvh(m, d, rc)`; `refit_scene_bvh` launches bounds
   over `(d.nworld, rc.bvh_ngeom)` reading `d.geom_xpos`/`d.geom_xmat` (all worlds).
4. Negative self-tests: missing / reordered / conditional / duplicated refit and a
   wrong upstream signature must all be rejected by the same detector.

Zero GPU: pure AST/text analysis; no capture, no device.
"""

from __future__ import annotations

import ast
import pathlib
import sys

REPO = pathlib.Path(__file__).resolve().parents[5]
MAIN = REPO / "scripts/capture_render_perworld.py"
HELPER = REPO / "scripts/perworld_capture_meta.py"
UPSTREAM = REPO / "target/mjwarp-parallel/upstream"
UP_INIT = UPSTREAM / "mujoco_warp/__init__.py"
UP_BVH = UPSTREAM / "mujoco_warp/_src/bvh.py"
REFIT = "refit_scene_before_render"
RENDER_FN = "render_images"


def _call_name(node):
    if not isinstance(node, ast.Call):
        return None
    func = node.func
    if isinstance(func, ast.Name):
        return func.id
    if isinstance(func, ast.Attribute):
        return func.attr
    return None


def _attr_call(node, name, attr):
    if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute):
        return isinstance(node.func.value, ast.Name) and node.func.value.id == name and node.func.attr == attr
    return False


def analyze_main(source: str) -> list[str]:
    problems: list[str] = []
    tree = ast.parse(source)
    functions = {node.name: node for node in tree.body if isinstance(node, ast.FunctionDef)}
    render_fn = functions.get(RENDER_FN)
    if render_fn is None:
        return [f"{RENDER_FN} function missing"]
    render_idx = [i for i, stmt in enumerate(render_fn.body)
                  if isinstance(stmt, ast.Expr) and _attr_call(stmt.value, "mjw", "render")]
    if len(render_idx) != 1:
        problems.append(f"expected exactly one mjw.render in {RENDER_FN}, found {len(render_idx)}")
    else:
        index = render_idx[0]
        previous = render_fn.body[index - 1] if index > 0 else None
        if not (isinstance(previous, ast.Expr) and _call_name(previous.value) == REFIT):
            problems.append(f"{REFIT} is not the statement immediately before mjw.render")
        refit_count = sum(
            1 for stmt in render_fn.body
            if isinstance(stmt, ast.Expr) and _call_name(stmt.value) == REFIT
        )
        if refit_count != 1:
            problems.append(f"expected exactly one {REFIT} in {RENDER_FN}, found {refit_count}")
    capture = functions.get("capture_once")
    if capture is None:
        problems.append("capture_once function missing")
    else:
        renders = [stmt for stmt in ast.walk(capture) if _call_name(stmt) == RENDER_FN]
        if len(renders) != 1:
            problems.append(f"capture_once must call {RENDER_FN} exactly once, found {len(renders)}")
    main_fn = functions.get("main")
    if main_fn is None:
        problems.append("main function missing")
    else:
        capture_calls = [node for node in ast.walk(main_fn) if _call_name(node) == "capture_once"]
        context_calls = [node for node in ast.walk(main_fn) if _call_name(node) == "create_render_context"]
        if len(capture_calls) != 2:
            problems.append(f"main must call capture_once exactly twice (repeat 2), found {len(capture_calls)}")
        if len(context_calls) != 1:
            problems.append(f"main must call create_render_context exactly once, found {len(context_calls)}")
        if capture_calls and context_calls:
            context_line = min(getattr(node, "lineno", 0) for node in context_calls)
            if any(getattr(node, "lineno", 0) <= context_line for node in capture_calls):
                problems.append("capture_once is called before create_render_context")
    imports_refit = any(
        isinstance(node, ast.ImportFrom) and node.module == "perworld_capture_meta"
        and any(alias.name == REFIT for alias in node.names)
        for node in tree.body
    )
    if not imports_refit:
        problems.append(f"{REFIT} is not imported from perworld_capture_meta")
    return problems


def analyze_helper(source: str) -> list[str]:
    problems: list[str] = []
    tree = ast.parse(source)
    function = next((node for node in tree.body
                     if isinstance(node, ast.FunctionDef) and node.name == REFIT), None)
    if function is None:
        return [f"{REFIT} helper missing"]
    control = [node for node in ast.walk(function) if isinstance(node, (ast.If, ast.For, ast.While, ast.Try))]
    if control:
        problems.append("helper must not guard the refit with control flow")
    refits = [node for node in ast.walk(function) if _attr_call(node, "mjw", "refit_bvh")]
    if len(refits) != 1:
        problems.append(f"helper must call mjw.refit_bvh exactly once, found {len(refits)}")
    else:
        names = [arg.id for arg in refits[0].args if isinstance(arg, ast.Name)]
        if names != ["m", "d", "rc"]:
            problems.append(f"refit_bvh argument order must be (m, d, rc), found {names}")
    return problems


def analyze_upstream(init_source: str, bvh_source: str) -> list[str]:
    problems: list[str] = []
    if "refit_bvh as refit_bvh" not in init_source:
        problems.append("mujoco_warp.__init__ does not export refit_bvh")
    tree = ast.parse(bvh_source)
    function = next((node for node in tree.body
                     if isinstance(node, ast.FunctionDef) and node.name == "refit_bvh"), None)
    if function is None:
        problems.append("upstream _src/bvh.py has no refit_bvh")
    else:
        names = [arg.arg for arg in function.args.args]
        if names != ["m", "d", "rc"]:
            problems.append(f"upstream refit_bvh signature must be (m, d, rc), found {names}")
    scene = next((node for node in tree.body
                  if isinstance(node, ast.FunctionDef) and node.name == "refit_scene_bvh"), None)
    if scene is None:
        problems.append("upstream refit_scene_bvh missing")
    else:
        text = ast.get_source_segment(bvh_source, scene) or ""
        for needle in ("d.nworld", "rc.bvh_ngeom", "d.geom_xpos", "d.geom_xmat"):
            if needle not in text:
                problems.append(f"refit_scene_bvh does not reference {needle}")
    return problems


MAIN_BAD_MISSING = '''
from perworld_capture_meta import refit_scene_before_render

def render_images(mjm, m, d, rc):
    import mujoco_warp as mjw
    mjw.render(m, d, rc)

def capture_once(mjm, m, d, rc):
    return render_images(mjm, m, d, rc)

def main():
    rc = mjw.create_render_context(mjm)
    first = capture_once(mjm, m, d, rc)
    second = capture_once(mjm, m, d, rc)
'''

MAIN_BAD_ORDER = '''
from perworld_capture_meta import refit_scene_before_render

def render_images(mjm, m, d, rc):
    import mujoco_warp as mjw
    mjw.render(m, d, rc)
    refit_scene_before_render(m, d, rc)

def capture_once(mjm, m, d, rc):
    return render_images(mjm, m, d, rc)

def main():
    rc = mjw.create_render_context(mjm)
    first = capture_once(mjm, m, d, rc)
    second = capture_once(mjm, m, d, rc)
'''

MAIN_BAD_CONDITIONAL = '''
from perworld_capture_meta import refit_scene_before_render

def render_images(mjm, m, d, rc):
    import mujoco_warp as mjw
    if True:
        refit_scene_before_render(m, d, rc)
    mjw.render(m, d, rc)

def capture_once(mjm, m, d, rc):
    return render_images(mjm, m, d, rc)

def main():
    rc = mjw.create_render_context(mjm)
    first = capture_once(mjm, m, d, rc)
    second = capture_once(mjm, m, d, rc)
'''

MAIN_BAD_DUPLICATE = '''
from perworld_capture_meta import refit_scene_before_render

def render_images(mjm, m, d, rc):
    import mujoco_warp as mjw
    refit_scene_before_render(m, d, rc)
    refit_scene_before_render(m, d, rc)
    mjw.render(m, d, rc)

def capture_once(mjm, m, d, rc):
    return render_images(mjm, m, d, rc)

def main():
    rc = mjw.create_render_context(mjm)
    first = capture_once(mjm, m, d, rc)
    second = capture_once(mjm, m, d, rc)
'''

HELPER_BAD_CONDITIONAL = '''
def refit_scene_before_render(m, d, rc) -> None:
    import mujoco_warp as mjw
    if True:
        mjw.refit_bvh(m, d, rc)
'''

HELPER_BAD_ARGS = '''
def refit_scene_before_render(m, d, rc) -> None:
    import mujoco_warp as mjw
    mjw.refit_bvh(d, m, rc)
'''

UPSTREAM_BAD_INIT = "from mujoco_warp._src.bvh import build_bvh as build_bvh\n"
UPSTREAM_BAD_BVH = "def refit_bvh(m, d):\n    return None\n"


def main() -> int:
    checks: list[tuple[str, bool]] = []

    def record(name: str, problems: list[str], expect_ok: bool = True) -> None:
        ok = (not problems) if expect_ok else bool(problems)
        checks.append((name, ok))
        detail = f" problems={problems}" if problems else ""
        print(f"{'PASS' if ok else 'FAIL'} {name}{detail}")

    record("main-render-has-preceding-refit", analyze_main(MAIN.read_text(encoding="utf-8")))
    record("helper-no-control-flow-single-refit", analyze_helper(HELPER.read_text(encoding="utf-8")))
    record("upstream-export-and-signature", analyze_upstream(UP_INIT.read_text(encoding="utf-8"),
                                                            UP_BVH.read_text(encoding="utf-8")))
    record("negative-missing-refit", analyze_main(MAIN_BAD_MISSING), expect_ok=False)
    record("negative-refit-after-render", analyze_main(MAIN_BAD_ORDER), expect_ok=False)
    record("negative-conditional-refit", analyze_main(MAIN_BAD_CONDITIONAL), expect_ok=False)
    record("negative-duplicate-refit", analyze_main(MAIN_BAD_DUPLICATE), expect_ok=False)
    record("negative-helper-conditional", analyze_helper(HELPER_BAD_CONDITIONAL), expect_ok=False)
    record("negative-helper-arg-order", analyze_helper(HELPER_BAD_ARGS), expect_ok=False)
    record("negative-upstream-signature", analyze_upstream(UPSTREAM_BAD_INIT, UPSTREAM_BAD_BVH), expect_ok=False)

    passed = sum(1 for _, ok in checks if ok)
    print(f"SUMMARY {'PASS' if passed == len(checks) else 'FAIL'} {passed}/{len(checks)}")
    return 0 if passed == len(checks) else 1


if __name__ == "__main__":
    sys.exit(main())
