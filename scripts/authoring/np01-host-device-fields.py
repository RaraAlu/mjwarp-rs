#!/usr/bin/env python3
"""NP01 B2：host/device 字段级位差分账（离线、只读归档、零 GPU）。

把已归档的三类差异分开算、分开报，避免「把 recipe 差写成类型偏离」：
  1. 类型偏离（NP01 目标）：不据数值相等反推，依据 = 逐字段表（30 产品 + 3 host）+ CUDA token 门禁 +
     host source 门禁；本脚本只登记「0 项」及其依据引用。
  2. recipe 差（同码不同后端）：`g10-p5-g2-gpu-reference-capture/compare-cpu-vs-gpu.json`
     ——CPU 参考 recipe vs GPU 参考 recipe，逐字段 bit_equal 比例与 max_abs/max_ulp。
  3. 同 recipe 实现残差：`g10-p5-gpu-kernels/gpu-window/result.json`
     ——设备（产品探针）vs 上游 CUDA（两侧都是 fma.rn recipe、都全 f32），残差只能归实现/算法顺序差。
  4. 引擎语义差：native C vs warp（A0 已单独出证据），本片只引用、不重复判定。
  5. 已知偏差：witness cylinder-box-sweep（引用 verified 卡），归类为同 recipe 实现残差，不冒充类型偏离。

输出：docs/development-evidence/np01-host-device-fields/host_device_fields.json
"""

import json
import os

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
EV = os.path.join(ROOT, "docs", "development-evidence")
RECIPE = os.path.join(EV, "g10-p5-g2-gpu-reference-capture", "compare-cpu-vs-gpu.json")
RESIDUAL = os.path.join(EV, "g10-p5-gpu-kernels", "gpu-window", "result.json")
QA_COMPARE = os.path.join(EV, "g10-p5-gpu-kernels-independent-check", "qa-compare-gpu.json")
B1 = os.path.join(EV, "np01-host-device-expr", "host_device_expr.json")
A0 = os.path.join(EV, "np01-e-recompute", "e_recompute.json")
OUT = os.path.join(EV, "np01-host-device-fields")

FIELDS = ["dist", "pos", "frame", "includemargin", "friction", "solref", "solreffriction", "solimp", "adhesion"]


def load(path):
    with open(path, encoding="utf-8") as handle:
        return json.load(handle)


def summarize(field_entry):
    if not isinstance(field_entry, dict) or "components" not in field_entry:
        return None
    return {
        "components": field_entry.get("components"),
        "bit_equal": field_entry.get("bit_equal"),
        "bit_equal_ratio": (field_entry.get("bit_equal") / field_entry["components"])
        if field_entry.get("components")
        else None,
        "zero_vs_nonzero": field_entry.get("zero_vs_nonzero"),
        "max_abs": field_entry.get("max_abs"),
        "max_ulp_nonzero": field_entry.get("max_ulp_nonzero", field_entry.get("max_ulp")),
    }


def main():
    recipe = load(RECIPE)
    residual = load(RESIDUAL)
    qa = load(QA_COMPARE)
    b1 = load(B1)
    a0 = load(A0)

    # ---- 2. recipe 差：CPU 参考 vs GPU 参考（同码不同后端） ----
    recipe_cases = {}
    for case, entry in recipe["cases"].items():
        fields = {}
        for name in FIELDS:
            summary = summarize(entry.get("float_fields", {}).get(name))
            if summary:
                fields[name] = summary
        for extra in ("geom_xpos", "geom_xmat"):
            key = f"{extra}_float"
            summary = summarize(entry.get(key))
            if summary and summary.get("max_ulp") is not None:
                fields.pop(f"{extra}", None)
                fields[extra] = summary
        recipe_cases[case] = {
            "counts_equal": entry.get("count_old") == entry.get("count_new"),
            "two_captures_bitwise_identical": entry.get("two_captures_bitwise_identical"),
            "integer_fields_bit_equal": entry.get("integer_fields_bit_equal"),
            "fields": fields,
        }

    # ---- 3. 同 recipe 实现残差：设备 vs 上游 CUDA ----
    residual_cases = {}
    for case in residual["cases"]:
        residual_cases[case["case"]] = {
            "dist": {
                "components": case.get("dist_components"),
                "bit_equal": case.get("dist_bit_equal"),
                "bit_equal_ratio": (case.get("dist_bit_equal") or 0) / case["dist_components"]
                if case.get("dist_components")
                else None,
                "max_ulp": case.get("dist_max_ulp"),
            },
            "pos": {
                "components": case.get("pos_components"),
                "bit_equal": case.get("pos_bit_equal"),
                "bit_equal_ratio": (case.get("pos_bit_equal") or 0) / case["pos_components"]
                if case.get("pos_components")
                else None,
                "max_abs": case.get("pos_max_abs"),
                "max_ulp_at_scale": case.get("pos_max_ulps_at_scale"),
            },
            "frame": {
                "components": case.get("frame_components"),
                "bit_equal": case.get("frame_bit_equal"),
                "bit_equal_ratio": (case.get("frame_bit_equal") or 0) / case["frame_components"]
                if case.get("frame_components")
                else None,
                "max_abs": case.get("frame_max_abs"),
            },
            "params_bit_equal": case.get("params_bit_equal"),
            "ints_bit_equal": case.get("ints_bit_equal"),
        }

    # ---- 5. 已知偏差：witness 扫描（QA 归档） ----
    known = []
    for failure in qa.get("failures", []):
        if isinstance(failure, str) and "witness" in failure:
            known.append(
                {
                    "record": failure,
                    "source": "g10-p5-gpu-kernels-independent-check/qa-compare-gpu.json",
                    "registry_card": "2026-10-25-g10-p5-deep-epa-witness-sampling",
                    "classification": "同 recipe 实现残差（x1/x2 witness 分量超 4·ULP@scale）；不是类型偏离、不是 recipe 差",
                }
            )

    report = {
        "schema": 1,
        "task_id": "2026-11-04-precision-np01-host-device-fields",
        "kind": "np01-field-level-difference-ledger",
        "inputs": {
            "recipe_difference": "docs/development-evidence/g10-p5-g2-gpu-reference-capture/compare-cpu-vs-gpu.json",
            "same_recipe_residual": "docs/development-evidence/g10-p5-gpu-kernels/gpu-window/result.json",
            "qa_compare": "docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-compare-gpu.json",
            "b1_mechanism": "docs/development-evidence/np01-host-device-expr/host_device_expr.json",
            "a0_native_semantics": "docs/development-evidence/np01-e-recompute/e_recompute.json",
            "field_tables": "docs/development-evidence/np01-type-mapping/{field-tables,host-tables}/",
        },
        # ---- 1. 类型偏离：0 项，但不从数值相等反推 ----
        "type_deviation": {
            "count": 0,
            "basis": [
                "30 条产品路径 + 3 条 host 站点的逐字段类型表（NP01 B1–B3 批）",
                "CUDA 精度 token 门禁：产品 .cu 代码区不含 double/half/bfloat16/tf32（docs/development-evidence/np01-type-mapping/cuda-surface.json）",
                "host source token 门禁：src/**/*.rs 代码区精度 token 白名单活性（host-source-gate 卡）",
            ],
            "explicitly_not_inferred_from": "数值位级相等/ULP 计数——数值差可以来自 recipe 或引擎语义，不能反推类型",
        },
        # ---- 2. recipe 差 ----
        "recipe_difference": {
            "definition": "同码不同后端：warp CPU 参考 recipe（每乘加两次舍入）vs warp CUDA 参考 recipe（NVVM 默认 fma.rn 收缩）",
            "mechanism": {
                "source": "B1 逐表达式 PTX 定序表",
                "summary": "12 条关键表达式中 8 条带乘加者在 --fmad=true 下全部出 fma.rn 且 mul.rn/add.rn 归零；--fmad=false 下 fma.rn 归零、指令数与 host 两次舍入逐项对应；face_norm2 的 head→tail→norm2 三步写法在 device 默认下仍融合为 2 条 fma.rn",
                "expression_count": b1.get("expression_count"),
            },
            "cases": recipe_cases,
        },
        # ---- 3. 同 recipe 实现残差 ----
        "same_recipe_residual": {
            "definition": "设备（产品探针）vs 上游 CUDA 接触数组：两侧同为 fma.rn recipe、同为 f32，残差不能归因 recipe/类型",
            "criterion": residual.get("criterion"),
            "device_probe": residual.get("device_probe"),
            "failed_cases": residual.get("failed_cases"),
            "unmet": residual.get("unmet"),
            "cases": residual_cases,
        },
        # ---- 4. 引擎语义差（引用，不在本片判定） ----
        "engine_semantic": {
            "definition": "native MuJoCo C vs warp 实现：不同引擎的接触选择/裁剪语义差",
            "reference": "A0 e_recompute.json（native 与 warp 接触数不等 / 可配对 case 的 pos 差 4.5e-4~7.3e-2）",
            "status": "A0 已单独出证据；本片只引用，不做新判定、不参与类型结论",
            "native_unpairable_cases": list(a0.get("candidates", {}).get("native_f64", {}).get("excluded", {}).keys()),
        },
        # ---- 5. 已知偏差 ----
        "known_deviations": known,
        "verdict": {
            "type_deviation_is_zero": True,
            "does_not_mean_bitwise_equal_across_recipes": True,
            "statement": "类型偏离 0 项 与 「host/device 位级一致」是两件事：前者靠类型表与 token 门禁，后者需要同 recipe + 同编译形态前提（B1/A0 已给机制）；本片不宣称任何位级等价或 NP01/NP03 通过。",
        },
    }
    os.makedirs(OUT, exist_ok=True)
    path = os.path.join(OUT, "host_device_fields.json")
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(report, handle, ensure_ascii=False, indent=2)
        handle.write("\n")
    print("wrote", path)
    print("recipe cases:", list(recipe_cases))
    print("residual cases:", list(residual_cases))
    print("known deviations:", len(known))


if __name__ == "__main__":
    main()
