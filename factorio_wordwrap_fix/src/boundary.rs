//! Unicode line boundaries over visible text, with Factorio rich references as objects.
use unicode_linebreak::linebreaks;
use unicode_segmentation::UnicodeSegmentation;

fn no_start(c: char) -> bool {
    "，。！？：；、）］｝〕〉》」』】〗〙〛’”％‰℃°,.!?;:%)]}".contains(c)
}
fn no_end(c: char) -> bool {
    "（［｛〔〈《「『【〖〘〚‘“([{".contains(c)
}
fn format_tag(tag: &str) -> bool {
    ["color=", "font=", "heading=", "/color", "/font", "/heading"]
        .iter()
        .any(|p| tag.starts_with(p))
}

/// `fit` is a byte count measured by the game's actual font/rich-text renderer.
/// Return a source UTF-8 boundary, keeping markup and references intact.
pub fn boundary(text: &str, fit: usize, rich: bool) -> usize {
    if fit >= text.len() {
        return text.len();
    }
    let mut visible = String::with_capacity(text.len());
    // (visible boundary, source boundary); zero-width closing styles stay left.
    let mut map = vec![(0usize, 0usize)];
    let mut atoms = Vec::new();
    let mut i = 0;
    while i < text.len() {
        if rich
            && text.as_bytes()[i] == b'['
            && let Some(end) = text[i..].find(']')
        {
            let end = i + end + 1;
            let tag = &text[i + 1..end - 1];
            if format_tag(tag) {
                if tag.starts_with('/')
                    && let Some(last) = map.last_mut()
                {
                    last.1 = end;
                }
                i = end;
                continue;
            }
            if tag.contains('=') || tag == "." {
                atoms.push(visible.len());
                visible.push('\u{fffc}');
                atoms.push(visible.len());
                map.push((visible.len(), end));
                i = end;
                continue;
            }
        }
        let c = text[i..].chars().next().unwrap();
        visible.push(c);
        i += c.len_utf8();
        map.push((visible.len(), i));
    }
    if visible.is_empty() {
        return text.len();
    }
    let mut breaks: Vec<usize> = linebreaks(&visible).map(|(n, _)| n).collect();
    // A measured prefix may end just before a space. The space itself need
    // not fit: native code already trims/skips ASCII whitespace between lines.
    breaks.extend(
        visible
            .char_indices()
            .filter(|(_, c)| c.is_ascii_whitespace())
            .map(|(n, _)| n),
    );
    breaks.extend(atoms);
    breaks.sort_unstable();
    breaks.dedup();
    let mut best = 0;
    for n in breaks {
        if n == 0 {
            continue;
        }
        let left = visible[..n].chars().next_back();
        let right = visible[n..].chars().next();
        if left.is_some_and(no_end) || right.is_some_and(no_start) {
            continue;
        }
        if let Ok(k) = map.binary_search_by_key(&n, |&(v, _)| v) {
            let source = map[k].1;
            if source <= fit {
                best = source;
            }
        }
    }
    if best > 0 {
        return best;
    }
    // Oversized first word: preserve grapheme clusters. A rich reference is one
    // object, so an object wider than the control intentionally occupies a line.
    let mut fallback = 0;
    for (n, g) in visible.grapheme_indices(true) {
        let end = n + g.len();
        if visible[..end].chars().next_back().is_some_and(no_end)
            || visible[end..].chars().next().is_some_and(no_start)
        {
            continue;
        }
        if let Ok(k) = map.binary_search_by_key(&end, |&(v, _)| v) {
            let source = map[k].1;
            if source <= fit {
                fallback = source;
            } else if fallback == 0 {
                return source;
            } else {
                break;
            }
        }
    }
    if fallback == 0 { text.len() } else { fallback }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mixed_words() {
        let s = "中文Factorio自动化";
        assert_eq!(&s[..boundary(s, 12, false)], "中文");
    }
    #[test]
    fn chinese_punctuation() {
        let s = "甲乙，丙丁";
        assert_eq!(&s[..boundary(s, 6, false)], "甲");
    }
    #[test]
    fn opening_punctuation() {
        let s = "甲（乙丙";
        assert_eq!(&s[..boundary(s, 6, false)], "甲");
    }
    #[test]
    fn references() {
        let s = "甲[entity=fusion-reactor]乙";
        let end = s.find(']').unwrap() + 1;
        for n in 4..end {
            assert_eq!(boundary(s, n, true), 3);
        }
        assert_eq!(boundary(s, end, true), end);
    }
    #[test]
    fn ascii_adjacent_icon() {
        let s = "abc[item=iron-plate]xyz";
        assert_eq!(boundary(s, 12, true), 3);
        assert_eq!(boundary(s, 20, true), 20);
    }
    #[test]
    fn oversized_icon() {
        let s = "[entity=fusion-reactor]甲";
        assert_eq!(boundary(s, 3, true), 23);
    }
    #[test]
    fn styles() {
        let s = "[color=red]甲乙[/color]丙";
        let n = boundary(s, s.len() - 2, true);
        assert_eq!(&s[..n], "[color=red]甲乙[/color]");
    }
    #[test]
    fn combining_and_emoji() {
        for s in ["e\u{301}xxxx", "👨‍👩‍👧‍👦xx"] {
            let n = boundary(s, 1, false);
            assert_eq!(s[..n].graphemes(true).count(), 1);
        }
    }
    #[test]
    fn every_byte_is_safe() {
        for s in [
            "中文English中文。",
            "[item=iron-plate]甲乙",
            "a\u{301}👩‍🔬中文",
            "（【",
            "[color=red][/color]",
            "甲，。！？",
        ] {
            for fit in 0..s.len() {
                let n = boundary(s, fit, true);
                assert!(n > 0 && s.is_char_boundary(n));
            }
        }
    }
    #[test]
    fn narrow_control_keeps_punctuation_with_glyph() {
        assert_eq!(boundary("甲，乙", 3, false), "甲，".len());
        assert_eq!(boundary("（甲乙", 3, false), "（甲".len());
    }
    #[test]
    fn latin_is_not_cjk() {
        assert_eq!(boundary("中文café中文", 10, false), 6);
        assert_eq!(boundary("中文hello world", 11, false), 11);
    }
    #[test]
    fn empty_and_unlimited() {
        assert_eq!(boundary("", 0, true), 0);
        let style = "[color=red][/color]";
        assert_eq!(boundary(style, 1, true), style.len());
        let text = "中文\n\nEnglish";
        assert_eq!(boundary(text, usize::MAX, true), text.len());
    }
}
