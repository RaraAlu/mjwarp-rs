"""QA independent verification of the S7a render PTX gate (A') and launch ABI.

The gate reference here is a from-scratch implementation of the required allowlist
semantics; it is applied to the same PTX inputs the product gate saw (files written by
the QA harness), so verdicts are compared 1:1 without reading delivered test assertions.
"""

from __future__ import annotations

import hashlib
import json
import pathlib
import re
import subprocess

ROOT = pathlib.Path(r"D:\Rust\mjwarp-rs")
BASE = ROOT / "target/qa-render-bvh-s7a"
REPORT = BASE / "harness-report.json"
OUT = BASE / "harness-compare.json"

SVD_EPS = "0d3D719799812DEA11"
SIN_COS = "0d3BF921FB54442D19"
ALLOWED = [
    f".global .align 8 .f64 _ZN2wp11_svd_configIdE11SVD_EPSILONE = {SVD_EPS};",
    f".global .align 8 .f64 _ZN2wp11_svd_configIdE17QR_GIVENS_EPSILONE = {SVD_EPS};",
]
FORBIDDEN_TOKENS = {"f16", "f16x2", "bf16", "bf16x2", "tf32"}


def reference_gate(ptx: str, entry: str) -> str | None:
    """Return None if accepted, else a rejection reason (independent implementation)."""
    lines = []
    for raw in ptx.splitlines():
        code = raw.split("//", 1)[0].strip()
        if code:
            lines.append(code)
    ops = lambda line: [w for w in re.split(r"[\s,;]+", line) if w]
    seen = [0, 0]
    reductions = 0
    has_f64_reg = False
    index = 0
    while index < len(lines):
        code = lines[index]
        if code.startswith(".reg .f64 ") or code.startswith(".reg .f64\t"):
            has_f64_reg = True
            index += 1
            continue
        current = ops(code)
        if len(current) == 3 and current[0] == "cvt.rn.f64.s64" and index + 2 < len(lines):
            second = ops(lines[index + 1])
            third = ops(lines[index + 2])
            if (
                len(second) == 4
                and second[0] in ("mul.rn.f64", "mul.f64")
                and second[2] == current[1]
                and second[3] == SIN_COS
                and len(third) == 3
                and third[0] == "cvt.rn.f32.f64"
                and third[2] == second[1]
            ):
                reductions += 1
                index += 3
                continue
        f64_line = ".f64 " in code or ".f64;" in code or code.endswith(".f64")
        if f64_line:
            if code.startswith(".global"):
                if code in ALLOWED:
                    slot = ALLOWED.index(code)
                    if seen[slot]:
                        return "duplicate allowlisted constant"
                    seen[slot] = 1
                    index += 1
                    continue
                return "unexpected f64 global"
            if code in ALLOWED:
                return "misplaced allowlisted constant"
            return "unexpected f64 line"
        tokens = []
        for word in code.split():
            tokens.extend(word.split(".")[1:])
        for token in tokens:
            cleaned = re.sub(r"[^A-Za-z0-9]", "", token)
            if cleaned in FORBIDDEN_TOKENS:
                return f"forbidden token {cleaned}"
        index += 1
    if has_f64_reg and reductions == 0:
        return "f64 register without sin/cos reduction"
    if f".visible .entry {entry}" not in ptx:
        return "missing entry"
    return None


def f64_token_scan(ptx: str) -> dict:
    globals_found = []
    other = []
    for number, raw in enumerate(ptx.splitlines(), start=1):
        code = raw.split("//", 1)[0]
        if ".f64" not in code:
            continue
        trimmed = code.strip()
        if trimmed.startswith(".global") and "svd_config" in trimmed:
            globals_found.append((number, trimmed))
        else:
            other.append((number, trimmed))
    return {
        "global_svd_lines": globals_found,
        "non_global_f64_lines": other,
        "global_count": len(globals_found),
    }


def main() -> int:
    report = json.loads(REPORT.read_text(encoding="utf-8"))
    real_ptx = (BASE / "w15-real.ptx").read_text(encoding="utf-8")
    failures: list[str] = []

    variants = report["gate"]["variants"]
    variant_results = {}
    for item in variants:
        name = item["name"]
        path = BASE / f"variant-{name}.ptx"
        text = path.read_text(encoding="utf-8")
        entry = "s7a_w15_probe_missing" if name == "missing_entry" else "s7a_w15_probe"
        reason = reference_gate(text, entry)
        qa_ok = reason is None
        product_ok = bool(item["ok"])
        expected = bool(item["expected_ok"])
        # The sin/cos triplet variant is permitted by both implementations (f64 register
        # lines are only legal when a reduction triplet accounts for them).
        variant_results[name] = {
            "qa_ok": qa_ok,
            "qa_reason": reason,
            "product_ok": product_ok,
            "product_error": item["error"],
            "verdicts_equal": qa_ok == product_ok,
            "matches_expected": qa_ok == expected == product_ok,
        }
        if qa_ok != product_ok:
            failures.append(f"gate verdict mismatch for {name}: qa={reason} product={item['error']}")
        if qa_ok != expected:
            failures.append(f"gate expectation mismatch for {name}: qa_ok={qa_ok} expected={expected}")

    # Real PTX: all f64 tokens must be exactly the two allowlisted dead-data constants.
    scan = f64_token_scan(real_ptx)
    if scan["global_count"] != 2:
        failures.append(f"real PTX has {scan['global_count']} f64 globals, expected 2")
    if scan["non_global_f64_lines"]:
        failures.append(f"real PTX has non-global f64 lines: {scan['non_global_f64_lines'][:5]}")
    expected_globals = [
        [str(ALLOWED[0]), 0],
        [str(ALLOWED[1]), 0],
    ]
    found_lines = [line for _, line in scan["global_svd_lines"]]
    for expected in ALLOWED:
        if expected not in found_lines:
            failures.append(f"real PTX missing allowlisted constant: {expected}")
    if sorted(found_lines) != sorted(ALLOWED):
        failures.append("real PTX f64 globals differ from the allowlist")
    # literal/symbol byte-exact check
    for line in found_lines:
        match = re.match(r"\.global \.align 8 \.f64 (_ZN2wp11_svd_configIdE\w+) = (0d[0-9A-F]+);$", line)
        if not match:
            failures.append(f"unexpected global f64 shape: {line}")
        elif match.group(2) != SVD_EPS:
            failures.append(f"unexpected f64 literal: {line}")
    ptx_report = report["ptx"]
    ptx_sha = hashlib.sha256(real_ptx.encode()).hexdigest()
    if ptx_sha != ptx_report["sha256"]:
        failures.append("harness ptx sha mismatch")
    if ptx_report["sha256"] != "d075f329132b5ffec750f004682fde52236880c7a85e1fde641edcedefba501e":
        failures.append(f"PTX sha256 {ptx_report['sha256']} != delivered d075f329...")
    if ptx_report["source_sha256"] != "0a319b89585e67d74c7c1e63d6e03f3903582221e62c58d6bb8b7c6654126a5d":
        failures.append(f"source sha256 {ptx_report['source_sha256']} != delivered 0a319b89...")
    if len(found_lines) == 2 and sorted(found_lines) == sorted(ALLOWED) and not scan["non_global_f64_lines"]:
        exclusive = True
    else:
        exclusive = False

    # ABI independent recomputation.
    abi = report["abi"]
    expected_elements = {
        "i32": (4, 4),
        "u32": (4, 4),
        "u64": (8, 8),
        "bool": (1, 1),
        "f32": (4, 4),
        "vec2f": (8, 4),
        "vec3f": (12, 4),
        "vec4f": (16, 4),
        "vec2i": (8, 4),
        "vec3i": (12, 4),
        "mat33f": (36, 4),
        "quatf": (16, 4),
        "texture2d_t": (24, 8),
    }
    element_checks = {}
    got_elements = {item["name"]: (item["bytes"], item["align"]) for item in abi["elements"]}
    for name, expected in expected_elements.items():
        ok = got_elements.get(name) == expected
        element_checks[name] = {"expected": expected, "actual": got_elements.get(name), "ok": ok}
        if not ok:
            failures.append(f"element {name} layout {got_elements.get(name)} != {expected}")
    slot_checks = {
        "slots_81": abi["slots"] == 81,
        "arrays_73": abi["arrays"] == 73,
        "scalars_7": abi["scalars"] == 7,
        "one_bounds_slot": abi["slots"] - abi["arrays"] - abi["scalars"] == 1,
        "all_classes_present": set(got_elements) == set(expected_elements),
        "texture2d_size_24": abi["size_of_texture2d"] == 24,
        "texture2d_align_8": abi["align_of_texture2d"] == 8,
        "render_array_56": abi["size_of_render_array"] == 56,
        "texture2d_bytes_const_24": abi["texture2d_bytes_const"] == 24,
        "signature_len_81": abi["signature_len"] == 81,
        "signature_name": abi["signature_name"] == "_render_megakernel",
        "block_threads_256": abi["signature_block_threads"] == 256,
        "plan_block_256": abi["plan_block_dims"][0] == 256,
    }
    for key, ok in slot_checks.items():
        if not ok:
            failures.append(f"ABI check failed: {key}")
    if sum(abi["per_kind"].values()) != 80:
        failures.append(f"per_kind sum {sum(abi['per_kind'].values())} != 80 (73 arrays + 7 scalars)")
    if abi["slots"] != 1 + 73 + 7:
        failures.append("81 slot composition mismatch")

    recipe = report["recipe"]
    recipe_checks = {
        "purpose_render": recipe["purpose"] == "Render",
        "gate_f32_render": recipe["gate"] == "F32Render",
        "fmad": recipe["fmad"] is True,
        "fast_math_off": recipe["fast_math"] is False,
        "render_admitted": recipe["render_admitted"] is True,
        "diagnostic_rejected_for_render": recipe["diagnostic_render_rejected"] is True,
        "physics_gate_distinct": recipe["physics_gate_enum"] == "F32Physics",
    }
    for key, ok in recipe_checks.items():
        if not ok:
            failures.append(f"recipe check failed: {key}")

    rejection_results = {}
    expected_errors = {
        "arg_count": "RenderArgCount",
        "arg_kind": "RenderArgKind",
        "bounds_slot": "InvalidSignature",
        "block_rule": "RenderBounds",
        "align_vec3": "RenderArrayAlign",
        "align_texture2d": "RenderArrayAlign",
        "capacity": "Capacity",
        "context_mismatch": "ContextMismatch",
        "shape_empty": "InvalidArray",
        "shape_zero": "InvalidArray",
    }
    for item in report["rejections"]:
        name = item["name"]
        error = item["error"] or ""
        expected = expected_errors.get(name, "")
        ok = bool(item["rejected"]) and (expected in error if expected else True)
        rejection_results[name] = {"rejected": item["rejected"], "error": error, "ok": ok}
        if not ok:
            failures.append(f"rejection {name} not rejected as {expected}: {error}")
    if len(report["rejections"]) != 10:
        failures.append("rejection count != 10")

    # Gate source untouched + ratchet + artifacts (git level).
    ptx_diff = subprocess.run(
        ["git", "-C", str(ROOT), "diff", "--name-only", "a0d9af0^..a0d9af0", "--", "src/runtime/transfer/kernel/ptx.rs"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout.strip()
    if ptx_diff:
        failures.append(f"check_physics_ptx file changed: {ptx_diff}")
    launch_blob = subprocess.check_output(["git", "-C", str(ROOT), "cat-file", "blob", "HEAD:src/render/kernel/launch.rs"])
    launch_lines = launch_blob.count(b"\n")
    if launch_lines != 500:
        failures.append(f"launch.rs has {launch_lines} lines, expected 500 ratchet")

    result = {
        "failures": failures,
        "gate": {
            "allowed_constants_byte_exact": exclusive,
            "real_ptx_f64_globals": scan,
            "variants": variant_results,
            "ptx_bytes": ptx_report["bytes"],
            "ptx_sha256": ptx_report["sha256"],
            "source_sha256": ptx_report["source_sha256"],
            "precision_gate": report["gate"]["precision_gate"],
            "recipe_gate": report["gate"]["recipe_gate"],
        },
        "abi": {
            "slot_checks": slot_checks,
            "element_checks": element_checks,
            "per_kind": abi["per_kind"],
            "sizes": {
                "texture2d": [abi["size_of_texture2d"], abi["align_of_texture2d"]],
                "render_array": abi["size_of_render_array"],
            },
            "signature": {
                "len": abi["signature_len"],
                "name": abi["signature_name"],
                "threads": abi["signature_block_threads"],
            },
        },
        "recipe": recipe_checks,
        "rejections": rejection_results,
        "physics_gate_file_unchanged": not ptx_diff,
        "launch_rs_lines": launch_lines,
    }
    OUT.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(
        json.dumps(
            {
                "failures": failures,
                "gate_verdicts": {name: (item["qa_ok"], item["product_ok"]) for name, item in variant_results.items()},
                "real_f64_globals": scan["global_count"],
                "ptx_sha": ptx_report["sha256"],
                "slots": abi["slots"],
                "launch_lines": launch_lines,
            },
            indent=2,
        )
    )
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
