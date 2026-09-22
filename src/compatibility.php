<?php

declare(strict_types=1);

// Backward compatibility class aliases for PHPThaiNLP
if (!class_exists('PHPThaiNLP\Tokenize\ThaiTokenizer', false)) {
    class_alias(\ThaiBreak\ThaiTokenizer::class, 'PHPThaiNLP\Tokenize\ThaiTokenizer');
    class_alias(\ThaiBreak\ThaiLineBreaker::class, 'PHPThaiNLP\Tokenize\ThaiLineBreaker');
    class_alias(\ThaiBreak\ThaiTrie::class, 'PHPThaiNLP\Tokenize\ThaiTrie');
    class_alias(\ThaiBreak\ThaiTCC::class, 'PHPThaiNLP\Tokenize\ThaiTCC');
    class_alias(\ThaiBreak\BigramModel::class, 'PHPThaiNLP\Tokenize\BigramModel');
    class_alias(\ThaiBreak\DictionaryLoader::class, 'PHPThaiNLP\Tokenize\DictionaryLoader');
    class_alias(\ThaiBreak\WeightedTokenizer::class, 'PHPThaiNLP\Tokenize\WeightedTokenizer');
    class_alias(\ThaiBreak\Laravel\ThaiBreakServiceProvider::class, 'PHPThaiNLP\Laravel\ThaiNLPServiceProvider');
    class_alias(\ThaiBreak\Laravel\Facades\ThaiBreak::class, 'PHPThaiNLP\Laravel\Facades\ThaiNLP');
}
