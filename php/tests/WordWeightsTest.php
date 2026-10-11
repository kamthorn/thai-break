<?php

declare(strict_types=1);

namespace ThaiBreak\Tests;

use PHPUnit\Framework\TestCase;
use ThaiBreak\ThaiTokenizer;
use ThaiBreak\ThaiTrie;

class WordWeightsTest extends TestCase
{
    public function testSetWeightReplacesTheWeightOfADictionaryWord(): void
    {
        $flat = new ThaiTrie();
        $flat->addMany(['แมว', 'แมวน้ำ', 'หมา']);
        $dawg = ThaiTrie::fromDawgFile(dirname(__DIR__, 2) . '/data/words.dawg');

        foreach (['flat' => $flat, 'dawg' => $dawg] as $name => $trie) {
            $base = $trie->totalWeight();
            $this->assertTrue($trie->setWeight('แมว', 5.0), $name);
            $this->assertSame($base + 4.0, $trie->totalWeight(), $name);
            $this->assertTrue($trie->setWeight('แมว', 2.0), $name);
            $this->assertSame($base + 1.0, $trie->totalWeight(), $name);
            $this->assertFalse($trie->setWeight('ฟลีบโลก', 3.0), "{$name}: a word outside the dictionary is not added");
            $this->assertSame(2.0, $trie->getWeight('แมว'), $name);
            $this->assertSame(1, $trie->applyWeights("# comment\nหมา\t3.5\nฟลีบโลก\t2\n"), $name);
        }

        $this->expectException(\InvalidArgumentException::class);
        $flat->applyWeights("หมา\tx\n");
    }

    public function testDictionaryWordsOfUpTo40CharactersAreMatched(): void
    {
        // พระบาทสมเด็จพระเจ้าอยู่หัว has 26 characters, one more than the old limit of 25
        $title = 'พระบาทสมเด็จพระเจ้าอยู่หัว';
        $long  = 'สมเด็จพระนางเจ้าสิริกิติ์พระบรมราชินีนาถ';
        $this->assertSame(40, mb_strlen($long, 'UTF-8'));
        $trie = new ThaiTrie();
        $trie->addMany(['พระ', 'บาท', 'สมเด็จ', $title, $long, 'เสด็จ']);
        $tokenizer = new ThaiTokenizer($trie);
        foreach ([$title, $long] as $word) {
            $this->assertSame([$word, 'เสด็จ'], $tokenizer->tokenize($word . 'เสด็จ'));
        }
    }

    public function testDefaultDictionaryHasWordWeights(): void
    {
        $trie = ThaiTokenizer::withDefaultDict()->getTrie();
        $this->assertGreaterThan(1.0, $trie->getWeight('ที่'));
    }
}
