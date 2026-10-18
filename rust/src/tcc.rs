use once_cell::sync::Lazy;

const C: &str = "[ก-ฮ]";
const T: &str = "[่-๋]?";
const D: &str = "[ุู]";
const K: &str = "([ก-ฮ][ก-ฮ]?[ุูิ]?์)?";

/// Cluster rules, tried in order; the first rule that matches wins (leftmost-first, like a regex
/// alternation). `c` is a consonant, `t` an optional tone mark, `d` a below vowel, `k` an optional
/// silent ending (karan).
const GENERAL_RULES: &[&str] = &[
    // Mai Han-akat always has a final: -ัวะ, or a consonant (with or without a tone mark).
    "cั[่-๋]?วะ",
    "cั[่-๋]?ck",
    "cั[่-๋]?",
    "เc็ck",
    "เcctาะk",
    "เccีtยะk",
    "เ(?:c[รลว]|หc)็ck",
    "เcิc์ck",
    // A true cluster or ห-led onset between เ and the vowel belongs to the syllable (เครื่อง, เพลิง, เปล่า).
    "เ(?:[กขคตปพผบด][รล]|[กขค]ว|ทร|ห[งญนมยรลว])ิtck",
    "เcิtck",
    "เcีtยะ?k",
    "เ(?:[กขคตปพผบด][รล]|[กขค]ว|ทร|ห[งญนมยรลว])ืtอะ?k",
    "เcืtอะk",
    "เcืtอ?k",
    "เ(?:[กขคตปพผบด][รล]|[กขค]ว|ทร|ห[งญนมยรลว])tาk",
    "เctา?ะ?k",
    "c[ึื]tck",
    "c[ะ-ู]tk",
    "c[ิุู]์",
    "cรรc์",
    "c็",
    "ct[ะาำ]?k",
    "แc็ck",
    "แcc์k",
    "แctะk",
    "แ(?:c[รลว]|หc)็ck",
    "แccc์k",
    "โctะk",
    "[เ-ไ]ctk",
    "ก็",
    "อึ",
    "หึ",
];

/// Rules that only apply when the next character can start a syllable (all of them start with เ).
const LOOKAHEAD_RULES: &[&str] = &["เccีtยk", "เc[ิีุู]tยk"];

fn expand(rule: &str) -> String {
    rule.replace('k', K).replace('c', C).replace('t', T).replace('d', D)
}

/// One instruction of the rules' NFA. A set of characters is a bit mask over U+0E00..U+0E7F,
/// since every rule matches Thai characters only.
#[derive(Clone, Copy, Debug)]
enum Inst {
    Char(u128),
    /// Try the first branch, then the second (greedy `?` and alternation order).
    Split(usize, usize),
    Jmp(usize),
    Match,
}

/// Parsed rule: a set of characters, an optional (greedy) node, or alternatives of sequences.
enum Node {
    Set(u128),
    Opt(Box<Node>),
    Alt(Vec<Vec<Node>>),
}

fn thai_bit(ch: char) -> u128 {
    let off = (ch as u32).wrapping_sub(0x0E00);
    assert!(off < 128, "TCC rules use Thai characters only: {ch:?}");
    1u128 << off
}

struct Parser<'a> {
    chars: &'a [char],
    pos: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    /// alt := seq ('|' seq)*
    fn alt(&mut self) -> Vec<Vec<Node>> {
        let mut branches = vec![self.seq()];
        while self.peek() == Some('|') {
            self.pos += 1;
            branches.push(self.seq());
        }
        branches
    }

    /// seq := (atom '?'?)*
    fn seq(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();
        while let Some(ch) = self.peek() {
            if ch == '|' || ch == ')' {
                break;
            }
            let atom = self.atom();
            if self.peek() == Some('?') {
                self.pos += 1;
                nodes.push(Node::Opt(Box::new(atom)));
            } else {
                nodes.push(atom);
            }
        }
        nodes
    }

    fn atom(&mut self) -> Node {
        let ch = self.peek().expect("atom");
        self.pos += 1;
        match ch {
            '[' => {
                let mut mask = 0u128;
                while self.peek() != Some(']') {
                    let lo = self.peek().expect("class");
                    self.pos += 1;
                    if self.peek() == Some('-') && self.chars.get(self.pos + 1) != Some(&']') {
                        let hi = self.chars[self.pos + 1];
                        self.pos += 2;
                        for cp in lo as u32..=hi as u32 {
                            mask |= thai_bit(char::from_u32(cp).unwrap());
                        }
                    } else {
                        mask |= thai_bit(lo);
                    }
                }
                self.pos += 1;
                Node::Set(mask)
            }
            '(' => {
                if self.peek() == Some('?') {
                    assert_eq!(self.chars.get(self.pos + 1), Some(&':'));
                    self.pos += 2;
                }
                let branches = self.alt();
                assert_eq!(self.peek(), Some(')'));
                self.pos += 1;
                Node::Alt(branches)
            }
            ch => Node::Set(thai_bit(ch)),
        }
    }
}

fn emit(node: &Node, prog: &mut Vec<Inst>) {
    match node {
        Node::Set(mask) => prog.push(Inst::Char(*mask)),
        Node::Opt(inner) => {
            let split = prog.len();
            prog.push(Inst::Match);
            emit(inner, prog);
            prog[split] = Inst::Split(split + 1, prog.len());
        }
        Node::Alt(branches) => emit_alt(branches, prog),
    }
}

fn emit_alt(branches: &[Vec<Node>], prog: &mut Vec<Inst>) {
    let mut jumps = Vec::new();
    for (k, branch) in branches.iter().enumerate() {
        let last = k + 1 == branches.len();
        let split = prog.len();
        if !last {
            prog.push(Inst::Match);
        }
        for node in branch {
            emit(node, prog);
        }
        if !last {
            jumps.push(prog.len());
            prog.push(Inst::Match);
            prog[split] = Inst::Split(split + 1, prog.len());
        }
    }
    let end = prog.len();
    for j in jumps {
        prog[j] = Inst::Jmp(end);
    }
}

/// NFA of the alternation of `rules`, in priority order.
fn compile(rules: &[&str]) -> Vec<Inst> {
    let branches: Vec<Vec<Node>> = rules
        .iter()
        .map(|rule| {
            let chars: Vec<char> = expand(rule).chars().collect();
            let mut p = Parser { chars: &chars, pos: 0 };
            let alt = p.alt();
            assert_eq!(p.pos, chars.len(), "unparsed rule {rule}");
            vec![Node::Alt(alt)]
        })
        .collect();
    let mut prog = Vec::new();
    emit_alt(&branches, &mut prog);
    prog.push(Inst::Match);
    prog
}

/// Threads reachable from `pcs` without reading a character, in priority order. Threads after
/// a `Match` are dropped: a match of a higher-priority rule wins over anything they could match.
fn closure(prog: &[Inst], pcs: impl IntoIterator<Item = usize>) -> Vec<usize> {
    let mut out = Vec::new();
    let mut seen = vec![false; prog.len()];
    for pc in pcs {
        let mut stack = vec![pc];
        while let Some(pc) = stack.pop() {
            if std::mem::replace(&mut seen[pc], true) {
                continue;
            }
            match prog[pc] {
                Inst::Char(_) => out.push(pc),
                Inst::Split(a, b) => stack.extend([b, a]),
                Inst::Jmp(a) => stack.push(a),
                Inst::Match => {
                    out.push(pc);
                    return out;
                }
            }
        }
    }
    out
}

const DEAD: u16 = 0;

/// Anchored leftmost-first DFA over classes of Thai characters, built from the NFA by subset
/// construction on ordered thread lists (as a leftmost-first regex DFA is). It reports the same
/// match as a regex alternation of the rules: the match of the first rule that matches.
struct Dfa {
    /// Class of each character of the Thai block; other characters never match.
    class: [u8; 128],
    classes: usize,
    /// `trans[state * classes + class]`, state 0 is dead
    trans: Vec<u16>,
    is_match: Vec<bool>,
    start: u16,
}

impl Dfa {
    fn new(rules: &[&str]) -> Self {
        let prog = compile(rules);
        let masks: Vec<u128> = prog
            .iter()
            .filter_map(|inst| match inst {
                Inst::Char(m) => Some(*m),
                _ => None,
            })
            .collect();

        // Characters that every rule treats alike share a class
        let mut class = [0u8; 128];
        let mut reps: Vec<usize> = Vec::new();
        let mut signatures: Vec<Vec<bool>> = Vec::new();
        for (off, slot) in class.iter_mut().enumerate() {
            let sig: Vec<bool> = masks.iter().map(|m| m >> off & 1 == 1).collect();
            let id = signatures.iter().position(|s| *s == sig).unwrap_or_else(|| {
                signatures.push(sig);
                reps.push(off);
                signatures.len() - 1
            });
            *slot = u8::try_from(id).expect("too many TCC character classes");
        }
        let classes = reps.len();

        let mut states: Vec<Vec<usize>> = vec![Vec::new()];
        let mut index = std::collections::HashMap::new();
        index.insert(Vec::new(), DEAD);
        let start = closure(&prog, [0]);
        index.insert(start.clone(), 1);
        states.push(start);
        let mut trans = vec![DEAD; 2 * classes];
        let mut done = 1;
        while done < states.len() {
            for (c, &off) in reps.iter().enumerate() {
                let next = closure(
                    &prog,
                    states[done].iter().filter_map(|&pc| match prog[pc] {
                        Inst::Char(m) if m >> off & 1 == 1 => Some(pc + 1),
                        _ => None,
                    }),
                );
                let id = *index.entry(next.clone()).or_insert_with(|| {
                    states.push(next);
                    trans.resize(trans.len() + classes, DEAD);
                    u16::try_from(states.len() - 1).expect("too many TCC DFA states")
                });
                trans[done * classes + c] = id;
            }
            done += 1;
        }
        let is_match = states
            .iter()
            .map(|s| s.last().is_some_and(|&pc| matches!(prog[pc], Inst::Match)))
            .collect();
        Self {
            class,
            classes,
            trans,
            is_match,
            start: 1,
        }
    }

    /// End of the match starting at `start`, if any.
    fn find(&self, chars: &[char], start: usize) -> Option<usize> {
        let mut state = self.start;
        let mut found = None;
        for (pos, &ch) in chars.iter().enumerate().skip(start) {
            let off = (ch as u32).wrapping_sub(0x0E00);
            if off >= 128 {
                break;
            }
            state = self.trans[state as usize * self.classes + self.class[off as usize] as usize];
            if state == DEAD {
                break;
            }
            if self.is_match[state as usize] {
                found = Some(pos + 1);
            }
        }
        found
    }
}

static GENERAL: Lazy<Dfa> = Lazy::new(|| Dfa::new(GENERAL_RULES));
static LOOKAHEAD: Lazy<Dfa> = Lazy::new(|| Dfa::new(LOOKAHEAD_RULES));

#[inline]
fn is_thai_consonant(ch: char) -> bool {
    ('ก'..='ฮ').contains(&ch)
}

/// End of a matched cluster `chars[start..end]`. A final consonant followed by a dependent
/// vowel or mark starts the next cluster instead: "รึยัง" is "รึ" + "ยัง", not
/// "รึย" + "ัง". Except ว before ะ, which is part of the vowel -ัวะ ("ผัวะ").
fn cluster_end(chars: &[char], start: usize, end: usize) -> usize {
    if let Some(&next) = chars.get(end) {
        let last = chars[end - 1];
        if end - start >= 2 && is_thai_consonant(last) && is_dependent_thai(next) && !(last == 'ว' && next == 'ะ') {
            return end - 1;
        }
    }
    end
}

/// Whether a character can never start a cluster: ะ ั า ำ ิ–ฺ ๅ ็–๎.
fn is_dependent_thai(ch: char) -> bool {
    matches!(ch as u32, 0x0E30..=0x0E3A | 0x0E45 | 0x0E47..=0x0E4E)
}

#[inline]
fn is_followed_by_lookahead_char(next: Option<char>) -> bool {
    match next {
        None => true,
        Some(ch) => {
            let cp = ch as u32;
            (0x0E01..=0x0E2E).contains(&cp) || (0x0E40..=0x0E44).contains(&cp) || !(0x0E00..=0x0E7F).contains(&cp)
        }
    }
}

/// Compute valid TCC break positions for a slice of characters.
///
/// Returns a boolean vector of size `chars.len() + 1` where `true` indicates
/// a safe word boundary.
pub fn tcc_pos_array(chars: &[char]) -> Vec<bool> {
    let n = chars.len();
    let mut valid = vec![false; n + 1];
    valid[0] = true;
    valid[n] = true;

    let mut pos = 0;
    while pos < n {
        // First, check lookahead rules (all of them start with เ)
        if chars[pos] == 'เ' {
            if let Some(end) = LOOKAHEAD.find(chars, pos) {
                if is_followed_by_lookahead_char(chars.get(end).copied()) {
                    pos = end;
                    valid[pos] = true;
                    continue;
                }
            }
        }

        // Second, check general rules; otherwise advance one character
        pos = match GENERAL.find(chars, pos) {
            Some(end) => cluster_end(chars, pos, end),
            None => pos + 1,
        };
        valid[pos] = true;
    }

    for i in 0..n {
        let ch = chars[i];
        if !(0x0E00..=0x0E7F).contains(&(ch as u32)) {
            // Non-Thai characters are always valid boundaries
            valid[i] = true;
            valid[i + 1] = true;
        } else if i > 0 && is_dependent_thai(ch) {
            // Never a boundary before a dependent vowel or mark (e.g. inside "เมื่อ")
            valid[i] = false;
        }
    }

    valid
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    /// The regex implementation the matcher replaced, kept as the reference.
    fn alternation(rules: &[&str]) -> Regex {
        let patterns: Vec<String> = rules.iter().map(|r| expand(r)).collect();
        Regex::new(&format!("^(?:{})", patterns.join("|"))).unwrap()
    }

    static REF_GENERAL: Lazy<Regex> = Lazy::new(|| alternation(GENERAL_RULES));
    static REF_LOOKAHEAD: Lazy<Regex> = Lazy::new(|| alternation(LOOKAHEAD_RULES));

    fn reference(chars: &[char]) -> Vec<bool> {
        let general = &*REF_GENERAL;
        let lookahead = &*REF_LOOKAHEAD;
        let n = chars.len();
        let mut valid = vec![false; n + 1];
        valid[0] = true;
        valid[n] = true;
        let text: String = chars.iter().collect();
        let (mut byte_pos, mut char_pos) = (0, 0);
        while byte_pos < text.len() {
            let sub = &text[byte_pos..];
            if sub.starts_with('เ') {
                if let Some(m) = lookahead.find(sub) {
                    if is_followed_by_lookahead_char(sub[m.end()..].chars().next()) {
                        byte_pos += m.end();
                        char_pos += m.as_str().chars().count();
                        valid[char_pos] = true;
                        continue;
                    }
                }
            }
            let len = match general.find(sub) {
                Some(m) => {
                    let matched: Vec<char> = m.as_str().chars().collect();
                    let rest: Vec<char> = sub[m.end()..].chars().take(1).collect();
                    let mut all = matched.clone();
                    all.extend(rest);
                    let end = cluster_end(&all, 0, matched.len());
                    all[..end].iter().map(|c| c.len_utf8()).sum()
                }
                None => sub.chars().next().unwrap().len_utf8(),
            };
            char_pos += sub[..len].chars().count();
            byte_pos += len;
            valid[char_pos] = true;
        }
        for (i, &ch) in chars.iter().enumerate() {
            if !(0x0E00..=0x0E7F).contains(&(ch as u32)) {
                valid[i] = true;
                valid[i + 1] = true;
            }
        }
        for i in 1..n {
            if is_dependent_thai(chars[i]) {
                valid[i] = false;
            }
        }
        valid
    }

    fn check(text: &str) {
        let chars: Vec<char> = text.chars().collect();
        assert_eq!(tcc_pos_array(&chars), reference(&chars), "{text:?}");
    }

    #[test]
    fn matches_regex_on_examples() {
        for text in [
            "",
            "ก",
            "เครื่อง",
            "เพลิง",
            "เปล่า",
            "ผัวะ",
            "รึยัง",
            "เกียะ",
            "เสียงเรียก",
            "กรรม์",
            "แข็ง",
            "ก็",
            "น้ำ",
            "เเ",
            "ไทยabc๑๒๓",
            "ั่",
            "เ",
            "สวัสดีครับ ภาษาไทย",
        ] {
            check(text);
        }
    }

    #[test]
    fn matches_regex_on_random_strings() {
        // xorshift, so the test needs no extra dependency
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let pool: Vec<char> = "กขคงจชซญดตทนบปผพฟมยรลวศสหอฮะัาำิีึืฺุูเแโใไๅ็่้๊๋์ํ๎ๆฯ a1".chars().collect();
        for _ in 0..200_000 {
            let len = (next() % 24) as usize;
            let text: String = (0..len)
                .map(|_| {
                    let r = next();
                    if r % 7 == 0 {
                        char::from_u32(0x0E00 + (r >> 8) as u32 % 128).unwrap()
                    } else {
                        pool[(r >> 8) as usize % pool.len()]
                    }
                })
                .collect();
            check(&text);
        }
    }
}
