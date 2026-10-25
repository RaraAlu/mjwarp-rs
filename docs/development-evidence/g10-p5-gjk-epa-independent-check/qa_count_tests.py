import re
from pathlib import Path

text = Path(r"D:\Rust\mjwarp-rs\target\qa-g2\unit-full.log").read_text(encoding="utf-8", errors="replace")
p = f = i = 0
rows = 0
for m in re.finditer(r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored", text):
    rows += 1
    p += int(m.group(2))
    f += int(m.group(3))
    i += int(m.group(4))
print("suites", rows, "passed", p, "failed", f, "ignored", i, "total", p + f + i)
print("ignored tests:", re.findall(r"test (\S+) \.\.\. ignored", text))
