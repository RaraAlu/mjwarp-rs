"""B0: compare run A / run B raw capture JSON byte-for-byte.

Usage: python -B compare_primitive_capture.py <run-a.json> <run-b.json>
Exit 0 = byte-identical; exit 2 = mismatch (STOP, report to Arch; never accept canonical-only equality).
"""
import hashlib
import sys
from pathlib import Path

left = Path(sys.argv[1])
right = Path(sys.argv[2])
a = left.read_bytes()
b = right.read_bytes()
sha_a = hashlib.sha256(a).hexdigest()
sha_b = hashlib.sha256(b).hexdigest()
print(f"run_a={left} sha256={sha_a}")
print(f"run_b={right} sha256={sha_b}")
if a == b:
    print("RAW BYTE-IDENTICAL: PASS")
    sys.exit(0)
print("RAW MISMATCH: STOP and report to Arch (canonical-only equality is not accepted)")
for index, (x, y) in enumerate(zip(a, b)):
    if x != y:
        print(f"first_diff_offset={index} a=0x{x:02x} b=0x{y:02x}")
        break
sys.exit(2)