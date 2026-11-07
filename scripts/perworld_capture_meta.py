"""Capture-metadata prerequisites for the per-world authoring script (spec v2 §11.1).

OFFLINE ONLY: imported by `scripts/capture_render_perworld.py`; not imported by
product builds/tests/gates. This module owns the tooling/metadata surface of the
S-2 capture-window prerequisites:

1. `command`: the actual command line (`argv` verbatim + rendered + cwd);
2. `captured_at`: strict non-empty ISO8601 from `MJWARP_CAPTURE_AT` (missing or
   empty -> explicit non-zero failure; never write an empty string);
3. `per_world_scope`: new coverage wording (per-world light poses + per-world
   geom poses, with the host-only light semantics premise of spec v2 §10.3);
4. reference identity: keep the script task id as the provenance source and add
   `reference_tag` (reference directory name by convention = spec §2.2 tag),
   instead of parametrising `task_id`.

Zero GPU: pure metadata helpers; the module also provides a CPU self-check with
positive/negative cases (`python -B scripts/perworld_capture_meta.py`).

Boundary: this module produces no reference and makes no capture-validity or
numeric claim.
"""

from __future__ import annotations

import datetime
import pathlib
import subprocess
import sys

TASK_ID_SOURCE = "script-constant"
TASK_ID = "2026-10-30-render-perworld-capture-authoring"
SCOPE_COVERS = (
    "per-world light poses (light_xpos/light_xdir) AND per-world geom poses "
    "(geom_xpos/geom_xmat) with nworld > 1; device-path light semantics are "
    "host-only retransmission (kernel light arrays stay 1-D, worldid == 0 per "
    "launch, .cu unchanged) as fixed in spec v2 §10.3"
)
SCOPE_NOT_COVERS = (
    "AA x perworld (nsamples>1 still Unsupported), single-launch performance "
    "plan K, continuous animation / large world counts, and any "
    "upstream-implementation equivalence claim"
)
OLD_SCOPE_DOES_NOT_COVER = "per-world light poses (device path indexes lights single-world)"


def fail(message: str) -> None:
    raise SystemExit(f"perworld capture metadata: {message}")


def command_line(argv, cwd) -> dict:
    """Actual command line; argv is recorded verbatim (non-empty)."""
    items = [str(item) for item in argv]
    while items and not items[-1].strip():
        items.pop()
    if not items:
        fail("command argv is empty (the actual command line must be recorded)")
    rendered = subprocess.list2cmdline(items)
    if not rendered.strip():
        fail("command line rendering is empty")
    return {"argv": items, "cwd": str(cwd), "rendered": rendered}


def require_capture_at(env=None) -> str:
    """Strict non-empty ISO8601 `MJWARP_CAPTURE_AT`; explicit failure otherwise."""
    if env is None:
        env = __import__("os").environ
    raw = env.get("MJWARP_CAPTURE_AT")
    if raw is None:
        fail("MJWARP_CAPTURE_AT is missing; set an explicit non-empty ISO8601 timestamp before capture")
    if not str(raw).strip():
        fail("MJWARP_CAPTURE_AT is empty; set an explicit non-empty ISO8601 timestamp before capture")
    text = str(raw).strip()
    try:
        datetime.datetime.fromisoformat(text.replace("Z", "+00:00"))
    except ValueError:
        fail(f"MJWARP_CAPTURE_AT is not ISO8601: {text!r}")
    return text


def capture_at_env_echo(env=None) -> dict:
    """Dry-run state echo only; dry-run never consumes the env value."""
    if env is None:
        env = __import__("os").environ
    raw = env.get("MJWARP_CAPTURE_AT")
    return {"present": raw is not None, "non_empty": bool(str(raw or "").strip())}


def per_world_scope() -> dict:
    """New coverage wording; field names kept for existing consumers."""
    return {"covers": SCOPE_COVERS, "does_not_cover": SCOPE_NOT_COVERS}


def reference_tag(cli_tag, out_dir) -> str:
    """Reference identity: CLI override wins, else the reference directory name (spec §2.2)."""
    if cli_tag is not None and str(cli_tag).strip():
        return str(cli_tag).strip()
    name = pathlib.Path(out_dir).name.strip()
    if not name:
        fail(f"cannot derive reference_tag from out_dir {out_dir!r}; pass --reference-tag")
    return name


def dry_run_echo(argv, cwd, cli_tag, out_dir, env=None) -> dict:
    """Additive dry-run echo: command / reference_tag / captured_at_env (env not consumed)."""
    return {
        "command": command_line(argv, cwd),
        "reference_tag": reference_tag(cli_tag, out_dir),
        "captured_at_env": capture_at_env_echo(env),
    }


def capture_metadata(argv, cwd, cli_tag, out_dir, env=None) -> dict:
    """Concrete capture.json metadata block for the four §11.1 prerequisites."""
    return {
        "command": command_line(argv, cwd),
        "captured_at": require_capture_at(env),
        "reference_tag": reference_tag(cli_tag, out_dir),
        "per_world_scope": per_world_scope(),
        "task_id": TASK_ID,
        "task_id_source": TASK_ID_SOURCE,
    }


def verify_capture_metadata(capture: dict) -> None:
    """Post-write self-check: non-empty command/captured_at/reference_tag and new scope wording."""
    command = capture.get("command") or {}
    if not command.get("argv") or not str(command.get("rendered") or "").strip():
        fail("capture.json command is empty (spec v2 §11.1 item 1)")
    captured_at = str(capture.get("captured_at") or "").strip()
    if not captured_at:
        fail("capture.json captured_at is empty (spec v2 §11.1 item 2)")
    try:
        datetime.datetime.fromisoformat(captured_at.replace("Z", "+00:00"))
    except ValueError:
        fail(f"capture.json captured_at is not ISO8601: {captured_at!r}")
    if not str(capture.get("reference_tag") or "").strip():
        fail("capture.json reference_tag is empty (spec v2 §11.1 item 4)")
    scope = capture.get("per_world_scope") or {}
    covers = str(scope.get("covers") or "")
    does_not_cover = str(scope.get("does_not_cover") or "")
    if "per-world light poses" not in covers or "geom poses" not in covers:
        fail("capture.json per_world_scope.covers does not state per-world light + geom coverage")
    if OLD_SCOPE_DOES_NOT_COVER in does_not_cover:
        fail("capture.json per_world_scope.does_not_cover still carries the old light-poses wording")
    if not str(capture.get("recipe", {}).get("task_id") or "").strip():
        fail("capture.json recipe.task_id is missing (task-id provenance source)")


def _case(name: str, body) -> tuple[str, bool]:
    try:
        body()
        return name, True
    except SystemExit as error:
        print(f"FAIL {name}: unexpected exit {error}")
        return name, False


def _case_expect_exit(name: str, body, needle: str) -> tuple[str, bool]:
    try:
        body()
    except SystemExit as error:
        text = str(error)
        if needle in text:
            return name, True
        print(f"FAIL {name}: exit message missing {needle!r}: {text}")
        return name, False
    print(f"FAIL {name}: expected a non-zero exit")
    return name, False


def selfcheck() -> int:
    """CPU positive/negative cases for the four prerequisites (no GPU, no capture)."""
    iso = "2026-11-07T13:00:00+08:00"
    results = [
        _case("command-positive", lambda: command_line([sys.executable, "scripts/capture_render_perworld.py", "--dry-run"], pathlib.Path.cwd())),
        _case_expect_exit("command-negative-empty-argv", lambda: command_line([], pathlib.Path.cwd()), "argv is empty"),
        _case("captured-at-positive", lambda: require_capture_at({"MJWARP_CAPTURE_AT": iso})),
        _case_expect_exit("captured-at-negative-missing", lambda: require_capture_at({}), "is missing"),
        _case_expect_exit("captured-at-negative-empty", lambda: require_capture_at({"MJWARP_CAPTURE_AT": "   "}), "is empty"),
        _case_expect_exit("captured-at-negative-not-iso", lambda: require_capture_at({"MJWARP_CAPTURE_AT": "not-a-time"}), "not ISO8601"),
        _case("scope-positive-new-wording", lambda: _scope_asserts()),
        _case_expect_exit("scope-negative-old-wording", lambda: _scope_old_negative(), "old light-poses wording"),
        _case("reference-tag-positive-from-dir", lambda: _tag_from_dir()),
        _case("reference-tag-positive-cli-override", lambda: _tag_override()),
        _case_expect_exit("reference-tag-negative-empty-dir", lambda: reference_tag(None, ""), "cannot derive reference_tag"),
        _case("capture-metadata-positive", lambda: verify_capture_metadata(_synthetic_capture(iso))),
        _case_expect_exit("capture-metadata-negative-empty-command", lambda: verify_capture_metadata(_synthetic_capture(iso, command={"argv": [], "rendered": ""})), "command is empty"),
        _case_expect_exit("capture-metadata-negative-old-scope", lambda: verify_capture_metadata(_synthetic_capture(iso, scope={"covers": SCOPE_COVERS, "does_not_cover": OLD_SCOPE_DOES_NOT_COVER})), "old light-poses wording"),
    ]
    passed = sum(1 for _, ok in results if ok)
    print(f"SELFCHECK {'PASS' if passed == len(results) else 'FAIL'} {passed}/{len(results)}")
    return 0 if passed == len(results) else 1


def _scope_asserts() -> None:
    scope = per_world_scope()
    if "per-world light poses" not in scope["covers"] or "geom poses" not in scope["covers"]:
        fail("new scope wording misses light/geom coverage")
    if OLD_SCOPE_DOES_NOT_COVER in scope["does_not_cover"]:
        fail("new scope still carries the old light-poses wording")


def _scope_old_negative() -> None:
    legacy = {"covers": "per-world geom poses", "does_not_cover": OLD_SCOPE_DOES_NOT_COVER}
    if OLD_SCOPE_DOES_NOT_COVER in legacy["does_not_cover"]:
        fail("old light-poses wording is still present in the legacy shape")


def _tag_from_dir() -> None:
    if reference_tag(None, "fixtures/render-bvh/scene-a-light/perworld/nw3-light-hinge02040") != "nw3-light-hinge02040":
        fail("reference_tag derivation from directory name failed")


def _tag_override() -> None:
    if reference_tag("tag-x", "fixtures/whatever") != "tag-x":
        fail("reference_tag CLI override failed")


def _synthetic_capture(captured_at: str, command=None, scope=None) -> dict:
    return {
        "command": command if command is not None else command_line([sys.executable, "script.py"], pathlib.Path.cwd()),
        "captured_at": captured_at,
        "reference_tag": "nw3-light-hinge02040",
        "per_world_scope": scope if scope is not None else per_world_scope(),
        "recipe": {"task_id": TASK_ID},
    }


if __name__ == "__main__":
    raise SystemExit(selfcheck())
