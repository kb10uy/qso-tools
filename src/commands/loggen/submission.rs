use std::{collections::HashMap, str::FromStr};

use adif_manip::document::Record as AdifRecord;
use anyhow::{Context, Result, anyhow, bail};
use inquire::{CustomType, Select, Text, validator::Validation};
use time::{
    Date, OffsetDateTime, UtcOffset, format_description::BorrowedFormatItem,
    macros::format_description,
};
use tracing::warn;

use crate::{
    commands::loggen::definition::{Category, ContestDefinition},
    core::{
        config::Config,
        jcx::{JAPAN_DXCC, Jcx, JcxCode},
        qso::get_optional_field,
    },
};

const DATE: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]");

/// Summary items with their prompts and kinds; values are given as `-A KEY=VALUE`.
const ITEMS: &[(&str, &str, ItemKind)] = &[
    ("callsign", "Callsign", ItemKind::Text),
    ("op_callsign", "Guest operator callsign", ItemKind::Text),
    ("operator", "Operator in config", ItemKind::Text),
    ("name", "Name", ItemKind::Text),
    ("address", "Address", ItemKind::Text),
    ("tel", "TEL", ItemKind::Text),
    ("email", "Email", ItemKind::Text),
    ("license_date", "License date (YYYY-MM-DD)", ItemKind::Date),
    ("age", "Age", ItemKind::Integer),
    ("power", "Power (W)", ItemKind::Number),
    ("fd_coeff", "FD coefficient", ItemKind::Integer),
    ("op_place", "Operating place", ItemKind::Text),
    ("power_supply", "Power supply", ItemKind::Text),
    ("comments", "Comments", ItemKind::Text),
    ("multi_op_list", "Operators", ItemKind::Text),
    ("reg_club_number", "Registered club number", ItemKind::Text),
    ("date", "Date of oath (YYYY-MM-DD)", ItemKind::Date),
];
const CHOICE_LINES: usize = 15;
const ALWAYS_REQUIRED: &[&str] = &["callsign", "name", "address", "power"];

/// Submission information independent of output formats.
#[derive(Debug, Clone, PartialEq)]
pub struct Submission {
    pub contest_name: String,
    pub contest_id: Option<String>,
    pub category_code: String,
    pub category_name: String,
    pub cabrillo_categories: Vec<(String, String)>,
    pub callsign: String,
    pub op_callsign: Option<String>,
    pub total_score: i64,
    pub address: Option<String>,
    pub name: Option<String>,
    pub tel: Option<String>,
    pub email: Option<String>,
    pub license_date: Option<Date>,
    pub age: Option<u32>,
    pub power: Option<f64>,
    pub fd_coeff: Option<i64>,
    pub op_place: Option<String>,
    pub power_supply: Option<String>,
    pub comments: Option<String>,
    pub multi_op_list: Option<String>,
    pub reg_club_number: Option<String>,
    pub date: Date,
}

/// Kind of summary item value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Text,
    Integer,
    Number,
    Date,
}

/// Asks the user for values missing in arguments.
pub trait Prompter {
    fn value(&mut self, prompt: &str, kind: ItemKind) -> Result<String>;
    fn choice(&mut self, prompt: &str, items: &[String]) -> Result<usize>;
}

/// Prompter on the terminal.
pub struct TerminalPrompter;

impl Prompter for TerminalPrompter {
    fn value(&mut self, prompt: &str, kind: ItemKind) -> Result<String> {
        let value = match kind {
            ItemKind::Text => Text::new(prompt)
                .with_validator(|input: &str| {
                    Ok(if input.trim().is_empty() {
                        Validation::Invalid("Enter a value".into())
                    } else {
                        Validation::Valid
                    })
                })
                .prompt()?,
            ItemKind::Integer => CustomType::<u32>::new(prompt).prompt()?.to_string(),
            ItemKind::Number => CustomType::<f64>::new(prompt).prompt()?.to_string(),
            ItemKind::Date => Text::new(prompt)
                .with_validator(|input: &str| {
                    Ok(match Date::parse(input, DATE) {
                        Ok(_) => Validation::Valid,
                        Err(_) => Validation::Invalid("Enter a date as YYYY-MM-DD".into()),
                    })
                })
                .prompt()?,
        };
        Ok(value)
    }

    fn choice(&mut self, prompt: &str, items: &[String]) -> Result<usize> {
        let selected = Select::new(prompt, items.to_vec())
            .with_page_size(CHOICE_LINES)
            .raw_prompt()?;
        Ok(selected.index)
    }
}

/// Selects category by `-A category=CODE`, or asks the user if there are multiple.
pub fn select_category<'a>(
    definition: &'a ContestDefinition,
    args: &HashMap<String, String>,
    prompter: &mut impl Prompter,
) -> Result<&'a Category> {
    if let Some(code) = args.get("category") {
        return definition
            .category(code)
            .ok_or_else(|| anyhow!("unknown category {code}"));
    }
    if let [category] = definition.categories.as_slice() {
        return Ok(category);
    }

    let items: Vec<_> = definition
        .categories
        .iter()
        .map(|c| format!("{} {}", c.code, c.name))
        .collect();
    let index = prompter
        .choice("Category", &items)
        .context("category is not selected; give it with -A category=CODE")?;
    Ok(&definition.categories[index])
}

/// Sources of submission information.
pub struct SubmissionSource<'a> {
    pub definition: &'a ContestDefinition,
    pub category: &'a Category,
    pub args: &'a HashMap<String, String>,
    pub config: &'a Config,
    pub records: &'a [&'a AdifRecord],
    pub jcx: &'a Jcx,
    pub contest_date: Option<Date>,
    pub offset: UtcOffset,
}

impl SubmissionSource<'_> {
    /// Builds submission; values not in arguments are taken from QSOs and operator config,
    /// and required ones still missing are asked.
    pub fn build(&self, total_score: i64, prompter: &mut impl Prompter) -> Result<Submission> {
        for required in &self.category.requires {
            if !ITEMS.iter().any(|(key, _, _)| key == required) {
                bail!(
                    "unknown required item {required} in category {}",
                    self.category.code
                );
            }
        }

        let mut values = HashMap::new();
        for (key, prompt, kind) in ITEMS {
            let value = match self.args.get(*key) {
                Some(value) => Some(value.clone()),
                None => self.default_value(key, &values),
            };
            let required =
                ALWAYS_REQUIRED.contains(key) || self.category.requires.iter().any(|r| r == key);
            let value =
                match value.filter(|v| !v.is_empty()) {
                    Some(value) => Some(value),
                    None if required => Some(prompter.value(prompt, *kind).with_context(|| {
                        format!("{key} is missing; give it with -A {key}=VALUE")
                    })?),
                    None => None,
                };
            if let Some(value) = value {
                values.insert(*key, value);
            }
        }

        let mut take = |key: &str| values.remove(key).filter(|v| !v.is_empty());
        let parse_date = |key: &str, value: Option<String>| -> Result<Option<Date>> {
            value
                .map(|v| Date::parse(&v, DATE).with_context(|| format!("invalid {key} {v}")))
                .transpose()
        };

        let date = match parse_date("date", take("date"))? {
            Some(date) => date,
            None => OffsetDateTime::now_local()
                .unwrap_or_else(|_| OffsetDateTime::now_utc().to_offset(self.offset))
                .date(),
        };
        Ok(Submission {
            contest_name: self.definition.name.clone(),
            contest_id: self.definition.contest_id.clone(),
            category_code: self.category.code.clone(),
            category_name: self.category.name.clone(),
            cabrillo_categories: self.category.cabrillo.clone(),
            callsign: take("callsign").ok_or_else(|| anyhow!("callsign is missing"))?,
            op_callsign: take("op_callsign"),
            total_score,
            address: take("address"),
            name: take("name"),
            tel: take("tel"),
            email: take("email"),
            license_date: parse_date("license_date", take("license_date"))?,
            age: parse_value("age", take("age"))?,
            power: parse_value("power", take("power"))?,
            fd_coeff: parse_value("fd_coeff", take("fd_coeff"))?,
            op_place: take("op_place"),
            power_supply: take("power_supply"),
            comments: take("comments"),
            multi_op_list: take("multi_op_list"),
            reg_club_number: take("reg_club_number"),
            date,
        })
    }

    fn default_value(&self, key: &str, values: &HashMap<&str, String>) -> Option<String> {
        let first_field = |name| {
            self.records
                .first()
                .and_then(|r| get_optional_field(r, name))
                .map(str::to_string)
        };
        let operator = || {
            let key = values.get("operator")?;
            let operator = self.config.operators.get(key);
            if operator.is_none() {
                warn!("operator {key} is not found in config");
            }
            operator
        };
        let contest = || operator()?.contest.as_ref();

        match key {
            "callsign" => first_field("STATION_CALLSIGN"),
            "operator" => first_field("OPERATOR").or_else(|| values.get("callsign").cloned()),
            "name" => contest()
                .and_then(|c| c.name.clone())
                .or_else(|| operator().map(|o| o.name.clone())),
            "address" => contest().and_then(|c| c.address.clone()),
            "tel" => contest().and_then(|c| c.tel.clone()),
            "email" => contest().and_then(|c| c.email.clone()),
            "license_date" => contest()
                .and_then(|c| c.licensed_at)
                .and_then(|d| d.format(DATE).ok()),
            "age" => {
                let birthday = contest().and_then(|c| c.birthday)?;
                Some(calculate_age(self.contest_date?, birthday).to_string())
            }
            "op_place" => {
                let is_japan = first_field("MY_DXCC").is_some_and(|d| d == JAPAN_DXCC.to_string());
                let code = first_field("MY_CNTY")?.parse::<JcxCode>().ok()?;
                is_japan.then(|| self.jcx.full_name_ja(&code)).flatten()
            }
            "power" => self
                .records
                .iter()
                .filter_map(|r| get_optional_field(r, "TX_PWR")?.parse::<f64>().ok())
                .reduce(f64::max)
                .map(|p| p.to_string()),
            _ => None,
        }
    }
}

fn parse_value<T: FromStr>(key: &str, value: Option<String>) -> Result<Option<T>> {
    value
        .map(|v| v.parse().map_err(|_| anyhow!("invalid {key} {v}")))
        .transpose()
}

/// Calculates age at `today`.
pub fn calculate_age(today: Date, birthday: Date) -> u32 {
    let had_birthday = (today.month(), today.day()) >= (birthday.month(), birthday.day());
    let age = today.year() - birthday.year() - if had_birthday { 0 } else { 1 };
    age.max(0) as u32
}

#[cfg(test)]
mod tests {
    use adif_manip::{LengthMode, read_adi};
    use mlua::Lua;
    use time::macros::date;

    use super::*;
    use crate::core::config::{ContestOperatorConfig, OperatorConfig};

    struct FakePrompter(HashMap<&'static str, &'static str>);

    impl Prompter for FakePrompter {
        fn value(&mut self, prompt: &str, _: ItemKind) -> Result<String> {
            self.0
                .get(prompt)
                .map(|v| v.to_string())
                .ok_or_else(|| anyhow!("not a terminal"))
        }

        fn choice(&mut self, _: &str, _: &[String]) -> Result<usize> {
            bail!("not a terminal")
        }
    }

    fn definition(lua: &Lua) -> ContestDefinition {
        lua.load(
            "return {
                name = 'Test',
                categories = {
                    XA = { name = 'Mixed' },
                    PN = { name = 'Newcomer', requires = { 'license_date' } },
                    XS = { name = 'Silver', requires = { 'age' } },
                },
                score_qso = function() end,
            }",
        )
        .eval()
        .unwrap()
    }

    #[test]
    fn calculates_age() {
        let birthday = date!(1998 - 06 - 03);
        assert_eq!(calculate_age(date!(2026 - 06 - 02), birthday), 27);
        assert_eq!(calculate_age(date!(2026 - 06 - 03), birthday), 28);
        assert_eq!(calculate_age(date!(1990 - 01 - 01), birthday), 0);
    }

    #[test]
    fn selects_category() {
        let lua = Lua::new();
        let definition = definition(&lua);
        let mut prompter = FakePrompter(HashMap::new());
        let args = HashMap::from([("category".to_string(), "PN".to_string())]);
        assert_eq!(
            select_category(&definition, &args, &mut prompter)
                .unwrap()
                .code,
            "PN"
        );

        let args = HashMap::from([("category".to_string(), "ZZ".to_string())]);
        assert!(select_category(&definition, &args, &mut prompter).is_err());
        assert!(select_category(&definition, &HashMap::new(), &mut prompter).is_err());
    }

    #[test]
    fn builds_submission() {
        let lua = Lua::new();
        let definition = definition(&lua);
        let document = read_adi(
            "Test\n<EOH>\n<STATION_CALLSIGN:6>N0CALL<TX_PWR:2>10<EOR>\n<STATION_CALLSIGN:6>N0CALL<TX_PWR:2>50<EOR>\n",
            LengthMode::Bytes,
        )
        .unwrap();
        let records: Vec<_> = document.records().iter().collect();
        let jcx = Jcx::default();
        let mut config = Config::default();
        config.operators.insert(
            "N0CALL".to_string(),
            OperatorConfig {
                name: "No Call".to_string(),
                contest: Some(ContestOperatorConfig {
                    name: Some("Contest Name".to_string()),
                    address: Some("Somewhere".to_string()),
                    birthday: Some(date!(1950 - 07 - 01)),
                    ..Default::default()
                }),
            },
        );
        let args = HashMap::from([
            ("date".to_string(), "2026-06-10".to_string()),
            ("comments".to_string(), String::new()),
        ]);
        let source = |code| SubmissionSource {
            definition: &definition,
            category: definition.category(code).unwrap(),
            args: &args,
            config: &config,
            records: &records,
            jcx: &jcx,
            contest_date: Some(date!(2026 - 06 - 06)),
            offset: UtcOffset::UTC,
        };

        let mut prompter = FakePrompter(HashMap::new());
        let submission = source("XS").build(100, &mut prompter).unwrap();
        assert_eq!(submission.callsign, "N0CALL");
        assert_eq!(submission.name.as_deref(), Some("Contest Name"));
        assert_eq!(submission.address.as_deref(), Some("Somewhere"));
        assert_eq!(submission.power, Some(50.0));
        assert_eq!(submission.age, Some(75));
        assert_eq!(submission.comments, None);
        assert_eq!(submission.date, date!(2026 - 06 - 10));

        assert!(source("PN").build(100, &mut prompter).is_err());
        let mut prompter =
            FakePrompter(HashMap::from([("License date (YYYY-MM-DD)", "2025-09-19")]));
        let submission = source("PN").build(100, &mut prompter).unwrap();
        assert_eq!(submission.license_date, Some(date!(2025 - 09 - 19)));
    }
}
