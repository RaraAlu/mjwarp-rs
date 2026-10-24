"""Approved S0a judging (user ruling 2026-10-24, option A).

The strict element-wise comparison stays recorded as the initial conclusion
(`strict_elementwise_parity = fail`). The approved re-judging keeps the plan's
own model-float rule and the render-relevant scope:

- scope: render-relevant model arrays (plan S0a acceptance item 3);
- int/byte arrays: exact equality;
- float32 arrays: f32 bit equality;
- float64 arrays: judged by the f32 bit view first (task acceptance item 3
  "float f32_bits first"); the double-layer numbers stay in parity.json;
- `geom_sameframe` is excluded as `render_relevant=false`: it is a collision
  frame-reuse flag, the upstream MJWarp renderer `_src` tree has no sameframe
  reference (grep-verified by Chief 2026-10-24), and MuJoCo 3.15 changed the
  enum semantics BODY(1) -> BODYROT(3).

No tolerance is changed and no double-layer difference number is removed.
Explicitly not claimed: the two model versions are bit-identical, or MuJoCo
3.12/3.15 are equivalent; only render-relevant arrays are equivalent at the
f32 bit level.
"""

from __future__ import annotations

APPROVED_JUDGING = {
    "decision": "user 2026-10-24 ruling, option A: close S0a under the already "
    "approved tolerance hierarchy",
    "applies_to": "render-relevant model arrays (plan S0a acceptance item 3)",
    "criterion": "int/byte exact; float32 f32_bits exact; float64 f32_bits first "
    "(f32_bit_differences == 0); excluded non-render field: geom_sameframe",
    "tolerance_changed": False,
    "double_layer_numbers_removed": False,
    "not_claimed": [
        "both model versions are bit-identical",
        "MuJoCo 3.12 and 3.15 are equivalent models",
        "strict element-wise parity passed",
    ],
    "record": "strict_elementwise_parity=fail is preserved as the initial strict "
    "conclusion in version_difference_classification",
}

EXCLUDED_FIELDS = {
    "geom_sameframe": {
        "render_relevant": False,
        "reason": "collision frame-reuse flag, not read by the scene renderer "
        "(upstream MJWarp renderer _src tree has no sameframe reference, "
        "grep-verified by Chief 2026-10-24); MuJoCo 3.15 also changed the enum "
        "semantics for geoms whose frame differs from the body frame only in "
        "position (3.12 BODY(1) vs 3.15 BODYROT(3))",
        "values_312": [1, 0, 1, 0, 0, 1, 1],
        "values_315": [1, 0, 3, 0, 0, 3, 3],
    }
}


def classify_differences(parity_fields, mismatched):
    """Split mismatches into float64 tail (f32 identical) and other differences."""
    version_differences = []
    for name in mismatched:
        comparison = next(field for field in parity_fields if field["field"] == name)
        category = "value_difference"
        detail = {}
        if comparison.get("kind") == "float64" and comparison.get("f32_bits_equal"):
            category = "float64_tail_f32_identical"
            detail = {
                "max_abs": comparison["max_absolute"],
                "max_ulp_f64": comparison["max_ulp"],
                "f32_bit_differences": comparison.get("f32_bit_differences", 0),
            }
        elif comparison.get("kind") in ("int32", "int64"):
            category = "integer_value_difference"
            detail = {"mismatches": comparison.get("mismatches", [])[:8]}
        exclusion = EXCLUDED_FIELDS.get(name, {})
        version_differences.append(
            {
                "field": name,
                "kind": comparison["kind"],
                "category": category,
                "render_relevant": exclusion.get("render_relevant", True),
                "note": exclusion.get("reason", ""),
                "detail": detail,
            }
        )
    f32_tail_only = [
        item["field"]
        for item in version_differences
        if item["category"] == "float64_tail_f32_identical"
    ]
    other_differences = [
        item["field"]
        for item in version_differences
        if item["category"] != "float64_tail_f32_identical"
    ]
    return version_differences, f32_tail_only, other_differences


def judged_equal(field):
    """Approved criterion per field; returns (equal, basis) or (None, excluded)."""
    if field["field"] in EXCLUDED_FIELDS:
        return None, "excluded_non_render"
    if field.get("equal"):
        return True, "strict_equal"
    if field.get("kind") == "float64" and field.get("f32_bits_equal"):
        return True, "f32_bits"
    return False, "difference"


def judge_fields(parity_fields, counts_match, versions_ok, deterministic):
    judged_failures = []
    basis = {}
    for field in parity_fields:
        equal, reason = judged_equal(field)
        basis[field["field"]] = reason
        if equal is False:
            judged_failures.append(field["field"])
    passed = not judged_failures and counts_match and versions_ok and deterministic
    return {
        "passed": passed,
        "judged_failures": judged_failures,
        "basis": basis,
        "criterion": APPROVED_JUDGING,
    }
