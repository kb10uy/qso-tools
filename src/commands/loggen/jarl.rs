use std::fmt::Write;

use anyhow::{Result, anyhow};
use time::{Date, UtcOffset, format_description::BorrowedFormatItem, macros::format_description};

use crate::{
    commands::loggen::{
        log::{ContestLog, LoggedQso},
        submission::Submission,
    },
    core::qso::band::Band,
};

const LOG_DATE: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]");
const LOG_TIME: &[BorrowedFormatItem<'_>] = format_description!("[hour]:[minute]");
const LABEL_OFFSET: &[BorrowedFormatItem<'_>] =
    format_description!("[offset_hour sign:mandatory]:[offset_minute]");

const LOG_TYPE: &str = "qso-tools";
const NO_VALUE: &str = "-";
const OATH: &str = "私は、JARL制定のコンテスト規約および電波法令にしたがい運用した結果、ここに提出するサマリーシートおよびログシートなどが事実と相違ないものであることを、私の名誉において誓います。";
const JST: UtcOffset = match UtcOffset::from_hms(9, 0, 0) {
    Ok(offset) => offset,
    Err(_) => panic!("valid offset"),
};

/// Writes JARL eLog (summary sheet R2.1 and log sheet in JARL format).
pub fn write_elog(log: &ContestLog, out: &mut impl Write) -> Result<()> {
    write_summary(&log.submission, out)?;
    write_log(&log.qsos, log.offset, out)
}

fn write_summary(submission: &Submission, out: &mut impl Write) -> Result<()> {
    writeln!(out, "<SUMMARYSHEET VERSION=R2.1>")?;
    write_tag(out, "CONTESTNAME", Some(&submission.contest_name))?;
    write_tag(out, "CATEGORYCODE", Some(&submission.category_code))?;
    write_tag(out, "CALLSIGN", Some(&submission.callsign))?;
    write_tag(out, "OPCALLSIGN", submission.op_callsign.as_ref())?;
    write_tag(out, "TOTALSCORE", Some(submission.total_score))?;
    write_tag(out, "ADDRESS", submission.address.as_ref())?;
    write_tag(out, "NAME", submission.name.as_ref())?;
    write_tag(out, "TEL", submission.tel.as_ref())?;
    write_tag(out, "EMAIL", submission.email.as_ref())?;
    write_tag(
        out,
        "LICENSEDATE",
        submission.license_date.map(japanese_date),
    )?;
    write_tag(out, "AGE", submission.age)?;
    write_tag(out, "POWER", submission.power)?;
    write_tag(out, "FDCOEFF", submission.fd_coeff)?;
    write_tag(out, "OPPLACE", submission.op_place.as_ref())?;
    write_tag(out, "POWERSUPPLY", submission.power_supply.as_ref())?;
    write_tag(out, "COMMENTS", submission.comments.as_ref())?;
    write_tag(out, "MULTIOPLIST", submission.multi_op_list.as_ref())?;
    write_tag(out, "REGCLUBNUMBER", submission.reg_club_number.as_ref())?;
    write_tag(out, "OATH", Some(OATH))?;
    write_tag(out, "DATE", Some(japanese_date(submission.date)))?;
    write_tag(out, "SIGNATURE", submission.name.as_ref())?;
    writeln!(out, "</SUMMARYSHEET>")?;
    Ok(())
}

fn write_log(qsos: &[LoggedQso], offset: UtcOffset, out: &mut impl Write) -> Result<()> {
    let header = [
        format!("DATE({})", offset_label(offset)),
        "TIME".to_string(),
        "BAND".to_string(),
        "MODE".to_string(),
        "CALLSIGN".to_string(),
        "SENTNo".to_string(),
        "RCVDNo".to_string(),
        "Multi".to_string(),
        "PTS".to_string(),
    ];
    let rows: Vec<_> = qsos
        .iter()
        .map(|q| columns(q, offset))
        .collect::<Result<_>>()?;

    let mut widths = header.each_ref().map(String::len);
    for row in &rows {
        for (width, column) in widths.iter_mut().zip(row) {
            *width = (*width).max(column.len());
        }
    }

    writeln!(out, "<LOGSHEET TYPE=\"{LOG_TYPE}\">")?;
    for row in [&header].into_iter().chain(&rows) {
        let padded: Vec<_> = row
            .iter()
            .zip(widths)
            .map(|(c, w)| format!("{c:w$}"))
            .collect();
        writeln!(out, "{}", padded.join(" ").trim_end())?;
    }
    writeln!(out, "</LOGSHEET>")?;
    Ok(())
}

fn columns(qso: &LoggedQso, offset: UtcOffset) -> Result<[String; 9]> {
    let datetime = qso.datetime.to_offset(offset);
    let band = jarl_band(qso.band)
        .ok_or_else(|| anyhow!("band {} is not in JARL log format", qso.band))?;
    let value = |v: &Option<String>| v.clone().unwrap_or_else(|| NO_VALUE.to_string());
    let multiplier = match qso.multipliers.as_slice() {
        [] => NO_VALUE.to_string(),
        multipliers => multipliers.join(","),
    };

    Ok([
        datetime.format(LOG_DATE)?,
        datetime.format(LOG_TIME)?,
        band.to_string(),
        jarl_mode(&qso.mode),
        qso.call.to_string(),
        format!("{} {}", value(&qso.sent.report), value(&qso.sent.number)),
        format!(
            "{} {}",
            value(&qso.received.report),
            value(&qso.received.number)
        ),
        multiplier,
        qso.point.to_string(),
    ])
}

/// Returns band notation of JARL log format, or `None` if the band is not used in Japan.
fn jarl_band(band: Band) -> Option<&'static str> {
    match band {
        Band::Meter2190 => Some("0.135"),
        Band::Meter630 => Some("0.475"),
        Band::Meter160 => Some("1.9"),
        Band::Meter80 => Some("3.5"),
        Band::Meter40 => Some("7"),
        Band::Meter20 => Some("14"),
        Band::Meter17 => Some("18"),
        Band::Meter15 => Some("21"),
        Band::Meter12 => Some("24"),
        Band::Meter10 => Some("28"),
        Band::Meter6 => Some("50"),
        Band::Meter2 => Some("144"),
        Band::Centimeter70 => Some("430"),
        Band::Centimeter23 => Some("1200"),
        Band::Centimeter13 => Some("2400"),
        Band::Centimeter6 => Some("5600"),
        Band::Centimeter3 => Some("10G"),
        Band::Centimeter1P25 => Some("24G"),
        Band::Millimeter6 => Some("47G"),
        Band::Millimeter4 => Some("77G"),
        Band::Millimeter2 => Some("135G"),
        Band::Millimeter1 => Some("248G"),
        _ => None,
    }
}

/// Returns mode notation of JARL log format from ADIF mode.
fn jarl_mode(mode: &str) -> String {
    match mode.to_ascii_uppercase().as_str() {
        "DIGITALVOICE" => "DV".to_string(),
        otherwise => otherwise.to_string(),
    }
}

fn japanese_date(date: Date) -> String {
    format!("{}年{}月{}日", date.year(), date.month() as u8, date.day())
}

fn offset_label(offset: UtcOffset) -> String {
    if offset == UtcOffset::UTC {
        "UTC".to_string()
    } else if offset == JST {
        "JST".to_string()
    } else {
        let formatted = offset.format(LABEL_OFFSET);
        format!("UTC{}", formatted.unwrap_or_else(|_| "?".to_string()))
    }
}

/// Writes a tag line; empty values are omitted as inapplicable items.
fn write_tag(out: &mut impl Write, tag: &str, value: Option<impl ToString>) -> Result<()> {
    if let Some(value) = value.map(|v| v.to_string()).filter(|v| !v.is_empty()) {
        writeln!(out, "<{tag}>{value}</{tag}>")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use time::macros::{date, datetime};

    use super::*;
    use crate::commands::loggen::log::LoggedExchange;

    fn submission() -> Submission {
        Submission {
            contest_name: "ALL JA1".to_string(),
            contest_id: None,
            category_code: "XMA".to_string(),
            category_name: "Mixed".to_string(),
            cabrillo_categories: vec![],
            callsign: "JL1HIS".to_string(),
            op_callsign: None,
            total_score: 1200,
            address: Some("Tochigi".to_string()),
            name: Some("Name".to_string()),
            tel: Some(String::new()),
            email: Some("me@example.com".to_string()),
            license_date: Some(date!(2025 - 09 - 19)),
            age: None,
            power: Some(25.0),
            fd_coeff: None,
            op_place: None,
            power_supply: None,
            comments: None,
            multi_op_list: None,
            reg_club_number: None,
            date: date!(2026 - 06 - 07),
        }
    }

    fn qso(
        datetime: time::OffsetDateTime,
        band: Band,
        call: &str,
        multipliers: &[&str],
        point: i64,
    ) -> LoggedQso {
        LoggedQso {
            datetime,
            band,
            frequency: 7.0,
            mode: "CW".into(),
            submode: None,
            call: call.into(),
            station_callsign: None,
            sent: LoggedExchange {
                report: Some("599".to_string()),
                number: Some("09M".to_string()),
            },
            received: LoggedExchange {
                report: Some("599".to_string()),
                number: Some("1001H".to_string()),
            },
            multipliers: multipliers.iter().map(|m| m.to_string()).collect(),
            point,
            dupe: point == 0,
        }
    }

    #[test]
    fn writes_summary_sheet() {
        let mut out = String::new();
        write_summary(&submission(), &mut out).unwrap();
        assert_eq!(
            out,
            format!(
                "<SUMMARYSHEET VERSION=R2.1>
<CONTESTNAME>ALL JA1</CONTESTNAME>
<CATEGORYCODE>XMA</CATEGORYCODE>
<CALLSIGN>JL1HIS</CALLSIGN>
<TOTALSCORE>1200</TOTALSCORE>
<ADDRESS>Tochigi</ADDRESS>
<NAME>Name</NAME>
<EMAIL>me@example.com</EMAIL>
<LICENSEDATE>2025年9月19日</LICENSEDATE>
<POWER>25</POWER>
<OATH>{OATH}</OATH>
<DATE>2026年6月7日</DATE>
<SIGNATURE>Name</SIGNATURE>
</SUMMARYSHEET>
"
            )
        );
    }

    #[test]
    fn writes_log_sheet() {
        let qsos = [
            qso(
                datetime!(2026-06-06 12:00 UTC),
                Band::Meter40,
                "JA1RL",
                &["1001"],
                1,
            ),
            qso(
                datetime!(2026-06-06 15:05 UTC),
                Band::Centimeter70,
                "JA1ZZZ",
                &[],
                0,
            ),
            qso(
                datetime!(2026-06-06 15:06 UTC),
                Band::Meter20,
                "JA1YYY",
                &["1", "W"],
                1,
            ),
        ];

        let mut out = String::new();
        write_log(&qsos, JST, &mut out).unwrap();
        assert_eq!(
            out,
            "<LOGSHEET TYPE=\"qso-tools\">
DATE(JST)  TIME  BAND MODE CALLSIGN SENTNo  RCVDNo    Multi PTS
2026-06-06 21:00 7    CW   JA1RL    599 09M 599 1001H 1001  1
2026-06-07 00:05 430  CW   JA1ZZZ   599 09M 599 1001H -     0
2026-06-07 00:06 14   CW   JA1YYY   599 09M 599 1001H 1,W   1
</LOGSHEET>
"
        );

        let invalid = [qso(
            datetime!(2026-06-06 12:00 UTC),
            Band::Meter5,
            "JA1RL",
            &[],
            1,
        )];
        assert!(write_log(&invalid, JST, &mut String::new()).is_err());
    }

    #[test]
    fn labels_offset() {
        assert_eq!(offset_label(JST), "JST");
        assert_eq!(offset_label(UtcOffset::UTC), "UTC");
        assert_eq!(
            offset_label(UtcOffset::from_hms(-5, 0, 0).unwrap()),
            "UTC-05:00"
        );
    }
}
