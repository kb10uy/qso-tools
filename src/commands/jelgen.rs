mod contest;
mod elog;

use std::{
    collections::{HashMap, HashSet},
    io::Write,
    path::{PathBuf, absolute},
};

use adif_reader::document::Record as AdifRecord;
use anyhow::{Context, Result, anyhow, bail};
use clap::Args;
use compact_str::ToCompactString;
use mlua::prelude::*;
use time::{
    Date, OffsetDateTime, UtcOffset, format_description::BorrowedFormatItem,
    macros::format_description,
};
use tracing::{Level, info, span, warn};

use crate::{
    commands::jelgen::{
        contest::{ContestConfig, ContestEntry, ContestSummary, QsoScore},
        elog::{LogLine, LogSheet, SummarySheet, calculate_age, jarl_band, jarl_mode},
    },
    core::{
        adif::AdifInput,
        config::{Config, OperatorConfig},
        output::OutputFile,
        qso::{exchange::QsoExchanges, get_optional_field, record::QsoRecord},
        schope::{args::ScriptArg, engine::load_script},
    },
};

const CONFIG_DATE: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]");
const CONFIG_OFFSET: &[BorrowedFormatItem<'_>] =
    format_description!("[offset_hour]:[offset_minute]");
const DEFAULT_OFFSET: &str = "+09:00";
const DEFAULT_LOG_TYPE: &str = "qso-tools";
const NO_VALUE: &str = "-";

/// Generates JARL eLog (summary sheet R2.1 and log sheet) through a Lua script.
#[derive(Debug, Clone, Args)]
pub struct Arguments {
    /// Contest script file.
    pub script_path: PathBuf,

    #[command(flatten)]
    pub input: AdifInput,

    /// Specify arguments passed to script as KEY or KEY=VALUE.
    #[arg(short = 'A', long = "args", value_name = "KEY[=VALUE]")]
    pub script_args: Vec<ScriptArg>,

    #[command(flatten)]
    pub output: OutputFile,
}

/// QSO accepted by the script.
struct ContestQso<'a> {
    qso: &'a QsoRecord,
    record: &'a AdifRecord,
}

#[derive(Debug, Default)]
struct Tally {
    keys: HashSet<String>,
    multiplier_keys: HashSet<String>,
    summary: ContestSummary,
}

pub fn run(args: Arguments, config: &Config) -> Result<()> {
    let script_path = absolute(&args.script_path)
        .with_context(|| format!("invalid script path {}", args.script_path.display()))?;
    let script_args: HashMap<_, _> = args
        .script_args
        .into_iter()
        .map(|a| (a.key, a.value))
        .collect();

    let document = args.input.read()?;
    let import_offset = args.input.import_offset();
    let mut qsos = vec![];
    for (i, record) in document.records().iter().enumerate() {
        let span = span!(Level::ERROR, "record_read", index = i);
        let _enter = span.enter();

        match QsoRecord::new(record, import_offset) {
            Ok(qso) => qsos.push((qso, record)),
            Err(e) => warn!("skipped invalid QSO: {e}"),
        }
    }
    qsos.sort_by_key(|(qso, _)| qso.datetime);

    let (_lua, script) = load_script(&script_path)?;
    let initialize: LuaFunction = script.get("initialize")?;
    let process_qso: LuaFunction = script.get("process_qso")?;
    let summarize: Option<LuaFunction> = script.get("summarize")?;

    let contest: ContestConfig =
        span!(Level::ERROR, "initialize").in_scope(|| initialize.call(script_args))?;
    let offset = parse_offset(contest.offset.as_deref().unwrap_or(DEFAULT_OFFSET))?;

    let mut tally = Tally::default();
    let mut accepted = vec![];
    let mut lines = vec![];
    for (qso, record) in &qsos {
        let span = span!(Level::ERROR, "process_qso", call = %qso.call, datetime = %qso.datetime);
        let _enter = span.enter();

        let score: Option<QsoScore> = process_qso.call(build_entry(qso, record))?;
        let Some(score) = score else {
            continue;
        };
        lines.push(tally.add(qso, record, score)?);
        accepted.push(ContestQso { qso, record });
    }
    info!(
        "{} QSOs, {} dupes, {} points, {} multipliers",
        tally.summary.qsos, tally.summary.dupes, tally.summary.points, tally.summary.multipliers
    );

    let total_score = match summarize {
        Some(summarize) => {
            span!(Level::ERROR, "summarize").in_scope(|| summarize.call(tally.summary))?
        }
        None => tally.summary.default_total(),
    };
    let summary = build_summary(contest.clone(), &accepted, total_score, offset, config)?;
    let log = LogSheet {
        log_type: contest
            .log_type
            .unwrap_or_else(|| DEFAULT_LOG_TYPE.to_string()),
        offset,
        lines,
    };

    let mut elog = String::new();
    summary.write_to(&mut elog)?;
    log.write_to(&mut elog)?;
    let mut writer = args.output.writer()?;
    writer.write_all(elog.as_bytes())?;
    writer.flush()?;

    Ok(())
}

impl Tally {
    fn add(&mut self, qso: &QsoRecord, record: &AdifRecord, score: QsoScore) -> Result<LogLine> {
        let is_dupe = !self.keys.insert(score.key.clone());
        let is_new_multiplier = !is_dupe
            && score
                .multiplier_key()
                .is_some_and(|k| self.multiplier_keys.insert(k.to_string()));

        if is_dupe {
            warn!("duplicate QSO: {}", score.key);
            self.summary.dupes += 1;
        } else {
            self.summary.qsos += 1;
            self.summary.points += score.point;
        }
        if is_new_multiplier {
            self.summary.multipliers += 1;
        }

        let band = match score.band {
            Some(band) => band,
            None => jarl_band(qso.band)
                .ok_or_else(|| {
                    anyhow!(
                        "band {} is not in JARL log format; return band from process_qso()",
                        qso.band
                    )
                })?
                .to_string(),
        };
        let exchanges = QsoExchanges::new(record);
        let exchange_value = |value: Option<String>, adif: Option<&str>, name: &str| {
            value.or(adif.map(str::to_string)).unwrap_or_else(|| {
                warn!("{name} is missing");
                NO_VALUE.to_string()
            })
        };

        Ok(LogLine {
            datetime: qso.datetime.into(),
            band,
            mode: score.mode.unwrap_or_else(|| jarl_mode(&qso.mode)),
            call: qso.call.to_string(),
            sent_rst: exchange_value(
                score.tx_report,
                exchanges.sent.report.as_deref(),
                "sent report",
            ),
            sent_number: exchange_value(
                score.tx_number,
                exchanges.sent.number.as_deref(),
                "sent number",
            ),
            received_rst: exchange_value(
                score.rx_report,
                exchanges.received.report.as_deref(),
                "received report",
            ),
            received_number: exchange_value(
                score.rx_number,
                exchanges.received.number.as_deref(),
                "received number",
            ),
            multiplier: score.multiplier.filter(|_| is_new_multiplier),
            point: if is_dupe { 0 } else { score.point },
        })
    }
}

fn build_entry(qso: &QsoRecord, record: &AdifRecord) -> ContestEntry {
    let field = |name| get_optional_field(record, name).map(|v| v.to_compact_string());
    ContestEntry {
        qso: qso.clone().into(),
        exchange: QsoExchanges::new(record).into(),
        station_callsign: field("STATION_CALLSIGN"),
        operator: field("OPERATOR"),
        power: tx_power(record),
        contest_id: field("CONTEST_ID"),
    }
}

/// Builds summary sheet; values not given by the script are taken from QSOs and operator config.
fn build_summary(
    contest: ContestConfig,
    accepted: &[ContestQso],
    total_score: i64,
    offset: UtcOffset,
    config: &Config,
) -> Result<SummarySheet> {
    let first = accepted.first();
    let first_field = |name| first.and_then(|q| get_optional_field(q.record, name));

    let Some(callsign) = contest
        .callsign
        .or_else(|| first_field("STATION_CALLSIGN").map(str::to_string))
    else {
        bail!("callsign is unknown; return callsign from initialize()");
    };
    let operator_key = contest
        .operator
        .as_deref()
        .or(first_field("OPERATOR"))
        .unwrap_or(&callsign);
    let operator = config.operators.get(operator_key);
    if operator.is_none() {
        warn!("operator {operator_key} is not found in config");
    }
    let operator_value = |value: Option<String>, get: fn(&OperatorConfig) -> Option<&String>| {
        value.or_else(|| operator.and_then(get).cloned())
    };

    let contest_date = first.map(|q| {
        OffsetDateTime::from(q.qso.datetime)
            .to_offset(offset)
            .date()
    });
    let age = contest.age.or_else(|| {
        let birthday = operator.and_then(|o| o.birthday)?;
        Some(calculate_age(contest_date?, birthday))
    });
    let license_date = match contest.license_date {
        Some(d) => Some(parse_date(&d)?),
        None => operator.and_then(|o| o.licensed_at),
    };
    let date = match contest.date {
        Some(d) => parse_date(&d)?,
        None => OffsetDateTime::now_local()
            .unwrap_or_else(|_| OffsetDateTime::now_utc().to_offset(offset))
            .date(),
    };
    let power = contest.power.or_else(|| {
        accepted
            .iter()
            .filter_map(|q| tx_power(q.record))
            .reduce(f64::max)
    });

    let summary = SummarySheet {
        contest_name: contest.contest_name,
        category_code: contest.category_code,
        callsign,
        op_callsign: contest.op_callsign,
        total_score,
        address: operator_value(contest.address, |o| o.address.as_ref()),
        name: operator_value(contest.name, |o| Some(&o.name)),
        tel: operator_value(contest.tel, |o| o.tel.as_ref()),
        email: operator_value(contest.email, |o| o.email.as_ref()),
        license_date,
        age,
        power,
        fd_coeff: contest.fd_coeff,
        op_place: contest.op_place,
        power_supply: contest.power_supply,
        comments: contest.comments,
        multi_op_list: contest.multi_op_list,
        reg_club_number: contest.reg_club_number,
        date,
    };
    for (name, missing) in [
        ("address", summary.address.is_none()),
        ("name", summary.name.is_none()),
        ("power", summary.power.is_none()),
    ] {
        if missing {
            warn!("{name} is missing in summary sheet");
        }
    }

    Ok(summary)
}

fn tx_power(record: &AdifRecord) -> Option<f64> {
    let power = get_optional_field(record, "TX_PWR")?;
    match power.parse() {
        Ok(power) => Some(power),
        Err(e) => {
            warn!("invalid TX_PWR {power}: {e}");
            None
        }
    }
}

fn parse_date(date: &str) -> Result<Date> {
    Date::parse(date, CONFIG_DATE).with_context(|| format!("invalid date {date}"))
}

fn parse_offset(offset: &str) -> Result<UtcOffset> {
    UtcOffset::parse(offset, CONFIG_OFFSET).with_context(|| format!("invalid offset {offset}"))
}

#[cfg(test)]
mod tests {
    use adif_reader::{LengthMode, read_adi};

    use super::*;

    #[test]
    fn parses_offset() {
        let jst = UtcOffset::from_hms(9, 0, 0).unwrap();
        assert_eq!(parse_offset("+09:00").unwrap(), jst);
        assert_eq!(parse_offset("09:00").unwrap(), jst);
        assert_eq!(
            parse_offset("-05:00").unwrap(),
            UtcOffset::from_hms(-5, 0, 0).unwrap()
        );
        assert!(parse_offset("JST").is_err());
    }

    #[test]
    fn tallies_qsos() {
        let document = read_adi(
            "Test
<EOH>
<QSO_DATE:8>20260606<TIME_ON:4>1200<BAND:3>40m<FREQ:5>7.010<MODE:2>CW<CALL:5>JA1RL<RST_SENT:3>599<STX_STRING:3>09M<RST_RCVD:3>599<SRX_STRING:5>1001H<EOR>
<QSO_DATE:8>20260606<TIME_ON:4>1201<BAND:3>40m<FREQ:5>7.010<MODE:2>CW<CALL:5>JA1RL<RST_SENT:3>599<STX_STRING:3>09M<RST_RCVD:3>599<SRX_STRING:5>1001H<EOR>
<QSO_DATE:8>20260606<TIME_ON:4>1202<BAND:3>40m<FREQ:5>7.010<MODE:2>CW<CALL:6>JA1ZZZ<RST_SENT:3>599<STX_STRING:3>09M<RST_RCVD:3>599<SRX_STRING:5>1001M<EOR>
<QSO_DATE:8>20260606<TIME_ON:4>1203<BAND:4>70cm<FREQ:7>433.000<MODE:2>FM<CALL:6>JA1YYY<RST_SENT:2>59<STX_STRING:3>09M<EOR>
",
            LengthMode::Bytes,
        )
        .unwrap();
        let scores = [
            ("40m-JA1RL", "1001"),
            ("40m-JA1RL", "1001"),
            ("40m-JA1ZZZ", "1001"),
            ("70cm-JA1YYY", "1002"),
        ];

        let mut tally = Tally::default();
        let lines: Vec<_> = document
            .records()
            .iter()
            .zip(scores)
            .map(|(record, (key, multiplier))| {
                let qso = QsoRecord::new(record, UtcOffset::UTC).unwrap();
                let score = QsoScore {
                    key: key.to_string(),
                    point: 1,
                    multiplier: Some(multiplier.to_string()),
                    ..Default::default()
                };
                tally.add(&qso, record, score).unwrap()
            })
            .collect();

        assert_eq!(
            tally.summary,
            ContestSummary {
                qsos: 3,
                dupes: 1,
                points: 3,
                multipliers: 2,
            }
        );
        let columns: Vec<_> = lines
            .iter()
            .map(|l| {
                (
                    l.band.as_str(),
                    l.received_rst.as_str(),
                    l.received_number.as_str(),
                    l.multiplier.as_deref(),
                    l.point,
                )
            })
            .collect();
        assert_eq!(
            columns,
            [
                ("7", "599", "1001H", Some("1001"), 1),
                ("7", "599", "1001H", None, 0),
                ("7", "599", "1001M", None, 1),
                ("430", NO_VALUE, NO_VALUE, Some("1002"), 1),
            ]
        );
    }
}
