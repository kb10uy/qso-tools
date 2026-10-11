use std::fmt::Write;

use anyhow::Result;
use time::{format_description::BorrowedFormatItem, macros::format_description};
use tracing::warn;

use crate::commands::loggen::log::{ContestLog, LoggedQso, align_columns};

const QSO_DATE: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]");
const QSO_TIME: &[BorrowedFormatItem<'_>] = format_description!("[hour][minute]");
const CREATED_BY: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));
const NO_VALUE: &str = "-";

/// Writes Cabrillo 3.0 log.
pub fn write_cabrillo(log: &ContestLog, out: &mut impl Write) -> Result<()> {
    let submission = &log.submission;
    let contest = submission.contest_id.as_deref().unwrap_or_else(|| {
        warn!("contest_id is not defined; CONTEST is written with contest name");
        &submission.contest_name
    });
    let operators = submission
        .multi_op_list
        .as_ref()
        .or(submission.op_callsign.as_ref());

    writeln!(out, "START-OF-LOG: 3.0")?;
    write_header(out, "CALLSIGN", Some(&submission.callsign))?;
    write_header(out, "CONTEST", Some(contest))?;
    for (key, value) in &submission.cabrillo_categories {
        write_header(
            out,
            &format!("CATEGORY-{}", key.to_ascii_uppercase()),
            Some(value),
        )?;
    }
    write_header(out, "CLAIMED-SCORE", Some(submission.total_score))?;
    write_header(out, "OPERATORS", operators)?;
    write_header(out, "NAME", submission.name.as_ref())?;
    write_header(out, "ADDRESS", submission.address.as_ref())?;
    write_header(out, "EMAIL", submission.email.as_ref())?;
    write_header(out, "CREATED-BY", Some(CREATED_BY))?;
    write_header(out, "SOAPBOX", submission.comments.as_ref())?;

    let rows: Vec<_> = log
        .qsos
        .iter()
        .map(|q| columns(q, &submission.callsign))
        .collect::<Result<_>>()?;
    for line in align_columns(&rows, &[0]) {
        writeln!(out, "QSO: {line}")?;
    }
    writeln!(out, "END-OF-LOG:")?;
    Ok(())
}

fn columns(qso: &LoggedQso, callsign: &str) -> Result<[String; 10]> {
    let datetime = qso.datetime.to_offset(time::UtcOffset::UTC);
    let value = |v: &Option<String>| v.clone().unwrap_or_else(|| NO_VALUE.to_string());
    Ok([
        format!("{}", (qso.frequency * 1000.0).round() as u64),
        cabrillo_mode(&qso.mode).to_string(),
        datetime.format(QSO_DATE)?,
        datetime.format(QSO_TIME)?,
        qso.station_callsign
            .as_deref()
            .unwrap_or(callsign)
            .to_string(),
        value(&qso.sent.report),
        value(&qso.sent.number),
        qso.call.to_string(),
        value(&qso.received.report),
        value(&qso.received.number),
    ])
}

/// Returns Cabrillo mode from ADIF mode.
fn cabrillo_mode(mode: &str) -> &'static str {
    match mode.to_ascii_uppercase().as_str() {
        "CW" => "CW",
        "SSB" | "AM" => "PH",
        "FM" => "FM",
        "RTTY" => "RY",
        _ => "DG",
    }
}

fn write_header(out: &mut impl Write, tag: &str, value: Option<impl ToString>) -> Result<()> {
    if let Some(value) = value.map(|v| v.to_string()).filter(|v| !v.is_empty()) {
        writeln!(out, "{tag}: {value}")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use time::{
        UtcOffset,
        macros::{date, datetime},
    };

    use super::*;
    use crate::{
        commands::loggen::{log::LoggedExchange, submission::Submission},
        core::qso::band::Band,
    };

    fn qso(
        datetime: time::OffsetDateTime,
        frequency: f64,
        mode: &str,
        call: &str,
        number: Option<&str>,
    ) -> LoggedQso {
        LoggedQso {
            datetime,
            band: Band::Meter40,
            frequency,
            mode: mode.into(),
            submode: None,
            call: call.into(),
            station_callsign: None,
            sent: LoggedExchange {
                report: Some("599".to_string()),
                number: Some("15006M".to_string()),
            },
            received: LoggedExchange {
                report: Some("599".to_string()),
                number: number.map(str::to_string),
            },
            multipliers: vec![],
            point: 1,
            dupe: false,
        }
    }

    #[test]
    fn writes_cabrillo() {
        let log = ContestLog {
            submission: Submission {
                contest_name: "全市全郡".to_string(),
                contest_id: Some("JA-ACAG".to_string()),
                category_code: "XAM".to_string(),
                category_name: "Mixed".to_string(),
                cabrillo_categories: vec![
                    ("operator".to_string(), "SINGLE-OP".to_string()),
                    ("power".to_string(), "LOW".to_string()),
                ],
                callsign: "JL1HIS".to_string(),
                op_callsign: None,
                total_score: 9,
                address: None,
                name: Some("Yu Kobayashi".to_string()),
                tel: None,
                email: Some("me@example.com".to_string()),
                license_date: None,
                age: None,
                power: Some(50.0),
                fd_coeff: None,
                op_place: None,
                power_supply: None,
                comments: Some(String::new()),
                multi_op_list: None,
                reg_club_number: None,
                date: date!(2026 - 10 - 12),
            },
            offset: UtcOffset::from_hms(9, 0, 0).unwrap(),
            qsos: vec![
                qso(
                    datetime!(2026-10-11 12:00 UTC),
                    7.0123,
                    "CW",
                    "JA1RL",
                    Some("100116H"),
                ),
                qso(
                    datetime!(2026-10-11 12:05 UTC),
                    144.21,
                    "SSB",
                    "JA1ZZZ",
                    None,
                ),
            ],
        };

        let mut out = String::new();
        write_cabrillo(&log, &mut out).unwrap();
        assert_eq!(
            out,
            format!(
                "START-OF-LOG: 3.0
CALLSIGN: JL1HIS
CONTEST: JA-ACAG
CATEGORY-OPERATOR: SINGLE-OP
CATEGORY-POWER: LOW
CLAIMED-SCORE: 9
NAME: Yu Kobayashi
EMAIL: me@example.com
CREATED-BY: {CREATED_BY}
QSO:   7012 CW 2026-10-11 1200 JL1HIS 599 15006M JA1RL  599 100116H
QSO: 144210 PH 2026-10-11 1205 JL1HIS 599 15006M JA1ZZZ 599 -
END-OF-LOG:
"
            )
        );
    }

    #[test]
    fn maps_mode() {
        assert_eq!(cabrillo_mode("ssb"), "PH");
        assert_eq!(cabrillo_mode("RTTY"), "RY");
        assert_eq!(cabrillo_mode("FT8"), "DG");
    }
}
