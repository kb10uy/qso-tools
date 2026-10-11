use std::fmt::{Result as FmtResult, Write};

use time::{
    Date, OffsetDateTime, UtcOffset, format_description::BorrowedFormatItem,
    macros::format_description,
};

use crate::core::qso::band::Band;

const LOG_DATE: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]");
const LOG_TIME: &[BorrowedFormatItem<'_>] = format_description!("[hour]:[minute]");
const LABEL_OFFSET: &[BorrowedFormatItem<'_>] =
    format_description!("[offset_hour sign:mandatory]:[offset_minute]");

const OATH: &str = "私は、JARL制定のコンテスト規約および電波法令にしたがい運用した結果、ここに提出するサマリーシートおよびログシートなどが事実と相違ないものであることを、私の名誉において誓います。";
const JST: UtcOffset = match UtcOffset::from_hms(9, 0, 0) {
    Ok(offset) => offset,
    Err(_) => panic!("valid offset"),
};

/// Summary sheet of JARL eLog R2.1.
#[derive(Debug, Clone, PartialEq)]
pub struct SummarySheet {
    pub contest_name: String,
    pub category_code: String,
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

/// Log sheet of JARL eLog in JARL log format.
#[derive(Debug, Clone, PartialEq)]
pub struct LogSheet {
    pub log_type: String,
    pub offset: UtcOffset,
    pub lines: Vec<LogLine>,
}

/// QSO line of JARL log format.
#[derive(Debug, Clone, PartialEq)]
pub struct LogLine {
    pub datetime: OffsetDateTime,
    pub band: String,
    pub mode: String,
    pub call: String,
    pub sent_rst: String,
    pub sent_number: String,
    pub received_rst: String,
    pub received_number: String,
    pub multiplier: Option<String>,
    pub point: i64,
}

impl SummarySheet {
    pub fn write_to(&self, out: &mut impl Write) -> FmtResult {
        writeln!(out, "<SUMMARYSHEET VERSION=R2.1>")?;
        write_tag(out, "CONTESTNAME", Some(&self.contest_name))?;
        write_tag(out, "CATEGORYCODE", Some(&self.category_code))?;
        write_tag(out, "CALLSIGN", Some(&self.callsign))?;
        write_tag(out, "OPCALLSIGN", self.op_callsign.as_ref())?;
        write_tag(out, "TOTALSCORE", Some(self.total_score))?;
        write_tag(out, "ADDRESS", self.address.as_ref())?;
        write_tag(out, "NAME", self.name.as_ref())?;
        write_tag(out, "TEL", self.tel.as_ref())?;
        write_tag(out, "EMAIL", self.email.as_ref())?;
        write_tag(out, "LICENSEDATE", self.license_date.map(japanese_date))?;
        write_tag(out, "AGE", self.age)?;
        write_tag(out, "POWER", self.power)?;
        write_tag(out, "FDCOEFF", self.fd_coeff)?;
        write_tag(out, "OPPLACE", self.op_place.as_ref())?;
        write_tag(out, "POWERSUPPLY", self.power_supply.as_ref())?;
        write_tag(out, "COMMENTS", self.comments.as_ref())?;
        write_tag(out, "MULTIOPLIST", self.multi_op_list.as_ref())?;
        write_tag(out, "REGCLUBNUMBER", self.reg_club_number.as_ref())?;
        write_tag(out, "OATH", Some(OATH))?;
        write_tag(out, "DATE", Some(japanese_date(self.date)))?;
        write_tag(out, "SIGNATURE", self.name.as_ref())?;
        writeln!(out, "</SUMMARYSHEET>")
    }
}

impl LogSheet {
    pub fn write_to(&self, out: &mut impl Write) -> FmtResult {
        let header = [
            format!("DATE({})", offset_label(self.offset)),
            "TIME".to_string(),
            "BAND".to_string(),
            "MODE".to_string(),
            "CALLSIGN".to_string(),
            "SENTNo".to_string(),
            "RCVDNo".to_string(),
            "Multi".to_string(),
            "PTS".to_string(),
        ];
        let rows: Vec<_> = self
            .lines
            .iter()
            .map(|l| l.columns(self.offset))
            .collect::<Result<_, _>>()
            .map_err(|_| std::fmt::Error)?;

        let mut widths = header.each_ref().map(String::len);
        for row in &rows {
            for (width, column) in widths.iter_mut().zip(row) {
                *width = (*width).max(column.len());
            }
        }

        writeln!(out, "<LOGSHEET TYPE=\"{}\">", self.log_type)?;
        for row in [&header].into_iter().chain(&rows) {
            let padded: Vec<_> = row
                .iter()
                .zip(widths)
                .map(|(c, w)| format!("{c:w$}"))
                .collect();
            writeln!(out, "{}", padded.join(" ").trim_end())?;
        }
        writeln!(out, "</LOGSHEET>")
    }
}

impl LogLine {
    fn columns(&self, offset: UtcOffset) -> Result<[String; 9], time::error::Format> {
        let datetime = self.datetime.to_offset(offset);
        Ok([
            datetime.format(LOG_DATE)?,
            datetime.format(LOG_TIME)?,
            self.band.clone(),
            self.mode.clone(),
            self.call.clone(),
            format!("{} {}", self.sent_rst, self.sent_number),
            format!("{} {}", self.received_rst, self.received_number),
            self.multiplier.clone().unwrap_or_else(|| "-".to_string()),
            self.point.to_string(),
        ])
    }
}

/// Returns band notation of JARL log format, or `None` if the band is not used in Japan.
pub fn jarl_band(band: Band) -> Option<&'static str> {
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
pub fn jarl_mode(mode: &str) -> String {
    match mode.to_ascii_uppercase().as_str() {
        "DIGITALVOICE" => "DV".to_string(),
        otherwise => otherwise.to_string(),
    }
}

/// Calculates age at `today`.
pub fn calculate_age(today: Date, birthday: Date) -> u32 {
    let had_birthday = (today.month(), today.day()) >= (birthday.month(), birthday.day());
    let age = today.year() - birthday.year() - if had_birthday { 0 } else { 1 };
    age.max(0) as u32
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
fn write_tag(out: &mut impl Write, tag: &str, value: Option<impl ToString>) -> FmtResult {
    match value.map(|v| v.to_string()).filter(|v| !v.is_empty()) {
        Some(value) => writeln!(out, "<{tag}>{value}</{tag}>"),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use time::macros::{date, datetime};

    use super::*;

    #[test]
    fn calculates_age() {
        let birthday = date!(1998 - 06 - 03);
        assert_eq!(calculate_age(date!(2026 - 06 - 02), birthday), 27);
        assert_eq!(calculate_age(date!(2026 - 06 - 03), birthday), 28);
        assert_eq!(calculate_age(date!(2026 - 10 - 11), birthday), 28);
        assert_eq!(calculate_age(date!(1990 - 01 - 01), birthday), 0);
    }

    #[test]
    fn writes_summary_sheet() {
        let summary = SummarySheet {
            contest_name: "ALL JA1".to_string(),
            category_code: "XMA".to_string(),
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
        };

        let mut out = String::new();
        summary.write_to(&mut out).unwrap();
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
        let line = |datetime, band: &str, call: &str, multiplier: Option<&str>, point| LogLine {
            datetime,
            band: band.to_string(),
            mode: "CW".to_string(),
            call: call.to_string(),
            sent_rst: "599".to_string(),
            sent_number: "09M".to_string(),
            received_rst: "599".to_string(),
            received_number: "1001H".to_string(),
            multiplier: multiplier.map(str::to_string),
            point,
        };
        let log = LogSheet {
            log_type: "qso-tools".to_string(),
            offset: JST,
            lines: vec![
                line(
                    datetime!(2026-06-06 12:00 UTC),
                    "7",
                    "JA1RL",
                    Some("1001"),
                    1,
                ),
                line(datetime!(2026-06-06 15:05 UTC), "430", "JA1ZZZ", None, 0),
            ],
        };

        let mut out = String::new();
        log.write_to(&mut out).unwrap();
        assert_eq!(
            out,
            "<LOGSHEET TYPE=\"qso-tools\">
DATE(JST)  TIME  BAND MODE CALLSIGN SENTNo  RCVDNo    Multi PTS
2026-06-06 21:00 7    CW   JA1RL    599 09M 599 1001H 1001  1
2026-06-07 00:05 430  CW   JA1ZZZ   599 09M 599 1001H -     0
</LOGSHEET>
"
        );
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
