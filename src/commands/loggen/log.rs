use compact_str::CompactString;
use time::{OffsetDateTime, UtcOffset};
use unicode_width::UnicodeWidthStr;

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

/// Aligns columns of rows by display width, separated by a space.
/// Columns in `right_aligned` are padded on the left.
pub fn align_columns<const N: usize>(rows: &[[String; N]], right_aligned: &[usize]) -> Vec<String> {
    let mut widths = [0; N];
    for row in rows {
        for (width, column) in widths.iter_mut().zip(row) {
            *width = (*width).max(column.width());
        }
    }

    rows.iter()
        .map(|row| {
            let padded: Vec<_> = row
                .iter()
                .zip(widths)
                .enumerate()
                .map(|(i, (column, width))| {
                    let fill = " ".repeat(width - column.width());
                    if right_aligned.contains(&i) {
                        format!("{fill}{column}")
                    } else {
                        format!("{column}{fill}")
                    }
                })
                .collect();
            padded.join(" ").trim_end().to_string()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aligns_columns_by_display_width() {
        let rows = [
            ["7010".to_string(), "小林".to_string(), "JA1RL".to_string()],
            ["144210".to_string(), "Yu".to_string(), "JA1ZZZ".to_string()],
        ];
        assert_eq!(
            align_columns(&rows, &[0]),
            ["  7010 小林 JA1RL", "144210 Yu   JA1ZZZ"]
        );
    }
}
