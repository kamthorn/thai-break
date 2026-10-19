package thaibreak

import (
	"strings"
	"testing"
)

// segmentationCases maps input to the expected words joined by '|'.
var segmentationCases = []struct{ name, input, want string }{
	{"a word before a full stop is not cut into an abbreviation", "เขากินข้าว.", "เขา|กิน|ข้าว|."},
	{"a word before a full stop, then more text", "ฉันรักเธอ.ไปเที่ยวกัน", "ฉัน|รัก|เธอ|.|ไป|เที่ยว|กัน"},
	{"abbreviations are still recognized", "เมื่อ 5 มิ.ย. ที่ จ.พิษณุโลก", "เมื่อ|5|มิ.ย.|ที่|จ.|พิษณุโลก"},
	{"an abbreviation after a word keeps the word whole", "ในเขตจ.พิจิตร", "ใน|เขต|จ.|พิจิตร"},
	{"an abbreviation after a word that ends like one", "ในเดือนพ.ย.", "ใน|เดือน|พ.ย."},
	{"ties keep the earlier word whole", "บอกว่าอึดอัด", "บอก|ว่า|อึดอัด"},
	{"ties keep the earlier word whole (2)", "ลาออกจากรองประธาน", "ลาออก|จาก|รอง|ประธาน"},
	{"ก็ is not swallowed by the cluster before it", "ทะเลก็สวย", "ทะเล|ก็|สวย"},
	{"a final consonant before a vowel starts the next cluster", "รึยัง", "รึ|ยัง"},
	{"no-break spaces are whitespace, not words", "ราคา 100 บาท", "ราคา|100|บาท"},
	{"two sara e are matched as sara ae, the text is kept", "เเข็งเเรงมาก", "เเข็งเเรง|มาก"},
	{"nikhahit + sara aa is matched as sara am, the text is kept", "นํ้าตาลทราย", "นํ้าตาล|ทราย"},
	{"an out-of-vocabulary name made of dictionary words is kept whole", "ชวรัตน์", "ชวรัตน์"},
	{"an out-of-vocabulary name is kept whole in a sentence", "นายศุภชัยกล่าวว่ามีการประชุม", "นาย|ศุภชัย|กล่าว|ว่า|มี|การ|ประชุม"},
	{"the repetition mark is not part of an unknown word", "อื่นๆ", "อื่น|ๆ"},
	{"an unknown word does not swallow the function words after it", "มาร์คฮิวจ์สไม่ได้สามารถ", "มาร์ค|ฮิวจ์ส|ไม่|ได้|สามารถ"},
	{"an unknown word does not swallow a conjunction", "ภานุวัฒน์ จินตะและสุทธินันท์", "ภานุวัฒน์|จินตะ|และ|สุทธิ|นันท์"},
	{"an unknown word does not swallow a function word inside it", "ชวรัตน์ที่ลืมไว้", "ชวรัตน์|ที่|ลืม|ไว้"},
	{"a transliterated name may contain มา", "อะห์มาดีเนญาดกล่าวว่า", "อะห์มาดีเนญาด|กล่าว|ว่า"},
	{"an unknown name does not swallow the title before it", "พี่เต๋อมาแล้ว", "พี่|เต๋อ|มา|แล้ว"},
	{"spelled-out letters are one token", "เอทีเอ็ม", "เอทีเอ็ม"},
	{"spelled-out letters of an acronym", "ดีเอสไอ", "ดีเอสไอ"},
	{"spelled-out letters next to words", "ยูเอ็นกล่าวว่าไอทีสำคัญ", "ยูเอ็น|กล่าว|ว่า|ไอที|สำคัญ"},
	{"a single letter name stays a word", "ไอ", "ไอ"},
	{"a tone mark keeps ที่ from being the letter T", "ที่ดีเอสไอ", "ที่|ดีเอสไอ"},
	{"a word and a final full stop are not an abbreviation", "ต่อไป.", "ต่อ|ไป|."},
	{"a word with a tone mark and a final full stop", "หรือไม่.", "หรือ|ไม่|."},
	{"a function word does not start an abbreviation", "ของส.ส.", "ของ|ส.ส."},
	{"a function word before a long abbreviation", "ของพ.ต.ท.", "ของ|พ.ต.ท."},
	{"an abbreviation followed by a name stays whole", "จ.เชียงใหม่", "จ.|เชียงใหม่"},
	{"a run of abbreviations stays whole", "ก.พ.", "ก.พ."},
	{"a tone mark after sara am is matched before it, the text is kept", "นำ้ตาลทราย", "นำ้ตาล|ทราย"},
	{"nikhahit + sara aa + tone mark is matched as tone + sara am", "นํา้ตาลทราย", "นํา้ตาล|ทราย"},
	{"Thai digits join ASCII digits in one token", "ราคา 1๐.๐๐ บาท", "ราคา|1๐.๐๐|บาท"},
	{"Thai digits after ASCII digits", "ปี 2567๑", "ปี|2567๑"},
	{"a -ีย vowel before a non-Thai character is one cluster", "ค่าเฉลี่ย)", "ค่า|เฉลี่ย|)"},
	{"a -ีย vowel before an ASCII digit is one cluster", "ค่าเฉลี่ย1", "ค่า|เฉลี่ย|1"},
	{"U+0085 is white space and is dropped", "เขา\u0085กิน", "เขา|กิน"},
	{"a -ีย vowel before a space is one cluster", "แนวทางไกร่เกลี่ย หรือแก", "แนวทาง|ไกร่เกลี่ย|หรือ|แก"},
	{"a common word does not start an abbreviation", "พบพล.อ.ประยุทธ์", "พบ|พล.อ.|ประยุทธ์"},
	{"a closed -็อ- syllable is kept as one TCC cluster", "คุณต็อดเล่นติ๊กต็อก", "คุณ|ต็อด|เล่น|ติ๊กต็อก"},
	{"ก็ before อ- is not merged into a -็อ- cluster", "เขาก็อดทน", "เขา|ก็|อดทน"},
	{"นาย does not steal the initial ก of a following name", "นายกษิตกล่าวว่า", "นาย|กษิต|กล่าว|ว่า"},
	{"หมอ does not steal the initial ก of a following name", "หมอกฤษฏ์คอนเฟิร์ม", "หมอ|กฤษฏ์|คอนเฟิร์ม"},
	{"นายก before an abbreviation stays นายก", "นายกอบต.", "นายก|อบต."},
	{"an abbreviation does not steal the final consonant of a cอc word", "เดินตามรอยพ.ต.ท.ทักษิณ", "เดิน|ตาม|รอย|พ.ต.ท.|ทักษิณ"},
	{"จ. does not steal the final consonant of a cอc word", "ไฟฟ้าน้ำพองจ.ขอนแก่น", "ไฟฟ้า|น้ำ|พอง|จ.|ขอนแก่น"},
}

func TestLongTextIsSegmentedToTheEnd(t *testing.T) {
	// Longer than the 50,000-edge limit that used to leave the rest of the text as one token
	sentence := "การประชุมสามัญผู้ถือหุ้นประจำปีจัดขึ้นที่โรงแรมในกรุงเทพมหานคร"
	words := Words(strings.Repeat(sentence, 2000))
	if want := 2000 * len(Words(sentence)); len(words) != want {
		t.Fatalf("got %d words, want %d", len(words), want)
	}
	for _, w := range words {
		if len([]rune(w)) >= 20 {
			t.Fatalf("unexpected long token of %d characters", len([]rune(w)))
		}
	}
}

func TestTCCNeverSplitsBeforeAVowelOrToneMark(t *testing.T) {
	for _, word := range []string{"เมื่อ", "เนื้อ", "เบื่อ", "ต้น", "เกล็ด", "เหม็น", "ลั๊วะ", "รึยัง"} {
		runes := []rune(word)
		valid := TCCPosArray(runes)
		for i := 1; i < len(runes); i++ {
			if isDependentThai(runes[i]) && valid[i] {
				t.Errorf("boundary before %q in %q", runes[i], word)
			}
		}
	}
}

func TestTCCKeepsAMaiHanAkatSyllableWhole(t *testing.T) {
	// Mai Han-akat is always followed by a final (or -ัวะ), so the syllable is one cluster
	for _, word := range []string{"ผัวะ", "จั๊วะ", "ยัง", "ยั่ง", "หัว", "สัญ", "กัณฐ์"} {
		runes := []rune(word)
		valid := TCCPosArray(runes)
		for i := 1; i < len(runes); i++ {
			if valid[i] {
				t.Errorf("%q was split before %q", word, runes[i])
			}
		}
	}
}

func TestTCCKeepsAClusterOnsetAfterSaraEWhole(t *testing.T) {
	// เ + a true cluster or ห-led onset + -ิ / -ือ / -า is one syllable
	for _, word := range []string{"เปล่า", "เหล้า", "เหงา", "เพลิง", "เหลือ", "เกลือ", "เครือ"} {
		runes := []rune(word)
		valid := TCCPosArray(runes)
		for i := 1; i < len(runes); i++ {
			if valid[i] {
				t.Errorf("%q was split before %q", word, runes[i])
			}
		}
	}
	// a consonant pair that is not a cluster can still start the next word
	for _, c := range []struct {
		text     string
		boundary int
	}{{"เทลง", 2}, {"ทะเลว่า", 4}} {
		if !TCCPosArray([]rune(c.text))[c.boundary] {
			t.Errorf("no boundary at %d in %q", c.boundary, c.text)
		}
	}
}

func TestSegmentation(t *testing.T) {
	for _, c := range segmentationCases {
		if got := strings.Join(Words(c.input), "|"); got != c.want {
			t.Errorf("%s: Words(%q) = %q, want %q", c.name, c.input, got, c.want)
		}
	}
}

func TestDictionaryWordsOfUpTo40CharactersAreMatched(t *testing.T) {
	// พระบาทสมเด็จพระเจ้าอยู่หัว has 26 characters, one more than the old limit of 25
	title := "พระบาทสมเด็จพระเจ้าอยู่หัว"
	long := "สมเด็จพระนางเจ้าสิริกิติ์พระบรมราชินีนาถ"
	if n := len([]rune(long)); n != 40 {
		t.Fatalf("test word has %d characters", n)
	}
	trie, err := LoadTsv(strings.NewReader("พระ\nบาท\nสมเด็จ\n" + title + "\n" + long + "\nเสด็จ\n"))
	if err != nil {
		t.Fatal(err)
	}
	tok := NewTokenizer(trie, nil)
	for _, w := range []string{title, long} {
		if got := strings.Join(tok.Tokenize(w+"เสด็จ", false), "|"); got != w+"|เสด็จ" {
			t.Errorf("Tokenize(%qเสด็จ) = %q", w, got)
		}
	}
}
