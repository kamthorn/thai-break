package thaibreak

import (
	"strings"
	"testing"
)

func TestDawgMatchesTsv(t *testing.T) {
	trieTsv, err := LoadTsv(strings.NewReader(defaultWordsData))
	if err != nil {
		t.Fatalf("failed to load TSV: %v", err)
	}

	trieDawg, err := LoadDawg(defaultDawgData)
	if err != nil {
		t.Fatalf("failed to load DAWG: %v", err)
	}

	sentences := []string{
		"สถาบันวิจัยดาราศาสตร์แห่งชาติกระทรวงการอุดมศึกษาวิทยาศาสตร์วิจัยและนวัตกรรม",
		"กรุงเทพมหานครอมรรัตนโกสินทร์มหินทรายุธยามหาดิลกภพนพรัตน์ราชธานีบุรีรมย์",
		"ทดสอบคำสั้นคำยาว กก กกหู กงการ กฎหมาย",
		"ปัญญาประดิษฐ์กำลังเปลี่ยนแปลงวิถีชีวิตและระบบการทำงานของมนุษย์ทั่วโลก",
	}

	tokTsv := NewTokenizer(trieTsv, nil)
	tokDawg := NewTokenizer(trieDawg, nil)

	for _, s := range sentences {
		resTsv := tokTsv.Tokenize(s, false)
		resDawg := tokDawg.Tokenize(s, false)

		if strings.Join(resTsv, "|") != strings.Join(resDawg, "|") {
			t.Errorf("Mismatch on '%s':\nTSV:  %v\nDAWG: %v", s, resTsv, resDawg)
		}
	}
}

func TestDawgDynamicAdd(t *testing.T) {
	trieDawg, err := LoadDawg(defaultDawgData)
	if err != nil {
		t.Fatalf("failed to load DAWG: %v", err)
	}

	sample := "สวัสดีชาวโลกและทดสอบคำแปลกใหม่จ้า"
	tok := NewTokenizer(trieDawg, nil)
	words1 := tok.Tokenize(sample, false)

	// Add dynamic new word
	trieDawg.Add("คำแปลกใหม่", 2.0)
	words2 := tok.Tokenize(sample, false)

	found := false
	for _, w := range words2 {
		if w == "คำแปลกใหม่" {
			found = true
			break
		}
	}
	if !found {
		t.Errorf("Expected dynamically added word 'คำแปลกใหม่' to be segmented, got %v (was %v)", words2, words1)
	}
}

func TestLoadDawgRejectsCorruptData(t *testing.T) {
	header := func(numStates uint32, body ...byte) []byte {
		data := []byte{'T', 'B', 'D', '1', byte(numStates), byte(numStates >> 8), byte(numStates >> 16), byte(numStates >> 24), 0, 0, 0, 0}
		return append(data, body...)
	}
	cases := map[string][]byte{
		"more states than bytes":      header(0xFFFFFFFF, 0),
		"no states":                   header(0),
		"state runs past the end":     header(1, 3, 'a', 0, 0, 0),
		"edge to a state that is not": header(1, 1, 'a', 0, 5, 0),
		"truncated":                   header(2, 0),
	}
	for name, data := range cases {
		if _, err := LoadDawg(data); err == nil {
			t.Errorf("%s: LoadDawg accepted corrupt data", name)
		}
	}
	// The bundled dictionary and a minimal valid one still load
	if _, err := LoadDawg(defaultDawgData); err != nil {
		t.Errorf("bundled DAWG rejected: %v", err)
	}
	if _, err := LoadDawg(header(1, 0)); err != nil {
		t.Errorf("minimal DAWG rejected: %v", err)
	}
}
