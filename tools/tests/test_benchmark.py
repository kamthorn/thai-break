#!/usr/bin/env python3
"""Unit tests for tools/corpora.py and tools/benchmark.py (synthetic data only)."""

import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from benchmark import evaluate
from corpora import blackboard_chunks, lst20_chunks, without_texts


class TestCorpora(unittest.TestCase):

    def test_lst20_chunks_split_at_spaces_sentences_and_keep_last(self):
        with tempfile.TemporaryDirectory() as tmp:
            Path(tmp, "T1.txt").write_text(
                "ฉัน\tPR\tO\tB_CLS\nรัก\tVV\tO\tI_CLS\n_\tPU\tO\tI_CLS\nไทย\tNN\tO\tE_CLS\n"
                "\n"
                "เขา\tPR\tO\tB_CLS\nมา\tVV\tO\tE_CLS\n",
                encoding="utf-8",
            )
            Path(tmp, "._T1.txt").write_text("garbage", encoding="utf-8")
            self.assertEqual(list(lst20_chunks(Path(tmp))), [["ฉัน", "รัก"], ["ไทย"], ["เขา", "มา"]])

    def test_blackboard_chunks_word_and_subword_levels(self):
        with tempfile.TemporaryDirectory() as tmp:
            Path(tmp, "section_00.conll").write_text(
                "# Tree 1\n1\tนายก\tนายก\tNN\n2\tยัน|ปฏิเสธ\tx\tVV\n3\t_\t_\tPU\n4\tข่าว\tx\tNN\n\n",
                encoding="utf-8",
            )
            self.assertEqual(list(blackboard_chunks(Path(tmp))), [["นายก", "ยันปฏิเสธ"], ["ข่าว"]])
            self.assertEqual(list(blackboard_chunks(Path(tmp), subword=True)), [["นายก", "ยัน", "ปฏิเสธ"], ["ข่าว"]])

    def test_without_texts(self):
        chunks = [["ก", "ข"], ["ค"]]
        self.assertEqual(list(without_texts(chunks, {"กข"})), [["ค"]])


class TestMetrics(unittest.TestCase):

    def test_evaluate_scores_and_error_causes(self):
        gold = [["ประจำ", "ปี"], ["ต่อมา"]]
        pred = [["ประ", "จำปี"], ["ต่อ", "มา"]]
        result = evaluate(gold, pred, dict_words={"ประจำ", "ปี", "ต่อ", "มา"})
        self.assertEqual(result["gold_words"], 3)
        self.assertAlmostEqual(result["word"]["recall"], 0.0)
        # Gold boundary after "ประจำ" is missed, predicted ones after "ประ" and "ต่อ" are wrong
        self.assertAlmostEqual(result["boundary"]["precision"], 0.0)
        self.assertEqual(result["errors"], {"in dictionary but mis-segmented": 2, "not in dictionary": 1})
        self.assertEqual(result["top_oov"], [("ต่อมา", 1)])

    def test_evaluate_ignores_whitespace_inside_gold_words(self):
        gold = [["ณ\u00a0อยุธยา", "ไป"]]
        pred = [["ณ", "อยุธยา", "ไป"]]
        result = evaluate(gold, pred)
        self.assertEqual(result["skipped_chunks"], 0)
        self.assertEqual(result["gold_words"], 2)
        self.assertAlmostEqual(result["word"]["precision"], 1 / 3)

    def test_evaluate_perfect_and_skips_text_mismatch(self):
        gold = [["ฉัน", "รัก"], ["ไทย"]]
        pred = [["ฉัน", "รัก"], ["ไท"]]
        result = evaluate(gold, pred)
        self.assertEqual(result["skipped_chunks"], 1)
        self.assertAlmostEqual(result["word"]["f1"], 1.0)
        self.assertAlmostEqual(result["boundary"]["f1"], 1.0)


if __name__ == "__main__":
    unittest.main()
