"""Smoke test for an installed wheel: run it outside the source tree."""
import os
import sys

import thaibreak

assert "site-packages" in os.path.abspath(thaibreak.__file__), thaibreak.__file__
assert thaibreak.words("ฉันรักภาษาไทย") == ["ฉัน", "รัก", "ภาษา", "ไทย"], thaibreak.words("ฉันรักภาษาไทย")
assert thaibreak.lines("ประเทศไทย", marker="|") == "ประเทศ|ไทย"
assert "\n" in thaibreak.wrap("ฉันรักภาษาไทยมากที่สุดในโลก", 12)
assert thaibreak.display_width("ภาษาไทย") == 7
print("ok", thaibreak.__version__, sys.platform)
