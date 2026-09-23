import os
import sys
import unittest

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

import thaibreak


class TestThaiBreak(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        dict_path = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../data/words.txt"))
        cls.initialized = thaibreak.init(dict_path, None)

    def test_init(self):
        self.assertTrue(self.initialized, "Failed to init")

    def test_words(self):
        tokens = thaibreak.words("ฉันรักภาษาไทย")
        self.assertEqual(tokens, ["ฉัน", "รัก", "ภาษา", "ไทย"])

    def test_lines_html(self):
        html = '<div class="title"><b>สวัสดี</b> &amp; ประเทศไทย</div><script>var x = "สวัสดีประเทศไทย";</script><!-- หมายเหตุ -->'
        broken = thaibreak.lines(html, marker="|", is_html=True)
        self.assertEqual('<div class="title"><b>สวัสดี</b> &amp; ประเทศ|ไทย</div><script>var x = "สวัสดีประเทศไทย";</script><!-- หมายเหตุ -->', broken)


    def test_display_width(self):
        self.assertEqual(thaibreak.display_width("ภาษาไทย"), 7)
        self.assertEqual(thaibreak.display_width("ก"), 1)

    def test_wrap(self):
        wrapped = thaibreak.wrap("ฉันรักภาษาไทยมากที่สุดในโลก", 12)
        self.assertIn("\n", wrapped)


if __name__ == "__main__":
    unittest.main()

