//! Built-in Vietnamese Telex typing for the Android builds (ARTCRAFT-TOAN).
//!
//! Android hands the app a hardware keyboard's keys before any input method sees them (the native
//! activity consumes them in the pre-IME stage), and the app's hidden input view does not give
//! keyboards such as Laban Key a real editor to run their own Telex on. So the Vietnamese
//! composition happens here instead, the way UniKey does it on a desktop: every word's raw
//! keystrokes are kept, the word is recomposed after each key, and the app receives the difference
//! as Backspaces plus the new text.
//!
//! Rules: Telex (`aa â`, `aw ă`, `ee ê`, `oo ô`, `ow ơ`, `uw ư`, `w ư`, `dd đ`; tones `s f r x j`,
//! `z` clears the tone), pressing a key twice undoes it (`ass` → `as`), and a word that does not
//! spell a Vietnamese syllable is left exactly as typed, so most English and every formula stays
//! untouched. A word that follows `=`, `(`, digits or other formula characters is never composed.
//! Tones go on the classic position (`hòa`, `thủy`).
//!
//! Pure Rust with no platform code, so the rules are unit-tested on any host.

/// What the app must do to its text after one key: delete `backspaces` characters before the
/// caret, then insert `insert`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Edit {
    pub backspaces: usize,
    pub insert: String,
}

impl Edit {
    fn plain(c: char) -> Self {
        Self { backspaces: 0, insert: c.to_string() }
    }

    /// Fold `next` into `self` as if both were applied one after the other.
    fn then(mut self, next: Edit) -> Self {
        let mut remove = next.backspaces;
        while remove > 0 && !self.insert.is_empty() {
            self.insert.pop();
            remove -= 1;
        }
        self.backspaces += remove;
        self.insert.push_str(&next.insert);
        self
    }
}

/// The Telex state for the word being typed.
#[derive(Debug, Clone)]
pub struct Telex {
    enabled: bool,
    /// Keys of the current word as typed (or the composed letters left after a Backspace).
    raw: Vec<char>,
    /// What the app currently shows for the current word.
    out: Vec<char>,
    /// The word follows a formula/code character: pass keys through untouched.
    code_word: bool,
}

impl Default for Telex {
    fn default() -> Self {
        Self::new()
    }
}

/// Longest words we compose; anything longer is not a syllable and passes through.
const MAX_WORD: usize = 16;

impl Telex {
    pub fn new() -> Self {
        Self { enabled: true, raw: Vec::new(), out: Vec::new(), code_word: false }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.reset();
    }

    /// Forget the current word (the caret moved, a click, Enter, focus change…).
    pub fn reset(&mut self) {
        self.raw.clear();
        self.out.clear();
        self.code_word = false;
    }

    /// The app deleted one character before the caret.
    pub fn backspace(&mut self) {
        if self.out.pop().is_some() {
            self.raw = self.out.clone();
        } else {
            self.reset();
        }
    }

    /// Feed a run of text (one key's text, or what a soft keyboard committed).
    pub fn feed_str(&mut self, text: &str) -> Edit {
        text.chars().fold(Edit::default(), |acc, c| acc.then(self.feed(c)))
    }

    /// Feed one typed character.
    pub fn feed(&mut self, c: char) -> Edit {
        if !self.enabled {
            return Edit::plain(c);
        }
        if !is_word_char(c) {
            let code = !(c.is_whitespace() || "\"'“”‘’«»-–—/.,;:?!…)]}".contains(c));
            self.reset();
            self.code_word = code;
            return Edit::plain(c);
        }
        if self.code_word || self.raw.len() >= MAX_WORD {
            self.raw.push(c);
            self.out.push(c);
            return Edit::plain(c);
        }
        self.raw.push(c);
        let new: Vec<char> = compose(&self.raw).chars().collect();
        let common = self.out.iter().zip(new.iter()).take_while(|(a, b)| a == b).count();
        let edit = Edit {
            backspaces: self.out.len() - common,
            insert: new[common..].iter().collect(),
        };
        self.out = new;
        edit
    }
}

// ---------------------------------------------------------------------------------------------
// Letters

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mark {
    None,
    /// â ê ô
    Circ,
    /// ă
    Breve,
    /// ơ ư
    Horn,
}

#[derive(Debug, Clone, Copy)]
struct Letter {
    /// Lowercase base: an ASCII letter or `đ`.
    base: char,
    mark: Mark,
    upper: bool,
}

impl Letter {
    fn is_vowel(&self) -> bool {
        matches!(self.base, 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
    }

    /// The lowercase letter with its mark (no tone).
    fn plain(&self) -> char {
        match (self.base, self.mark) {
            ('a', Mark::Breve) => 'ă',
            ('a', Mark::Circ) => 'â',
            ('e', Mark::Circ) => 'ê',
            ('o', Mark::Circ) => 'ô',
            ('o', Mark::Horn) => 'ơ',
            ('u', Mark::Horn) => 'ư',
            (b, _) => b,
        }
    }
}

/// Tone forms of every vowel, in tone order: none, sắc, huyền, hỏi, ngã, nặng.
const TONES: [(char, [char; 6]); 12] = [
    ('a', ['a', 'á', 'à', 'ả', 'ã', 'ạ']),
    ('ă', ['ă', 'ắ', 'ằ', 'ẳ', 'ẵ', 'ặ']),
    ('â', ['â', 'ấ', 'ầ', 'ẩ', 'ẫ', 'ậ']),
    ('e', ['e', 'é', 'è', 'ẻ', 'ẽ', 'ẹ']),
    ('ê', ['ê', 'ế', 'ề', 'ể', 'ễ', 'ệ']),
    ('i', ['i', 'í', 'ì', 'ỉ', 'ĩ', 'ị']),
    ('o', ['o', 'ó', 'ò', 'ỏ', 'õ', 'ọ']),
    ('ô', ['ô', 'ố', 'ồ', 'ổ', 'ỗ', 'ộ']),
    ('ơ', ['ơ', 'ớ', 'ờ', 'ở', 'ỡ', 'ợ']),
    ('u', ['u', 'ú', 'ù', 'ủ', 'ũ', 'ụ']),
    ('ư', ['ư', 'ứ', 'ừ', 'ử', 'ữ', 'ự']),
    ('y', ['y', 'ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ']),
];

/// Split a (lowercase) Vietnamese letter into base, mark and tone.
fn decompose(c: char) -> Option<(char, Mark, usize)> {
    if c == 'đ' {
        return Some(('đ', Mark::None, 0));
    }
    for (plain, forms) in TONES {
        if let Some(tone) = forms.iter().position(|f| *f == c) {
            let (base, mark) = match plain {
                'ă' => ('a', Mark::Breve),
                'â' => ('a', Mark::Circ),
                'ê' => ('e', Mark::Circ),
                'ô' => ('o', Mark::Circ),
                'ơ' => ('o', Mark::Horn),
                'ư' => ('u', Mark::Horn),
                b => (b, Mark::None),
            };
            return Some((base, mark, tone));
        }
    }
    None
}

fn lower(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphabetic() || decompose(lower(c)).is_some()
}

fn render(l: &Letter, tone: usize) -> char {
    let plain = l.plain();
    let toned = TONES
        .iter()
        .find(|(p, _)| *p == plain)
        .and_then(|(_, forms)| forms.get(tone).copied())
        .unwrap_or(plain);
    if l.upper { toned.to_uppercase().next().unwrap_or(toned) } else { toned }
}

// ---------------------------------------------------------------------------------------------
// Composition

fn tone_of(key: char) -> Option<usize> {
    match key {
        's' => Some(1),
        'f' => Some(2),
        'r' => Some(3),
        'x' => Some(4),
        'j' => Some(5),
        _ => None,
    }
}

/// Indices of the last run of vowels in `letters`.
fn last_vowel_group(letters: &[Letter]) -> Option<(usize, usize)> {
    let end = letters.iter().rposition(Letter::is_vowel)?;
    let mut start = end;
    while start > 0 && letters[start - 1].is_vowel() {
        start -= 1;
    }
    Some((start, end + 1))
}

/// A partial word is acceptable while it has no vowel yet, or when it spells a syllable.
fn acceptable(letters: &[Letter], tone: usize) -> bool {
    !letters.iter().any(Letter::is_vowel) || place_tone(letters, tone).is_some()
}

fn render_plain(letters: &[Letter]) -> String {
    letters.iter().map(|l| render(l, 0)).collect()
}

/// Compose a word's raw keys into Vietnamese, or return them unchanged when the result would not
/// be a Vietnamese syllable. Every Telex rule only fires when the word still spells a syllable
/// after it, so English such as `class` keeps its letters.
fn compose(raw: &[char]) -> String {
    let typed: String = raw.iter().collect();
    let mut letters: Vec<Letter> = Vec::new();
    let mut tone = 0usize;
    let mut changed = false;
    let mut undone = false;
    let mut tone_locked = false;
    let mut mark_locked = false;

    for &k in raw {
        let lk = lower(k);
        let upper = k != lk;
        let literal = Letter { base: lk, mark: Mark::None, upper };

        // Composed letters (left behind by a Backspace, or sent by a keyboard) keep their marks.
        if !lk.is_ascii() {
            match decompose(lk) {
                Some((base, mark, t)) => {
                    letters.push(Letter { base, mark, upper });
                    if t != 0 {
                        tone = t;
                    }
                }
                None => letters.push(literal),
            }
            continue;
        }

        let has_vowel = letters.iter().any(Letter::is_vowel);

        // Tones.
        if let Some(t) = tone_of(lk).filter(|_| has_vowel && !tone_locked) {
            if tone == t {
                tone = 0;
                tone_locked = true;
                undone = true;
                letters.push(literal);
                continue;
            }
            if acceptable(&letters, t) {
                tone = t;
                changed = true;
                continue;
            }
            letters.push(literal);
            continue;
        }
        if lk == 'z' && tone != 0 && !tone_locked {
            tone = 0;
            changed = true;
            continue;
        }

        // Marks: try the rule on a copy and keep it only if the word still spells a syllable.
        if !mark_locked {
            if let Some((next, undo)) = apply_mark(&letters, lk, upper) {
                if undo {
                    letters = next;
                    letters.push(literal);
                    mark_locked = true;
                    undone = true;
                    continue;
                }
                if acceptable(&next, tone) {
                    letters = next;
                    changed = true;
                    continue;
                }
            }
        }

        letters.push(literal);
    }

    if !changed && !undone && tone == 0 {
        return typed;
    }
    if !letters.iter().any(Letter::is_vowel) {
        return render_plain(&letters);
    }
    match place_tone(&letters, tone) {
        Some(word) => word,
        None if undone && tone == 0 => render_plain(&letters),
        None => typed,
    }
}

/// The letters after a mark key (`w`, `a`/`e`/`o`, `d`), and whether the key undid that mark
/// instead. `None` when the key marks nothing and is just a letter.
fn apply_mark(letters: &[Letter], lk: char, upper: bool) -> Option<(Vec<Letter>, bool)> {
    let mut next = letters.to_vec();
    match lk {
        'w' => {
            let Some((s, e)) = last_vowel_group(&next) else {
                // A w with no vowel before it is ư (tw → tư).
                next.push(Letter { base: 'u', mark: Mark::Horn, upper });
                return Some((next, false));
            };
            // "uo" + w → ươ
            if let Some(i) = (s..e.saturating_sub(1))
                .find(|&i| next[i].base == 'u' && next[i + 1].base == 'o')
            {
                let undo = next[i].mark == Mark::Horn && next[i + 1].mark == Mark::Horn;
                let m = if undo { Mark::None } else { Mark::Horn };
                next[i].mark = m;
                next[i + 1].mark = m;
                return Some((next, undo));
            }
            let i = (s..e).rev().find(|&i| matches!(next[i].base, 'a' | 'o' | 'u'))?;
            let want = if next[i].base == 'a' { Mark::Breve } else { Mark::Horn };
            let undo = next[i].mark == want;
            next[i].mark = if undo { Mark::None } else { want };
            Some((next, undo))
        }
        'a' | 'e' | 'o' => {
            let (s, e) = last_vowel_group(&next)?;
            let i = (s..e).rev().find(|&i| next[i].base == lk)?;
            let undo = next[i].mark == Mark::Circ;
            next[i].mark = if undo { Mark::None } else { Mark::Circ };
            Some((next, undo))
        }
        'd' => {
            let first = next.first_mut()?;
            if first.base == 'd' {
                first.base = 'đ';
                return Some((next, false));
            }
            if first.base == 'đ' && letters.len() == 1 {
                first.base = 'd';
                return Some((next, true));
            }
            None
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------------------------
// Spelling and tone position

const INITIALS: [&str; 28] = [
    "ngh", "ng", "nh", "gh", "gi", "kh", "ph", "th", "tr", "ch", "qu", "b", "c", "d", "đ", "g",
    "h", "k", "l", "m", "n", "p", "r", "s", "t", "v", "x", "",
];

const FINALS: [&str; 9] = ["ng", "nh", "ch", "c", "m", "n", "p", "t", ""];

const CLUSTERS: [&str; 61] = [
    "a", "ă", "â", "e", "ê", "i", "o", "ô", "ơ", "u", "ư", "y", "ai", "ao", "au", "ay", "âu",
    "ây", "eo", "êu", "ia", "iê", "iu", "oa", "oă", "oe", "oi", "ôi", "ơi", "oo", "ua", "uâ",
    "uê", "ui", "uô", "uơ", "uy", "ưa", "ưi", "ươ", "ưu", "yê", "iêu", "oai", "oay", "oeo",
    "uây", "uôi", "uya", "uyê", "uyu", "ươi", "ươu",
    // Still being typed: the tone can come before the â/ê/ô/ơ that finishes the vowel.
    "ie", "uo", "ye", "uye", "ieu", "uoi", "uou", "ưo",
];

/// Check the syllable and render it with the tone on its right vowel.
fn place_tone(letters: &[Letter], tone: usize) -> Option<String> {
    // ươ needs a letter after it (ương, người, rượu); at the end of a word it is uơ (thuở, huơ).
    let mut letters = letters.to_vec();
    for i in 0..letters.len().saturating_sub(1) {
        if letters[i].base == 'u' && letters[i + 1].base == 'o' && letters[i + 1].mark == Mark::Horn {
            letters[i].mark = if i + 2 < letters.len() { Mark::Horn } else { Mark::None };
        }
    }
    let letters = letters.as_slice();
    let plain: Vec<char> = letters.iter().map(Letter::plain).collect();
    let text: String = plain.iter().collect();

    for initial in INITIALS {
        let Some(rest) = text.strip_prefix(initial) else { continue };
        let ini_len = initial.chars().count();
        for fin in FINALS {
            let Some(cluster) = rest.strip_suffix(fin) else { continue };
            if !CLUSTERS.contains(&cluster) && !(cluster == "yêu" && initial.is_empty()) {
                continue;
            }
            // Stop finals only take sắc or nặng.
            if matches!(fin, "c" | "ch" | "p" | "t") && !matches!(tone, 0 | 1 | 5) {
                return None;
            }
            let n = cluster.chars().count();
            let marked = (0..n).rev().find(|&i| {
                letters.get(ini_len + i).is_some_and(|l| l.mark != Mark::None && l.is_vowel())
            });
            let pos = match (marked, n) {
                (Some(i), _) => i,
                (None, 3) => 1,
                (None, 2) if !fin.is_empty() => 1,
                _ => 0,
            };
            let at = ini_len + pos;
            return Some(
                letters
                    .iter()
                    .enumerate()
                    .map(|(i, l)| if i == at { render(l, tone) } else { render(l, 0) })
                    .collect(),
            );
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Type `keys` and return what the app ends up showing.
    fn typed(keys: &str) -> String {
        let mut t = Telex::new();
        let mut shown = String::new();
        for c in keys.chars() {
            let e = t.feed(c);
            for _ in 0..e.backspaces {
                shown.pop();
            }
            shown.push_str(&e.insert);
        }
        shown
    }

    #[test]
    fn telex_words() {
        assert_eq!(typed("tieengs vieetj"), "tiếng việt");
        assert_eq!(typed("Coongj hoaf xax hooij chur nghiax Vieetj Nam"), "Cộng hòa xã hội chủ nghĩa Việt Nam");
        assert_eq!(typed("DDoocj laapj - Tuwj do - Hanhj phucs"), "Độc lập - Tự do - Hạnh phúc");
        assert_eq!(typed("dduwowngf"), "đường");
        assert_eq!(typed("nguyeenx"), "nguyễn");
        assert_eq!(typed("thuyr"), "thủy");
        assert_eq!(typed("quys"), "quý");
        assert_eq!(typed("gias"), "giá");
        assert_eq!(typed("gif"), "gì");
        assert_eq!(typed("khoong"), "không");
        assert_eq!(typed("tw"), "tư");
        assert_eq!(typed("hoaf"), "hòa");
        assert_eq!(typed("toans"), "toán");
        assert_eq!(typed("TIEENGS"), "TIẾNG");
        assert_eq!(typed("tiene"), "tiên");
        assert_eq!(typed("ddaf Nawngx"), "đà Nẵng");
    }

    #[test]
    fn more_words() {
        let cases = [
            ("tiesen", "tiến"), ("tieesng", "tiếng"), ("nguowif", "người"), ("muaf", "mùa"), ("Truwowngf", "Trường"),
            ("thangs", "tháng"), ("nawm", "năm"), ("ngayf", "ngày"), ("Soos", "Số"),
            ("quyeets ddinhj", "quyết định"), ("Nowi nhaanj", "Nơi nhận"), ("vawn banr", "văn bản"),
            ("uyr ban", "ủy ban"), ("khuyeen", "khuyên"), ("giuwax", "giữa"), ("thuowr", "thuở"),
            ("hieeur", "hiểu"), ("ruouwj", "rượu"), ("yeeu", "yêu"), ("ddooij", "đội"),
            ("Kieemr soats vieen khoong luwu", "Kiểm soát viên không lưu"),
        ];
        for (keys, want) in cases {
            assert_eq!(typed(keys), want, "{keys}");
        }
        for word in ["class", "world", "this is a test", "window", "email", "office", "Excel", "Word", "file"] {
            println!("{word} -> {}", typed(word));
        }
    }

    #[test]
    fn double_key_undoes() {
        assert_eq!(typed("ass"), "as");
        assert_eq!(typed("aaa"), "aa");
        assert_eq!(typed("ddd"), "dd");
        assert_eq!(typed("asz"), "a");
    }

    #[test]
    fn english_and_formulas_stay_as_typed() {
        assert_eq!(typed("class"), "class");
        assert_eq!(typed("text"), "text");
        assert_eq!(typed("=SUMIFS(A1:B2)"), "=SUMIFS(A1:B2)");
        assert_eq!(typed("=AS1"), "=AS1");
        assert_eq!(typed("hello world"), "hello world");
        assert_eq!(typed("week"), "week");
    }

    #[test]
    fn backspace_then_continue() {
        let mut t = Telex::new();
        for c in "tieeng".chars() {
            t.feed(c);
        }
        t.backspace(); // app removed the g: "tiên"
        t.backspace(); // "tiê"
        let e = t.feed_str("ns");
        // "tiê" + n + s → "tiến": the ê gets the tone, so the app rewrites from there.
        let mut shown = String::from("tiê");
        for _ in 0..e.backspaces {
            shown.pop();
        }
        shown.push_str(&e.insert);
        assert_eq!(shown, "tiến");
    }

    #[test]
    fn disabled_passes_through() {
        let mut t = Telex::new();
        t.set_enabled(false);
        assert_eq!(t.feed('a').insert, "a");
        assert_eq!(t.feed('s').insert, "s");
    }

    #[test]
    fn hostile_input_never_panics() {
        let mut t = Telex::new();
        for s in ["", "wwwwwwwwwwwwwwwwwwwwww", "zzzz", "ĐĐĐ", "𝔘𝔫𝔦", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "\u{0301}s"] {
            t.feed_str(s);
            t.backspace();
            t.backspace();
        }
    }
}
