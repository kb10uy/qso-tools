use compact_str::CompactString;
use time::{OffsetDateTime, UtcOffset};

use crate::{commands::loggen::submission::Submission, core::qso::band::Band};

/// Contest log independent of output formats.
#[derive(Debug, Clone, PartialEq)]
pub struct ContestLog {
    pub submission: Submission,
    pub offset: UtcOffset,
    pub qsos: Vec<LoggedQso>,
}

/// QSO written in the log.
#[derive(Debug, Clone, PartialEq)]
pub struct LoggedQso {
    pub datetime: OffsetDateTime,
    pub band: Band,
    pub frequency: f64,
    pub mode: CompactString,
    pub submode: Option<CompactString>,
    pub call: CompactString,
    pub station_callsign: Option<CompactString>,
    pub sent: LoggedExchange,
    pub received: LoggedExchange,
    pub multipliers: Vec<String>,
    pub point: i64,
    pub dupe: bool,
}

/// Report and number of exchange.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LoggedExchange {
    pub report: Option<String>,
    pub number: Option<String>,
}
