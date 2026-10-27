"""Scan attempt1 evidence logs for encoding, BOM and line endings."""
from pathlib import Path

EV = Path(r"D:\Rust\mjwarp-rs\docs\development-evidence\product-collision-inputs-wiring\logs")
OUT = Path(r"D:\Rust\mjwarp-rs\target\qa-s3win\encoding-scan.txt")


def classify(raw: bytes):
    bom = raw.startswith(b"\xef\xbb\xbf")
    crlf = b"\r\n" in raw
    zero_ratio = raw.count(b"\x00") / max(1, len(raw))
    encoding = "unknown"
    text = None
    if zero_ratio > 0.2 and len(raw) % 2 == 0:
        try:
            text = raw.decode("utf-16-le")
            encoding = "utf-16-le"
        except UnicodeDecodeError:
            encoding = "utf-16-le-invalid"
    else:
        try:
            text = raw.decode("utf-8")
            encoding = "utf-8"
        except UnicodeDecodeError:
            encoding = "not-utf8"
    return encoding, bom, crlf, text


def main():
    rows = ["# attempt1 evidence log encoding scan", "# file | bytes | encoding | bom | crlf | lines"]
    bad = []
    for path in sorted(EV.glob("attempt1-*")):
        raw = path.read_bytes()
        encoding, bom, crlf, text = classify(raw)
        lines = len(text.splitlines()) if text is not None else -1
        rows.append(f"{path.name} | {len(raw)} | {encoding} | bom={bom} | crlf={crlf} | lines={lines}")
        if encoding != "utf-8" or bom:
            bad.append(f"{path.name}: {encoding} bom={bom} crlf={crlf}")
    rows.append("")
    rows.append(f"non-utf8-or-bom: {len(bad)}")
    rows.extend(bad)
    OUT.write_bytes(("\n".join(rows) + "\n").encode("utf-8"))
    print("\n".join(rows))


if __name__ == "__main__":
    main()
