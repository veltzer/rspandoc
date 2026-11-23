//! Display width of text, as doclayout measures it in a narrow context:
//! combining marks take no column, East Asian wide characters and emoji take
//! two, ambiguous characters one. Ported from doclayout 0.5.0.3
//! (`Text.DocLayout`, BSD-3-Clause, Copyright John `MacFarlane`).

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

/// The width class of a character in one of the Unicode blocks that doclayout
/// special-cases, or `None` when the generic tables decide.
fn block_class(c: char) -> Option<Class> {
    Some(match c {
        '\0'..='\u{1F}' | '\u{7F}'..='\u{9F}' | '\u{AD}' => Class::Control,
        '\u{20}'..='\u{7E}' | '\u{A0}'..='\u{2FF}' => Class::Narrow,
        '\u{300}'..='\u{36F}' => Class::Combining,
        '\u{3250}'..='\u{A4CF}' => match c {
            '\u{4DC0}'..='\u{4DFF}' => Class::Narrow,
            _ => Class::Wide,
        },
        '\u{600}'..='\u{6FF}' => match c {
            '\u{600}'..='\u{605}' | '\u{61C}' | '\u{6DD}' => Class::Control,
            '\u{610}'..='\u{61A}'
            | '\u{64B}'..='\u{65F}'
            | '\u{670}'
            | '\u{6D6}'..='\u{6DC}'
            | '\u{6DF}'..='\u{6E4}'
            | '\u{6E7}'..='\u{6E8}'
            | '\u{6EA}'..='\u{6ED}' => Class::Combining,
            _ => Class::Narrow,
        },
        '\u{900}'..='\u{97F}' => match c {
            '\u{900}'..='\u{902}'
            | '\u{93A}'
            | '\u{93C}'
            | '\u{941}'..='\u{948}'
            | '\u{94D}'
            | '\u{951}'..='\u{957}'
            | '\u{962}'
            | '\u{963}' => Class::Combining,
            _ => Class::Narrow,
        },
        '\u{980}'..='\u{A02}' => match c {
            '\u{980}'
            | '\u{982}'..='\u{9BB}'
            | '\u{9BD}'..='\u{9C0}'
            | '\u{9C5}'..='\u{9CC}'
            | '\u{9CE}'..='\u{9E1}'
            | '\u{9E4}'..='\u{9FD}' => Class::Narrow,
            _ => Class::Combining,
        },
        '\u{370}'..='\u{58F}' => match c {
            '\u{483}'..='\u{489}' => Class::Combining,
            _ => Class::Narrow,
        },
        '\u{2E80}'..='\u{324F}' => match c {
            '\u{302A}'..='\u{302D}' | '\u{3097}'..='\u{309A}' => Class::Combining,
            '\u{303F}' => Class::Narrow,
            '\u{2E80}'..='\u{3029}' | '\u{302E}'..='\u{3096}' | '\u{309B}'..='\u{3247}' => {
                Class::Wide
            }
            _ => Class::Ambiguous,
        },
        '\u{AC00}'..='\u{D7A3}'
        | '\u{F900}'..='\u{FAFF}'
        | '\u{FF01}'..='\u{FF60}'
        | '\u{FFE0}'..='\u{FFE6}'
        | '\u{20000}'..='\u{3FFFF}' => Class::Wide,
        '\u{C00}'..='\u{C80}' => match c {
            '\u{C00}'
            | '\u{C04}'
            | '\u{C3A}'..='\u{C3C}'
            | '\u{C3E}'..='\u{C40}'
            | '\u{C45}'..='\u{C56}'
            | '\u{C62}'
            | '\u{C63}' => Class::Combining,
            _ => Class::Narrow,
        },
        '\u{B80}'..='\u{BFF}' => match c {
            '\u{B80}'..='\u{B82}' | '\u{BC0}' | '\u{BCD}' => Class::Combining,
            _ => Class::Narrow,
        },
        '\u{E00}'..='\u{E7F}' => match c {
            '\u{E31}' | '\u{E34}'..='\u{E3A}' | '\u{E47}'..='\u{E4E}' => Class::Combining,
            _ => Class::Narrow,
        },
        _ => return None,
    })
}

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
    match c {
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
        '\u{1F3FB}'..='\u{1F3FF}' => match emoji_info(st.last) {
            Some((_, true)) => MatchState {
                first: false,
                total: st.total,
                last: c,
                tentative: 2,
            },
            _ => wide,
        },
        _ => match block_class(c) {
            Some(Class::Narrow | Class::Ambiguous) => narrow,
            Some(Class::Wide) => wide,
            Some(Class::Combining) => combining,
            Some(Class::Control) => control,
            Some(Class::Zwj | Class::EmojiPresentationMod | Class::EmojiSkinToneMod) | None => {
                resolve(st, c, class_of(c))
            }
        },
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
        Class::Zwj if emoji_like => MatchState {
            first: false,
            total: st.total - 2,
            last: c,
            tentative: 2,
        },
        Class::Control | Class::Zwj => with(0),
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
