//! Fuzzy search over Hide My Email addresses (replaces Fuse.js).
//!
//! Every whitespace-separated token of the query must match one of the
//! fields (label, address, note). A contiguous substring match beats a
//! scattered subsequence match, an earlier position beats a later one, and
//! the label weighs more than the address, which weighs more than the note.
//! Equal scores keep the input order (newest first).

use crate::model::HmeEmail;

const LABEL_WEIGHT: i64 = 3;
const ADDRESS_WEIGHT: i64 = 2;
const NOTE_WEIGHT: i64 = 1;

fn lower_chars(s: &str) -> Vec<char> {
    s.chars().flat_map(char::to_lowercase).collect()
}

/// Score of `needle` against `hay`, or `None` when it doesn't match.
fn score_field(hay: &[char], needle: &[char]) -> Option<i64> {
    if needle.is_empty() {
        return Some(0);
    }
    if needle.len() > hay.len() {
        return None;
    }
    // Contiguous substring.
    if let Some(pos) = hay.windows(needle.len()).position(|w| w == needle) {
        let at_word_start = pos == 0 || !hay[pos - 1].is_alphanumeric();
        return Some(1000 - pos as i64 + if at_word_start { 200 } else { 0 });
    }
    // Subsequence: every needle char in order, penalising the gaps.
    let mut gaps = 0i64;
    let mut it = hay.iter().enumerate();
    let mut last: Option<usize> = None;
    for c in needle {
        let (i, _) = it.find(|(_, h)| *h == c)?;
        if let Some(l) = last {
            gaps += (i - l - 1) as i64;
        }
        last = Some(i);
    }
    // Too scattered to be meaningful.
    if gaps > (needle.len() as i64) * 4 {
        return None;
    }
    Some(300 - gaps * 10)
}

fn score_item(item: &HmeEmail, tokens: &[Vec<char>]) -> Option<i64> {
    let fields = [
        (lower_chars(&item.label), LABEL_WEIGHT),
        (lower_chars(&item.hme), ADDRESS_WEIGHT),
        (lower_chars(&item.note), NOTE_WEIGHT),
    ];
    let mut total = 0;
    for token in tokens {
        total += fields
            .iter()
            .filter_map(|(hay, w)| score_field(hay, token).map(|s| s * w))
            .max()?;
    }
    Some(total)
}

/// Indices into `items` of the matches, best first. An empty query returns
/// every index in the original order.
pub fn search(items: &[HmeEmail], query: &str) -> Vec<usize> {
    let tokens: Vec<Vec<char>> = query.split_whitespace().map(lower_chars).collect();
    if tokens.is_empty() {
        return (0..items.len()).collect();
    }
    let mut scored: Vec<(usize, i64)> = items
        .iter()
        .enumerate()
        .filter_map(|(i, item)| score_item(item, &tokens).map(|s| (i, s)))
        .collect();
    // Stable sort keeps newest-first among equal scores.
    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored.into_iter().map(|(i, _)| i).collect()
}

/// Splits `text` into `(segment, is_match)` runs, marking the contiguous,
/// case-insensitive occurrences of the query's tokens (for `<mark>`).
pub fn highlight(text: &str, query: &str) -> Vec<(String, bool)> {
    let chars: Vec<char> = text.chars().collect();
    let lower: Vec<char> = chars
        .iter()
        .map(|c| c.to_lowercase().next().unwrap_or(*c))
        .collect();
    let mut marked = vec![false; chars.len()];
    for token in query.split_whitespace() {
        let t: Vec<char> = token
            .chars()
            .map(|c| c.to_lowercase().next().unwrap_or(c))
            .collect();
        if t.is_empty() || t.len() > lower.len() {
            continue;
        }
        if let Some(pos) = lower.windows(t.len()).position(|w| w == t.as_slice()) {
            marked[pos..pos + t.len()].iter_mut().for_each(|m| *m = true);
        }
    }
    let mut out: Vec<(String, bool)> = Vec::new();
    for (c, m) in chars.into_iter().zip(marked) {
        match out.last_mut() {
            Some((seg, last)) if *last == m => seg.push(c),
            _ => out.push((c.to_string(), m)),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(label: &str, hme: &str, note: &str) -> HmeEmail {
        HmeEmail {
            anonymous_id: hme.into(),
            hme: hme.into(),
            origin: String::new(),
            domain: String::new(),
            forward_to_email: String::new(),
            is_active: true,
            label: label.into(),
            note: note.into(),
            create_timestamp: 0.0,
            recipient_mail_id: String::new(),
        }
    }

    fn items() -> Vec<HmeEmail> {
        vec![
            item("GitHub", "quiet_plover@icloud.com", ""),
            item("Netflix", "red_fox@icloud.com", "family plan"),
            item("gitlab.com", "zany.ghost@icloud.com", ""),
            item("Amazon", "github_alt@icloud.com", "for git stuff"),
        ]
    }

    #[test]
    fn empty_query_keeps_everything_in_order() {
        assert_eq!(search(&items(), "   "), vec![0, 1, 2, 3]);
    }

    #[test]
    fn label_beats_address_beats_note() {
        let r = search(&items(), "git");
        assert_eq!(r[0], 0, "label prefix 'GitHub' first");
        assert!(r.contains(&2) && r.contains(&3));
        assert!(!r.contains(&1));
    }

    #[test]
    fn case_insensitive_and_fuzzy() {
        assert_eq!(search(&items(), "NETFLIX"), vec![1]);
        assert_eq!(search(&items(), "ntflx"), vec![1]);
        assert_eq!(search(&items(), "family"), vec![1]);
    }

    #[test]
    fn all_tokens_must_match() {
        assert_eq!(search(&items(), "git ghost"), vec![2]);
        assert!(search(&items(), "netflix ghost").is_empty());
    }

    #[test]
    fn highlights_matches() {
        assert_eq!(
            highlight("DigitalOcean", "git"),
            vec![("Di".to_string(), false), ("git".to_string(), true), ("alOcean".to_string(), false)]
        );
        assert_eq!(highlight("GitHub", ""), vec![("GitHub".to_string(), false)]);
        assert_eq!(highlight("", "x"), Vec::<(String, bool)>::new());
        assert_eq!(highlight("Хабр", "ХА")[0], ("Ха".to_string(), true));
    }

    #[test]
    fn unicode() {
        let v = vec![item("Госуслуги", "a@icloud.com", "")];
        assert_eq!(search(&v, "госу"), vec![0]);
        assert_eq!(search(&v, "ГОСУСЛУГИ"), vec![0]);
    }
}
