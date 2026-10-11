use std::str::FromStr;

use time::{
    OffsetDateTime, PrimitiveDateTime, UtcOffset,
    error::Parse as TimeParseError,
    format_description::{BorrowedFormatItem, well_known::Rfc3339},
    macros::format_description,
};

const OFFSET_WITHOUT_SECONDS: &[BorrowedFormatItem<'_>] = format_description!(
    "[year]-[month]-[day]T[hour]:[minute][offset_hour sign:mandatory]:[offset_minute]"
);
const LOCAL: &[BorrowedFormatItem<'_>] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]");
const LOCAL_WITHOUT_SECONDS: &[BorrowedFormatItem<'_>] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]");

/// Datetime given from the command line; a datetime without offset is resolved in the log offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContestTime {
    Offset(OffsetDateTime),
    Local(PrimitiveDateTime),
}

impl FromStr for ContestTime {
    type Err = TimeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(datetime) = OffsetDateTime::parse(s, &Rfc3339) {
            return Ok(ContestTime::Offset(datetime));
        }
        if let Ok(datetime) = OffsetDateTime::parse(s, OFFSET_WITHOUT_SECONDS) {
            return Ok(ContestTime::Offset(datetime));
        }
        if let Ok(datetime) = PrimitiveDateTime::parse(s, LOCAL) {
            return Ok(ContestTime::Local(datetime));
        }
        PrimitiveDateTime::parse(s, LOCAL_WITHOUT_SECONDS).map(ContestTime::Local)
    }
}

impl ContestTime {
    pub fn resolve(self, offset: UtcOffset) -> OffsetDateTime {
        match self {
            ContestTime::Offset(datetime) => datetime,
            ContestTime::Local(datetime) => datetime.assume_offset(offset),
        }
    }
}

/// Contest period; `since` is inclusive and `until` is exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Period {
    pub since: Option<OffsetDateTime>,
    pub until: Option<OffsetDateTime>,
}

impl Period {
    pub fn contains(&self, datetime: OffsetDateTime) -> bool {
        self.since.is_none_or(|s| s <= datetime) && self.until.is_none_or(|u| datetime < u)
    }
}

#[cfg(test)]
mod tests {
    use time::macros::{datetime, offset};

    use super::*;

    #[test]
    fn parses_contest_time() {
        let jst = offset!(+9);
        let parse = |s: &str| s.parse::<ContestTime>().unwrap().resolve(jst);
        assert_eq!(
            parse("2026-06-06T21:00:00+09:00"),
            datetime!(2026-06-06 12:00 UTC)
        );
        assert_eq!(
            parse("2026-06-06T12:00:00Z"),
            datetime!(2026-06-06 12:00 UTC)
        );
        assert_eq!(
            parse("2026-06-06T21:00+09:00"),
            datetime!(2026-06-06 12:00 UTC)
        );
        assert_eq!(
            parse("2026-06-06T21:00:30"),
            datetime!(2026-06-06 12:00:30 UTC)
        );
        assert_eq!(parse("2026-06-06T21:00"), datetime!(2026-06-06 12:00 UTC));
        assert!("2026-06-06".parse::<ContestTime>().is_err());
    }

    #[test]
    fn checks_period() {
        let period = Period {
            since: Some(datetime!(2026-06-06 12:00 UTC)),
            until: Some(datetime!(2026-06-07 12:00 UTC)),
        };
        assert!(!period.contains(datetime!(2026-06-06 11:59 UTC)));
        assert!(period.contains(datetime!(2026-06-06 12:00 UTC)));
        assert!(!period.contains(datetime!(2026-06-07 12:00 UTC)));
        assert!(Period::default().contains(datetime!(2000-01-01 0:00 UTC)));
    }
}
