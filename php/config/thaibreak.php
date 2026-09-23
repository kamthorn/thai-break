<?php

declare(strict_types=1);

return [
    /*
    |--------------------------------------------------------------------------
    | Custom Dictionary Path
    |--------------------------------------------------------------------------
    |
    | Path to a custom dictionary file (format: word or word<TAB>weight).
    | If set to null, the default built-in dictionary (data/words.txt) will be used.
    |
    */
    'dict_path' => env('THAIBREAK_DICT_PATH', null),

    /*
    |--------------------------------------------------------------------------
    | Custom Bigram Model Path
    |--------------------------------------------------------------------------
    |
    | Path to a custom bigram TSV file (format: prev_word<TAB>word<TAB>count).
    | If set to null, no bigram model will be used by default.
    |
    */
    'bigram_path' => env('THAIBREAK_BIGRAM_PATH', null),

    /*
    |--------------------------------------------------------------------------
    | Custom Words & Domain Vocabularies
    |--------------------------------------------------------------------------
    |
    | Additional domain words to inject into the tokenizer upon booting.
    | Keys are words, and values are optional weight numbers (higher = higher priority).
    |
    | Example:
    |   'custom_words' => [
    |       'คนละครึ่ง' => 50000.0,
    |       'เราเที่ยวด้วยกัน' => 50000.0,
    |   ],
    |
    */
    'custom_words' => [],

    /*
    |--------------------------------------------------------------------------
    | Line Breaker & Typographic Wrapping
    |--------------------------------------------------------------------------
    |
    | Settings for Thai line breaks (HTML, PDF, or Plain Text wrapping).
    |
    */
    'line_breaker' => [
        // Default break opportunity marker: Zero-Width Space (U+200B)
        'default_marker' => "\u{200B}",

        // Default display column width for wrapping
        'default_wrap_width' => 60,

        // Automatically detect HTML markup and preserve tags/entities
        'auto_detect_html' => true,
    ],
];
