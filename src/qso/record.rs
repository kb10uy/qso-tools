use adif_reader::document::Record;
use compact_str::{CompactString, ToCompactString};
use time::{
    Date, OffsetDateTime, Time, UtcDateTime, UtcOffset, format_description::BorrowedFormatItem,
    macros::format_description,
};

use crate::qso::{band::Band, error::QsoError, get_required_field};

const ADIF_DATE: &[BorrowedFormatItem<'_>] = format_description!(
    "[year repr:full padding:zero][month repr:numerical padding:zero][day padding:zero]"
);
const ADIF_TIME: &[BorrowedFormatItem<'_>] =
    format_description!("[hour repr:24 padding:zero][minute padding:zero][second padding:zero]");
const ADIF_TIME_SHORT: &[BorrowedFormatItem<'_>] =
    format_description!("[hour repr:24 padding:zero][minute padding:zero]");

#[derive(Debug, Clone, PartialEq)]
pub struct QsoRecord {
    pub datetime: UtcDateTime,
    pub band: Band,
    pub mode: CompactString,
    pub call: CompactString,
    pub frequency: f64,
    pub frequency_str: CompactString,
}

impl QsoRecord {
    pub fn new(record: &Record, offset: UtcOffset) -> Result<QsoRecord, QsoError> {
        let time_on = get_required_field(record, "TIME_ON")?;
        let time = match time_on.len() {
            4 => Time::parse(time_on, ADIF_TIME_SHORT)?,
            _ => Time::parse(time_on, ADIF_TIME)?,
        };
        let datetime = OffsetDateTime::new_in_offset(
            Date::parse(get_required_field(record, "QSO_DATE")?, ADIF_DATE)?,
            time,
            offset,
        )
        .to_utc();

        let band = get_required_field(record, "BAND")?;
        let mode = get_required_field(record, "MODE")?;
        let call = get_required_field(record, "CALL")?;
        let frequency_str = get_required_field(record, "FREQ")?;

        Ok(QsoRecord {
            datetime,
            band: band.to_ascii_lowercase().parse()?,
            mode: mode.to_compact_string(),
            call: call.to_compact_string(),
            frequency: frequency_str.parse()?,
            frequency_str: frequency_str.to_compact_string(),
        })
    }
}
