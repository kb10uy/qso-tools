use std::str::FromStr;

use adif_reader::document::Record;

use crate::qso::{error::QsoError, get_optional_field};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QslReceiveStatus {
    Invalid,
    Unconfirmed,
    Confirmed,
    Requested,
    Verified,
}

impl FromStr for QslReceiveStatus {
    type Err = QsoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "I" => Ok(QslReceiveStatus::Invalid),
            "N" => Ok(QslReceiveStatus::Unconfirmed),
            "Y" => Ok(QslReceiveStatus::Confirmed),
            "R" => Ok(QslReceiveStatus::Requested),
            "V" => Ok(QslReceiveStatus::Verified),
            _ => Err(QsoError::QslParse(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QslSendStatus {
    Invalid,
    NotSent,
    Sent,
    Requested,
    Queued,
}

impl FromStr for QslSendStatus {
    type Err = QsoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "I" => Ok(QslSendStatus::Invalid),
            "N" => Ok(QslSendStatus::NotSent),
            "Y" => Ok(QslSendStatus::Sent),
            "R" => Ok(QslSendStatus::Requested),
            "Q" => Ok(QslSendStatus::Queued),
            _ => Err(QsoError::QslParse(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QslStatus {
    pub receive: Option<QslReceiveStatus>,
    pub send: Option<QslSendStatus>,
}

impl QslStatus {
    pub fn new(record: &Record) -> Result<QslStatus, QsoError> {
        let receive = get_optional_field(record, "QSL_RCVD")
            .map(FromStr::from_str)
            .transpose()?;
        let send = get_optional_field(record, "QSL_SENT")
            .map(FromStr::from_str)
            .transpose()?;
        Ok(QslStatus { receive, send })
    }

    pub fn should_send(&self) -> bool {
        matches!(
            self.send,
            Some(QslSendStatus::Queued | QslSendStatus::Requested)
        )
    }

    pub fn received(&self) -> bool {
        matches!(
            self.receive,
            Some(QslReceiveStatus::Confirmed | QslReceiveStatus::Verified)
        )
    }
}
