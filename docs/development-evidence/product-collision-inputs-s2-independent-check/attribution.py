"""QA evidence for the S2 third-case fixture attribution mismatch."""
import hashlib
import json
from pathlib import Path

ROOT = Path(r"D:\Rust\mjwarp-rs")
OUT = ROOT / "target/qa-s2/fixture-attribution.txt"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def words(path: Path) -> int:
    return path.stat().st_size // 4


def main():
    manifest = json.loads(
        (ROOT / "docs/development-evidence/mesh-multicontact-m3/mesh-case-manifest.json").read_text(encoding="utf-8")
    )
    case = manifest["cases"]["mesh-mesh"]
    case_dir = ROOT / "docs/development-evidence/mesh-multicontact-m3/cases/mesh-mesh"
    mesh_mesh = ROOT / "fixtures/g10-p5-mesh-multicontact/mesh-mesh.xml"
    two_assets = ROOT / "fixtures/g10-p5-mesh-multicontact/mesh-mesh-two-assets.xml"
    lines = [
        "# S2 third case (mesh-mesh) fixture attribution check",
        "",
        "## what the M3 case manifest says",
        f"M3 manifest case xml   = {case['xml']}",
        f"M3 manifest xml_sha256  = {case['xml_sha256']}",
        f"M3 manifest qpos        = {case['qpos']}",
        f"mesh-mesh.xml actual sha256 = {sha(mesh_mesh)}  match={sha(mesh_mesh) == case['xml_sha256']}",
        "",
        "## the two candidate fixtures (both exist, different content)",
        f"mesh-mesh.xml            sha256={sha(mesh_mesh)} bytes={mesh_mesh.stat().st_size} "
        "(asset: cube + cube_b scaled 0.9; qpos z=0.18)",
        f"mesh-mesh-two-assets.xml sha256={sha(two_assets)} bytes={two_assets.stat().st_size} "
        "(asset: cube + tetra; qpos z=0.09; used by the S3/S4 U6 branch case)",
        "",
        "## the M3 case bins themselves",
        f"ngeom={words(case_dir / 'geom_type.bin')} nmesh={words(case_dir / 'mesh_vertadr.bin')} "
        f"nmeshvert={words(case_dir / 'mesh_vert.bin') // 3} nmeshpoly={words(case_dir / 'mesh_polynormal.bin') // 3}",
        "mesh_vertnum = " + str(list((case_dir / "mesh_vertnum.bin").read_bytes()[:8])),
        "mesh_polynum = " + str(list((case_dir / "mesh_polynum.bin").read_bytes()[:8])),
        "=> 2 meshes x (8 verts, 6 polys) = cube + cube_b, i.e. mesh-mesh.xml.",
        "=> cube + tetra would be 8+4=12 verts and 6+4=10 polys, and qpos z=0.09.",
        "",
        "## what the S2 record says (mismatch)",
        "README.md:212     mesh-mesh (fixture = fixtures/g10-p5-mesh-multicontact/mesh-mesh-two-assets.xml)",
        "README.md:215     the 32-array bins of mesh-mesh-two-assets come from the M3 capture window",
        "receipt.json:325  \"mesh-mesh (fixture mesh-mesh-two-assets)\"",
        "delivery record line 91: third fixture (mesh-mesh-two-assets) ... M3 capture-window case",
        "",
        "# Impact: the bins and the S2 tests are unaffected (tests read bins only, never the XML),",
        "# but evidence traceability points at the wrong fixture and future slices may mix the two",
        "# (the S3/S4 U6 case genuinely uses mesh-mesh-two-assets.xml).",
    ]
    OUT.write_bytes(("\n".join(lines) + "\n").encode("utf-8"))
    print("wrote", OUT)


if __name__ == "__main__":
    main()
