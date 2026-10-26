"""Fill probe_template.rs with verbatim function bodies + real mjModel arrays.

Run with the pinned venv python (any python3 works; no third-party imports here).
Output: target/qa-m0afix/probe.rs
"""
import json
from pathlib import Path

QA = Path(r"D:\Rust\mjwarp-rs\target\qa-m0afix")


def extract_fn(text, name, new_name=None):
    lines = text.split("\n")
    start = None
    for index, line in enumerate(lines):
        if line.startswith("fn " + name + "("):
            start = index
            break
    if start is None:
        raise SystemExit("fn " + name + " not found")
    end = None
    for cursor in range(start + 1, len(lines)):
        if lines[cursor] == "}":
            end = cursor
            break
    if end is None:
        raise SystemExit("fn " + name + " end not found")
    body = lines[start : end + 1]
    if new_name:
        body[0] = body[0].replace("fn " + name + "(", "fn " + new_name + "(", 1)
    return "\n".join(body), [start + 1, end + 1]


old_text = (QA / "abi-old.rs").read_text(encoding="utf-8")
new_text = (QA / "abi-new.rs").read_text(encoding="utf-8")
old_fn, old_range = extract_fn(old_text, "validate_mesh_poly", "validate_mesh_poly_old")
new_fn, new_range = extract_fn(new_text, "validate_mesh_poly")

model = json.loads((QA / "cube-poly-model.json").read_text(encoding="utf-8"))
arr = model["fixtures"]["mesh-box"]["arrays"]

decls = []
for name in [
    "polynum",
    "polyadr",
    "polynormal",
    "polyvertadr",
    "polyvertnum",
    "polyvert",
    "polymapadr",
    "polymapnum",
    "polymap",
]:
    values = arr[name]
    if name == "polynormal":
        rendered = ", ".join(repr(float(value)) for value in values)
        decls.append("    let polynormal: Vec<f32> = vec![" + rendered + "];")
    else:
        rendered = ", ".join(str(int(value)) for value in values)
        decls.append("    let " + name + ": Vec<i32> = vec![" + rendered + "];")

text = (QA / "probe_template.rs").read_text(encoding="utf-8")
text = text.replace("// __OLD_FN__", old_fn)
text = text.replace("// __NEW_FN__", new_fn)
text = text.replace("    // __ARRAY_DECLS__", "\n".join(decls))
text = text.replace("__OLD_RANGE_JSON__", str(old_range))
text = text.replace("__NEW_RANGE_JSON__", str(new_range))

(QA / "probe.rs").write_text(text, encoding="utf-8")
print("old fn lines", old_range, "new fn lines", new_range)
print("wrote", QA / "probe.rs", len(text), "bytes")
