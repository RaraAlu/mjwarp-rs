"""QA independent table for S2: per-case 32-array bin sizes/sha256 + case counts."""
import hashlib
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
OUT = ROOT / "target/qa-s2/case-bins-table.txt"

CASES = [
    ("mesh-box", "docs/development-evidence/mesh-multicontact-m0b/cases/mesh-box"),
    ("mesh-cylinder", "docs/development-evidence/mesh-multicontact-m0b/cases/mesh-cylinder"),
    ("mesh-mesh", "docs/development-evidence/mesh-multicontact-m3/cases/mesh-mesh"),
]

FIELDS = [
    "geom_xpos", "geom_xmat", "geom_size", "geom_margin", "geom_gap", "geom_rbound",
    "geom_aabb", "geom_type", "geom_dataid", "mesh_vertadr", "mesh_vertnum", "mesh_graphadr",
    "mesh_octadr", "mesh_vert", "mesh_graph", "mesh_polynum", "mesh_polyadr", "mesh_polynormal",
    "mesh_polyvertadr", "mesh_polyvertnum", "mesh_polyvert", "mesh_polymapadr", "mesh_polymapnum",
    "mesh_polymap", "hfield_size", "hfield_nrow", "hfield_ncol", "hfield_adr", "hfield_data",
    "oct_aabb", "oct_child", "oct_coeff",
]


def words(path: Path) -> int:
    return path.stat().st_size // 4


def main():
    lines = [
        "# S2 case bins, read directly by QA (32 arrays per case, LE bytes)",
        "# sha256 is over the raw .bin bytes; count is scalars (4-byte host width)",
        "#",
    ]
    for name, case_dir in CASES:
        root = ROOT / case_dir
        lines.append(f"## {name}  ({case_dir})")
        files = sorted(p.name for p in root.iterdir() if p.is_file())
        lines.append(f"# files={len(files)} (bin={sum(1 for f in files if f.endswith('.bin'))}, meta={('meta.txt' in files)})")
        for field in FIELDS:
            raw = (root / f"{field}.bin").read_bytes()
            lines.append(
                f"{field} count={len(raw) // 4} bytes={len(raw)} sha256={hashlib.sha256(raw).hexdigest()}"
            )
        meta = (root / "meta.txt").read_text(encoding="utf-8")
        keep = [line for line in meta.splitlines() if line.split()[:1] and line.split()[0] in
                {"nworld", "ngeom", "naconmax", "naccdmax", "efc_width", "g1", "g2"}]
        lines.append("# meta: " + "; ".join(keep))
        counts = {
            "ngeom": words(root / "geom_type.bin"),
            "nmesh": words(root / "mesh_vertadr.bin"),
            "nmeshvert": words(root / "mesh_vert.bin") // 3,
            "nmeshgraph": words(root / "mesh_graph.bin"),
            "nmeshpoly": words(root / "mesh_polynormal.bin") // 3,
            "nmeshpolyvert": words(root / "mesh_polyvert.bin"),
            "nmeshpolymap": words(root / "mesh_polymap.bin"),
            "nhfield": words(root / "hfield_nrow.bin"),
            "nhfielddata": words(root / "hfield_data.bin"),
            "noct": words(root / "oct_aabb.bin") // 6,
        }
        lines.append("# derived counts: " + ", ".join(f"{k}={v}" for k, v in counts.items()))
        lines.append("")
    OUT.write_bytes(("\n".join(lines) + "\n").encode("utf-8"))
    print(f"wrote {OUT} lines={len(lines)}")


if __name__ == "__main__":
    main()
