"""Offline judgment: U038-3D under the native-first two-reference rule.

OFFLINE ONLY. Reads the generated native reference (first reference), the frozen
secondary MJWarp f32 evidence and the product f32 evidence; never starts Python
from the product side and never recomputes native values from GPU outputs.

Rule (docs/numerical-precision-contract.md, native_error_v2):
    t_i = atol_native + rtol_native * abs(native_i)
    E_candidate = max_i abs(candidate_i - native_i) / t_i
    E_mjwarp    = max_i abs(mjwarp_i    - native_i) / t_i
    redline     = 1.05 * E_mjwarp
Native pass admits the block (no secondary reference needed). Only when native
fails does the secondary reference decide: E_candidate <= E_mjwarp (comparison
pass) or <= redline (within-redline pass), else failed.

Native blocks and frozen tolerances:
- kinematics fields: G01 main chain / low-level, 2e-5 + 2e-5*|native|
- qfrc_gravcomp: G01 gravity certificate block U, 2e-4 absolute/relative
"""

from __future__ import annotations

import hashlib
import json
import pathlib
import subprocess

import numpy as np

ROOT = pathlib.Path(__file__).resolve().parents[2]
EVIDENCE = ROOT / "docs" / "development-evidence" / "u038-3d" / "native-two-reference"
DEBT2 = ROOT / "docs" / "development-evidence" / "u038-3d" / "debt-2"
FIXTURES = ROOT / "fixtures" / "gravity-upstream-abi"
TASK_ID = "2026-10-24-u038-native-two-reference"
RELATIVE_SLACK = 0.05
FIELD_ORDER = [
    "xpos",
    "xquat_wxyz",
    "xipos",
    "xmat",
    "xanchor",
    "xaxis",
    "subtree_com",
    "cdof",
]
EXPECTED_HASHES = {
    "debt-2/product-fields-raw.json": "5d8bfbf2f2820ba02111be90f4e4092613cdba000906214d8de92b7085da964b",
    "debt-2/product-gravity-raw.json": "cfba6a2858cc9ae8971a95dfd7cbe83ff97474a433a5e9a50320dffe2ecfba85",
    "debt-2/upstream-gravity-raw.json": "c93fee14e0526d572824f3ed8c45bf1a954b7cd3247b4af3329ced68665581c8",
    "fixtures/gravity-upstream-abi/three-d-kinematics/raw-bits.json": (
        "1c03af97bd0607095d662edf3c445ac4e96edcfda8705324b15a638bd7836187"
    ),
    "debt-2/run.json": "620d9d9f7d664991ce6ac04515a6d5d4aad73776cf87f1951eac458c05c547c8",
}


def digest(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path: pathlib.Path):
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: pathlib.Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes((json.dumps(value, indent=2) + "\n").encode())


def from_hex_bits(hex_bits) -> np.ndarray:
    return np.array(
        [
            np.frombuffer(int(bits, 16).to_bytes(4, "little"), dtype=np.float32)[0]
            for bits in hex_bits
        ],
        dtype=np.float32,
    )


def score(actual: np.ndarray, native: np.ndarray, atol: float, rtol: float) -> dict:
    errors = np.abs(actual.astype(np.float64) - native)
    limits = atol + rtol * np.abs(native)
    normalized = np.divide(
        errors,
        limits,
        out=np.zeros_like(errors),
        where=limits != 0.0,
    )
    violations = int(np.count_nonzero(errors > limits))
    worst = int(np.argmax(normalized))
    return {
        "count": int(actual.size),
        "violations": violations,
        "max_normalized": float(normalized[worst]),
        "worst_index": worst,
        "worst": {
            "index": worst,
            "native": float(native[worst]),
            "value": float(actual[worst]),
            "error": float(errors[worst]),
            "limit": float(limits[worst]),
            "normalized": float(normalized[worst]),
        },
        "max_absolute": float(errors.max()),
    }


def status_name(block: dict) -> str:
    if block["native"]["violations"] == 0:
        return "native_passed"
    if block["secondary_used"]:
        return "mjwarp_passed" if block["admitted"] else "failed"
    return "pending"


def load_inputs() -> tuple[dict, dict, dict, dict, dict]:
    native = load(EVIDENCE / "native-reference.json")
    product_fields = load(DEBT2 / "product-fields-raw.json")
    product_gravity = load(DEBT2 / "product-gravity-raw.json")
    frozen_fields = load(FIXTURES / "three-d-kinematics" / "raw-bits.json")
    frozen_gravity = load(DEBT2 / "upstream-gravity-raw.json")
    for relative, expected in EXPECTED_HASHES.items():
        if relative.startswith("debt-2/"):
            path = ROOT / "docs" / "development-evidence" / "u038-3d" / relative
        else:
            path = ROOT / relative
        if digest(path) != expected:
            raise RuntimeError(f"judgment input drift: {relative}")
    return native, product_fields, product_gravity, frozen_fields, frozen_gravity


def field_values(payload: dict, field: str) -> np.ndarray:
    for entry in payload["fields"]:
        if entry["field"] == field:
            return from_hex_bits(entry["bits"])
    raise RuntimeError(f"missing field: {field}")


def main() -> int:
    native, product_fields, product_gravity, frozen_fields, frozen_gravity = load_inputs()
    if native["task_id"] != TASK_ID:
        raise RuntimeError("native reference task id mismatch")
    fields = native["fields"]
    excluded = set(fields["xaxis"]["native_equivalent"]["excluded_flat_indices"])
    mask = np.array(
        [index not in excluded for index in range(fields["xaxis"]["count"])], dtype=bool
    )

    blocks = []
    for name in FIELD_ORDER:
        atol = rtol = 2e-5
        source = "contract: G01 main chain/low-level 2e-5 + 2e-5*abs(native)"
        if name == "xaxis":
            source += "; xaxis coverage limited to slide+hinge (free/ball have no native single axis)"
        native_values = np.asarray(fields[name]["f64"], dtype=np.float64)
        candidate = field_values(product_fields, name).astype(np.float64)
        secondary = field_values(frozen_fields, name).astype(np.float64)
        selected = np.ones(native_values.size, dtype=bool)
        if name == "xaxis":
            selected = mask
        native_values, candidate, secondary = (
            native_values[selected],
            candidate[selected],
            secondary[selected],
        )
        native_primary = score(candidate, native_values, atol, rtol)
        secondary_score = score(secondary, native_values, atol, rtol)
        native_f32 = np.asarray(fields[name]["f32"], dtype=np.float64)[selected]
        f32_variant = {
            "candidate": score(candidate, native_f32, atol, rtol),
            "mjwarp": score(secondary, native_f32, atol, rtol),
        }
        redline = secondary_score["max_normalized"] * (1.0 + RELATIVE_SLACK)
        block = {
            "block": name,
            "coverage": {
                "values": int(native_values.size),
                "excluded_values": int(fields[name]["count"]) - int(native_values.size),
                "excluded_reason": fields[name].get("native_equivalent", {}).get("note", ""),
            },
            "tolerance": {
                "absolute": atol,
                "relative": rtol,
                "source": source,
                "native_reference": "f64 raw value (primary); f32 round-to-nearest variant is diagnostic only",
            },
            "native": native_primary,
            "mjwarp": secondary_score,
            "redline": redline,
            "secondary_used": False,
            "admitted": False,
            "status": "",
            "admission_label": "",
            "reason": "",
            "f32_quantized_variant": f32_variant,
        }
        if native_primary["violations"] == 0:
            block["status"] = "native_passed"
            block["admission_label"] = "原生放行"
            block["reason"] = "native_within_tolerance"
            block["admitted"] = True
        else:
            block["secondary_used"] = True
            if native_primary["max_normalized"] <= secondary_score["max_normalized"]:
                block["status"] = "mjwarp_passed"
                block["admission_label"] = "对照放行"
                block["reason"] = "native_error_not_worse_than_mjwarp"
                block["admitted"] = True
            elif native_primary["max_normalized"] <= redline:
                block["status"] = "mjwarp_passed"
                block["admission_label"] = "红线内放行"
                block["reason"] = "native_error_within_mjwarp_redline"
                block["admitted"] = True
            else:
                block["status"] = "failed"
                block["admission_label"] = "失败不放行"
                block["reason"] = "native_error_exceeds_mjwarp_redline"
        blocks.append(block)

    # qfrc_gravcomp uses the frozen gravity-certificate U tolerance.
    native_gravity = np.asarray(native["gravity"]["f64"], dtype=np.float64)
    candidate_gravity = from_hex_bits(product_gravity["field_bits"][0]).astype(np.float64)
    secondary_gravity = from_hex_bits(frozen_gravity["field_bits"][0]).astype(np.float64)
    if not (
        native_gravity.size == candidate_gravity.size == secondary_gravity.size
    ):
        raise RuntimeError("gravity vector length mismatch")
    atol = rtol = 2e-4
    native_primary = score(candidate_gravity, native_gravity, atol, rtol)
    secondary_score = score(secondary_gravity, native_gravity, atol, rtol)
    native_gravity_f32 = np.asarray(native["gravity"]["f32"], dtype=np.float64)
    redline = secondary_score["max_normalized"] * (1.0 + RELATIVE_SLACK)
    gravity_block = {
        "block": "qfrc_gravcomp",
        "coverage": {"values": int(native_gravity.size), "excluded_values": 0, "excluded_reason": ""},
        "tolerance": {
            "absolute": atol,
            "relative": rtol,
            "source": "contract: G01 gravity certificate block U, 2e-4 absolute/relative",
            "native_reference": "f64 raw value (primary); f32 round-to-nearest variant is diagnostic only",
        },
        "native": native_primary,
        "mjwarp": secondary_score,
        "redline": redline,
        "secondary_used": False,
        "admitted": False,
        "status": "",
        "admission_label": "",
        "reason": "",
        "f32_quantized_variant": {
            "candidate": score(candidate_gravity, native_gravity_f32, atol, rtol),
            "mjwarp": score(secondary_gravity, native_gravity_f32, atol, rtol),
        },
    }
    if native_primary["violations"] == 0:
        gravity_block.update(
            status="native_passed",
            admission_label="原生放行",
            reason="native_within_tolerance",
            admitted=True,
        )
    else:
        gravity_block["secondary_used"] = True
        if native_primary["max_normalized"] <= secondary_score["max_normalized"]:
            gravity_block.update(
                status="mjwarp_passed",
                admission_label="对照放行",
                reason="native_error_not_worse_than_mjwarp",
                admitted=True,
            )
        elif native_primary["max_normalized"] <= redline:
            gravity_block.update(
                status="mjwarp_passed",
                admission_label="红线内放行",
                reason="native_error_within_mjwarp_redline",
                admitted=True,
            )
        else:
            gravity_block.update(
                status="failed",
                admission_label="失败不放行",
                reason="native_error_exceeds_mjwarp_redline",
            )
    blocks.append(gravity_block)

    counts = {"native_passed": 0, "mjwarp_passed": 0, "failed": 0, "pending": 0}
    for block in blocks:
        counts[block["status"]] += 1
    compared_scalars = sum(block["native"]["count"] for block in blocks)
    admitted = (
        compared_scalars > 0 and counts["failed"] == 0 and counts["pending"] == 0
    )
    secondary_used = any(block["secondary_used"] for block in blocks)
    if not admitted:
        admission_label = "失败不放行" if counts["failed"] else "待验收不放行"
    elif secondary_used:
        admission_label = "放行，含二级参考"
    else:
        admission_label = "原生放行"

    strict = load(DEBT2 / "run.json")
    strict_record = {
        "policy": "historical strict upstream bitwise tolerance 2e-6",
        "total_values": strict["total_compared_values"],
        "total_bit_differences": strict["total_bit_differences"],
        "total_outside_tolerance": strict["total_outside_tolerance"],
        "max_absolute": strict["max_absolute"],
        "absolute_tolerance": strict["absolute_tolerance"],
        "note": "unchanged and recorded for context; it is not the native_error_v2 admission basis "
        "and this verdict does not claim upstream bitwise alignment",
    }

    verdict = {
        "schema": 2,
        "task_id": TASK_ID,
        "kind": "u038-native-two-reference-verdict",
        "policy": "native_error_v2",
        "gate": "NP03",
        "scope": "u038-3d-free-ball-hinge-slide-7worlds",
        "judged_at_head": subprocess.check_output(
            ["git", "-C", str(ROOT), "rev-parse", "HEAD"], text=True
        ).strip(),
        "product_evidence_head": "3b22f5f8f56cfd20f2c19264e922e4f640d16897",
        "conditions": "same frozen model/qpos/gravity switch as the U038 3D package "
        "(upstream 71da24d, Warp 1.15.0); native MuJoCo 3.15.0 f64 CPU first reference; "
        "frozen secondary MJWarp f32 evidence; product f32 chain evidence; no per-element best-of picking",
        "rules": {
            "formula": "t_i = atol + rtol*abs(native_i); E = max_i abs(value_i - native_i)/t_i; redline = 1.05*E_mjwarp",
            "relative_slack": RELATIVE_SLACK,
            "native_tolerance_source": "docs/numerical-precision-contract.md lines 142-199 and 326-339, table lines 373-374",
            "admission_classes": {
                "native_passed": "原生放行",
                "mjwarp_passed": "对照放行 / 红线内放行",
                "failed": "失败不放行",
                "pending": "待验收不放行",
            },
        },
        "inputs": {
            "native_reference": {
                "path": "docs/development-evidence/u038-3d/native-two-reference/native-reference.json",
                "sha256": digest(EVIDENCE / "native-reference.json"),
            },
            "candidate": {
                "path": "docs/development-evidence/u038-3d/debt-2/product-fields-raw.json",
                "sha256": digest(DEBT2 / "product-fields-raw.json"),
                "gravity_path": "docs/development-evidence/u038-3d/debt-2/product-gravity-raw.json",
                "gravity_sha256": digest(DEBT2 / "product-gravity-raw.json"),
            },
            "secondary": {
                "path": "fixtures/gravity-upstream-abi/three-d-kinematics/raw-bits.json",
                "sha256": digest(FIXTURES / "three-d-kinematics" / "raw-bits.json"),
                "gravity_path": "docs/development-evidence/u038-3d/debt-2/upstream-gravity-raw.json",
                "gravity_sha256": digest(DEBT2 / "upstream-gravity-raw.json"),
            },
            "native_generation": "docs/development-evidence/u038-3d/native-two-reference/authoring.json",
        },
        "coverage": {
            "blocks": len(blocks),
            "compared_scalars": compared_scalars,
            "excluded_scalars": sum(
                block["coverage"]["excluded_values"] for block in blocks
            ),
            "excluded_reason": "xaxis entries of free/ball joints: MuJoCo has no single Cartesian "
            "axis for those joints and ignores jnt_axis for ball, so they are not the same field "
            "(contract: missing equivalent fields must not be force-compared)",
        },
        "blocks": blocks,
        "summary": {
            "evaluated_blocks": len(blocks),
            "values": compared_scalars,
            **counts,
            "secondary_reference_used": secondary_used,
            "np03_admitted": admitted,
            "admission_label": admission_label,
            "admission_scope": "recorded field blocks only (U038 3D chain); not full U038, "
            "not full product, not upstream bitwise alignment",
        },
        "upstream_strict_tolerance_record": strict_record,
        "not_claimed": [
            "整体 U038 三维链验收通过",
            "上游逐位对齐（bitwise）",
            "完整三维状态空间或完整产品精度验收",
        ],
    }
    write_json(EVIDENCE / "verdict.json", verdict)
    print(
        json.dumps(
            {
                "verdict": admission_label,
                "np03_admitted": admitted,
                "counts": counts,
                "blocks": {
                    block["block"]: {
                        "status": block["status"],
                        "E_candidate": round(block["native"]["max_normalized"], 6),
                        "E_mjwarp": round(block["mjwarp"]["max_normalized"], 6),
                        "redline": round(block["redline"], 6),
                    }
                    for block in blocks
                },
            },
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
