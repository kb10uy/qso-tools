use std::sync::Arc;

use mlua::prelude::*;

use crate::core::jcx::{Jcx, JcxCode};

/// JARL contest helpers backed by JCC/JCG definitions.
pub struct JarlModule {
    jcx: Arc<Jcx>,
}

/// Contest number split into area number and power code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContestNumber {
    pub area: String,
    pub power: Option<char>,
}

impl JarlModule {
    pub fn new(jcx: Arc<Jcx>) -> JarlModule {
        JarlModule { jcx }
    }

    pub fn create_module_table(self, lua: &Lua) -> LuaResult<LuaTable> {
        let t = lua.create_table()?;
        let this = Arc::new(self);

        t.set(
            "parse_number",
            lua.create_function(|lua, number: Option<String>| {
                let Some(parsed) = number.as_deref().and_then(parse_number) else {
                    return Ok(LuaNil);
                };
                let table = lua.create_table()?;
                table.set("area", parsed.area)?;
                table.set("power", parsed.power.map(String::from))?;
                Ok(LuaValue::Table(table))
            })?,
        )?;
        t.set(
            "jcx",
            lua.create_function(move |lua, code: Option<String>| {
                this.jcx_table(lua, code.as_deref())
            })?,
        )?;
        t.set(
            "area_of_prefecture",
            lua.create_function(|_, code: String| Ok(area_of_prefecture(&code)))?,
        )?;
        t.set(
            "area_of_call",
            lua.create_function(|_, call: String| Ok(area_of_call(&call)))?,
        )?;

        Ok(t)
    }

    fn jcx_table(&self, lua: &Lua, code: Option<&str>) -> LuaResult<LuaValue> {
        let Some(code) = code.and_then(|c| c.parse::<JcxCode>().ok()) else {
            return Ok(LuaNil);
        };
        let Some(county) = self.jcx.county(&code) else {
            return Ok(LuaNil);
        };
        let prefecture = code.prefecture_code().unwrap_or(code.county());

        let table = lua.create_table()?;
        table.set("code", code.county())?;
        table.set("kind", county.kind.as_deref())?;
        table.set("name", self.jcx.full_name_ja(&code))?;
        table.set("prefecture", prefecture)?;
        table.set("area", area_of_prefecture(prefecture))?;
        table.set("has_wards", self.jcx.has_wards(&code))?;
        Ok(LuaValue::Table(table))
    }
}

/// Splits contest number like `100116L` into area number and power code.
pub fn parse_number(number: &str) -> Option<ContestNumber> {
    let number = number.trim().to_ascii_uppercase();
    let (area, power) = match number.chars().last()? {
        c @ ('P' | 'L' | 'M' | 'H') => (&number[..number.len() - 1], Some(c)),
        _ => (number.as_str(), None),
    };
    if area.is_empty() || !area.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(ContestNumber {
        area: area.to_string(),
        power,
    })
}

/// Returns call area of prefecture number, including Hokkaido sub-prefectures (101-114) and Ogasawara (48).
pub fn area_of_prefecture(code: &str) -> Option<u8> {
    let number: u32 = code.trim().parse().ok()?;
    match number {
        1 | 101..=114 => Some(8),
        2..=7 => Some(7),
        8 | 9 => Some(0),
        10..=17 | 48 => Some(1),
        18..=21 => Some(2),
        22..=27 => Some(3),
        28..=30 => Some(9),
        31..=35 => Some(4),
        36..=39 => Some(5),
        40..=47 => Some(6),
        _ => None,
    }
}

/// Returns call area of Japanese callsign, or the area digit of portable suffix like `/3`.
pub fn area_of_call(call: &str) -> Option<u8> {
    let call = call.trim().to_ascii_uppercase();
    let mut segments = call.split('/');
    let home = segments.next()?;
    if let Some(digit) = segments.find_map(|s| match s.as_bytes() {
        [d] if d.is_ascii_digit() => Some(d - b'0'),
        _ => None,
    }) {
        return Some(digit);
    }

    let &[first, second, area, ..] = home.as_bytes() else {
        return None;
    };
    let is_japanese = matches!(
        (first, second),
        (b'J', b'A'..=b'S') | (b'7', b'J'..=b'N') | (b'8', b'J'..=b'N')
    );
    (is_japanese && area.is_ascii_digit()).then_some(area - b'0')
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::core::jcx::County;

    #[test]
    fn parses_number() {
        assert_eq!(
            parse_number("100116l"),
            Some(ContestNumber {
                area: "100116".to_string(),
                power: Some('L'),
            })
        );
        assert_eq!(
            parse_number("10"),
            Some(ContestNumber {
                area: "10".to_string(),
                power: None,
            })
        );
        assert_eq!(parse_number("H"), None);
        assert_eq!(parse_number("1001X"), None);
        assert_eq!(parse_number(""), None);
    }

    #[test]
    fn finds_area() {
        assert_eq!(area_of_prefecture("10"), Some(1));
        assert_eq!(area_of_prefecture("01"), Some(8));
        assert_eq!(area_of_prefecture("106"), Some(8));
        assert_eq!(area_of_prefecture("48"), Some(1));
        assert_eq!(area_of_prefecture("49"), None);

        assert_eq!(area_of_call("JL1HIS"), Some(1));
        assert_eq!(area_of_call("JA1RL/3"), Some(3));
        assert_eq!(area_of_call("7K4XXX"), Some(4));
        assert_eq!(area_of_call("8J0YYY"), Some(0));
        assert_eq!(area_of_call("W1AW"), None);
        assert_eq!(area_of_call("W1AW/JA1"), None);
    }

    #[test]
    fn looks_up_jcx() {
        let county = |kind: &str, name_ja: &str| County {
            kind: Some(kind.to_string()),
            name_ja: Some(name_ja.to_string()),
            name_en: None,
        };
        let jcx = Jcx::new(
            HashMap::from([
                ("10".to_string(), county("prefecture", "東京都")),
                ("100116".to_string(), county("ward", "豊島区")),
                ("0601".to_string(), county("city", "仙台市")),
                ("060101".to_string(), county("ward", "青葉区")),
                ("1002".to_string(), county("city", "八王子市")),
            ]),
            HashMap::new(),
        );
        let lua = Lua::new();
        let module = JarlModule::new(Arc::new(jcx))
            .create_module_table(&lua)
            .unwrap();
        lua.globals().set("jarl", module).unwrap();

        let (kind, name, prefecture, area): (String, String, String, u8) = lua
            .load("local j = jarl.jcx('100116'); return j.kind, j.name, j.prefecture, j.area")
            .eval()
            .unwrap();
        assert_eq!(
            (kind.as_str(), name.as_str(), prefecture.as_str(), area),
            ("ward", "東京都豊島区", "10", 1)
        );
        let prefecture: String = lua.load("return jarl.jcx('10').prefecture").eval().unwrap();
        assert_eq!(prefecture, "10");
        let has_wards: (bool, bool, bool) = lua
            .load("return jarl.jcx('0601').has_wards, jarl.jcx('060101').has_wards, jarl.jcx('1002').has_wards")
            .eval()
            .unwrap();
        assert_eq!(has_wards, (true, false, false));
        let unknown: LuaValue = lua.load("return jarl.jcx('100199')").eval().unwrap();
        assert!(unknown.is_nil());
        let power: String = lua
            .load("return jarl.parse_number('100116L').power")
            .eval()
            .unwrap();
        assert_eq!(power, "L");
    }
}
