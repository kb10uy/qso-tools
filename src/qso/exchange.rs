use adif_reader::document::Record;
use compact_str::{CompactString, ToCompactString};

use crate::qso::{get_optional_field, get_optional_field_oneof};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QsoExchange {
    pub report: Option<CompactString>,
    pub number: Option<CompactString>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QsoExchanges {
    pub sent: QsoExchange,
    pub received: QsoExchange,
}

impl QsoExchanges {
    pub fn new(record: &Record) -> QsoExchanges {
        let sent = QsoExchange {
            report: get_optional_field(record, "RST_SENT").map(|s| s.to_compact_string()),
            number: get_optional_field_oneof(record, &["STX", "STX_STRING"])
                .map(|s| s.to_compact_string()),
        };
        let received = QsoExchange {
            report: get_optional_field(record, "RST_RCVD").map(|s| s.to_compact_string()),
            number: get_optional_field_oneof(record, &["SRX", "SRX_STRING"])
                .map(|s| s.to_compact_string()),
        };
        QsoExchanges { sent, received }
    }
}
