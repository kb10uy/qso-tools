mod card;
mod cli;
mod data;
mod source;

use std::{
    collections::HashMap,
    fs::read_to_string,
    io::stdout,
    path::{Path, PathBuf, absolute},
    sync::LazyLock,
};

use adif_reader::document::Record as AdifRecord;
use anyhow::{Context, Result, anyhow, bail};
use callfind::grid_locator::GridLocator;
use compact_str::{CompactString, ToCompactString};
use mlua::prelude::*;
use regex::Regex;
use time::UtcOffset;
use tracing::{Level, info, span, warn};

use crate::{
    commands::qcgen::{
        card::{
            QslCardEntry, QslCounty, QslInstrument, QslLocation, QslOperator, QslPark,
            QslReferences, QslRouting, QslState, QslStation,
        },
        data::{Instrument, Park},
        source::read_document,
    },
    core::{
        config::{Config, OperatorConfig, read_items_from_tomls},
        jcx::{Jcx, JcxCode},
        qso::{exchange::QsoExchanges, get_optional_field, qsl::QslStatus, record::QsoRecord},
        schope::engine::{initialize_lua, lua_to_json},
    },
};

pub use cli::Arguments;

const INSTRUMENTS_FILENAME: &str = "instruments.toml";
const PARKS_FILENAME: &str = "parks.toml";
const SUBDIVISIONS_FILENAME: &str = "subdivisions.toml";

static RE_EXTRA_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"!(\w+):(\S+)").expect("valid regex"));

struct EntryContext<'a> {
    instruments: HashMap<String, Instrument>,
    parks: HashMap<String, Park>,
    jcx: Jcx,
    operators: &'a HashMap<String, OperatorConfig>,
    default_instrument: Option<&'a str>,
    default_power: Option<f64>,
    subdivisions: HashMap<u32, HashMap<String, String>>,
}

pub fn run(args: Arguments, config: &Config) -> Result<()> {
    let script_path = absolute(&args.script_path)
        .with_context(|| format!("invalid script path {}", args.script_path.display()))?;
    let script_args: HashMap<_, _> = args
        .script_args
        .into_iter()
        .map(|a| (a.key, a.value))
        .collect();

    let document = read_document(
        args.adif.as_deref(),
        args.lenient_length.unwrap_or_default().into(),
    )?;

    let context = EntryContext {
        instruments: read_items_from_tomls(
            config
                .sibling_file(INSTRUMENTS_FILENAME)
                .into_iter()
                .chain(args.instruments_files),
        )?,
        parks: read_items_from_tomls::<Park>(config.sibling_file(PARKS_FILENAME))?
            .into_iter()
            .map(|(k, v)| (k.to_ascii_uppercase(), v))
            .collect(),
        jcx: Jcx::load(config)?,
        operators: &config.operators,
        default_instrument: args.instrument.as_deref(),
        default_power: args.power,
        subdivisions: read_subdivisions(config.sibling_file(SUBDIVISIONS_FILENAME))?,
    };

    let mut entries = vec![];
    for (i, record) in document.records().iter().enumerate() {
        let span = span!(Level::ERROR, "record_process", index = i);
        let _enter = span.enter();

        let qsl_status = QslStatus::new(record)?;
        entries.push(build_entry(record, qsl_status, &context)?);
    }
    info!("processing {} QSOs", entries.len());

    let processed_json = run_script(&script_path, script_args, entries)?;
    serde_json::to_writer(stdout().lock(), &processed_json)?;

    Ok(())
}

fn build_entry(
    record: &AdifRecord,
    qsl_status: QslStatus,
    context: &EntryContext,
) -> Result<QslCardEntry> {
    let qso_record = QsoRecord::new(record, UtcOffset::UTC)?;
    let qso_exchanges = QsoExchanges::new(record);
    let station = build_station(record, context);

    let mut instrument_key = context.default_instrument;
    let comment = get_optional_field(record, "COMMENT").unwrap_or_default();
    for extra_tag in RE_EXTRA_TAG.captures_iter(comment) {
        let key = extra_tag.get(1).expect("group must exist");
        let value = extra_tag.get(2).expect("group must exist");
        match key.as_str() {
            "inst" => instrument_key = Some(value.as_str()),
            otherwise => warn!("unknown extra tag: {otherwise}"),
        }
    }

    let instrument = instrument_key.and_then(|k| context.instruments.get(k));
    if let (Some(key), None) = (instrument_key, instrument) {
        warn!("unknown instrument: {key}");
    }
    let power = get_optional_field(record, "TX_PWR")
        .and_then(|p| match p.parse() {
            Ok(power) => Some(power),
            Err(e) => {
                warn!("invalid TX_PWR {p}: {e}");
                None
            }
        })
        .filter(is_valid_power)
        .or(context.default_power.filter(is_valid_power))
        .or(instrument
            .and_then(|i| i.default_power)
            .filter(is_valid_power));

    let operator_callsign = get_optional_field(record, "OPERATOR");
    let operator_name = operator_callsign
        .and_then(|c| context.operators.get(c))
        .map(|o| o.name.to_compact_string());

    Ok(QslCardEntry {
        qso: qso_record.into(),
        exchange: qso_exchanges.into(),
        station,
        operator: QslOperator {
            callsign: operator_callsign.map(|s| s.to_compact_string()),
            name: operator_name,
        },
        instrument: QslInstrument {
            antenna: instrument.map(|i| i.antenna.to_compact_string()),
            rig: instrument.map(|i| i.rig.to_compact_string()),
            power,
        },
        qsl: QslRouting {
            should_send: qsl_status.should_send(),
            received: qsl_status.received(),
            via: compact_field(record, "QSL_VIA"),
            sent_via: compact_field(record, "QSL_SENT_VIA"),
        },
    })
}

/// Checks whether the power value is valid; zero is treated as missing.
fn is_valid_power(power: &f64) -> bool {
    *power != 0.0
}

fn build_station(record: &AdifRecord, context: &EntryContext) -> QslStation {
    let grid =
        get_optional_field(record, "MY_GRIDSQUARE").and_then(|g| match g.parse::<GridLocator>() {
            Ok(grid) => Some(grid),
            Err(e) => {
                warn!("invalid MY_GRIDSQUARE {g}: {e}");
                None
            }
        });

    let dxcc = get_optional_field(record, "MY_DXCC").and_then(|d| d.parse::<u32>().ok());
    let state = get_optional_field(record, "MY_STATE").map(|code| QslState {
        code: code.to_compact_string(),
        name: dxcc
            .and_then(|d| context.subdivisions.get(&d))
            .and_then(|s| s.get(&code.to_ascii_uppercase()))
            .map(|n| n.to_compact_string()),
    });

    QslStation {
        callsign: compact_field(record, "STATION_CALLSIGN"),
        location: QslLocation {
            grid,
            city: compact_field(record, "MY_CITY"),
            county: get_optional_field(record, "MY_CNTY").map(|c| build_county(c, &context.jcx)),
            state,
            country: compact_field(record, "MY_COUNTRY"),
        },
        references: QslReferences {
            pota: get_optional_field(record, "MY_POTA_REF")
                .map(|r| build_parks(r, &context.parks))
                .unwrap_or_default(),
            sota: compact_field(record, "MY_SOTA_REF"),
            wwff: compact_field(record, "MY_WWFF_REF"),
            iota: compact_field(record, "MY_IOTA"),
            sig: compact_field(record, "MY_SIG"),
            sig_info: compact_field(record, "MY_SIG_INFO"),
        },
    }
}

/// Reads subdivision names keyed by DXCC entity code and then subdivision code.
fn read_subdivisions(file: Option<PathBuf>) -> Result<HashMap<u32, HashMap<String, String>>> {
    read_items_from_tomls::<HashMap<String, String>>(file)?
        .into_iter()
        .map(|(dxcc, names)| {
            let dxcc = dxcc
                .parse()
                .map_err(|_| anyhow!("invalid DXCC entity code in subdivisions: {dxcc}"))?;
            let names = names
                .into_iter()
                .map(|(k, v)| (k.to_ascii_uppercase(), v))
                .collect();
            Ok((dxcc, names))
        })
        .collect()
}

/// Resolves JCC/JCG code (with optional HAMLOG town suffix like `15006C`) into its names.
fn build_county(code: &str, jcx: &Jcx) -> QslCounty {
    let (county, town, has_town) = match code.parse::<JcxCode>() {
        Ok(c) => (jcx.county(&c), jcx.town(&c), c.town().is_some()),
        Err(_) => (None, None, false),
    };
    let code = code.trim().to_ascii_uppercase();
    if county.is_none() {
        if !jcx.is_empty() {
            warn!("unknown county: {code}");
        }
    } else if has_town && town.is_none() {
        warn!("unknown HAMLOG town: {code}");
    }

    QslCounty {
        kind: county
            .and_then(|c| c.kind.as_deref())
            .map(|k| k.to_compact_string()),
        name_ja: county
            .and_then(|c| c.name_ja.as_deref())
            .filter(|n| !n.is_empty())
            .map(|n| n.to_compact_string()),
        name_en: county
            .and_then(|c| c.name_en.as_deref())
            .filter(|n| !n.is_empty())
            .map(|n| n.to_compact_string()),
        town_ja: county
            .and(town)
            .and_then(|t| t.name_ja.as_deref())
            .filter(|n| !n.is_empty())
            .map(|n| n.to_compact_string()),
        code: code.into(),
    }
}

/// Splits comma-separated POTA references (with optional `@LOCATION`) and resolves park names.
fn build_parks(pota_ref: &str, parks: &HashMap<String, Park>) -> Vec<QslPark> {
    pota_ref
        .split(',')
        .map(str::trim)
        .filter(|r| !r.is_empty())
        .map(|r| {
            let (reference, location) = match r.split_once('@') {
                Some((reference, location)) => (reference.trim(), Some(location.trim())),
                None => (r, None),
            };
            let reference = reference.to_ascii_uppercase();
            let park = parks.get(&reference);
            if park.is_none() {
                warn!("unknown park: {reference}");
            }
            QslPark {
                location: location
                    .filter(|l| !l.is_empty())
                    .map(|l| l.to_ascii_uppercase().into()),
                name_en: park
                    .and_then(|p| p.name_en.as_deref())
                    .filter(|n| !n.is_empty())
                    .map(|n| n.to_compact_string()),
                name_ja: park
                    .and_then(|p| p.name_ja.as_deref())
                    .filter(|n| !n.is_empty())
                    .map(|n| n.to_compact_string()),
                reference: reference.into(),
            }
        })
        .collect()
}

fn compact_field(record: &AdifRecord, name: &str) -> Option<CompactString> {
    get_optional_field(record, name).map(|v| v.to_compact_string())
}

fn run_script(
    script_path: &Path,
    script_args: HashMap<String, String>,
    entries: Vec<QslCardEntry>,
) -> Result<serde_json::Value> {
    let script_text = read_to_string(script_path)
        .with_context(|| format!("failed to read {}", script_path.display()))?;
    let Some(script_base) = script_path.parent() else {
        bail!("script path is invalid");
    };

    let lua = initialize_lua(script_base)?;
    let script_table: LuaTable = lua
        .load(script_text)
        .set_name(format!("@{}", script_path.display()))
        .eval()?;
    let generate: LuaFunction = script_table.get("generate")?;
    let processed_value: LuaValue = generate.call((script_args, entries))?;

    Ok(lua_to_json(processed_value)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::jcx::{County, Town};

    #[test]
    fn builds_parks() {
        let parks = HashMap::from([(
            "JP-0001".to_string(),
            Park {
                name_en: Some("Park One".to_string()),
                name_ja: Some("公園一".to_string()),
            },
        )]);

        let built = build_parks("jp-0001@jp-13, JP-0002", &parks);
        assert_eq!(
            built,
            [
                QslPark {
                    reference: "JP-0001".into(),
                    location: Some("JP-13".into()),
                    name_en: Some("Park One".into()),
                    name_ja: Some("公園一".into()),
                },
                QslPark {
                    reference: "JP-0002".into(),
                    location: None,
                    name_en: None,
                    name_ja: None,
                },
            ]
        );
    }

    #[test]
    fn builds_county() {
        let jcx = Jcx::new(
            HashMap::from([
                (
                    "1001".to_string(),
                    County {
                        kind: Some("city".to_string()),
                        name_ja: Some("市一".to_string()),
                        name_en: None,
                    },
                ),
                (
                    "15006".to_string(),
                    County {
                        kind: Some("gun".to_string()),
                        name_ja: Some("郡一".to_string()),
                        name_en: None,
                    },
                ),
            ]),
            HashMap::from([(
                "15006".to_string(),
                HashMap::from([(
                    "C".to_string(),
                    Town {
                        name_ja: Some("町一".to_string()),
                    },
                )]),
            )]),
        );

        assert_eq!(
            build_county("1001", &jcx),
            QslCounty {
                code: "1001".into(),
                kind: Some("city".into()),
                name_ja: Some("市一".into()),
                name_en: None,
                town_ja: None,
            }
        );
        assert_eq!(
            build_county("15006c", &jcx),
            QslCounty {
                code: "15006C".into(),
                kind: Some("gun".into()),
                name_ja: Some("郡一".into()),
                name_en: None,
                town_ja: Some("町一".into()),
            }
        );
        assert_eq!(
            build_county("9999", &jcx),
            QslCounty {
                code: "9999".into(),
                kind: None,
                name_ja: None,
                name_en: None,
                town_ja: None,
            }
        );
    }
}
