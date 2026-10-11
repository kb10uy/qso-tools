mod contest;
mod definition;
mod jarl;
mod log;
mod period;
mod submission;
mod tally;

use std::{
    collections::{BTreeMap, HashMap},
    io::Write,
    path::{PathBuf, absolute},
    sync::Arc,
};

use adif_reader::document::Record as AdifRecord;
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use compact_str::ToCompactString;
use mlua::prelude::*;
use time::{UtcOffset, format_description::BorrowedFormatItem, macros::format_description};
use tracing::{Level, info, span, warn};

use crate::{
    commands::loggen::{
        contest::{ContestEntry, QsoScore},
        definition::ContestDefinition,
        log::ContestLog,
        period::{ContestTime, Period},
        submission::{SubmissionSource, TerminalPrompter, select_category},
        tally::{Multiplier, Tally},
    },
    core::{
        adif::AdifInput,
        config::Config,
        cty::load_cty,
        jcx::Jcx,
        output::OutputFile,
        qso::{exchange::QsoExchanges, get_optional_field, record::QsoRecord},
        schope::{
            args::ScriptArg,
            engine::{load_script_with, register_reference_modules},
        },
    },
};

const OFFSET: &[BorrowedFormatItem<'_>] = format_description!("[offset_hour]:[offset_minute]");
const DEFAULT_OFFSET: &str = "+09:00";

/// Generates contest logs through a Lua script.
#[derive(Debug, Clone, Args)]
pub struct Arguments {
    #[command(subcommand)]
    pub format: LogFormat,
}

#[derive(Debug, Clone, Subcommand)]
pub enum LogFormat {
    /// Generates JARL eLog (summary sheet R2.1 and log sheet).
    Jarl(CommonArguments),
}

/// Arguments shared by log formats.
#[derive(Debug, Clone, Args)]
pub struct CommonArguments {
    /// Contest script file.
    pub script_path: PathBuf,

    #[command(flatten)]
    pub input: AdifInput,

    /// Specify arguments passed to script as KEY or KEY=VALUE.
    /// Category and summary items are also given by this.
    #[arg(short = 'A', long = "args", value_name = "KEY[=VALUE]")]
    pub script_args: Vec<ScriptArg>,

    /// Accept QSOs at or after the datetime like 2026-06-06T21:00 or 2026-06-06T12:00Z.
    /// Datetime without offset is in the log offset of the script.
    #[arg(long, value_name = "DATETIME")]
    pub qso_since: Option<ContestTime>,

    /// Accept QSOs before the datetime.
    #[arg(long, value_name = "DATETIME")]
    pub qso_until: Option<ContestTime>,

    #[command(flatten)]
    pub output: OutputFile,
}

pub fn run(args: Arguments, config: &Config) -> Result<()> {
    match args.format {
        LogFormat::Jarl(args) => {
            let log = generate(&args, config)?;
            let mut text = String::new();
            jarl::write_elog(&log, &mut text)?;
            write_output(&args.output, &text)
        }
    }
}

fn generate(args: &CommonArguments, config: &Config) -> Result<ContestLog> {
    let script_path = absolute(&args.script_path)
        .with_context(|| format!("invalid script path {}", args.script_path.display()))?;
    let script_args: HashMap<_, _> = args
        .script_args
        .iter()
        .map(|a| (a.key.clone(), a.value.clone()))
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

    let jcx = Arc::new(Jcx::load(config)?);
    let cty = load_cty(config)?.map(Arc::new);
    let (lua, script) = load_script_with(&script_path, |lua| {
        register_reference_modules(lua, jcx.clone(), cty)
    })?;
    let definition = ContestDefinition::from_lua(LuaValue::Table(script), &lua)?;
    let offset = parse_offset(definition.offset.as_deref().unwrap_or(DEFAULT_OFFSET))?;
    let period = Period {
        since: args.qso_since.map(|t| t.resolve(offset)),
        until: args.qso_until.map(|t| t.resolve(offset)),
    };

    let mut prompter = TerminalPrompter::default();
    let category = select_category(&definition, &script_args, &mut prompter)?;
    let context = lua.create_table()?;
    context.set("category", &category.table)?;
    context.set("args", script_args.clone())?;

    let mut tally = Tally::default();
    let mut excluded: BTreeMap<String, u64> = BTreeMap::new();
    let mut accepted = vec![];
    let mut logged = vec![];
    for (qso, record) in &qsos {
        let span = span!(Level::ERROR, "score_qso", call = %qso.call, datetime = %qso.datetime);
        let _enter = span.enter();

        if !period.contains(qso.datetime.into()) {
            *excluded.entry("out of period".to_string()).or_default() += 1;
            continue;
        }
        if !definition.accepts(category, qso) {
            *excluded.entry("out of category".to_string()).or_default() += 1;
            continue;
        }

        let entry = build_entry(qso, record).into_lua(&lua)?;
        let (score, reason): (Option<QsoScore>, Option<String>) = definition
            .score_qso
            .call((entry.clone(), context.clone()))?;
        let Some(score) = score else {
            let reason = reason.unwrap_or_else(|| "excluded by script".to_string());
            *excluded.entry(reason).or_default() += 1;
            continue;
        };
        if let Some(reason) = reason {
            warn!("{reason}");
        }

        let dupe_key = match &definition.dupe_key {
            Some(dupe_key) => dupe_key.call((entry.clone(), context.clone()))?,
            None => definition.scope_key(&definition.dupe_scope, qso, &qso.call),
        };
        let multipliers = score
            .multipliers
            .iter()
            .map(|(kind, value)| {
                let key = match &definition.multiplier_key {
                    Some(multiplier_key) => multiplier_key.call((
                        entry.clone(),
                        kind.as_str(),
                        value.as_str(),
                        context.clone(),
                    ))?,
                    None => definition.scope_key(&definition.multiplier_scope, qso, value),
                };
                Ok(Multiplier {
                    kind: kind.clone(),
                    value: value.clone(),
                    key,
                })
            })
            .collect::<LuaResult<Vec<_>>>()?;

        logged.push(tally.add(qso, record, score, dupe_key, multipliers));
        accepted.push(*record);
    }

    for (reason, count) in &excluded {
        info!("excluded {count} QSOs: {reason}");
    }
    let summary = tally.summary();
    info!(
        "{} QSOs, {} dupes, {} points, {} multipliers",
        summary.qsos, summary.dupes, summary.points, summary.multipliers
    );
    if logged.is_empty() {
        warn!("no QSO is accepted");
    }

    let total_score = match &definition.total {
        Some(total) => span!(Level::ERROR, "total")
            .in_scope(|| total.call((summary.clone(), context.clone())))?,
        None => summary.default_total(),
    };
    let contest_date = period
        .since
        .or(logged.first().map(|q| q.datetime))
        .map(|d| d.to_offset(offset).date());
    let submission = SubmissionSource {
        definition: &definition,
        category,
        args: &script_args,
        config,
        records: &accepted,
        jcx: &jcx,
        contest_date,
        offset,
    }
    .build(total_score, &mut prompter)?;

    Ok(ContestLog {
        submission,
        offset,
        qsos: logged,
    })
}

fn build_entry(qso: &QsoRecord, record: &AdifRecord) -> ContestEntry {
    let field = |name| get_optional_field(record, name).map(|v| v.to_compact_string());
    ContestEntry {
        qso: qso.clone().into(),
        exchange: QsoExchanges::new(record).into(),
        station_callsign: field("STATION_CALLSIGN"),
        operator: field("OPERATOR"),
        power: get_optional_field(record, "TX_PWR").and_then(|p| p.parse().ok()),
        contest_id: field("CONTEST_ID"),
        name: field("NAME"),
    }
}

fn write_output(output: &OutputFile, text: &str) -> Result<()> {
    let mut writer = output.writer()?;
    writer.write_all(text.as_bytes())?;
    writer.flush()?;
    Ok(())
}

fn parse_offset(offset: &str) -> Result<UtcOffset> {
    UtcOffset::parse(offset, OFFSET).with_context(|| format!("invalid offset {offset}"))
}

#[cfg(test)]
mod tests {
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
}
