//! Display width of text, as doclayout measures it in a narrow context:
//! combining marks take no column, East Asian wide characters and emoji take
//! two, ambiguous characters one. Ported from doclayout 0.5.0.3
//! (`Text.DocLayout`, BSD-3-Clause, Copyright John MacFarlane).

use std::sync::OnceLock;

use super::width_tables::{EMOJI, RANGES};

/// A character's width class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Narrow,
    Wide,
    Combining,
    Control,
    Ambiguous,
    Zwj,
    EmojiPresentationMod,
    EmojiSkinToneMod,
}

/// What a character already measured leaves for the next one. `total` is
/// signed: a joiner after an emoji takes back the emoji's two columns, which
/// the next emoji then adds again, so it can dip below zero in between.
#[derive(Clone, Copy)]
struct MatchState {
    first: bool,
    total: i64,
    last: char,
    tentative: i64,
}

impl MatchState {
    /// The width measured so far; never negative, as the joiner's debt is
    /// at most the two columns it follows.
    fn extract(self) -> usize {
        usize::try_from(self.total + self.tentative).unwrap_or(0)
    }
}

/// The display width of `s`.
pub fn real_length(s: &str) -> usize {
    let mut st = MatchState {
        first: true,
        total: 0,
        last: ' ',
        tentative: 0,
    };
    for c in s.chars() {
        st = update_narrow(st, c);
    }
    st.extract()
}

/// The emoji info of a start character: (takes a variation selector,
/// takes a skin tone).
fn emoji_info(c: char) -> Option<(bool, bool)> {
    let code = u32::from(c);
    EMOJI
        .binary_search_by_key(&code, |e| e.0)
        .ok()
        .map(|i| (EMOJI[i].1, EMOJI[i].2))
}

const fn is_skin_tone(c: char) -> bool {
    c >= '\u{1F3FB}' && c <= '\u{1F3FF}'
}

#[allow(clippy::too_many_lines)] // One branch per Unicode block, as doclayout has it.
fn update_narrow(st: MatchState, c: char) -> MatchState {
    let total = st.total + st.tentative;
    let narrow = MatchState {
        first: false,
        total,
        last: c,
        tentative: 1,
    };
    let wide = MatchState {
        first: false,
        total,
        last: c,
        tentative: 2,
    };
    let combining = MatchState {
        first: false,
        total,
        last: c,
        tentative: i64::from(st.first),
    };
    let control = MatchState {
        first: false,
        total,
        last: c,
        tentative: 0,
    };
    let ambiguous = narrow;
    match c {
        '\0'..='\u{1F}' => control,
        '\u{20}'..='\u{7E}' => narrow,
        '\u{7F}'..='\u{9F}' => control,
        '\u{AD}' => control,
        '\u{A0}'..='\u{2FF}' => narrow,
        '\u{300}'..='\u{36F}' => combining,
        '\u{3250}'..='\u{A4CF}' => match c {
            '\u{3250}'..='\u{4DBF}' => wide,
            '\u{4DC0}'..='\u{4DFF}' => narrow,
            _ => wide,
        },
        '\u{600}'..='\u{6FF}' => match c {
            '\u{600}'..='\u{605}' => control,
            '\u{606}'..='\u{60F}' => narrow,
            '\u{610}'..='\u{61A}' => combining,
            '\u{61B}' => narrow,
            '\u{61C}' => control,
            '\u{61D}'..='\u{64A}' => narrow,
            '\u{64B}'..='\u{65F}' => combining,
            '\u{670}' => combining,
            '\u{660}'..='\u{6D5}' => narrow,
            '\u{6D6}'..='\u{6DC}' => combining,
            '\u{6DD}' => control,
            '\u{6DE}' => narrow,
            '\u{6DF}'..='\u{6E4}' => combining,
            '\u{6E5}'..='\u{6E6}' => narrow,
            '\u{6E9}' => narrow,
            '\u{6E7}'..='\u{6ED}' => combining,
            _ => narrow,
        },
        '\u{900}'..='\u{97F}' => match c {
            '\u{900}'..='\u{902}' => combining,
            '\u{903}'..='\u{939}' => narrow,
            '\u{93A}' | '\u{93C}' => combining,
            '\u{93B}' | '\u{93D}'..='\u{940}' => narrow,
            '\u{941}'..='\u{948}' => combining,
            '\u{94D}' => combining,
            '\u{949}'..='\u{950}' => narrow,
            '\u{951}'..='\u{957}' => combining,
            '\u{962}' | '\u{963}' => combining,
            _ => narrow,
        },
        '\u{980}'..='\u{A02}' => match c {
            '\u{981}' => combining,
            '\u{9BC}' => combining,
            '\u{980}'..='\u{9C0}' => narrow,
            '\u{9C1}'..='\u{9C4}' => combining,
            '\u{9CD}' => combining,
            '\u{9C5}'..='\u{9E1}' => narrow,
            '\u{9E2}'..='\u{9E3}' => combining,
            '\u{9E4}'..='\u{9FD}' => narrow,
            _ => combining,
        },
        '\u{370}'..='\u{58F}' => match c {
            '\u{370}'..='\u{482}' => narrow,
            '\u{483}'..='\u{489}' => combining,
            _ => narrow,
        },
        '\u{2E80}'..='\u{324F}' => match c {
            '\u{2E80}'..='\u{3029}' => wide,
            '\u{302A}'..='\u{302D}' => combining,
            '\u{303F}' => narrow,
            '\u{302E}'..='\u{3096}' => wide,
            '\u{3097}'..='\u{309A}' => combining,
            '\u{309B}'..='\u{3247}' => wide,
            _ => ambiguous,
        },
        '\u{AC00}'..='\u{D7A3}' => wide,
        '\u{C00}'..='\u{C80}' => match c {
            '\u{C00}' | '\u{C04}' => combining,
            '\u{C01}'..='\u{C03}' | '\u{C05}'..='\u{C39}' => narrow,
            '\u{C3D}' => narrow,
            '\u{C3A}'..='\u{C40}' => combining,
            '\u{C41}'..='\u{C44}' => narrow,
            '\u{C45}'..='\u{C56}' => combining,
            '\u{C62}' | '\u{C63}' => combining,
            _ => narrow,
        },
        '\u{B80}'..='\u{BFF}' => match c {
            '\u{B80}'..='\u{B82}' => combining,
            '\u{BC0}' => combining,
            '\u{BCD}' => combining,
            _ => narrow,
        },
        '\u{E00}'..='\u{E7F}' => match c {
            '\u{E31}' => combining,
            '\u{E34}'..='\u{E3A}' => combining,
            '\u{E47}'..='\u{E4E}' => combining,
            _ => narrow,
        },
        '\u{200D}' => {
            if emoji_info(st.last).is_some() || st.last == '\u{FE0F}' || is_skin_tone(st.last) {
                MatchState {
                    first: false,
                    total: st.total - 2,
                    last: c,
                    tentative: 2,
                }
            } else {
                control
            }
        }
        '\u{FE0F}' => match emoji_info(st.last) {
            Some((true, _)) => MatchState {
                first: false,
                total: st.total,
                last: c,
                tentative: 2,
            },
            _ => control,
        },
        '\u{F900}'..='\u{FAFF}' | '\u{FF01}'..='\u{FF60}' | '\u{FFE0}'..='\u{FFE6}' => wide,
        '\u{20000}'..='\u{3FFFF}' => wide,
        '\u{1F3FB}'..='\u{1F3FF}' => match emoji_info(st.last) {
            Some((_, true)) => MatchState {
                first: false,
                total: st.total,
                last: c,
                tentative: 2,
            },
            _ => wide,
        },
        _ => resolve(st, c, class_of(c)),
    }
}

/// doclayout's `resolveWidth`: the state after a character of class `cls`.
fn resolve(st: MatchState, c: char, cls: Class) -> MatchState {
    let total = st.total + st.tentative;
    let with = |tentative| MatchState {
        first: false,
        total,
        last: c,
        tentative,
    };
    let emoji_like =
        emoji_info(st.last).is_some() || st.last == '\u{FE0F}' || is_skin_tone(st.last);
    match cls {
        Class::Narrow | Class::Ambiguous => with(1),
        Class::Wide => with(2),
        Class::Combining => with(i64::from(st.first)),
        Class::Control => with(0),
        Class::Zwj if emoji_like => MatchState {
            first: false,
            total: st.total - 2,
            last: c,
            tentative: 2,
        },
        Class::Zwj => with(0),
        Class::EmojiPresentationMod => match emoji_info(st.last) {
            Some((true, _)) => MatchState {
                first: false,
                total: st.total,
                last: c,
                tentative: 2,
            },
            _ => with(0),
        },
        Class::EmojiSkinToneMod => match emoji_info(st.last) {
            Some((_, true)) => MatchState {
                first: false,
                total: st.total,
                last: c,
                tentative: 2,
            },
            _ => with(2),
        },
    }
}

/// The class of `c` in doclayout's narrow-context range map: the generated
/// ranges with ambiguous characters narrow, equal neighbours merged, and
/// the joiner and emoji modifiers inserted, each followed by the class that
/// was in force before it.
fn class_of(c: char) -> Class {
    static MAP: OnceLock<Vec<(u32, Class)>> = OnceLock::new();
    let map = MAP.get_or_init(|| {
        let mut merged: Vec<(u32, Class)> = Vec::new();
        for &(start, cls) in RANGES {
            let cls = if cls == Class::Ambiguous {
                Class::Narrow
            } else {
                cls
            };
            if merged.last().is_none_or(|&(_, last)| last != cls) {
                merged.push((start, cls));
            }
        }
        let mut map: std::collections::BTreeMap<u32, Class> = merged.into_iter().collect();
        for (k1, k2, cls) in [
            (0x1F3FB, 0x1F3FF, Class::EmojiSkinToneMod),
            (0xFE0F, 0xFE0F, Class::EmojiPresentationMod),
            (0x200D, 0x200D, Class::Zwj),
        ] {
            let before = map.range(..=k1).next_back().map(|(_, &v)| v);
            map.insert(k1, cls);
            if let Some(prev) = before {
                map.entry(k2 + 1).or_insert(prev);
            }
        }
        map.into_iter().collect()
    });
    let code = u32::from(c);
    match map.binary_search_by_key(&code, |e| e.0) {
        Ok(i) => map[i].1,
        Err(0) => Class::Control,
        Err(i) => map[i - 1].1,
    }
}

#[cfg(test)]
mod tests {
    use super::real_length;

    #[test]
    fn widths() {
        assert_eq!(real_length("abc"), 3);
        assert_eq!(real_length("©—→é"), 4);
        assert_eq!(real_length("中文"), 4);
        assert_eq!(real_length("e\u{301}"), 1);
        assert_eq!(real_length("\t"), 0);
        assert_eq!(real_length("👍🏽"), 2);
        assert_eq!(real_length("שלום"), 4);
    }
}
