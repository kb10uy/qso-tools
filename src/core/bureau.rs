use callfind::cty::{CtyDatabase, Scope};
use compact_str::{CompactString, ToCompactString};

/// Position of a QSL card in the order JARL QSL bureau asks members to bundle cards.
///
/// See <https://www.jarl.org/Japanese/5_Nyukai/qsl-sq.htm> for the rules.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BureauOrder {
    /// Card for a Japanese station, ordered by prefix group and then suffix.
    Domestic {
        section: DomesticSection,
        suffix: CompactString,
    },

    /// Card for a foreign station, ordered by entity prefix and then callsign.
    Foreign {
        entity: CompactString,
        callsign: CompactString,
    },
}

/// Prefix group of domestic cards in the order JARL lists them.
///
/// Areas are ranked from 1 to 9 and then 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DomesticSection {
    /// JA1 to JS0, ordered by area and then prefix.
    Regular { area: u8, prefix: u8 },

    /// 7J1 to 7J0 for foreign nationals, ordered by area.
    ForeignNational { area: u8 },

    /// 7K1 to 7N4, ordered by prefix and then area.
    Seven { prefix: u8, area: u8 },

    /// 8J1 to 8N0 for commemorative stations, ordered by prefix and then area.
    Commemorative { prefix: u8, area: u8 },

    /// SWL numbers like JA1-12345, placed after all QSL cards and ordered by number.
    Swl { area: u8, number: u32 },
}

impl BureauOrder {
    /// Determines the order of the card for `call`, routed to `via` instead if it looks like a callsign.
    /// Foreign cards are grouped by DXCC entity if `cty` is given, otherwise by callsign prefix.
    pub fn new(call: &str, via: Option<&str>, cty: Option<&CtyDatabase>) -> BureauOrder {
        let routed = via.filter(|v| is_callsign_like(v)).unwrap_or(call);
        let callsign = home_callsign(routed);
        if let Some((section, suffix)) = parse_domestic(&callsign) {
            return BureauOrder::Domestic {
                section,
                suffix: suffix.to_compact_string(),
            };
        }

        let entity = cty
            .and_then(|c| c.lookup_in(&callsign, Scope::Dxcc))
            .map(|r| r.entity.primary_prefix.to_compact_string())
            .unwrap_or_else(|| callsign_prefix(&callsign).to_compact_string());
        BureauOrder::Foreign { entity, callsign }
    }
}

/// Checks whether the text consists of letters, digits and slashes, containing both letters and digits.
fn is_callsign_like(text: &str) -> bool {
    let text = text.trim();
    text.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'/')
        && text.bytes().any(|b| b.is_ascii_digit())
        && text.bytes().any(|b| b.is_ascii_alphabetic())
}

/// Extracts the longest segment of portable notation like `JA1ABC/3` or `KH6/W1AW` in uppercase.
fn home_callsign(callsign: &str) -> CompactString {
    let longest = callsign
        .trim()
        .split('/')
        .reduce(|a, b| if b.len() > a.len() { b } else { a })
        .unwrap_or_default();
    longest.to_ascii_uppercase().into()
}

/// Extracts the prefix up to the last digit like `3D2` of `3D2AG`.
fn callsign_prefix(callsign: &str) -> &str {
    match callsign.rfind(|c: char| c.is_ascii_digit()) {
        Some(i) => &callsign[..=i],
        None => callsign,
    }
}

/// Parses a Japanese callsign or SWL number into its prefix group and suffix.
fn parse_domestic(callsign: &str) -> Option<(DomesticSection, &str)> {
    let &[first, second, area, ..] = callsign.as_bytes() else {
        return None;
    };
    if !area.is_ascii_digit() {
        return None;
    }
    let area = area_rank(area);
    let rest = &callsign[3..];

    if let (b'J', b'A'..=b'S') = (first, second)
        && let Some(number) = rest.strip_prefix('-')
    {
        if number.is_empty() || !number.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let number = number.parse().ok()?;
        return Some((DomesticSection::Swl { area, number }, ""));
    }

    if rest.is_empty() || !rest.bytes().all(|b| b.is_ascii_alphanumeric()) {
        return None;
    }
    let section = match (first, second) {
        (b'J', b'A'..=b'S') => DomesticSection::Regular {
            area,
            prefix: second,
        },
        (b'7', b'J') => DomesticSection::ForeignNational { area },
        (b'7', b'K'..=b'N') => DomesticSection::Seven {
            prefix: second,
            area,
        },
        (b'8', b'J'..=b'N') => DomesticSection::Commemorative {
            prefix: second,
            area,
        },
        _ => return None,
    };
    Some((section, rest))
}

/// Ranks area digits from 1 to 9 and then 0.
fn area_rank(digit: u8) -> u8 {
    match digit {
        b'0' => 10,
        d => d - b'0',
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CTY: &str = "\
Fiji Islands:             32:  56:  OC:  -17.78:  -177.92:   -12.0:  3D2:
    3D2;
United States:            05:  08:  NA:   37.53:    91.67:     5.0:  K:
    AA,K,N,W;
Hawaii:                   31:  61:  OC:   21.12:   157.48:    10.0:  KH6:
    KH6,KH7;
";

    fn sorted(calls: &[&str], cty: Option<&CtyDatabase>) -> Vec<String> {
        let mut calls = calls.to_vec();
        calls.sort_by_cached_key(|c| BureauOrder::new(c, None, cty));
        calls.into_iter().map(String::from).collect()
    }

    #[test]
    fn sorts_domestic_cards() {
        let expected = [
            "JA1AA",
            "JA1ZZ",
            "JD1BCD",
            "JE1ABC",
            "JS1XYZ",
            "JA2ABC",
            "JR2ZZZ",
            "JA9ABC",
            "JA0ABC",
            "JR0ABC",
            "7J1AAA",
            "7J0AAA",
            "7K1ABC",
            "7K4ABC",
            "7L1ABC",
            "7N4ABC",
            "8J1ABC",
            "8J0ABC",
            "8M1ABC",
            "8N1ABC",
            "JA1-100",
            "JA1-20000",
            "JA2-1",
        ];
        let mut shuffled = expected.to_vec();
        shuffled.reverse();
        shuffled.swap(3, 17);
        shuffled.swap(8, 12);

        assert_eq!(sorted(&shuffled, None), expected);
    }

    #[test]
    fn places_domestic_before_foreign() {
        assert_eq!(
            sorted(&["W1AW", "JA1ABC", "3D2AG"], None),
            ["JA1ABC", "3D2AG", "W1AW"]
        );
    }

    #[test]
    fn groups_foreign_cards_by_entity() {
        let cty = CtyDatabase::parse(CTY).unwrap();
        assert_eq!(
            sorted(&["W1AW", "KH6ABC", "N1XYZ", "3D2AG", "AA1ZZ"], Some(&cty)),
            ["3D2AG", "AA1ZZ", "N1XYZ", "W1AW", "KH6ABC"]
        );
    }

    #[test]
    fn routes_portable_callsigns_to_home_callsigns() {
        assert_eq!(
            BureauOrder::new("3d2/ja1abc", None, None),
            BureauOrder::new("JA1ABC", None, None)
        );
        assert_eq!(
            BureauOrder::new("JA1ABC/3", None, None),
            BureauOrder::new("JA1ABC", None, None)
        );
        assert_eq!(
            BureauOrder::new("KH6/W1AW", None, None),
            BureauOrder::new("W1AW", None, None)
        );
    }

    #[test]
    fn routes_cards_via_managers() {
        assert_eq!(
            BureauOrder::new("3D2AG", Some("JA1ABC"), None),
            BureauOrder::new("JA1ABC", None, None)
        );
        assert_eq!(
            BureauOrder::new("3D2AG", Some("BURO"), None),
            BureauOrder::new("3D2AG", None, None)
        );
    }

    #[test]
    fn rejects_malformed_domestic_callsigns() {
        assert_eq!(parse_domestic("JA1"), None);
        assert_eq!(parse_domestic("JA1-"), None);
        assert_eq!(parse_domestic("JA1-+5"), None);
        assert_eq!(parse_domestic("JT1ABC"), None);
        assert_eq!(parse_domestic("7O1ABC"), None);
    }
}
